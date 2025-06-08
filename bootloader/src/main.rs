#![feature(lang_items, core_intrinsics)]
#![allow(internal_features)]
#![no_main]
#![no_std]

use core::{
	self,
	mem::{
		self,
		offset_of,
	},
	panic,
	ptr,
	slice,
};

use acpi::{
	RootSystemDescriptionPointer,
	RootSystemDescriptionPointerEx,
};
use bootloader::elf::{
	Elf,
	Elf64ProgramHeader,
};
use uefi::{
	GUID,
	PhysicalAddress,
	memory::MemoryType,
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

const PAGE_SIZE: usize = 4096;
const LZMA_FILESYSTEM: GUID = GUID::new(0xEE4E5898, 0x3914, 0x4259, 0x9D6E_DC7BD79403CF);
const DXE_SERVICES: GUID = GUID::new(0x05AD34BA, 0x6F02, 0x4214, 0x952E_4DA0398E2BB9);
const HANDOFF_BLOCK_LIST: GUID = GUID::new(0x7739F24C, 0x93D7, 0x11D4, 0x9A3A_0090273FC14D);
const MEMORY_TYPE_INFO_TABLE: GUID = GUID::new(0x4C19049F, 0x4137, 0x4DD3, 0x9C10_8B97A83FFDFA);
const MEMORY_STATUS_CODE_RECORD: GUID = GUID::new(0x060CC026, 0x4C0D, 0x4DDA, 0x8F41_595FEF00A502);

// 00781CA1-5DE3-405F-ABB8-379C3C076984
// 1E2ED096-30E2-4254-BD89-863BBEF82325
// 4E28CA50-D582-44AC-A11F-E3D56526DB34
// C451ED2B-9694-45D3-BABA-ED9F8988A389

// static RSP: usize = 0;
// arch::global_asm!(
// 	".global efi_main",
// 	"efi_main:",
// 		"mov [{rsp}], rsp",
// 		"jmp {main}",
// 	rsp = sym RSP,
// 	main = sym efi_main,
// );

fn lower_nibble_to_hex(byte: u8) -> u8 {
	if byte <= 9 { b'0' + byte } else { b'A' + byte - 10 }
}

static SYSTEM_TABLE: core::sync::atomic::AtomicPtr<SystemTable> = core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

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
		uefi::CStr16::from_u16_with_nul_unchecked(slice::from_raw_parts(string as *const [u8] as *const u8 as *const u16, string.len() / 2))
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

	let config_tables = unsafe { core::slice::from_raw_parts((*system_table).configuration_tables, (*system_table).num_table_entries) };
	for table in config_tables {
		match table.vendorguid {
			LZMA_FILESYSTEM => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['L' as u16, 'Z' as u16, 'M' as u16, 'A' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			DXE_SERVICES => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['D' as u16, 'X' as u16, 'E' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			HANDOFF_BLOCK_LIST => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['H' as u16, 'A' as u16, 'N' as u16, 'D' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			MEMORY_TYPE_INFO_TABLE => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['M' as u16, 'E' as u16, 'M' as u16, 'T' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			DebugImageInfoTable::GUID => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['D' as u16, 'I' as u16, 'I' as u16, 'T' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			MEMORY_STATUS_CODE_RECORD => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['M' as u16, 'S' as u16, 'C' as u16, 'R' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			ConfigurationTable::EFI_ACPI_20_TABLE => {
				let acpi_table = table.vendortable as *const RootSystemDescriptionPointerEx;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.rsdp.signature != *b"RSD PTR " {
					return Status::ABORTED;
				}
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['A' as u16, 'C' as u16, 'P' as u16, 'I' as u16, '2' as u16, '0' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			ConfigurationTable::ACPI_10_TABLE => {
				let acpi_table = table.vendortable as *const RootSystemDescriptionPointer;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.signature != *b"RSD PTR " {
					return Status::ABORTED;
				}
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['A' as u16, 'C' as u16, 'P' as u16, 'I' as u16, '1' as u16, '0' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			ConfigurationTable::SAL_SYSTEM_TABLE => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['S' as u16, 'A' as u16, 'L' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			ConfigurationTable::SMBIOS_TABLE => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['S' as u16, 'M' as u16, 'B' as u16, 'I' as u16, 'O' as u16, 'S' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			ConfigurationTable::SMBIOS3_TABLE => {
				let prompt = unsafe {
					uefi::CStr16::from_u16_with_nul_unchecked(&[
						'S' as u16,
						'M' as u16,
						'B' as u16,
						'I' as u16,
						'O' as u16,
						'S' as u16,
						'3' as u16,
						'\r' as u16,
						'\n' as u16,
						'\0' as u16,
					])
				};
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			ConfigurationTable::MPS_TABLE => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['M' as u16, 'P' as u16, 'S' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			MemoryAttributesTable::GUID => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['U' as u16, 'M' as u16, 'A' as u16, 'T' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			SystemResourceTable::GUID => {
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['E' as u16, 'S' as u16, 'R' as u16, 'T' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
			guid => {
				let mut string = *b"U\0N\0K\0N\0O\0W\0N\0:\0 \0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0a\0\r\0\n\0\0\0";
				for (i, byte) in guid.data().to_ne_bytes().iter().rev().enumerate() {
					let hi = ((*byte) & 0xF0) >> 4;
					let lo = (*byte) & 0x0F;
					string[4 * i + 18] = lower_nibble_to_hex(hi);
					string[4 * i + 20] = lower_nibble_to_hex(lo);
				}
				let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(slice::from_raw_parts(&string as *const [u8] as *const u8 as *const u16, string.len() / 2)) };
				unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
			},
		}
	}

	let graphics_ptr = unsafe { (*(*graphics).mode).framebuffer_base }.to_ptr();
	let graphics_len = unsafe { (*(*graphics).mode).framebuffer_size } / size_of::<GraphicsPixel>();
	let pix_per_scan = unsafe { (*(*(*graphics).mode).info).pixels_per_scanline };
	let screen = unsafe { slice::from_raw_parts_mut(graphics_ptr, graphics_len) };

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

	let mask = PixelBitmask::new(0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000);
	let mut color = GraphicsOutputProtocol::grapics_color(0xFF00A5FF, &mask);
	// graphics.fill_pixel(&color, (50, 50), (100, 200))?;
	unsafe { ((*graphics).blt)(graphics, &mut color, GraphicsOutputBLTOperation::VIDEO_FILL, 0, 0, 50, 50, 100, 200, None) };
	let mut color2 = GraphicsOutputProtocol::grapics_color(0xFF0000FF, &mask);
	// graphics.fill_pixel(&color2, (60, 60), (80, 30))?;
	unsafe { ((*graphics).blt)(graphics, &mut color2, GraphicsOutputBLTOperation::VIDEO_FILL, 0, 0, 60, 60, 80, 30, None) };
	let white = GraphicsOutputProtocol::grapics_color(0xFFFFFFFF, &mask);
	bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'\0', 0, 0, 20, &white);

	let prompt = unsafe {
		uefi::CStr16::from_u16_with_nul_unchecked(&[
			'q' as u16,
			'=' as u16,
			'q' as u16,
			'u' as u16,
			'i' as u16,
			't' as u16,
			' ' as u16,
			'|' as u16,
			' ' as u16,
			'r' as u16,
			'=' as u16,
			'r' as u16,
			'e' as u16,
			'b' as u16,
			'o' as u16,
			'o' as u16,
			't' as u16,
			'\r' as u16,
			'\n' as u16,
			'\0' as u16,
		])
	};
	unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
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

	let (memory_map, map_key, memory_map_size) = {
		let (mut memory_map_size, memory_map, mut map_key, mut descriptor_size, mut descriptor_version) = (0, ptr::null_mut(), 0, 0, 0);
		let _x = unsafe { ((*(*system_table).boot_services).get_memory_map)(&mut memory_map_size, ptr::null_mut(), &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		let _y = unsafe { ((*(*system_table).boot_services).allocate_pool)(MemoryType::LOADER_DATA, memory_map_size, &mut (memory_map as *mut ())) };
		let _z = unsafe { ((*(*system_table).boot_services).get_memory_map)(&mut memory_map_size, memory_map, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		(memory_map, map_key, memory_map_size)
	};

	match unsafe { ((*(*system_table).boot_services).exit_boot_services)(image_handle, map_key) } {
		Status::SUCCESS => {
			bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'A', 5, 200, 20, &white);
			let start: fn(kernel::KernelData) -> ! = unsafe { mem::transmute(kernel_file_ptr.add(528)) };
			start(kernel::KernelData::new(graphics_ptr, graphics_len));
			// loop {}
		},
		_ => {
			let _x = unsafe { ((*(*system_table).boot_services).free_pages)(PhysicalAddress::new(kernel_file_ptr as u64), pages) };
			let _y = unsafe { ((*(*system_table).boot_services).free_pool)(memory_map as *mut ()) };
			unsafe { ((*kernel_file).close)(kernel_file) };
			Status::ABORTED
		},
	}
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
	let system_table = SYSTEM_TABLE.load(core::sync::atomic::Ordering::SeqCst);
	let message = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&['P' as u16, 'a' as u16, 'n' as u16, 'i' as u16, 'c' as u16, '\r' as u16, '\n' as u16, '\0' as u16]) };
	unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, message.as_ptr()) };
	unsafe { ((*(*system_table).runtime_services).reset_system)(ResetType::COLD, Status::SUCCESS, 0, ptr::null()) }
}
