// #![feature(lang_items, core_intrinsics)]
// #![allow(internal_features)]
#![no_main]
#![no_std]

use core::{
	self,
	ffi::c_void,
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
	SystemTablePointer,
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
			GraphicsOutputBLTOperation,
			GraphicsOutputProtocol,
			GraphicsPixel,
			PixelBitmask,
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

fn print_bytes<T>(data: &T) {
	let system_table = unsafe { SYSTEM_TABLE }.expect("SystemTablePointer should be initialized");
	let bytes = unsafe { core::slice::from_raw_parts(data as *const T as *const u8, size_of::<T>()) };
	for byte in bytes {
		let mut message = ['\0' as u16, '\0' as u16, ' ' as u16, '\0' as u16];
		message[0] = lower_nibble_to_hex(byte >> 4) as u16;
		message[1] = lower_nibble_to_hex(*byte) as u16;
		let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&message) };
		unsafe { ((*system_table.console_out).output_string)(system_table.console_out, prompt.as_ptr()) };
	}
}

fn println_bytes<T>(data: &T) {
	let system_table = unsafe { SYSTEM_TABLE }.expect("SystemTablePointer should be initialized");
	print_bytes(data);
	let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['\r' as u16, '\n' as u16, '\0' as u16]) };
	unsafe { ((*system_table.console_out).output_string)(system_table.console_out, prompt.as_ptr()) };
}

fn print_string(data: &str) {
	let system_table = unsafe { SYSTEM_TABLE }.expect("SystemTablePointer should be initialized");
	for ch in data.chars() {
		let mut message = ['\0' as u16, '\0' as u16];
		message[0] = ch as u16;
		let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&message) };
		unsafe { ((*system_table.console_out).output_string)(system_table.console_out, prompt.as_ptr()) };
	}
}

fn println_string(data: &str) {
	let system_table = unsafe { SYSTEM_TABLE }.expect("SystemTablePointer should be initialized");
	print_string(data);
	let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['\r' as u16, '\n' as u16, '\0' as u16]) };
	unsafe { ((*system_table.console_out).output_string)(system_table.console_out, prompt.as_ptr()) };
}

fn print_string_bytes(data: &[u8]) {
	let system_table = unsafe { SYSTEM_TABLE }.expect("SystemTablePointer should be initialized");
	for ch in data {
		let mut message = ['\0' as u16, '\0' as u16];
		message[0] = *ch as u16;
		let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&message) };
		unsafe { ((*system_table.console_out).output_string)(system_table.console_out, prompt.as_ptr()) };
	}
}

fn println_string_bytes(data: &[u8]) {
	let system_table = unsafe { SYSTEM_TABLE }.expect("SystemTablePointer should be initialized");
	print_string_bytes(data);
	let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['\r' as u16, '\n' as u16, '\0' as u16]) };
	unsafe { ((*system_table.console_out).output_string)(system_table.console_out, prompt.as_ptr()) };
}

static mut SYSTEM_TABLE: Option<SystemTablePointer> = None;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct PageTableEntries(u64);

#[repr(C, align(4096))]
struct PageTable {
	pub entries: [PageTableEntries; 512],
}

const PAGE_SIZE: usize = 4096;
const LZMA_FILESYSTEM: GUID = GUID::new(0xEE4E5898, 0x3914, 0x4259, 0x9D6E_DC7BD79403CF);
const DXE_SERVICES: GUID = GUID::new(0x05AD34BA, 0x6F02, 0x4214, 0x952E_4DA0398E2BB9);
const HANDOFF_BLOCK_LIST: GUID = GUID::new(0x7739F24C, 0x93D7, 0x11D4, 0x9A3A_0090273FC14D);
const MEMORY_TYPE_INFO_TABLE: GUID = GUID::new(0x4C19049F, 0x4137, 0x4DD3, 0x9C10_8B97A83FFDFA);
const MEMORY_STATUS_CODE_RECORD: GUID = GUID::new(0x060CC026, 0x4C0D, 0x4DDA, 0x8F41_595FEF00A502);

/// image: IN, system_table: IN
#[unsafe(no_mangle)]
extern "efiapi" fn efi_main(image_handle: &mut c_void, system_table: SystemTablePointer<'static>) -> Status {
	unsafe { SYSTEM_TABLE = Some(system_table) };

	if system_table.header.signature != SystemTable::SIGNATURE {
		return Status::INVALID_PARAMETER;
	}
	unsafe { ((*system_table.console_out).reset)(system_table.console_out, true) };
	unsafe { ((*system_table.std_err).reset)(system_table.std_err, true) };
	unsafe { ((*system_table.console_in).reset)(system_table.console_in, true) };
	let mut keyinput = text::InputKey::default();
	let events = [unsafe { (*system_table.console_in).wait_for_key }];

	let graphics = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*system_table.boot_services).locate_protocol)(&GraphicsOutputProtocol::GUID, ptr::null_mut(), &mut interface_ptr) };
		interface_ptr as *const GraphicsOutputProtocol
	};

	// Interesting GUIDs:
	// 00781CA1-5DE3-405F-ABB8-379C3C076984
	// 1E2ED096-30E2-4254-BD89-863BBEF82325
	// 4E28CA50-D582-44AC-A11F-E3D56526DB34
	// C451ED2B-9694-45D3-BABA-ED9F8988A389
	let config_tables = unsafe { core::slice::from_raw_parts(system_table.configuration_tables, system_table.num_table_entries) };
	for table in config_tables {
		match table.vendorguid {
			LZMA_FILESYSTEM => println_string("LZMA FILESYSTEM"),
			DXE_SERVICES => println_string("DXE SERVICES"),
			HANDOFF_BLOCK_LIST => println_string("HANDOFF BLOCK LIST"),
			MEMORY_TYPE_INFO_TABLE => println_string("MEMORY TYPE INFO TABLE"),
			DebugImageInfoTable::GUID => println_string("DEBUG IMAGE INFO TABLE"),
			MEMORY_STATUS_CODE_RECORD => println_string("MEMORY STATUS CODE RECORD"),
			ConfigurationTable::EFI_ACPI_20_TABLE => {
				let acpi_table = table.vendortable as *const acpi::RootSystemDescriptionPointerEx;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.rsdp.signature != *b"RSD PTR " {
					return Status::ABORTED;
				}
				println_string("ACPI20");
			},
			ConfigurationTable::ACPI_10_TABLE => {
				let acpi_table = table.vendortable as *const acpi::RootSystemDescriptionPointer;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.signature != *b"RSD PTR " {
					return Status::ABORTED;
				}
				println_string("ACPI10");
			},
			ConfigurationTable::SAL_SYSTEM_TABLE => println_string("SAL SYSTEM TABLE"),
			ConfigurationTable::SMBIOS_TABLE => println_string("SMBIOS TABLE"),
			ConfigurationTable::SMBIOS3_TABLE => println_string("SMBIOS3 TABLE"),
			ConfigurationTable::MPS_TABLE => println_string("MPS TABLE"),
			MemoryAttributesTable::GUID => println_string("UEFI MEMORY ATTRIBUTES TABLE"),
			SystemResourceTable::GUID => println_string("EFI SYSTEM RESOURCE TABLE"),
			guid => {
				print_string("UNKNOWN: ");
				println_bytes(&guid.data().to_ne_bytes());
			},
		}
	}

	let loaded_image = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*system_table.boot_services).handle_protocol)(image_handle, &LoadedImageProtocol::GUID, &mut interface_ptr) };
		interface_ptr as *const LoadedImageProtocol
	};

	let filesystem = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*system_table.boot_services).handle_protocol)((*loaded_image).device_handle as *mut c_void, &SimpleFileSystemProtocol::GUID, &mut interface_ptr) };
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
		unsafe { ((*system_table.boot_services).allocate_pool)(MemoryType::LOADER_DATA, size, &mut buffer) };
		unsafe { ((*kernel_file).get_info)(kernel_file, &FileInfo::GUID, &mut size, buffer) };

		let file_size = unsafe { *(buffer.byte_offset(offset_of!(FileInfo, file_size) as isize) as *const u64) };
		unsafe { ((*system_table.boot_services).free_pool)(buffer) };
		((file_size as usize).div_ceil(PAGE_SIZE), file_size as usize)
	};

	let kernel_file_ptr = {
		let mut file = PhysicalAddress::new(0);
		let mut file_size = file_size;
		let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, pages, &mut file) };
		if status != Status::SUCCESS {
			return Status::ABORTED;
		}
		let status = unsafe { ((*kernel_file).read)(kernel_file, &mut file_size, file.to_ptr()) };
		if status != Status::SUCCESS {
			return Status::ABORTED;
		}
		file.to_ptr::<u8>()
	};

	let Ok(elf) = Elf::new(kernel_file_ptr, file_size) else {
		return Status::ABORTED;
	};

	let Some(phoff) = elf.header().program_header_offset else {
		return Status::ABORTED;
	};

	// let uefi_cr3: *mut PageTable;
	// unsafe { core::arch::asm!("mov {}, cr3", out(reg) uefi_cr3) };
	// print_string("CR3: ");
	// println_bytes(&uefi_cr3);
	//
	// for entry in unsafe { (*uefi_cr3).entries } {
	// 	print_string("Page Table Entry: ");
	// 	println_bytes(&entry);
	// 	let _ = unsafe { (*system_table.boot_services).wait_for_event(&events) }.unwrap();
	// 	let _ = unsafe { ((*system_table.console_in).read_keystroke)(system_table.console_in, &mut keyinput) };
	// }

	let mut segments_loaded = 0usize;
	for i in 0..elf.header().program_header_num.expect("").into() {
		let program_header = unsafe { &*(kernel_file_ptr.add(usize::from(phoff) + (i * elf.header().program_header_entry_size) as usize) as *const Elf64ProgramHeader) };
		if program_header.p_type == bootloader::elf::ProgramHeaderType::LOAD
		/* PT_LOAD */
		{
			let file_offset = program_header.p_offset;
			let file_size = program_header.p_filesz;
			let mem_size = program_header.p_memsz;
			let segment_pages = mem_size.div_ceil(PAGE_SIZE);
			let mut mem_addr = PhysicalAddress::new(0);
			let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, segment_pages, &mut mem_addr) };

			if status != Status::SUCCESS {
				println_string("Failed to allocate page");
				match status {
					Status::OUT_OF_RESOURCES => println_string("OUT OF RESOURCES"),
					Status::INVALID_PARAMETER => println_string("INVALID PARAMETER"),
					Status::NOT_FOUND => println_string("NOT FOUND"),
					_ => unreachable!(),
				}
				let _index = unsafe { (*system_table.boot_services).wait_for_event(&events) }.unwrap();
				let _ = unsafe { ((*system_table.console_in).read_keystroke)(system_table.console_in, &mut keyinput) };
				return status;
			}

			print_string("Num pages: ");
			println_bytes(&segment_pages);
			print_string("Offset: ");
			println_bytes(&file_offset);
			print_string("File size: ");
			println_bytes(&file_size);
			print_string("Mem size: ");
			println_bytes(&mem_size);
			print_string("Mem addr: ");
			println_bytes(&mem_addr);

			if file_size > 0 {
				let ptr = unsafe { kernel_file_ptr.add(file_offset) };
				for j in 0..file_size {
					unsafe { *mem_addr.to_ptr::<u8>().add(j) = *ptr.add(j) }
				}
			}

			let zero_fill_count = mem_size - file_size;
			if zero_fill_count > 0 {
				let ptr = unsafe { mem_addr.to_ptr::<u8>().add(file_size) };
				for j in 0..zero_fill_count {
					unsafe { *ptr.add(j) = 0 }
				}
			}

			segments_loaded += 1;
		}
	}
	if segments_loaded == 0 {
		return Status::NOT_FOUND;
	}

	let graphics_mode = unsafe { (*graphics).mode };
	let graphics_ptr = unsafe { (*graphics_mode).framebuffer_base }.to_ptr();
	let graphics_len = unsafe { (*graphics_mode).framebuffer_size } / size_of::<GraphicsPixel>();
	let pix_per_scan = unsafe { (*(*graphics_mode).info).pixels_per_scanline };
	let screen = unsafe { slice::from_raw_parts_mut(graphics_ptr, graphics_len) };

	let mask = PixelBitmask::new(0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000);
	let mut color = GraphicsOutputProtocol::grapics_color(0xFF00A5FF, &mask);
	// graphics.fill_pixel(&color, (50, 50), (100, 200))?;
	unsafe { ((*graphics).blt)(graphics, &mut color, GraphicsOutputBLTOperation::VIDEO_FILL, 0, 0, 50, 50, 100, 200, None) };
	let mut color2 = GraphicsOutputProtocol::grapics_color(0xFF0000FF, &mask);
	// graphics.fill_pixel(&color2, (60, 60), (80, 30))?;
	unsafe { ((*graphics).blt)(graphics, &mut color2, GraphicsOutputBLTOperation::VIDEO_FILL, 0, 0, 60, 60, 80, 30, None) };
	let white = GraphicsOutputProtocol::grapics_color(0xFFFFFFFF, &mask);
	bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'\0', 0, 0, 20, &white);

	println_string("q = quit | r = reboot | c = continue | p = panic");
	loop {
		let index = unsafe { (*system_table.boot_services).wait_for_event(&events) }.unwrap();
		#[allow(clippy::single_match)]
		match index {
			0 => {
				// system_table.stdin().read_keystroke_into(&mut key)?;
				unsafe { ((*system_table.console_in).read_keystroke)(system_table.console_in, &mut keyinput) };
				match keyinput.unicodechar.try_into().unwrap() {
					'q' | 'Q' => unsafe { ((*system_table.runtime_services).reset_system)(ResetType::SHUTDOWN, Status::SUCCESS, 0, ptr::null()) },
					'r' | 'R' => unsafe { ((*system_table.runtime_services).reset_system)(ResetType::COLD, Status::SUCCESS, 0, ptr::null()) },
					'c' | 'C' => break,
					'p' | 'P' => panic!(),
					_ => continue,
				}
			},
			_ => {},
		}
	}

	unsafe { ((*system_table.console_out).reset)(system_table.console_out, true) };
	unsafe { ((*system_table.std_err).reset)(system_table.std_err, true) };
	unsafe { ((*system_table.console_in).reset)(system_table.console_in, true) };

	let (memory_map, map_key, memory_map_size, descriptor_size, descriptor_version) = {
		let (mut memory_map_size, mut memory_map, mut map_key, mut descriptor_size, mut descriptor_version) = (0, ptr::null_mut(), 0, 0, 0);
		let result = unsafe { ((*system_table.boot_services).get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		if result == Status::SUCCESS {
			print_string("UNEXPECTED SUCCESS");
			unsafe { ((*system_table.boot_services).stall)(1000000) };
			return Status::ABORTED;
		}
		let Status::SUCCESS = (unsafe { ((*system_table.boot_services).allocate_pool)(MemoryType::LOADER_DATA, memory_map_size, &mut memory_map) }) else {
			print_string("FAILED TO ALLOCATE BUFFER FOR MEMORY MAP");
			unsafe { ((*system_table.boot_services).stall)(1000000) };
			return Status::ABORTED;
		};
		let Status::SUCCESS =
			(unsafe { ((*system_table.boot_services).get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) })
		else {
			println_string("FAILED TO GET MEMORY MAP");
			unsafe { ((*system_table.boot_services).stall)(1000000) };
			return Status::ABORTED;
		};
		(memory_map, map_key, memory_map_size, descriptor_size, descriptor_version)
	};
	match unsafe { ((*system_table.boot_services).exit_boot_services)(image_handle, map_key) } {
		Status::SUCCESS => {
			// bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'A', 5, 200, 20, &white);
			let start: fn(kernel::KernelData) -> ! = unsafe { core::mem::transmute(elf.header().entry) };
			start(kernel::KernelData::new(graphics_ptr, graphics_len));
		},
		Status::INVALID_PARAMETER => {
			let num_descriptors = memory_map_size / descriptor_size;
			let mut loader_data_num = 0;
			let mut loader_code_num = 0;
			let mut boot_data_num = 0;
			let mut boot_code_num = 0;
			let mut runtime_data_num = 0;
			let mut runtime_code_num = 0;
			let mut conventional_num = 0;
			let mut unusable_num = 0;
			let mut acpi_reclaim_num = 0;
			let mut acpi_nvs_num = 0;
			let mut mmio_num = 0;
			let mut mmio_port_space_num = 0;
			let mut pal_num = 0;
			let mut persistent_num = 0;
			let mut unaccepted_num = 0;
			let mut reserved_num = 0;
			let mut unknown_num = 0;
			for i in 0..num_descriptors {
				let descriptor = unsafe { &*((memory_map as *mut u8).add(i * descriptor_size) as *mut MemoryDescriptor) };
				match descriptor.region_type {
					MemoryType::LOADER_DATA => loader_data_num += descriptor.num_pages,
					MemoryType::LOADER_CODE => loader_code_num += descriptor.num_pages,
					MemoryType::BOOT_SERVICES_DATA => boot_data_num += descriptor.num_pages,
					MemoryType::BOOT_SERVICES_CODE => boot_code_num += descriptor.num_pages,
					MemoryType::RUNTIME_SERVICES_DATA => runtime_data_num += descriptor.num_pages,
					MemoryType::RUNTIME_SERVICES_CODE => runtime_code_num += descriptor.num_pages,
					MemoryType::CONVENTIONAL_MEMORY => conventional_num += descriptor.num_pages,
					MemoryType::UNUSABLE_MEMORY => unusable_num += descriptor.num_pages,
					MemoryType::ACPI_RECLAIM_MEMORY => acpi_reclaim_num += descriptor.num_pages,
					MemoryType::ACPI_MEMORY_NVS => acpi_nvs_num += descriptor.num_pages,
					MemoryType::MEMORY_MAPPED_IO => mmio_num += descriptor.num_pages,
					MemoryType::MEMORY_MAPPED_IO_PORT_SPACE => mmio_port_space_num += descriptor.num_pages,
					MemoryType::PAL_CODE => pal_num += descriptor.num_pages,
					MemoryType::PERSISTENT_MEMORY => persistent_num += descriptor.num_pages,
					MemoryType::UNACCEPTED => unaccepted_num += descriptor.num_pages,
					MemoryType::RESERVED => reserved_num += descriptor.num_pages,
					_ => unknown_num += descriptor.num_pages,
				}
				// print_string("Physical Start: ", system_table);
				// println_bytes(&unsafe { (*descriptor).physical_start }, system_table);
				// print_string("Virtual Start: ", system_table);
				// println_bytes(&unsafe { (*descriptor).virtual_start }, system_table);
				// print_string("Number of Pages: ", system_table);
				// println_bytes(&unsafe { (*descriptor).num_pages }, system_table);
				// print_string("Attributes: ", system_table);
				// println_bytes(&unsafe { (*descriptor).attribute }, system_table);
			}
			print_string("Number of Descriptors: ");
			println_bytes(&num_descriptors);
			print_string("LOADER DATA PAGES: ");
			println_bytes(&loader_data_num);
			print_string("LOADER CODE PAGES: ");
			println_bytes(&loader_code_num);
			print_string("BOOT SERVICES DATA PAGES: ");
			println_bytes(&boot_data_num);
			print_string("BOOT SERVICES CODE PAGES: ");
			println_bytes(&boot_code_num);
			print_string("RUNTIME SERVICES DATA PAGES: ");
			println_bytes(&runtime_data_num);
			print_string("RUNTIME SERVICES CODE PAGES: ");
			println_bytes(&runtime_code_num);
			print_string("CONVENTIONAL MEMORY PAGES: ");
			println_bytes(&conventional_num);
			print_string("UNUSABLE MEMORY PAGES: ");
			println_bytes(&unusable_num);
			print_string("ACPI RECLAIM MEMORY PAGES: ");
			println_bytes(&acpi_reclaim_num);
			print_string("ACPI MEMORY NVS PAGES: ");
			println_bytes(&acpi_nvs_num);
			print_string("MEMORY MAPPED IO PAGES: ");
			println_bytes(&mmio_num);
			print_string("MEMORY MAPPED IO PORT SPACE PAGES: ");
			println_bytes(&mmio_port_space_num);
			print_string("PAL CODE PAGES: ");
			println_bytes(&pal_num);
			print_string("PERSISTENT MEMORY PAGES: ");
			println_bytes(&persistent_num);
			print_string("UNACCEPTED PAGES: ");
			println_bytes(&unaccepted_num);
			print_string("RESERVED PAGES: ");
			println_bytes(&reserved_num);
			print_string("UNKNOWN PAGES: ");
			println_bytes(&unknown_num);
			println_string("");

			println_string("Exit BootServices failed: Mapkey incorrect");
			print_string("Memory Map Pointer: ");
			println_bytes(&memory_map);
			print_string("Memory Map Key: ");
			println_bytes(&map_key);
			print_string("Memory Map Size: ");
			println_bytes(&memory_map_size);
			print_string("Memory Map Descriptor Size: ");
			println_bytes(&descriptor_size);
			print_string("Memory Map Descriptor Version: ");
			println_bytes(&descriptor_version);

			let _x = unsafe { ((*system_table.boot_services).free_pages)(PhysicalAddress::new(kernel_file_ptr as u64), pages) };
			let _y = unsafe { ((*system_table.boot_services).free_pool)(memory_map) };
			unsafe { ((*kernel_file).close)(kernel_file) };
			let _ = unsafe { (*system_table.boot_services).wait_for_event(&events) }.unwrap();
			let _ = unsafe { ((*system_table.console_in).read_keystroke)(system_table.console_in, &mut keyinput) };
			Status::ABORTED
		},
		_ => unreachable!(),
	}

	// let mut gdtr = arch::x86_64::DescriptorTablePointer { limit: 0, address: 0 };
	// unsafe { core::arch::asm!("sgdt [{}]", in(reg) &mut gdtr) };
	// print_string("GDTR: ");
	// println_bytes(&gdtr);

	// let mut idtr = arch::x86_64::DescriptorTablePointer { limit: 0, address: 0 };
	// unsafe { core::arch::asm!("sidt [{}]", in(reg) &mut idtr) };
	// print_string("IDTR: ");
	// println_bytes(&idtr);

	// let control_flag_strings = [
	// 	"PE,",
	// 	"MP,",
	// 	"EM,",
	// 	"TS,",
	// 	"ET,",
	// 	"NE,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"WP,",
	// 	"Reserved,",
	// 	"AM,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"NW,",
	// 	"CD,",
	// 	"PG,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// 	"Reserved,",
	// ];
	// let cr0: u64;
	// unsafe { core::arch::asm!("mov {}, cr0", out(reg) cr0) };
	// print_string("CR0: ");
	// for (i, string) in control_flag_strings.iter().enumerate() {
	// 	if cr0 & (1 << i) > 0 {
	// 		print_string(string);
	// 	}
	// }
	// println_string("");

	// let cr2: u64;
	// unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2) };
	// print_string("CR2: ");
	// println_bytes(&cr2);

	// Model specific extensions register
	// Can use CPUID to query support for feature except for performance counter extensions
	// let model_specific_features = [
	// 	"VME,",
	// 	"PVI,",
	// 	"TSD,",
	// 	"DE,",
	// 	"PSE,",
	// 	"PAE,",
	// 	"MCE,",
	// 	"PGE,",
	// 	"PCE,",
	// 	"OSFXSR,",
	// 	"OSXMMEXCPT,",
	// 	"UMIP,",
	// 	"LA57,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"FSGSBASE,",
	// 	"PCIDE,",
	// 	"OSXSAVE,",
	// 	"RESERVED,",
	// 	"SMEP,",
	// 	"SMAP,",
	// 	"PKE,",
	// 	"CET,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// ];
	// let cr4: u64;
	// unsafe { core::arch::asm!("mov {}, cr4", out(reg) cr4) };
	// print_string("CR4: ");
	// for (i, string) in model_specific_features.iter().enumerate() {
	// 	if cr4 & (1 << i) > 0 {
	// 		print_string(string);
	// 	}
	// }
	// println_string("");

	// Task Priority Register
	// lowest 4 bits for 1-15 task priority where 0 enables and 15 disables all external interrupts
	// let cr8: u64;
	// unsafe { core::arch::asm!("mov {}, cr8", out(reg) cr8) };
	// print_string("CR8: ");
	// println_bytes(&cr8);
	//
	// let max = core::arch::x86_64::__cpuid(0x80000000);
	// if max.eax < 0x80000008 {
	// 	return Status::INVALID_PARAMETER;
	// }
	// print_string("MAX CPUID: ");
	// println_bytes(&max.eax);

	// let vendor_info = unsafe { core::mem::transmute::<core::arch::x86_64::CpuidResult, [u8; size_of::<core::arch::x86_64::CpuidResult>()]>(core::arch::x86_64::__cpuid(0x00000000)) };
	// print_string("Vendor Info: ");
	// println_string_bytes(unsafe { core::slice::from_raw_parts(vendor_info.as_ptr().add(size_of::<u32>()), 3 * size_of::<u32>()) }, system_table);

	// let flag_strings = [
	// 	"FPU,",
	// 	"VME,",
	// 	"DE,",
	// 	"PSE,",
	// 	"TSC,",
	// 	"MSR,",
	// 	"PAE,",
	// 	"MCE,",
	// 	"CX8,",
	// 	"APIC,",
	// 	"RESERVED,",
	// 	"SEP,",
	// 	"MTRR,",
	// 	"PGE,",
	// 	"MCA,",
	// 	"CMOV,",
	// 	"PAT,",
	// 	"PSE-36,",
	// 	"PSN,",
	// 	"CLFSH,",
	// 	"RESERVED,",
	// 	"DS,",
	// 	"ACPI,",
	// 	"MMX,",
	// 	"FXSR,",
	// 	"SSE,",
	// 	"SSE2,",
	// 	"SS,",
	// 	"HTT,",
	// 	"TM,",
	// 	"RESERVED,",
	// 	"PBE,",
	// ];
	// let test = core::arch::x86_64::__cpuid(0x00000001);
	// print_string("Flags: ");
	// for (i, string) in flag_strings.iter().enumerate() {
	// 	if test.edx & (1 << i) > 0 {
	// 		print_string(string);
	// 	}
	// }
	// println_string("");

	// let tlb_info = core::arch::x86_64::__cpuid(0x00000002);
	// print_string("TLB Info: ");
	// println_bytes(&tlb_info);

	// let cache_info = core::arch::x86_64::__cpuid(0x00000002);
	// print_string("Cache Info: ");
	// println_bytes(&cache_info);

	// let addr_info = core::arch::x86_64::__cpuid(0x80000008);
	// print_string("Address Space Info: ");
	// println_bytes(&addr_info);

	// let efer_features = [
	// 	"SCE,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"LME,",
	// 	"RESERVED,",
	// 	"LMA,",
	// 	"NXE,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// ];
	// let efer = unsafe { arch::x86_64::rdmsr(0xC0000080) };
	// print_string("EFER: ");
	// for (i, string) in efer_features.iter().enumerate() {
	// 	if efer & (1 << i) > 0 {
	// 		print_string(string);
	// 	}
	// }
	// println_string("");
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
	match unsafe { SYSTEM_TABLE } {
		Some(system_table) => {
			let message = info.message().as_str().unwrap_or("");
			let location = info.location().unwrap();
			let file_name = location.file();
			let line_number = location.line();
			let column_number = location.column();
			print_string("Panicked at file name: ");
			println_string(file_name);
			print_string(" line number: ");
			println_bytes(&line_number);
			print_string(" column number: ");
			println_bytes(&column_number);
			println_string(message);
			unsafe { ((*system_table.boot_services).stall)(1000000) };
			unsafe { ((*system_table.runtime_services).reset_system)(ResetType::COLD, Status::SUCCESS, 0, ptr::null()) }
		},
		None => loop {},
	}
}
