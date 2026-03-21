#![feature(unwrap_infallible)]
// #![warn(missing_docs)]
#![no_main]
#![no_std]
//! # BOOTLOADER
//! Starting executable file for the UEFI bootloader for my hobby OS project.

use core::{
	self,
	ffi::c_void,
	ops::Index,
	panic,
	ptr,
	slice,
};

use arch::x86_64::paging::EntryFlags;
use bootloader::{
	PAGE_SIZE,
	SYSTEM_TABLE_POINTER,
	address_space::{
		AddressSpace,
		Page,
		PageIterator,
	},
	boot_info::{
		self,
	},
	elf::{
		Elf64ProgramHeader,
		Elf64SectionHeader,
		ElfHeader,
		ProgramHeaderFlags,
		ProgramHeaderType,
	},
	print::{
		Port,
		println,
	},
};
use uefi::{
	Char16,
	PhysicalAddress,
	SystemTablePointer,
	VirtualAddress,
	memory::{
		MemoryDescriptor,
		MemoryType,
	},
	protocols::{
		Protocol,
		file::{
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
		SystemTable,
	},
};

fn read_file<T: Default>(file: *const FileProtocol, offset: u64) -> T {
	let mut pos = 0;
	let mut header = T::default();
	let mut bytes_to_write = size_of::<T>();
	let status = unsafe { ((*file).set_position)(file, offset) };
	if status != Status::SUCCESS {
		panic!("Expected set_position to succeed but got {status}");
	}
	while bytes_to_write != 0 {
		let status = unsafe { ((*file).read)(file, &mut bytes_to_write, &mut header as *mut _ as *mut _) };
		if status != Status::SUCCESS {
			panic!("Huh?! {status}");
		}
		pos += bytes_to_write;
		bytes_to_write = size_of::<T>() - pos;
	}
	header
}

/// image: IN, system_table: IN
#[unsafe(export_name = "efi_main")]
pub extern "efiapi" fn main(image_handle: &mut c_void, system_table: SystemTablePointer) -> Status {
	unsafe {
		Port::COM1.init();
		SYSTEM_TABLE_POINTER = Some(system_table);
	}
	match system_table.header.signature {
		SystemTable::SIGNATURE => {},
		other => {
			panic!("The SystemTable signature was {other} but was expecting {}", SystemTable::SIGNATURE);
		},
	}
	unsafe { ((*system_table.console_out).reset)(system_table.console_out, true) };
	unsafe { ((*system_table.std_err).reset)(system_table.std_err, true) };
	unsafe { ((*system_table.console_in).reset)(system_table.console_in, true) };

	let loaded_image = {
		let mut interface_ptr = ptr::null();
		let status = unsafe { ((*system_table.boot_services).handle_protocol)(image_handle, &LoadedImageProtocol::GUID, &mut interface_ptr) };
		if status != Status::SUCCESS {
			panic!("Failed to find LoadedImageProtocol: {status}");
		}
		interface_ptr as *const LoadedImageProtocol
	};

	let filesystem = {
		let mut interface_ptr = ptr::null();
		let status = unsafe { ((*system_table.boot_services).handle_protocol)((*loaded_image).device_handle as *mut _, &SimpleFileSystemProtocol::GUID, &mut interface_ptr) };
		if status != Status::SUCCESS {
			panic!("Failed to find SimpleFileSystemProtocol: {status}")
		}
		interface_ptr as *const SimpleFileSystemProtocol
	};

	let root_filesystem = {
		let mut file_protocol = ptr::null();
		let status = unsafe { ((*filesystem).open_volume)(filesystem, &mut file_protocol) };
		if status != Status::SUCCESS {
			panic!("Failed to open volume: {status}");
		}
		file_protocol
	};

	let kernel_file = {
		let mut file_protocol = ptr::null();
		let path = "k\0e\0r\0n\0e\0l\0\\\0k\0e\0r\0n\0e\0l\0\0\0";
		let result = unsafe { ((*root_filesystem).open)(root_filesystem, &mut file_protocol, path.as_ptr() as *const Char16, FileProtocol::MODE_READ, 0) };
		if result != Status::SUCCESS {
			panic!("Failed to open file: {path}")
		}
		let status = unsafe { ((*root_filesystem).close)(root_filesystem) };
		if status != Status::SUCCESS {
			panic!("Failed to close filesystem: {status}");
		}
		file_protocol
	};

	let kernel_elf_header = read_file::<ElfHeader>(kernel_file, 0);
	if !kernel_elf_header.is_supported_and_valid() {
		panic!("Expected a supported and valid header but got {kernel_elf_header:#?}")
	}

	let Some(kernel_program_header_num) = kernel_elf_header.program_header_num else {
		panic!("Didn't find any program headers?!");
	};

	let Some(kernel_program_header_offset) = kernel_elf_header.program_header_offset else {
		panic!("Didn't find any program headers?!");
	};

	let Some(kernel_entry) = kernel_elf_header.entry else {
		panic!("ELF has no entry point");
	};

	let Some(kernel_section_header_num) = kernel_elf_header.section_header_num else {
		panic!("Didn't find any section headers");
	};

	let Some(kernel_section_header_offset) = kernel_elf_header.section_header_offset else {
		panic!("Didn't find any section headers");
	};

	// for i in 0..kernel_section_header_num.get() {
	// 	let section_header = read_file::<Elf64SectionHeader>(kernel_file, kernel_section_header_offset.get() as u64 + i as u64 * kernel_elf_header.section_header_entry_size as u64);
	// 	println(format_args!("{section_header:#?}"));
	// }

	let mut address_space = AddressSpace::create(4);

	let mut segments_loaded = 0;
	let mut pages_loaded = 0;
	for i in 0..kernel_program_header_num.get() {
		let program_header = read_file::<Elf64ProgramHeader>(kernel_file, kernel_program_header_offset.get() as u64 + i as u64 * kernel_elf_header.program_header_entry_size as u64);
		println(format_args!("{program_header:?}"));

		match program_header.p_type {
			ProgramHeaderType::LOAD => {
				let virtual_address = program_header.p_vaddr;
				let offset = virtual_address & 0xFFF;
				let file_offset = program_header.p_offset;
				let file_size = program_header.p_filesz;
				let mem_size = program_header.p_memsz;
				let align = program_header.p_align;
				let elf_flags = program_header.p_flags;
				let segment_pages = (offset + mem_size).div_ceil(PAGE_SIZE);

				if align > 0 && file_offset % align != virtual_address % align {
					panic!("Align: {align}");
				}

				let allocated_address = {
					let mut address = PhysicalAddress::new(0);
					let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, segment_pages, &mut address) };
					if status != Status::SUCCESS {
						println(format_args!("Failed to allocate page with status: {status}"));
						return status;
					}
					address
				};

				if program_header.p_vaddr.wrapping_add(mem_size) < program_header.p_vaddr {
					panic!("ELF address structure overflowed");
				}

				// unsafe { allocated_address.to_ptr::<u8>().write_bytes(0, segment_pages * 4096) };

				if file_size > 0 {
					let mut pos = 0;
					let mut bytes_to_write = file_size;
					let status = unsafe { ((*kernel_file).set_position)(kernel_file, file_offset as u64) };
					if status != Status::SUCCESS {
						panic!("Expected set_position to succeed but got {status}");
					}
					while bytes_to_write != 0 {
						let status = unsafe { ((*kernel_file).read)(kernel_file, &mut bytes_to_write, allocated_address.to_ptr::<u8>().add(offset + pos) as *mut _) };
						if status != Status::SUCCESS {
							panic!("More huh?! {status}");
						}
						pos += bytes_to_write;
						bytes_to_write = file_size - pos;
					}
				}

				let zero_fill_count = mem_size - file_size;
				if zero_fill_count > 0 {
					println(format_args!("Filling zeros: {zero_fill_count}"));
					unsafe { allocated_address.to_ptr::<u8>().add(offset + file_size).write_bytes(0, zero_fill_count) };
				}

				// Mapping page tables
				let page_permissions = {
					let w = if elf_flags.get() & ProgramHeaderFlags::W.get() != 0 {
						EntryFlags::READ_WRITE
					} else {
						EntryFlags::default()
					};
					let x = if elf_flags.get() & ProgramHeaderFlags::X.get() != 0 {
						EntryFlags::default()
					} else {
						EntryFlags::NOT_EXECUTABLE
					};
					EntryFlags::PRESENT | w | x
				};

				for i in 0..segment_pages {
					address_space.mmap(
						VirtualAddress::new((virtual_address + (i << 12)) as u64),
						PhysicalAddress::new(allocated_address.get() + (i << 12) as u64),
						page_permissions,
					);
				}

				println(format_args!("Segment_pages: {segment_pages}"));

				segments_loaded += 1;
				pages_loaded += segment_pages;
			},
			_ => continue,
		}
	}
	unsafe { ((*kernel_file).close)(kernel_file) };

	println(format_args!("Segments Loaded: {segments_loaded}, Pages Loaded: {pages_loaded}"));
	if segments_loaded == 0 {
		panic!("No segments?!");
	}

	let trampoline_page = {
		let mut address = PhysicalAddress::new(0);
		let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, 1, &mut address) };
		if status != Status::SUCCESS {
			println(format_args!("Failed to allocate page with status: {status}"));
			return status;
		}
		unsafe { address.to_ptr::<u8>().write_bytes(0, 4096) };
		address
	};

	address_space.mmap(VirtualAddress::new(trampoline_page.get()), trampoline_page, EntryFlags::PRESENT);

	let stack_page = {
		let mut address = PhysicalAddress::new(0);
		let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, 1, &mut address) };
		if status != Status::SUCCESS {
			println(format_args!("Failed to allocate page with status: {status}"));
			return status;
		}
		unsafe { address.to_ptr::<u8>().write_bytes(0, 4096) };
		address
	};

	address_space.mmap(VirtualAddress::new(0xFFFFFFFFFFFFFFFF), trampoline_page, EntryFlags::READ_WRITE | EntryFlags::PRESENT);

	// let entry = kernel_entry.as_ptr();
	// println(format_args!("Entry: {entry:#X?}"));

	// Interesting GUIDs:
	// 00781CA1-5DE3-405F-ABB8-379C3C076984
	// 1E2ED096-30E2-4254-BD89-863BBEF82325
	// 4E28CA50-D582-44AC-A11F-E3D56526DB34
	// C451ED2B-9694-45D3-BABA-ED9F8988A389
	let config_tables = unsafe { core::slice::from_raw_parts(system_table.configuration_tables, system_table.num_table_entries) };
	let mut root_system_description_pointer = None;
	let mut root_system_description_pointer_ex = None;
	for table in config_tables {
		println(format_args!("{table:?}"));
		match table.vendor_guid {
			ConfigurationTable::EFI_ACPI_20_TABLE => {
				let acpi_table = table.vendor_table as *const acpi::RootSystemDescriptionPointerEx;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.rsdp.signature != *b"RSD PTR " {
					println(format_args!("Not an ACPI20 table?"));
					return Status::ABORTED;
				}
				root_system_description_pointer_ex = Some(unsafe { (table.vendor_table as *const acpi::RootSystemDescriptionPointerEx).as_ref_unchecked() });
			},
			ConfigurationTable::ACPI_10_TABLE => {
				let acpi_table = table.vendor_table as *const acpi::RootSystemDescriptionPointer;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.signature != *b"RSD PTR " {
					println(format_args!("Not an ACPI10 table?"));
					return Status::ABORTED;
				}
				root_system_description_pointer = Some(unsafe { (table.vendor_table as *const acpi::RootSystemDescriptionPointer).as_ref_unchecked() });
			},
			ConfigurationTable::SMBIOS_TABLE => {
				// let smbios = table.vendortable as *const SMBIOSTable_64;
			},
			ConfigurationTable::SMBIOS3_TABLE => {
				// let smbios = table.vendortable as *const SMBIOSTable_64;
			},
			_ => continue,
		}
	}

	let graphics = {
		let mut interface_ptr = ptr::null();
		let status = unsafe { ((*system_table.boot_services).locate_protocol)(&GraphicsOutputProtocol::GUID, ptr::null_mut(), &mut interface_ptr) };
		if status != Status::SUCCESS {
			panic!("Failed to find GraphicsOutputProtocol: {status}");
		}
		interface_ptr as *const GraphicsOutputProtocol
	};

	let graphics_mode = unsafe { (*graphics).mode };
	// let graphics_max_mode = unsafe { (*graphics_mode).max_mode };
	// let mut query_mode = 0;
	// let (mut mode, mut w, mut h, mut format) = (0, 0, 0, GraphicsPixelFormat::RED_GREEN_BLUE_RESERVED_8BIT_PER_COLOR);
	// while query_mode < graphics_max_mode {
	// 	let mut size = 0;
	// 	let mut ptr = ptr::null();
	// 	let status = unsafe { ((*graphics).query_mode)(graphics, query_mode, &mut size, &mut ptr) };
	// 	match status {
	// 		Status::SUCCESS => println(format_args!("Successfully queried mode {mode}")),
	// 		e => {
	// 			println(format_args!("Expected Success but got {e} instead"));
	// 			continue;
	// 		},
	// 	}
	// 	let info = unsafe { *ptr };
	// 	if w < info.horizontal_resolution {
	// 		w = info.horizontal_resolution;
	// 		h = info.vertical_resolution;
	// 		format = info.pixel_format;
	// 		mode = query_mode;
	// 	}
	// 	println(format_args!("{info:?}"));
	// 	query_mode += 1;
	// }
	// let status = unsafe { ((*graphics).set_mode)(graphics, mode) };
	// if status != Status::SUCCESS {
	// 	panic!("Couldn't Set Mode {mode}");
	// }

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

	println(format_args!("q = quit | r = reboot | c = continue | p = panic"));

	let mut keyinput = text::InputKey::default();
	let events = [unsafe { (*system_table.console_in).wait_for_key }];
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

	let (memory_map, map_key, memory_map_size, descriptor_size, descriptor_version) = {
		let (mut memory_map_size, mut memory_map, mut map_key, mut descriptor_size, mut descriptor_version) = (0, ptr::null_mut(), 0, 0, 0);
		let result = unsafe { ((*system_table.boot_services).get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		if result == Status::SUCCESS {
			panic!("Unexpected Success in allocating memory map");
		}
		memory_map_size += 2 * descriptor_size;
		let Status::SUCCESS = (unsafe { ((*system_table.boot_services).allocate_pool)(MemoryType::LOADER_DATA, memory_map_size, &mut memory_map) }) else {
			panic!("Failed to allocate buffer for memory map");
		};
		let status = unsafe { ((*system_table.boot_services).get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		let Status::SUCCESS = status else {
			panic!("Failed to get memory map: {status}");
		};
		(memory_map, map_key, memory_map_size, descriptor_size, descriptor_version)
	};

	// for i in 0..memory_map_size / descriptor_size {
	// 	let descriptor = unsafe { (memory_map.add(i * descriptor_size) as *const MemoryDescriptor).as_ref_unchecked() };
	// 	println(format_args!("{descriptor:#X?}"));
	// }
	// println(format_args!("descriptor size: {descriptor_size}, type size: {}", size_of::<MemoryDescriptor>()));

	match unsafe { ((*system_table.boot_services).exit_boot_services)(image_handle, map_key) } {
		Status::SUCCESS => {
			println(format_args!("Exit Boot Services Succeeded!"));
			// let header = boot_info::KernelDataHeader {
			// 	graphics_len,
			// 	graphics_ptr,
			// 	root_system_description_pointer,
			// 	root_system_description_pointer_ex,
			// 	system_table,
			// 	virtual_mappings_count: pages_loaded,
			// };
			// let start: extern "C" fn(data: &boot_info::KernelDataHeader) -> ! = unsafe { core::mem::transmute(kernel_entry.as_ptr()) };
			// start(&header);
			// let start: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_entry.as_ptr()) };
			// start();
			// unsafe {
			// 	core::arch::asm!("mov cr3, {}", in(reg) address_space.ptr);
			// }
			let mut start = ptr::null::<u8>();
			let mut end = ptr::null::<u8>();
			let mut size = 0usize;

			unsafe {
				core::arch::asm!(
					"lea {start}, [2f]",
					"lea {end}, [3f]",
					"jmp 3f",
					"2:",
					"mov cr3, {address_space}",
					"jmp {kernel_entry}",
					"4:",
					"jmp 4b",
					"3:",
					"mov {size}, {end}",
					"sub {size}, {start}",
					"mov rsi, {start}",
					"mov rdi, {trampoline}",
					"mov rcx, {size}",
					"rep movsb",
					"xor rsp, rsp",
					"jmp {trampoline}",
					start = in(reg) start,
					end = in(reg) end,
					size = in(reg) size,
					// stack = in(reg) stack_addr,
					trampoline = in(reg) trampoline_page.to_ptr::<u8>(),
					address_space = in(reg) address_space.get_ptr(),
					kernel_entry = in(reg) kernel_entry.as_ptr(),
					options(noreturn)
				)
			}
		},
		Status::INVALID_PARAMETER => {
			println(format_args!("Exit Boot Services Failed!"));
			let num_descriptors = memory_map_size / descriptor_size;
			for i in 0..num_descriptors {
				let descriptor = unsafe { &*((memory_map as *mut u8).add(i * descriptor_size) as *mut MemoryDescriptor) };
				println(format_args!("{descriptor:?}"));
			}

			println(format_args!("Memory Map Pointer: {memory_map:X?}"));
			println(format_args!("Memory Map Key: {map_key}"));
			println(format_args!("Memory Map Size: {memory_map_size}"));
			println(format_args!("Memory Map Descriptor Size: {descriptor_size}"));
			println(format_args!("Memory Map Descriptor Version: {descriptor_version}"));
			Status::ABORTED
		},
		_ => unreachable!(),
	}
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
	let message = info.message();
	let location = info.location().unwrap();
	let file_name = location.file();
	let line_number = location.line();
	let column_number = location.column();
	println(format_args!(
		"Panicked at: {file_name}\n\tline number: {line_number}\n\tcolumn number: {column_number}\n\tmessage: {message}"
	));
	if let Some(ptr) = unsafe { SYSTEM_TABLE_POINTER } {
		unsafe { ((*ptr.runtime_services).reset_system)(ResetType::SHUTDOWN, Status::SUCCESS, 0, ptr::null()) }
	} else {
		loop {}
	}
}
