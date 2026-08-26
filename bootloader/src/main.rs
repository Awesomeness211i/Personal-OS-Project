// #![warn(missing_docs)]
#![no_main]
#![no_std]
//! # BOOTLOADER
//! Starting executable file for the UEFI bootloader for my hobby OS project.

use core::{
	self,
	ffi::c_void,
	panic,
	ptr,
};

use acpi::{
	RootSystemDescriptionPointer,
	RootSystemDescriptionPointerEx,
};
use arch::x86_64::{
	PAGE_SIZE,
	paging::EntryFlags,
};
use boot_protocol_structures::{
	KernelData,
	KernelDataStruct,
	address_space::{
		AddressSpace,
		Page,
	},
	debug_print::{
		Port,
		println,
	},
};
use elf::elf::{
	Elf,
	Elf64Dynamic,
	Elf64RTypeX86_64,
	Elf64Rel,
	Elf64Rela,
	ExecutableType,
	ProgramHeaderFlags,
	ProgramHeaderType,
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
		HasGUID,
		file::{
			FileInfo,
			FileInfoHeader,
			FileProtocol,
			SimpleFileSystemProtocol,
		},
		graphics::{
			GraphicsOutputProtocol,
			GraphicsPixel,
			GraphicsPixelFormat,
		},
		image::LoadedImageProtocol,
	},
	services::AllocateType,
	status::Status,
	tables::{
		ConfigurationTable,
		SystemTable,
	},
};

fn set_position(file: *const FileProtocol, offset: u64) -> () {
	let status = unsafe { ((*file).set_position)(file, offset) };
	if status != Status::SUCCESS {
		panic!("Expected set_position to succeed but got {status}");
	}
}

fn read_file_offset(file: *const FileProtocol, buffer: *mut u8, size: usize, offset: u64) {
	set_position(file, offset);
	read_file(file, buffer, size)
}

fn read_file(file: *const FileProtocol, buffer: *mut u8, size: usize) {
	let mut pos = 0;
	let mut bytes_to_write = size;
	while bytes_to_write != 0 {
		let status = unsafe { ((*file).read)(file, &mut bytes_to_write, buffer.add(pos) as *mut _) };
		if status != Status::SUCCESS {
			panic!("More huh?! {status}");
		}
		pos += bytes_to_write;
		bytes_to_write = size - pos;
	}
}

const KERNEL_PAGE: MemoryType = MemoryType::custom_os(0x80000000);
const TRAMPOLINE_PAGE: MemoryType = MemoryType::custom_os(0x80000001);
const STACK_PAGE: MemoryType = MemoryType::custom_os(0x80000002);
const ARGUMENTS: MemoryType = MemoryType::custom_os(0x80000003);
const ELF_FILE: MemoryType = MemoryType::custom_os(0x80000004);
const ADDRESS_SPACE: MemoryType = MemoryType::custom_os(0x80000005);

// main entry address: 0x64805e0

// Interesting GUIDs:
// 00781CA1-5DE3-405F-ABB8-379C3C076984
// 1E2ED096-30E2-4254-BD89-863BBEF82325
// 4E28CA50-D582-44AC-A11F-E3D56526DB34
// C451ED2B-9694-45D3-BABA-ED9F8988A389

/// image: IN, system_table: IN
#[unsafe(export_name = "efi_main")]
pub unsafe extern "efiapi" fn main(image_handle: *mut c_void, system_table: SystemTablePointer) -> Status {
	unsafe {
		Port::COM1.init();
	}
	match system_table.header().signature {
		SystemTable::SIGNATURE => {},
		other => {
			panic!("The SystemTable signature was {other} but was expecting {}", SystemTable::SIGNATURE);
		},
	}
	unsafe { (system_table.console_out().reset)(system_table.console_out(), true) };
	unsafe { (system_table.std_err().reset)(system_table.std_err(), true) };
	unsafe { (system_table.console_in().reset)(system_table.console_in(), true) };

	let ptr = main as *const u8;
	println(format_args!("{ptr:#X?} {system_table:#X?}"));

	// Disable watchdog timer
	unsafe { (system_table.boot_services().set_watchdog_timer)(0, 0, 0, None) };

	let loaded_image = system_table
		.boot_services()
		.handle_protocol::<LoadedImageProtocol>(image_handle)
		.unwrap_or_else(|e| panic!("Failed to find LoadedImageProtocol: {e}"));

	let filesystem = system_table
		.boot_services()
		.handle_protocol::<SimpleFileSystemProtocol>(loaded_image.device_handle as *mut _)
		.unwrap_or_else(|e| panic!("Failed to find SimpleFileSystemProtocol: {e}"));

	let root_filesystem = {
		let mut file_protocol = ptr::null();
		let status = unsafe { (filesystem.open_volume)(filesystem, &mut file_protocol) };
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

	let kernel = {
		let mut info_size = 0;
		let status = unsafe { ((*kernel_file).get_info)(kernel_file, &FileInfo::GUID, &mut info_size, ptr::null_mut()) };
		if status != Status::BUFFER_TOO_SMALL {
			panic!("unexpected success?!")
		}

		let mut info_address = ptr::null_mut();
		let status = unsafe { (system_table.boot_services().allocate_pool)(MemoryType::LOADER_DATA, info_size, &mut info_address) };
		if status != Status::SUCCESS {
			panic!("Failed to allocate page with status: {status}")
		}

		let status = unsafe { ((*kernel_file).get_info)(kernel_file, &FileInfo::GUID, &mut info_size, info_address) };
		if status != Status::SUCCESS {
			panic!("Failed to get info with: {status}, {info_size}")
		}

		let info = unsafe { (info_address as *const FileInfoHeader).read() };
		let size = info.file_size as usize;

		let status = unsafe { (system_table.boot_services().free_pool)(info_address) };
		if status != Status::SUCCESS {
			panic!("Failed to free info with status: {info_address:#X?} {status}")
		}

		let mut address = PhysicalAddress::new(0);
		let status = unsafe { (system_table.boot_services().allocate_pages)(AllocateType::ANY_PAGES, ELF_FILE, size.div_ceil(PAGE_SIZE), &mut address) };
		if status != Status::SUCCESS {
			panic!("Failed to allocate page with status: {status}")
		}
		unsafe { address.to_ptr::<u8>().write_bytes(0, size) };

		read_file(kernel_file, address.to_ptr(), size);

		Elf::new(address.to_ptr(), size).unwrap_or_else(|e|
			panic!("Expected a supported and valid header but got error: {e}")
		)
	};

	let kernel_elf_header = kernel.header();

	let Some(kernel_entry) = kernel_elf_header.entry else {
		panic!("ELF has no entry point");
	};

	let base_address = if kernel_elf_header.executable_type == ExecutableType::DYNAMIC {
		// TODO: Actually figure out what base address I want or even potentially do randomization?
		0xFFFF_8000_0000_0000
		// 0x0000_7FFF_FFFF_FFFF
	} else {
		0x0
	};

	// Allocate trampoline page
	let trampoline = {
		let number = 1;
		let mut address = PhysicalAddress::new(0);
		let status = unsafe { (system_table.boot_services().allocate_pages)(AllocateType::ANY_PAGES, TRAMPOLINE_PAGE, number, &mut address) };
		if status != Status::SUCCESS {
			println(format_args!("Failed to allocate page with status: {status}"));
			return status;
		}
		unsafe { address.to_ptr::<Page>().write_bytes(0, number) };
		address
	};

	// Allocate kernel pages
	let kernel_page_count = {
		let mut kernel_pages_needed = 0;
		for program_header in kernel.program_headers() {
			if program_header.p_type == ProgramHeaderType::LOAD {
				let offset = program_header.p_vaddr & (PAGE_SIZE - 1);
				let segment_pages = (offset + program_header.p_memsz).div_ceil(PAGE_SIZE);
				kernel_pages_needed += segment_pages;
			}
		}

		if kernel_pages_needed == 0 {
			panic!("No segments or pages?!");
		}

		let mut address = PhysicalAddress::new(0);
		let status = unsafe { (system_table.boot_services().allocate_pages)(AllocateType::ANY_PAGES, KERNEL_PAGE, kernel_pages_needed, &mut address) };
		if status != Status::SUCCESS {
			panic!("Failed to allocate page with status: {status}")
		}

		let mut page_iter = 0;
		for program_header in kernel.program_headers() {
			if program_header.p_type == ProgramHeaderType::LOAD {
				let offset = program_header.p_vaddr & (PAGE_SIZE - 1);
				let segment_pages = (offset + program_header.p_memsz).div_ceil(PAGE_SIZE);

				if program_header.p_align > 0 && program_header.p_offset % program_header.p_align != program_header.p_vaddr % program_header.p_align {
					panic!("Align: {:?}", program_header.p_align);
				}

				if program_header.p_vaddr.wrapping_add(program_header.p_memsz) < program_header.p_vaddr {
					panic!("ELF address structure overflowed");
				}

				if page_iter > kernel_pages_needed {
					panic!("Trying to write out of bounds is bad");
				}

				if program_header.p_filesz > 0 {
					// unsafe {
					// 	address
					// 		.to_ptr::<u8>()
					// 		.add(offset + (page_iter << 12))
					// 		.copy_from(kernel.ptr::<u8>().add(program_header.p_offset + offset + (page_iter << 12)), program_header.p_filesz)
					// };
					read_file_offset(
						kernel_file,
						unsafe { address.to_ptr::<u8>().add(offset + (page_iter << 12)) },
						program_header.p_filesz,
						program_header.p_offset as u64,
					);
				}

				let zero_fill_count = program_header.p_memsz - program_header.p_filesz;
				if zero_fill_count > 0 {
					println(format_args!("Filling zeros: {zero_fill_count}"));
					unsafe { address.to_ptr::<u8>().add((page_iter << 12) + offset + program_header.p_filesz).write_bytes(0, zero_fill_count) };
				}
				page_iter += segment_pages;
			}
		}
		kernel_pages_needed
	};

	// Allocate kernel argument pages
	let (kernel_arguments_physical, kernel_arguments_page_count) = {
		let number = size_of::<KernelDataStruct>().div_ceil(PAGE_SIZE);
		let mut address = PhysicalAddress::new(0);
		let status = unsafe { (system_table.boot_services().allocate_pages)(AllocateType::ANY_PAGES, ARGUMENTS, number, &mut address) };
		if status != Status::SUCCESS {
			println(format_args!("Failed to allocate page with status: {status}"));
			return status;
		}
		unsafe { address.to_ptr::<Page>().write_bytes(0, number) };
		(address, number)
	};

	// Allocate stack pages
	let stack_page_count = {
		let number = 5;
		let mut address = PhysicalAddress::new(0);
		let status = unsafe { (system_table.boot_services().allocate_pages)(AllocateType::ANY_PAGES, STACK_PAGE, number, &mut address) };
		if status != Status::SUCCESS {
			panic!("Failed to allocate page with status: {status}")
		}
		unsafe { address.to_ptr::<Page>().write_bytes(0, number) };
		number
	};

	unsafe { ((*kernel_file).close)(kernel_file) };

	let graphics = {
		let mut interface_ptr = ptr::null();
		let status = unsafe { (system_table.boot_services().locate_protocol)(&GraphicsOutputProtocol::GUID, ptr::null_mut(), &mut interface_ptr) };
		if status != Status::SUCCESS {
			panic!("Failed to find GraphicsOutputProtocol: {status}");
		}
		interface_ptr as *const GraphicsOutputProtocol
	};

	let graphics_mode = unsafe { (*graphics).mode };
	let graphics_max_mode = unsafe { (*graphics_mode).max_mode };
	let mut query_mode = 0;
	let (mut mode, mut w, mut h, mut pix_per_scan, mut format) = (0, 0, 0, 0, GraphicsPixelFormat::RED_GREEN_BLUE_RESERVED_8BIT_PER_COLOR);

	while query_mode < graphics_max_mode {
		let mut size = 0;
		let mut ptr = ptr::null();
		let status = unsafe { ((*graphics).query_mode)(graphics, query_mode, &mut size, &mut ptr) };
		match status {
			Status::SUCCESS => println(format_args!("Successfully queried mode {mode}")),
			e => {
				println(format_args!("Expected Success but got {e} instead"));
				continue;
			},
		}
		let info = unsafe { *ptr };
		if (w <= info.horizontal_resolution && h <= info.vertical_resolution) && (info.horizontal_resolution <= 1920 && info.vertical_resolution <= 1080) {
			w = info.horizontal_resolution;
			h = info.vertical_resolution;
			format = info.pixel_format;
			pix_per_scan = info.pixels_per_scanline;
			mode = query_mode;
		}
		println(format_args!("{info:?}"));
		query_mode += 1;
	}

	let status = unsafe { ((*graphics).set_mode)(graphics, mode) };
	if status != Status::SUCCESS {
		panic!("Couldn't Set Mode {mode}");
	}

	let graphics_ptr = unsafe { (*graphics_mode).framebuffer_base }.to_ptr::<GraphicsPixel>();
	let graphics_len = w as usize * h as usize * size_of::<GraphicsPixel>();

	let (memory_map, map_key, memory_map_size, descriptor_size, descriptor_version) = {
		let (mut memory_map_size, mut memory_map, mut map_key, mut descriptor_size, mut descriptor_version) = (0, ptr::null_mut(), 0, 0, 0);
		let result = unsafe { (system_table.boot_services().get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		if result == Status::SUCCESS {
			panic!("Unexpected Success in allocating memory map");
		}
		memory_map_size += 2 * descriptor_size;
		let Status::SUCCESS = (unsafe { (system_table.boot_services().allocate_pool)(MemoryType::LOADER_DATA, memory_map_size, &mut memory_map) }) else {
			panic!("Failed to allocate buffer for memory map");
		};
		let status = unsafe { (system_table.boot_services().get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) };
		let Status::SUCCESS = status else {
			panic!("Failed to get memory map: {status}");
		};
		(memory_map, map_key, memory_map_size, descriptor_size, descriptor_version)
	};

	let system_table = system_table.exit_boot_services(image_handle, map_key)
		.unwrap_or_else(|_system_table| 
			panic!("Exit Boot Services Failed!: Memory Map Pointer: {memory_map:#X?}, Memory Map Key: {map_key}, Memory Map Size: {memory_map_size}, Memory Map Descriptor Size: {descriptor_size}, Memory Map Descriptor Version: {descriptor_version}")
		);

	println(format_args!("Exit Boot Services Succeeded!"));

	let config_tables = system_table.config_tables();
	let mut root_system_description_pointer = RootSystemDescriptionPointer::default();
	let mut root_system_description_pointer_ex = RootSystemDescriptionPointerEx::default();
	for table in config_tables {
		println(format_args!("{table:?}"));
		match table.vendor_guid {
			ConfigurationTable::EFI_ACPI_20_TABLE => {
				let acpi_table = table.vendor_table as *const acpi::RootSystemDescriptionPointerEx;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.rsdp.signature != *b"RSD PTR " {
					panic!("Not an ACPI20 table?")
				}
				root_system_description_pointer_ex = t;
			},
			ConfigurationTable::ACPI_10_TABLE => {
				let acpi_table = table.vendor_table as *const acpi::RootSystemDescriptionPointer;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.signature != *b"RSD PTR " {
					panic!("Not an ACPI10 table?")
				}
				root_system_description_pointer = t;
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

	let mut min_page_map_num = 0;
	for i in 0..memory_map_size / descriptor_size {
		let descriptor = unsafe { &mut *(memory_map.add(i * descriptor_size) as *mut MemoryDescriptor) };
		match descriptor.region_type {
			MemoryType::ACPI_MEMORY_NVS | MemoryType::PERSISTENT_MEMORY | MemoryType::CONVENTIONAL_MEMORY | MemoryType::LOADER_DATA | MemoryType::LOADER_CODE | MemoryType::PAL_CODE => {
				descriptor.virtual_start = VirtualAddress::new(descriptor.physical_start.get());
				min_page_map_num += descriptor.num_pages;
			},
			MemoryType::RUNTIME_SERVICES_CODE | MemoryType::RUNTIME_SERVICES_DATA => {
				descriptor.virtual_start = VirtualAddress::new(descriptor.physical_start.get());
				min_page_map_num += descriptor.num_pages;
			},
			KERNEL_PAGE => {
				descriptor.virtual_start = VirtualAddress::new(base_address);
				min_page_map_num += descriptor.num_pages;
			},
			ARGUMENTS => {
				descriptor.virtual_start = VirtualAddress::new(base_address + (kernel_page_count << 12) as u64);
				min_page_map_num += descriptor.num_pages;
			},
			STACK_PAGE => {
				descriptor.virtual_start = VirtualAddress::new(base_address + ((1 + kernel_page_count + kernel_arguments_page_count) << 12) as u64);
				min_page_map_num += descriptor.num_pages;
			},
			TRAMPOLINE_PAGE => {
				descriptor.virtual_start = VirtualAddress::new(descriptor.physical_start.get());
				min_page_map_num += descriptor.num_pages;
			},
			MemoryType::BOOT_SERVICES_CODE | MemoryType::BOOT_SERVICES_DATA | ELF_FILE => {
				descriptor.region_type = MemoryType::CONVENTIONAL_MEMORY;
				descriptor.virtual_start = VirtualAddress::new(descriptor.physical_start.get());
				min_page_map_num += descriptor.num_pages;
			},
			_ => continue,
		}
	}

	let mut address_space_physical = PhysicalAddress::new(0);
	let mut address_space_virtual = VirtualAddress::new(0);
	let mut address_space_page_num = 0;
	for i in 0..memory_map_size / descriptor_size {
		let descriptor = unsafe { &mut *(memory_map.add(i * descriptor_size) as *mut MemoryDescriptor) };
		match descriptor.region_type {
			MemoryType::CONVENTIONAL_MEMORY => {
				if descriptor.num_pages >= min_page_map_num.div_ceil(512) {
					let count = kernel_page_count + kernel_arguments_page_count + stack_page_count + 1;
					let virtual_ptr = VirtualAddress::new(base_address + (count << 12) as u64);
					println(format_args!("{base_address:X} {count}"));
					descriptor.region_type = ADDRESS_SPACE;
					descriptor.virtual_start = virtual_ptr;
					address_space_physical = descriptor.physical_start;
					address_space_virtual = virtual_ptr;
					address_space_page_num = descriptor.num_pages;
					break;
				}
			},
			_ => continue,
		}
	}

	if address_space_page_num == 0 {
		panic!("Not enough space for page tables?!");
	}

	let mut kernel_address_space = AddressSpace::create(address_space_physical, address_space_virtual, address_space_page_num as usize, 4);

	let mut stack = None;
	let mut kernel_arguments = None;
	for i in 0..memory_map_size / descriptor_size {
		let descriptor = unsafe { (memory_map.add(i * descriptor_size) as *const MemoryDescriptor).read() };
		println(format_args!("{descriptor:#X?}"));
		match descriptor.region_type {
			MemoryType::CONVENTIONAL_MEMORY | MemoryType::ACPI_MEMORY_NVS | MemoryType::PERSISTENT_MEMORY => {
				for j in 0..descriptor.num_pages {
					kernel_address_space.mmap(
						VirtualAddress::new(descriptor.virtual_start.get() + (j << 12)),
						PhysicalAddress::new(descriptor.physical_start.get() + (j << 12)),
						EntryFlags::PRESENT,
						EntryFlags::NONE,
					);
				}
			},
			MemoryType::RUNTIME_SERVICES_CODE | MemoryType::RUNTIME_SERVICES_DATA | TRAMPOLINE_PAGE => {
				for j in 0..descriptor.num_pages {
					kernel_address_space.mmap(
						VirtualAddress::new(descriptor.virtual_start.get() + (j << 12)),
						PhysicalAddress::new(descriptor.physical_start.get() + (j << 12)),
						EntryFlags::PRESENT,
						EntryFlags::PRESENT,
					);
				}
			},
			KERNEL_PAGE => {
				let mut used_kernel_pages = 0;
				for program_header in kernel.program_headers() {
					// println(format_args!("{program_header:#X?}"));
					match program_header.p_type {
						ProgramHeaderType::LOAD => {
							let offset = program_header.p_vaddr & (PAGE_SIZE - 1);
							let segment_pages = (offset + program_header.p_memsz).div_ceil(PAGE_SIZE);

							let elf_flags = program_header.p_flags;
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

							for j in 0..segment_pages {
								kernel_address_space.mmap(
									VirtualAddress::new(descriptor.virtual_start.get() + ((used_kernel_pages + j) << 12) as u64),
									PhysicalAddress::new(descriptor.physical_start.get() + ((used_kernel_pages + j) << 12) as u64),
									EntryFlags::PRESENT | EntryFlags::READ_WRITE,
									page_permissions,
								);
							}

							used_kernel_pages += segment_pages;
						},
						ProgramHeaderType::DYNAMIC => {
							let mut rela = 0;
							let mut rela_size = 0;
							let mut rela_entry_size = 0;
							// let mut rela_count = 0;

							let mut rel = 0;
							let mut rel_size = 0;
							let mut rel_entry_size = 0;
							// let mut rel_count = 0;

							let mut j = 0;
							loop {
								let dynamic_entry = unsafe { kernel.offset(program_header.p_offset + j * size_of::<Elf64Dynamic>()) };
								// println(format_args!("{dynamic_entry:#?}"));
								match *dynamic_entry {
									Elf64Dynamic::HASH { ptr: _ } => {},
									Elf64Dynamic::STRTAB { ptr: _ } => {},
									Elf64Dynamic::SYMTAB { ptr: _ } => {},
									Elf64Dynamic::RELA { ptr } => rela = ptr,
									Elf64Dynamic::RELASZ { val } => rela_size = val,
									Elf64Dynamic::RELAENT { val } => rela_entry_size = val,
									// Elf64Dynamic::GnuRelaCount { val } => rela_count = val,
									Elf64Dynamic::REL { ptr } => rel = ptr,
									Elf64Dynamic::RELSZ { val } => rel_size = val,
									Elf64Dynamic::RELENT { val } => rel_entry_size = val,
									// Elf64Dynamic::GnuRelCount { val } => rel_count = val,
									Elf64Dynamic::STRSZ { val: _ } => {},
									Elf64Dynamic::SYMENT { val: _ } => {},
									Elf64Dynamic::Null => break,
									_ => {},
								}
								j += 1;
							}

							if rela_entry_size > 0 {
								println(format_args!("actual: {rela_entry_size}, expected: {}", size_of::<Elf64Rela>()));
								assert!(rela_entry_size as usize >= size_of::<Elf64Rela>());
								for j in 0..rela_size / rela_entry_size {
									let rela_entry = unsafe { kernel.offset::<Elf64Rela>(rela as usize + j as usize * rela_entry_size as usize) };
									let rela_type = Elf64RTypeX86_64::new(rela_entry.r_info);

									let offset = rela_entry.r_offset;
									let addend = rela_entry.r_addend;
									match rela_type {
										// Base address + addend
										Elf64RTypeX86_64::R_AMD64_RELATIVE => {
											let relative = descriptor.virtual_start.get() + addend as u64;
											let off = descriptor.physical_start.get() + offset;
											println(format_args!("writing: {relative:X} to: {off:X}"));
											// TODO: Check that this is correct
											unsafe {
												(descriptor.physical_start.to_ptr::<u8>().add(offset as usize) as *mut u64).write(relative);
											}
										},
										t => println(format_args!("type: {:#X}", t.get())),
									}
								}
							}

							if rel_entry_size > 0 {
								println(format_args!("actual: {rela_entry_size}, expected: {}", size_of::<Elf64Rela>()));
								assert!(rel_entry_size as usize >= size_of::<Elf64Rel>());
								for j in 0..rel_size / rel_entry_size {
									let rel_entry = unsafe { kernel.offset::<Elf64Rel>((rel + j * rel_entry_size) as usize) };
									println(format_args!("{rel_entry:#?}"));
								}
							}
						},
						_ => continue,
					}
				}
			},
			STACK_PAGE => {
				stack = Some(VirtualAddress::new(descriptor.virtual_start.get() + ((stack_page_count) << 12) as u64));
				for j in 0..descriptor.num_pages {
					kernel_address_space.mmap(
						VirtualAddress::new(descriptor.virtual_start.get() + (j << 12)),
						PhysicalAddress::new(descriptor.physical_start.get() + (j << 12)),
						EntryFlags::PRESENT | EntryFlags::READ_WRITE,
						EntryFlags::PRESENT | EntryFlags::READ_WRITE,
					);
				}
			},
			ARGUMENTS => {
				kernel_arguments = Some(descriptor.virtual_start);
				for j in 0..descriptor.num_pages {
					kernel_address_space.mmap(
						VirtualAddress::new(descriptor.virtual_start.get() + (j << 12)),
						PhysicalAddress::new(descriptor.physical_start.get() + (j << 12)),
						EntryFlags::PRESENT | EntryFlags::READ_WRITE,
						EntryFlags::PRESENT,
					);
				}
			},
			ADDRESS_SPACE => {
				for j in 0..descriptor.num_pages {
					kernel_address_space.mmap(
						VirtualAddress::new(descriptor.virtual_start.get() + (j << 12)),
						PhysicalAddress::new(descriptor.physical_start.get() + (j << 12)),
						EntryFlags::PRESENT | EntryFlags::READ_WRITE,
						EntryFlags::PRESENT | EntryFlags::READ_WRITE | EntryFlags::WRITE_THROUGH | EntryFlags::PAGE_CACHE_DISABLE,
					);
				}
			},
			_ => continue,
		}
	}

	let Some(kernel_arguments) = kernel_arguments else {
		panic!("No Arguments?");
	};

	let Some(stack) = stack else {
		panic!("No Stack?");
	};

	let data = KernelData::V1 {
		size: size_of::<KernelDataStruct>(),
		memory_map,
		stack_page_count,
		trampoline_page: trampoline,
		address_space: kernel_address_space.clone(),
		system_table,
		root_system_description_pointer,
		root_system_description_pointer_ex,
	};
	unsafe { kernel_arguments_physical.to_ptr::<KernelData>().write(data) };

	let mut start: *mut u8;
	let mut end: *mut u8;
	unsafe {
		core::arch::asm!(
			"lea {start}, [2f]",
			"lea {end}, [3f]",
			"jmp 3f",
			"2:",
			"mov cr3, rdx",
			"xor rbx, rbx",
			"xor rcx, rcx",
			"xor rdx, rdx",
			"xor rbp, rbp",
			"xor r8, r8",
			"xor r9, r9",
			"xor r10, r10",
			"xor r11, r11",
			"xor r12, r12",
			"xor r13, r13",
			"xor r14, r14",
			"xor r15, r15",
			"jmp rsi",
			"4:",
			"jmp 4b",
			"3:",
			start = out(reg) start,
			end = out(reg) end,
			options(nostack, readonly),
		);

		let size = end as usize - start as usize + 1;
		start.copy_to(trampoline.to_ptr(), size);

		// should have arguments rdi, rsi, rdx, rcx, r8
		// let tramp: extern "C" fn(kernel_args: *const u8, kernel_entry: *const u8, kernel_address_space: *const u8, stack: *const u8) -> ! = core::mem::transmute(trampoline);
		// tramp(
		// 	kernel_arguments.to_ptr(),
		// 	kernel_entry.as_ptr().add(base_address as usize),
		// 	kernel_address_space.get_ptr() as *const u8,
		// 	stack.to_ptr(),
		// );

		core::arch::asm!(
			// creating my own calling convention for my trampoline page
			// specifically using the usual conventions of C in a specific way
			"mov rsp, rcx",
			"jmp r8",
			in("rdi") kernel_arguments.to_ptr::<u8>(),
			in("rsi") kernel_entry.as_ptr().add(base_address as usize),
			in("rdx") kernel_address_space.get_ptr::<u8>(),
			in("rcx") stack.to_ptr::<u8>(),
			in("r8") trampoline.to_ptr::<u8>(),
			options(noreturn, nostack),
		);
	}
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
	let message = info.message();
	if let Some(location) = info.location() {
		let file_name = location.file();
		let line_number = location.line();
		let column_number = location.column();
		println(format_args!(
			"Panicked at: {file_name}\n\tline number: {line_number}\n\tcolumn number: {column_number}\n\tmessage: {message}"
		));
	} else {
		println(format_args!("Panicked: {message}"));
	}
	loop {}
}
