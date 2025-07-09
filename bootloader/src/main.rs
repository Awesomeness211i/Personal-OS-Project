#![feature(lang_items, core_intrinsics)]
#![allow(internal_features)]
#![no_main]
#![no_std]

use core::{
	self,
	mem::offset_of,
	panic,
	ptr,
	slice,
};

use bootloader::elf::{
	Elf,
	Elf64ProgramHeader,
};
use uefi::{
	GUID,
	PhysicalAddress,
	memory::{
		MemoryDescriptor,
		MemoryType,
	},
	protocols::{
		Protocol,
		file::{
			FileInfo,
			FileProtocol,
			SimpleFileSystemProtocol,
		},
		graphics::{
			// GraphicsOutputBLTOperation,
			GraphicsOutputProtocol,
			// GraphicsPixel,
			// PixelBitmask,
		},
		image::LoadedImageProtocol,
		text,
	},
	services::{
		AllocateType,
		ResetType,
	},
	status::Status,
	tables::{
		ConfigurationTable,
		DebugImageInfoTable,
		MemoryAttributesTable,
		SystemResourceTable,
		SystemTable,
	},
};

const PAGE_SIZE: usize = 4096;
const LZMA_FILESYSTEM: GUID = GUID::new(0xEE4E5898, 0x3914, 0x4259, 0x9D6E_DC7BD79403CF);
const DXE_SERVICES: GUID = GUID::new(0x05AD34BA, 0x6F02, 0x4214, 0x952E_4DA0398E2BB9);
const HANDOFF_BLOCK_LIST: GUID = GUID::new(0x7739F24C, 0x93D7, 0x11D4, 0x9A3A_0090273FC14D);
const MEMORY_TYPE_INFO_TABLE: GUID = GUID::new(0x4C19049F, 0x4137, 0x4DD3, 0x9C10_8B97A83FFDFA);
const MEMORY_STATUS_CODE_RECORD: GUID = GUID::new(0x060CC026, 0x4C0D, 0x4DDA, 0x8F41_595FEF00A502);

static SYSTEM_TABLE: core::sync::atomic::AtomicPtr<SystemTable> = core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

// static RSP: usize = 0;
// #[unsafe(naked)]
// #[unsafe(no_mangle)]
// unsafe extern "efiapi" fn efi_main() {
// 	core::arch::naked_asm!(
// 		"mov [{rsp}], rsp",
// 		"jmp {main}",
// 		rsp = sym RSP,
// 		main = sym main,
// 	)
// }

fn lower_nibble_to_hex(byte: u8) -> u8 {
	let value = byte & 0x0F;
	(if value < 10 { b'0' } else { b'A' - 10 }) + value
}

fn print_bytes<T>(data: &T, system_table: *mut SystemTable) {
	let bytes = unsafe { core::slice::from_raw_parts(data as *const T as *const u8, size_of::<T>()) };
	for byte in bytes {
		let mut message = ['\0' as u16, '\0' as u16, '\0' as u16];
		message[0] = lower_nibble_to_hex(byte >> 4) as u16;
		message[1] = lower_nibble_to_hex(*byte) as u16;
		let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&message) };
		unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
	}
	let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['\r' as u16, '\n' as u16, '\0' as u16]) };
	unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
}

fn print_string(data: &str, system_table: *mut SystemTable) {
	for ch in data.chars() {
		let mut message = ['\0' as u16, '\0' as u16];
		message[0] = ch as u16;
		let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&message) };
		unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
	}
	let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['\r' as u16, '\n' as u16, '\0' as u16]) };
	unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
}

fn print_string_bytes(data: &[u8], system_table: *mut SystemTable) {
	for ch in data {
		let mut message = ['\0' as u16, '\0' as u16];
		message[0] = *ch as u16;
		let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&message) };
		unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
	}
	let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['\r' as u16, '\n' as u16, '\0' as u16]) };
	unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
}

/// image: IN, system_table: IN
#[unsafe(no_mangle)]
extern "efiapi" fn efi_main(image_handle: *mut (), system_table: *mut SystemTable) -> Status {
	SYSTEM_TABLE.store(system_table, core::sync::atomic::Ordering::SeqCst);
	unsafe { ((*(*system_table).console_out).reset)((*system_table).console_out, true) };
	unsafe { ((*(*system_table).std_err).reset)((*system_table).std_err, true) };
	unsafe { ((*(*system_table).console_in).reset)((*system_table).console_in, true) };

	let graphics = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*(*system_table).boot_services).locate_protocol)(&GraphicsOutputProtocol::GUID, ptr::null_mut(), &mut interface_ptr) };
		interface_ptr as *const GraphicsOutputProtocol
	};

	let loaded_image = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*(*system_table).boot_services).handle_protocol)(image_handle, &LoadedImageProtocol::GUID, &mut interface_ptr) };
		interface_ptr as *const LoadedImageProtocol
	};

	let filesystem = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*(*system_table).boot_services).handle_protocol)((*loaded_image).device_handle as *mut (), &SimpleFileSystemProtocol::GUID, &mut interface_ptr) };
		interface_ptr as *const SimpleFileSystemProtocol
	};

	let root_filesystem = {
		let mut file_protocol = ptr::null();
		unsafe { ((*filesystem).open_volume)(filesystem, &mut file_protocol) };
		file_protocol
	};

	let filename = unsafe {
		let string = b"k\0e\0r\0n\0e\0l\0\\\0k\0e\0r\0n\0e\0l\0\0\0";
		uefi::CStr16::from_u16_with_nul_unchecked(slice::from_raw_parts(string as *const [u8; 28] as *const u8 as *const u16, string.len() / 2))
	};

	let kernel_file = {
		let mut file_protocol = ptr::null();
		let result = unsafe { ((*root_filesystem).open)(root_filesystem, &mut file_protocol, filename.as_ptr(), FileProtocol::MODE_READ, 0) };
		if result != Status::SUCCESS {
			return Status::ABORTED;
		}
		let status = unsafe { ((*root_filesystem).close)(root_filesystem) };
		if status != Status::SUCCESS {
			return Status::ABORTED;
		}
		file_protocol
	};

	let (pages, file_size) = {
		let mut size = 0;
		let mut buffer = ptr::null_mut();

		unsafe { ((*kernel_file).get_info)(kernel_file, &FileInfo::GUID, &mut size, ptr::null_mut()) };
		unsafe { ((*(*system_table).boot_services).allocate_pool)(MemoryType::LOADER_DATA, size, &mut buffer) };
		unsafe { ((*kernel_file).get_info)(kernel_file, &FileInfo::GUID, &mut size, buffer) };

		let file_size = unsafe { *(buffer.byte_offset(offset_of!(FileInfo, file_size) as isize) as *const u64) };
		unsafe { ((*(*system_table).boot_services).free_pool)(buffer) };
		((file_size as usize).div_ceil(PAGE_SIZE), file_size as usize)
	};

	let kernel_file_ptr = {
		let mut file = PhysicalAddress::new(0);
		let mut file_size = file_size;
		let status = unsafe { ((*(*system_table).boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, pages, &mut file) };
		if status != Status::SUCCESS {
			return Status::ABORTED;
		}
		let status = unsafe { ((*kernel_file).read)(kernel_file, &mut file_size, file.to_ptr()) };
		if status != Status::SUCCESS {
			return Status::ABORTED;
		}
		file.to_ptr::<u8>()
	};

	// 00781CA1-5DE3-405F-ABB8-379C3C076984
	// 1E2ED096-30E2-4254-BD89-863BBEF82325
	// 4E28CA50-D582-44AC-A11F-E3D56526DB34
	// C451ED2B-9694-45D3-BABA-ED9F8988A389
	let config_tables = unsafe { core::slice::from_raw_parts((*system_table).configuration_tables, (*system_table).num_table_entries) };
	for table in config_tables {
		match table.vendorguid {
			LZMA_FILESYSTEM => print_string("LZMA FILESYSTEM", system_table),
			DXE_SERVICES => print_string("DXE SERVICES", system_table),
			HANDOFF_BLOCK_LIST => print_string("HANDOFF BLOCK LIST", system_table),
			MEMORY_TYPE_INFO_TABLE => print_string("MEMORY TYPE INFO TABLE", system_table),
			DebugImageInfoTable::GUID => print_string("DEBUG IMAGE INFO TABLE", system_table),
			MEMORY_STATUS_CODE_RECORD => print_string("MEMORY STATUC CODE RECORD", system_table),
			ConfigurationTable::EFI_ACPI_20_TABLE => {
				let acpi_table = table.vendortable as *const acpi::RootSystemDescriptionPointerEx;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.rsdp.signature != *b"RSD PTR " {
					return Status::ABORTED;
				}
				print_string("ACPI20", system_table);
			},
			ConfigurationTable::ACPI_10_TABLE => {
				let acpi_table = table.vendortable as *const acpi::RootSystemDescriptionPointer;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.signature != *b"RSD PTR " {
					return Status::ABORTED;
				}
				print_string("ACPI10", system_table);
			},
			ConfigurationTable::SAL_SYSTEM_TABLE => print_string("SAL SYSTEM TABLE", system_table),
			ConfigurationTable::SMBIOS_TABLE => print_string("SMBIOS TABLE", system_table),
			ConfigurationTable::SMBIOS3_TABLE => print_string("SMBIOS3 TABLE", system_table),
			ConfigurationTable::MPS_TABLE => print_string("MPS TABLE", system_table),
			MemoryAttributesTable::GUID => print_string("UEFI MEMORY ATTRIBUTES TABLE", system_table),
			SystemResourceTable::GUID => print_string("EFI SYSTEM RESOURCE TABLE", system_table),
			guid => {
				let mut string = *b"UNKNOWN: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
				for (i, byte) in guid.data().to_ne_bytes().iter().rev().enumerate() {
					string[2 * i + 9] = lower_nibble_to_hex(*byte >> 4);
					string[2 * i + 10] = lower_nibble_to_hex(*byte);
				}
				print_string_bytes(&string, system_table);
			},
		}
	}

	let Ok(elf) = Elf::new(kernel_file_ptr, file_size) else {
		return Status::ABORTED;
	};

	match elf.header().program_header_offset {
		Some(phoff) => {
			let ph_table = unsafe { kernel_file_ptr.add(phoff.into()) };
			for i in 0..elf.header().program_header_num.expect("").into() {
				let ph = unsafe { &*(ph_table.add((i * elf.header().program_header_entry_size) as usize) as *const Elf64ProgramHeader) };
				if ph.p_type != 1
				/* PT_LOAD */
				{
					continue;
				}
				let total_pages = ph.p_memsz.div_ceil(PAGE_SIZE);
				let n_pages_from_file = ph.p_filesz.div_ceil(PAGE_SIZE);
				let n_alloc_pages = total_pages - n_pages_from_file;

				let seg_start_page = unsafe { kernel_file_ptr.add(ph.p_offset) };

				for x in 0..n_pages_from_file {
					let p_offset = x * PAGE_SIZE;
					// map seg_start_page + p_offset, ph.p_vaddr
				}
			}
		},
		None => return Status::ABORTED,
	}
	match elf.header().section_header_offset {
		Some(_shoff) => {
			// let sh_table = unsafe { kernel_file_ptr.add(shoff.into()) };
			// for i in 0..elf_header_ptr.section_header_num as usize {
			// 	let sh = unsafe { &*(sh_table.add(i * elf_header_ptr.section_header_entry_size as usize) as *const ElfSectionHeader) };
			// }
		},
		None => return Status::ABORTED,
	}

	let mut gdtr = arch::x86_64::DescriptorTablePointer { limit: 0, address: 0 };
	unsafe { core::arch::asm!("sgdt [{}]", in(reg) &mut gdtr) };
	print_bytes(&gdtr, system_table);

	let mut idtr = arch::x86_64::DescriptorTablePointer { limit: 0, address: 0 };
	unsafe { core::arch::asm!("sidt [{}]", in(reg) &mut idtr) };
	print_bytes(&idtr, system_table);

	let cr0: u64;
	unsafe { core::arch::asm!("mov {}, cr0", out(reg) cr0) };
	let cr0 = cr0.swap_bytes();
	print_bytes(&cr0, system_table);

	let cr2: u64;
	unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2) };
	let cr2 = cr2.swap_bytes();
	print_bytes(&cr2, system_table);

	let cr3: u64;
	unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3) };
	let cr3 = cr3.swap_bytes();
	print_bytes(&cr3, system_table);

	let cr4: u64;
	unsafe { core::arch::asm!("mov {}, cr4", out(reg) cr4) };
	let cr4 = cr4.swap_bytes();
	print_bytes(&cr4, system_table);

	let info = unsafe { core::arch::x86_64::__cpuid(0x00000000) };
	print_bytes(&info, system_table);

	let result = unsafe { core::arch::x86_64::__cpuid(0x80000008) };
	print_bytes(&result, system_table);

	// let graphics_ptr = unsafe { (*(*graphics).mode).framebuffer_base }.to_ptr();
	// let graphics_len = unsafe { (*(*graphics).mode).framebuffer_size } / size_of::<GraphicsPixel>();
	// let pix_per_scan = unsafe { (*(*(*graphics).mode).info).pixels_per_scanline };
	// let screen = unsafe { slice::from_raw_parts_mut(graphics_ptr, graphics_len) };

	// let mask = PixelBitmask::new(0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000);
	// let mut color = GraphicsOutputProtocol::grapics_color(0xFF00A5FF, &mask);
	// // graphics.fill_pixel(&color, (50, 50), (100, 200))?;
	// unsafe { ((*graphics).blt)(graphics, &mut color, GraphicsOutputBLTOperation::VIDEO_FILL, 0, 0, 50, 50, 100, 200, None) };
	// let mut color2 = GraphicsOutputProtocol::grapics_color(0xFF0000FF, &mask);
	// // graphics.fill_pixel(&color2, (60, 60), (80, 30))?;
	// unsafe { ((*graphics).blt)(graphics, &mut color2, GraphicsOutputBLTOperation::VIDEO_FILL, 0, 0, 60, 60, 80, 30, None) };
	// let white = GraphicsOutputProtocol::grapics_color(0xFFFFFFFF, &mask);
	// bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'\0', 0, 0, 20, &white);

	print_string("q = quit | r = reboot | c = continue | p = panic", system_table);
	let mut key = text::InputKey::default();
	let events = [unsafe { (*(*system_table).console_in).wait_for_key }];
	loop {
		let index = unsafe { (*(*system_table).boot_services).wait_for_event(&events) }.unwrap();
		#[allow(clippy::single_match)]
		match index {
			0 => {
				// system_table.stdin().read_keystroke_into(&mut key)?;
				unsafe { ((*(*system_table).console_in).read_keystroke)((*system_table).console_in, &mut key) };
				match key.unicodechar.try_into().unwrap() {
					'q' | 'Q' => unsafe { ((*(*system_table).runtime_services).reset_system)(ResetType::SHUTDOWN, Status::SUCCESS, 0, ptr::null()) },
					'r' | 'R' => unsafe { ((*(*system_table).runtime_services).reset_system)(ResetType::COLD, Status::SUCCESS, 0, ptr::null()) },
					'c' | 'C' => break,
					'p' | 'P' => panic!(),
					_ => continue,
				}
			},
			_ => {},
		}
	}

	unsafe { ((*(*system_table).console_out).reset)((*system_table).console_out, true) };
	unsafe { ((*(*system_table).std_err).reset)((*system_table).std_err, true) };
	unsafe { ((*(*system_table).console_in).reset)((*system_table).console_in, true) };

	let (memory_map, map_key, memory_map_size, descriptor_size, descriptor_version) = {
		let (mut memory_map_size, mut memory_map, mut map_key, mut descriptor_size, mut descriptor_version) = (0, ptr::null_mut(), 0, 0, 0);
		unsafe { ((*(*system_table).boot_services).get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		let allocation_status = unsafe { ((*(*system_table).boot_services).allocate_pool)(MemoryType::LOADER_DATA, memory_map_size, &mut memory_map) };
		match allocation_status {
			Status::SUCCESS => {},
			Status::OUT_OF_RESOURCES => {
				print_string("OUT OF RESOURCES", system_table);
				unsafe { ((*(*system_table).boot_services).stall)(1000000) };
				return Status::ABORTED;
			},
			Status::INVALID_PARAMETER => {
				print_string("INVALID PARAMETER", system_table);
				unsafe { ((*(*system_table).boot_services).stall)(1000000) };
				return Status::ABORTED;
			},
			_ => unreachable!(),
		}
		let result =
			unsafe { ((*(*system_table).boot_services).get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		match result {
			Status::SUCCESS => (memory_map, map_key, memory_map_size, descriptor_size, descriptor_version),
			Status::ERROR_BUFFER_TOO_SMALL => {
				print_string("BUFFER TOO SMALL", system_table);
				unsafe { ((*(*system_table).boot_services).stall)(1000000) };
				return Status::ABORTED;
			},
			Status::INVALID_PARAMETER => {
				print_string("INVALID PARAMETER", system_table);
				unsafe { ((*(*system_table).boot_services).stall)(1000000) };
				return Status::ABORTED;
			},
			_ => unreachable!(),
		}
	};

	let mut best_num_pages = 0;
	let mut best_alloc_start = PhysicalAddress::new(0);
	for i in 0..memory_map_size / descriptor_size {
		let desc = unsafe { &*((memory_map as *mut u8).add(descriptor_size * i) as *mut MemoryDescriptor) };
		if desc.region_type != MemoryType::CONVENTIONAL_MEMORY {
			continue;
		}
		if desc.num_pages > best_num_pages {
			best_num_pages = desc.num_pages;
			best_alloc_start = desc.physical_start;
		}
	}
	if best_alloc_start.to_ptr::<()>().is_null() {
		return Status::ABORTED;
	}

	match unsafe { ((*(*system_table).boot_services).exit_boot_services)(image_handle, map_key) } {
		Status::SUCCESS => {
			// bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'A', 5, 200, 20, &white);
			// let start: fn(kernel::KernelData) -> ! = unsafe { mem::transmute(kernel_file_ptr.add(528)) };
			// start(kernel::KernelData::new(graphics_ptr, graphics_len));
			loop {}
		},
		_ => {
			let _x = unsafe { ((*(*system_table).boot_services).free_pages)(PhysicalAddress::new(kernel_file_ptr as u64), pages) };
			let _y = unsafe { ((*(*system_table).boot_services).free_pool)(memory_map) };
			unsafe { ((*kernel_file).close)(kernel_file) };
			Status::ABORTED
		},
	}
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
	let system_table = SYSTEM_TABLE.load(core::sync::atomic::Ordering::SeqCst);
	print_string("Panic", system_table);
	unsafe { ((*(*system_table).boot_services).stall)(1000000) };
	unsafe { ((*(*system_table).runtime_services).reset_system)(ResetType::COLD, Status::SUCCESS, 0, ptr::null()) }
}
