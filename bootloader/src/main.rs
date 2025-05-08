#![feature(lang_items, core_intrinsics)]
#![allow(internal_features)]
#![no_main]
#![no_std]

use uefi::protocols::{
	Protocol,
	// path::DevicePathProtocol,
	file::{
		SimpleFileSystemProtocol,
		FileProtocol,
		FileInfo,
	},
	image::LoadedImageProtocol,
	graphics::{
		GraphicsOutputBLTOperation,
		GraphicsOutputProtocol,
		GraphicsPixel,
		PixelBitmask,
	}
};

use bootloader::elf::{
	ElfHeader,
	ExecutableType,
	ElfProgramHeader,
	ElfSectionHeader,
};

use core::{
	mem::offset_of,
};

/// image: IN, system_table: IN
#[unsafe(no_mangle)]
extern "efiapi" fn efi_main(image_handle: *mut (), system_table: *mut uefi::tables::SystemTable) -> uefi::status::Status {
	unsafe { ((*(*system_table).console_out).reset)((*system_table).console_out, true) };
	unsafe { ((*(*system_table).std_err).reset)((*system_table).std_err, true) };
	unsafe { ((*(*system_table).console_in).reset)((*system_table).console_in, true) };
	let graphics = {
		let mut interface_ptr = core::ptr::null();
		unsafe { ((*(*system_table).boot_services).locate_protocol)(&GraphicsOutputProtocol::GUID, core::ptr::null_mut(), &mut interface_ptr) };
		interface_ptr as *const GraphicsOutputProtocol
	};
	let loaded_image = {
		let mut interface_ptr = core::ptr::null();
		unsafe { ((*(*system_table).boot_services).handle_protocol)(image_handle, &LoadedImageProtocol::GUID, &mut interface_ptr) };
		interface_ptr as *const LoadedImageProtocol
	};
	// let device_path = {
	// 	let mut interface_ptr = core::ptr::null();
	// 	unsafe { ((*(*system_table).boot_services).handle_protocol)((*loaded_image).device_handle as *mut (), &DevicePathProtocol::GUID, &mut interface_ptr) };
	// 	interface_ptr as *const uefi::protocols::path::DevicePathProtocol
	// };
	let filesystem = {
		let mut interface_ptr = core::ptr::null();
		unsafe { ((*(*system_table).boot_services).handle_protocol)((*loaded_image).device_handle as *mut (), &SimpleFileSystemProtocol::GUID, &mut interface_ptr) };
		interface_ptr as *const SimpleFileSystemProtocol
	};
	let root_filesystem = {
		let mut file_protocol = core::ptr::null();
		unsafe { ((*filesystem).open_volume)(filesystem, &mut file_protocol) };
		file_protocol
	};
	let filename = unsafe {
		uefi::CStr16::from_u16_with_nul_unchecked(&['k' as u16, 'e' as u16, 'r' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\\' as u16, 'k' as u16, 'e' as u16, 'r' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\0' as u16 ])
	};
	let kernel_file = {
		let mut file_protocol = core::ptr::null();
		let _result = unsafe { ((*root_filesystem).open)(root_filesystem, &mut file_protocol, filename.as_ptr(), FileProtocol::MODE_READ, 0) };
		let _status = unsafe { ((*root_filesystem).close)(root_filesystem) };
		file_protocol
	};
	let file_size = {
		let mut size = 0;
		let mut buffer = core::ptr::null_mut();
		unsafe { ((*kernel_file).get_info)(kernel_file, &FileInfo::GUID, &mut size, core::ptr::null_mut()) };
		unsafe { ((*(*system_table).boot_services).allocate_pool)(uefi::memory::MemoryType::LOADER_DATA, size, &mut buffer) };
		unsafe { ((*kernel_file).get_info)(kernel_file, &FileInfo::GUID, &mut size, buffer) };
		let file_size = unsafe { *(buffer.byte_offset(offset_of!(FileInfo, file_size) as isize) as *const u64) };
		unsafe { ((*(*system_table).boot_services).free_pool)(buffer) };
		file_size
	};
	let kernel_file_ptr = {
		let mut file = core::ptr::null_mut();
		let mut file_size = file_size as usize;
		unsafe { ((*(*system_table).boot_services).allocate_pool)(uefi::memory::MemoryType::LOADER_DATA, file_size as usize, &mut file) };
		unsafe { ((*kernel_file).read)(kernel_file, &mut file_size, file) };
		file as *mut u8
	};

	let graphics_ptr = unsafe { (*(*graphics).mode).framebuffer_base }.to_ptr();
	let graphics_len = unsafe { (*(*graphics).mode).framebuffer_size } / size_of::<GraphicsPixel>();
	let pix_per_scan = unsafe { (*(*(*graphics).mode).info).pixels_per_scanline };
	let screen = unsafe { core::slice::from_raw_parts_mut(graphics_ptr, graphics_len) };

	let elf_header_ptr = unsafe { &*(kernel_file_ptr as *const ElfHeader) };
	if &elf_header_ptr.identifier[0..4] != b"\x7FELF" /* magic */ {
		return uefi::status::Status::ABORTED;
	}
	if elf_header_ptr.identifier[4] != 2 /* 64 bit */ {
		return uefi::status::Status::ABORTED;
	}
	if elf_header_ptr.identifier[5] != 1 /* little endian */ {
		return uefi::status::Status::ABORTED;
	}
	if elf_header_ptr.identifier[6] != 1 /* elf version */ {
		return uefi::status::Status::ABORTED;
	}
	if elf_header_ptr.identifier[7] != 0 /* system v */ {
		return uefi::status::Status::ABORTED;
	}
	if elf_header_ptr.identifier[8] != 0 /* abi version */ {
		return uefi::status::Status::ABORTED;
	}
	if &elf_header_ptr.identifier[9..] != b"\x00\x00\x00\x00\x00\x00\x00" /* padding */ {
		return uefi::status::Status::ABORTED;
	}
	if elf_header_ptr.executable_type != ExecutableType::DYNAMIC {
		return uefi::status::Status::ABORTED;
	}

	let ph_table = unsafe { core::slice::from_raw_parts(kernel_file_ptr.add(elf_header_ptr.phoff) as *const ElfProgramHeader, elf_header_ptr.phnum as usize) };
	for ph in ph_table {
		if ph.p_type != 1 /* PT_LOAD */ {
			continue;
		}
	}

	let mask = PixelBitmask::new(0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000);
	let mut color = GraphicsOutputProtocol::grapics_color(0xFF00A5FF, &mask);
	// graphics.fill_pixel(&color, (50, 50), (100, 200))?;
	unsafe { ((*graphics).blt)(graphics, &mut color, GraphicsOutputBLTOperation::VIDEO_FILL, 0, 0, 50, 50, 100, 200, None) };
	let mut color2 = GraphicsOutputProtocol::grapics_color(0xFF0000FF, &mask);
	// graphics.fill_pixel(&color2, (60, 60), (80, 30))?;
	unsafe { ((*graphics).blt)(graphics, &mut color2, GraphicsOutputBLTOperation::VIDEO_FILL, 0, 0, 60, 60, 80, 30, None) };

	let prompt = unsafe { uefi::CStr16::from_u16_with_nul_unchecked(&[ 'q' as u16, '=' as u16, 'q' as u16, 'u' as u16, 'i' as u16, 't' as u16, ' ' as u16, '|' as u16, ' ' as u16, 'r' as u16, '=' as u16, 'r' as u16, 'e' as u16, 'b' as u16, 'o' as u16, 'o' as u16, 't' as u16, '\r' as u16, '\n' as u16, '\0' as u16 ]) };
	unsafe { ((*(*system_table).console_out).output_string)((*system_table).console_out, prompt.as_ptr()) };
	let white = GraphicsOutputProtocol::grapics_color(0xFFFFFFFF, &mask);
	bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'\0', 0, 0, 20, &white);
	let mut key = uefi::protocols::text::InputKey::default();
	let events = [
		unsafe { (*(*system_table).console_in).wait_for_key },
	];
	loop {
		let index = unsafe { (*(*system_table).boot_services).wait_for_event(&events) }.unwrap();
		#[allow(clippy::single_match)]
		match index {
			0 => {
				// system_table.stdin().read_keystroke_into(&mut key)?;
				unsafe { ((*(*system_table).console_in).read_keystroke)((*system_table).console_in, &mut key) };
				match key.unicodechar.try_into().unwrap() {
					'q' | 'Q' => unsafe { ((*(*system_table).runtime_services).reset_system)(uefi::services::ResetType::SHUTDOWN, uefi::status::Status::SUCCESS, 0, core::ptr::null()) },
					'r' | 'R' => unsafe { ((*(*system_table).runtime_services).reset_system)(uefi::services::ResetType::COLD, uefi::status::Status::SUCCESS, 0, core::ptr::null()) },
					'c' | 'C' => break,
					'p' | 'P' => panic!(),
					_ => continue,
				}
			},
			_ => {},
		}
	}

	let (memory_map, map_key, memory_map_size) = {
		let (mut memory_map_size, memory_map, mut map_key, mut descriptor_size, mut descriptor_version) = (0, core::ptr::null_mut(), 0, 0, 0);
		let _x = unsafe { ((*(*system_table).boot_services).get_memory_map)(&mut memory_map_size, core::ptr::null_mut(), &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		let _y = unsafe { ((*(*system_table).boot_services).allocate_pool)(uefi::memory::MemoryType::LOADER_DATA, memory_map_size, &mut (memory_map as *mut ())) };
		let _z = unsafe { ((*(*system_table).boot_services).get_memory_map)(&mut memory_map_size, memory_map, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		(memory_map, map_key, memory_map_size)
	};

	match unsafe { ((*(*system_table).boot_services).exit_boot_services)(image_handle, map_key) } {
		uefi::status::Status::SUCCESS => {
			bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'A', 5, 200, 20, &white);
			let start: fn(kernel::KernelData) -> ! = unsafe { core::mem::transmute(kernel_file_ptr.add(528)) };
			start(kernel::KernelData::new(graphics_ptr, graphics_len));
			// loop {}
		},
		_ => {
			let _x = unsafe { ((*(*system_table).boot_services).free_pool)(kernel_file_ptr as *mut ()) };
			let _y = unsafe { ((*(*system_table).boot_services).free_pool)(memory_map as *mut ()) };
			unsafe { ((*kernel_file).close)(kernel_file) };
			uefi::status::Status::ABORTED
		},
	}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	// let system_table = unsafe { uefi::Environment::system_table() };
	// let message = unsafe {
	// 	uefi::CStr16::from_u16_with_nul_unchecked(&[ 'P' as u16, 'a' as u16, 'n' as u16, 'i' as u16, 'c' as u16, '\r' as u16, '\n' as u16, '\0' as u16 ])
	// };
	// system_table.stdout().output_string(message).unwrap();
	// system_table.boot_services().stall(1000000).unwrap();
	// system_table.runtime_services().reset_system(uefi::enums::ResetType::SHUTDOWN, uefi::status::Status::SUCCESS, None)
	loop {}
}
