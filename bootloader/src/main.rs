#![feature(unwrap_infallible, custom_test_frameworks)]
// #![test_runner(crate::test_runner)]
// #![warn(missing_docs)]
#![no_main]
#![no_std]
//! # BOOTLOADER
//! Starting executable file for the UEFI bootloader for my hobby OS project.

use core::{
	self,
	ffi::c_void,
	fmt::{
		Error,
		Write,
	},
	panic,
	ptr,
	slice,
};

use arch::x86_64::paging::{
	PageGlobalDirectory,
	PageGlobalDirectoryEntry,
	PageMiddleDirectoryEntry,
	PageTableEntry,
	PageUpperDirectoryEntry,
	PagingElement,
	PagingStructure,
};
use bootloader::{
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

fn println(args: core::fmt::Arguments<'_>) {
	let mut port = Port::COM1;
	let _ = writeln!(port, "{args}");
}

fn print(args: core::fmt::Arguments<'_>) {
	let mut port = Port::COM1;
	let _ = write!(port, "{args}");
}

const PAGE_SIZE: usize = 4096;

#[repr(transparent)]
struct Port(u16);

impl core::fmt::Write for Port {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		let str = s.as_bytes();
		let len = str.len();
		let result = unsafe { self.out_bytes(str) };
		if result == len { Ok(()) } else { Err(Error) }
	}
}

impl Port {
	pub const COM1: Self = Self(0x3F8);
	pub const COM2: Self = Self(0x2F8);
	pub const COM3: Self = Self(0x3E8);
	pub const COM4: Self = Self(0x2E8);
	pub const COM5: Self = Self(0x5F8);
	pub const COM6: Self = Self(0x4F8);
	pub const COM7: Self = Self(0x5E8);
	pub const COM8: Self = Self(0x4E8);

	const RX_TX_BUFFER: u16 = 0;
	const INT_ENABLE: u16 = 1;
	const FIFO_CTRL: u16 = 2;
	const LINE_CTRL: u16 = 3;
	const MODEM_CTRL: u16 = 4;
	const LINE_STATUS: u16 = 5;
	const MODEM_STATUS: u16 = 6;
	const SCRATCH: u16 = 7;

	pub unsafe fn init(&self) {
		unsafe {
			Self(self.0 + Self::INT_ENABLE).out(0x00);
			Self(self.0 + Self::LINE_CTRL).out(0x80);
			Self(self.0 + Self::RX_TX_BUFFER).out(0x03);

			Self(self.0 + Self::INT_ENABLE).out(0x00);
			Self(self.0 + Self::LINE_CTRL).out(0x03);
			Self(self.0 + Self::FIFO_CTRL).out(0x07);
			Self(self.0 + Self::MODEM_CTRL).out(0x1F);

			Self(self.0 + Self::RX_TX_BUFFER).out(0xAA);

			if Self::inb(self) != 0xAA {
				println(format_args!("Error: port not setup correctly"));
			}

			Self(self.0 + Self::MODEM_CTRL).out(0x0F);
		}
	}

	#[inline(always)]
	unsafe fn out(&self, value: u8) {
		unsafe {
			core::arch::asm!(
				"out dx, al",
				in("dx") self.0,
				in("al") value,
			)
		}
	}

	#[inline(always)]
	unsafe fn inb(&self) -> u8 {
		let value: u8;
		unsafe {
			core::arch::asm!(
				"in al, dx",
				in("dx") self.0,
				out("al") value,
			)
		}
		value
	}

	#[inline(always)]
	unsafe fn out_bytes(&self, bytes: &[u8]) -> usize {
		let unwritten_bytes: usize;
		unsafe {
			core::arch::asm!(
				"rep outsb",
				inout("rcx") bytes.len() => unwritten_bytes,
				in("dx") self.0,
				inout("rsi") bytes.as_ptr() => _,
			);
		}
		bytes.len() - unwritten_bytes
	}
}

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

static mut SYSTEM_TABLE_POINTER: Option<SystemTablePointer> = None;

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

	let address_space = AddressSpace::create(4);
	let _ = address_space;
	// println(format_args!("{address_space:#?}"));
	// address_space.mmap(, , );

	// Model specific extensions register
	// Can use CPUID to query support for feature except for performance counter extensions
	// let model_specific_features = [
	// 	"VME,",
	// 	"PVI,",
	// 	"TSD,",
	// 	"DE,",
	// 	"PSE,", // bit 4
	// 	"PAE,", // bit 5
	// 	"MCE,",
	// 	"PGE,", // bit 7
	// 	"PCE,",
	// 	"OSFXSR,",     // bit 9
	// 	"OSXMMEXCPT,", // bit 10
	// 	"UMIP,",
	// 	"LA57,", // bit 12
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"FSGSBASE,",
	// 	"PCIDE,", // bit 17
	// 	"OSXSAVE,",
	// 	"RESERVED,",
	// 	"SMEP,", // bit 20
	// 	"SMAP,", // bit 21
	// 	"PKE,",  // bit 22
	// 	"CET,",  // bit 23
	// 	"PKS,",  // bit 24
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
	// print(format_args!("CR4: "));
	// for (i, string) in model_specific_features.iter().enumerate() {
	// 	if cr4 & (1 << i) > 0 {
	// 		print(format_args!("{string}"));
	// 	}
	// }
	// println(format_args!(""));

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

	let page_global_directory_table = {
		let mut page_table_allocation = PhysicalAddress::new(0);
		let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, 1, &mut page_table_allocation) };
		if status != Status::SUCCESS {
			panic!("Failed to allocate page table allocation: {status}")
		}
		let page_global_directory_table = PageGlobalDirectory::get();
		println(format_args!("CR3: {:X?}", page_global_directory_table as *const PageGlobalDirectory));
		let ptr = unsafe {
			page_table_allocation.to_ptr::<u8>().write_bytes(0, PAGE_SIZE);
			page_table_allocation.to_ptr::<PageGlobalDirectory>().as_mut_unchecked()
		};
		for (i, entry) in page_global_directory_table.entries().iter().enumerate() {
			unsafe { ptr.get_entries()[i] = entry.clone() };
		}
		ptr
	};

	let high_half_base = 0xFFFF_8000_0000_0000;
	let index_mask = 0x1FF;

	let mut segments_loaded = 0;
	let mut pages_loaded = 0;
	for i in 0..kernel_program_header_num.get() {
		let program_header = read_file::<Elf64ProgramHeader>(kernel_file, kernel_program_header_offset.get() as u64 + i as u64 * kernel_elf_header.program_header_entry_size as u64);
		println(format_args!("{program_header:?}"));

		match program_header.p_type {
			ProgramHeaderType::LOAD => {
				let virtual_address = high_half_base | program_header.p_vaddr;
				let offset = program_header.p_vaddr & 0xFFF;
				let file_offset = program_header.p_offset;
				let file_size = program_header.p_filesz;
				let mem_size = program_header.p_memsz;
				let align = program_header.p_align;
				let elf_flags = program_header.p_flags;
				let permissive = PageGlobalDirectoryEntry::PRESENT | PageGlobalDirectoryEntry::READ_WRITE | PageGlobalDirectoryEntry::USER_ACCESSIBLE;
				let segment_pages = (offset + mem_size).div_ceil(PAGE_SIZE);

				if align > 0 && file_offset % align != virtual_address % align {
					panic!("Align: {align}");
				}

				let mut new = virtual_address >> 12;
				let page_table_index = new & index_mask;
				new >>= 9;
				let page_middle_directory_index = new & index_mask;
				new >>= 9;
				let page_upper_directory_index = new & index_mask;
				new >>= 9;
				let page_global_directory_index = new & index_mask;

				let allocated_address = {
					let mut address = PhysicalAddress::new(0);
					let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, segment_pages, &mut address) };
					if status != Status::SUCCESS {
						println(format_args!("Failed to allocate page with status: {status}"));
						return status;
					}
					address
				};
				println(format_args!("Mapping physical address: {allocated_address:#X} to virtual address: {virtual_address:#X}"));

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
				let mut page_global_directory_entry = if !page_global_directory_table.entries()[page_global_directory_index].exists() {
					let mut page_table_allocation = PhysicalAddress::new(0);
					let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, 1, &mut page_table_allocation) };
					if status != Status::SUCCESS {
						panic!("Failed to allocate page table allocation: {status}")
					}
					println(format_args!("Page Table Allocation Address: {page_table_allocation:#X?}"));
					unsafe {
						page_table_allocation.to_ptr::<u8>().write_bytes(0, PAGE_SIZE);
						page_global_directory_table.get_entries()[page_global_directory_index] = PageGlobalDirectoryEntry::new((page_table_allocation.to_ptr::<u8>() as u64) | permissive);
					}
					page_global_directory_table.entries()[page_global_directory_index].clone()
				} else {
					page_global_directory_table.entries()[page_global_directory_index].clone()
				};
				let page_upper_directory = unsafe { page_global_directory_entry.to_addr().get_entries() };

				let mut page_upper_directory_entry = if !page_upper_directory[page_upper_directory_index].exists() {
					let mut page_table_allocation = PhysicalAddress::new(0);
					let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, 1, &mut page_table_allocation) };
					if status != Status::SUCCESS {
						panic!("Failed to allocate page table allocation: {status}")
					}
					println(format_args!("Page Table Allocation Address: {page_table_allocation:#X?}"));
					unsafe {
						page_table_allocation.to_ptr::<u8>().write_bytes(0, PAGE_SIZE);
						page_upper_directory[page_upper_directory_index] = PageUpperDirectoryEntry::new((page_table_allocation.to_ptr::<u8>() as u64) | permissive);
					}
					page_upper_directory[page_upper_directory_index].clone()
				} else {
					page_upper_directory[page_upper_directory_index].clone()
				};
				let page_middle_directory = unsafe { page_upper_directory_entry.to_addr().get_entries() };

				let mut page_middle_directory_entry = if !page_middle_directory[page_middle_directory_index].exists() {
					let mut page_table_allocation = PhysicalAddress::new(0);
					let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, 1, &mut page_table_allocation) };
					if status != Status::SUCCESS {
						panic!("Failed to allocate page table allocation: {status}")
					}
					println(format_args!("Page Table Allocation Address: {page_table_allocation:#X?}"));
					unsafe {
						page_table_allocation.to_ptr::<u8>().write_bytes(0, PAGE_SIZE);
						page_middle_directory[page_middle_directory_index] = PageMiddleDirectoryEntry::new((page_table_allocation.to_ptr::<u8>() as u64) | permissive);
					}
					page_middle_directory[page_middle_directory_index].clone()
				} else {
					page_middle_directory[page_middle_directory_index].clone()
				};
				let page_tables = unsafe { page_middle_directory_entry.to_addr().get_entries() };

				let page_permissions = {
					let w = if elf_flags.get() & ProgramHeaderFlags::W.get() != 0 {
						PageGlobalDirectoryEntry::READ_WRITE
					} else {
						0
					};
					let x = if elf_flags.get() & ProgramHeaderFlags::X.get() != 0 {
						0
					} else {
						PageGlobalDirectoryEntry::NOT_EXECUTABLE
					};
					let result = PageGlobalDirectoryEntry::PRESENT | w | x;
					println(format_args!("Page Permissions: {result:#X}"));
					result
				};

				for i in 0..segment_pages {
					if !page_tables[page_table_index + i].exists() {
						page_tables[page_table_index + i] = unsafe { PageTableEntry::new((allocated_address.get() + i as u64 * 0x1000) | page_permissions) };
					} else {
						panic!("Tried to allocate {} index with original value {:#X?}", page_table_index + i, page_tables[page_table_index])
					}
				}
				println(format_args!("Segment_pages: {segment_pages}"));

				segments_loaded += 1;
				pages_loaded += segment_pages;
			},
			_ => continue,
		}
	}
	unsafe { ((*kernel_file).close)(kernel_file) };

	unsafe {
		core::arch::asm!("mov cr3, {}", in(reg) page_global_directory_table);
	}

	println(format_args!("Segments Loaded: {segments_loaded}"));
	if segments_loaded == 0 {
		panic!("No segments?!");
	}

	let entry = kernel_entry.as_ptr() as u64 | high_half_base as u64;
	println(format_args!("Entry: {entry:#X}"));

	// println(format_args!("Kernel Space:"));
	// for i in 0..pages_loaded * 4096 {
	// 	if i % 4096 == 0 {
	// 		println(format_args!(""));
	// 	}
	// 	let x = unsafe { *(high_half_base as *const u8).add(i) };
	// 	print(format_args!("{x:02X} "));
	// }
	// println(format_args!(""));

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

	for i in 0..memory_map_size / descriptor_size {
		let descriptor = unsafe { (memory_map.add(i * descriptor_size) as *const MemoryDescriptor).as_ref_unchecked() };
		println(format_args!("{descriptor:#X?}"));
	}
	println(format_args!("descriptor size: {descriptor_size}, type size: {}", size_of::<MemoryDescriptor>()));

	match unsafe { ((*system_table.boot_services).exit_boot_services)(image_handle, map_key) } {
		Status::SUCCESS => {
			println(format_args!("Exit Boot Services Succeeded!"));
			let header = boot_info::KernelDataHeader {
				graphics_len,
				graphics_ptr,
				root_system_description_pointer,
				root_system_description_pointer_ex,
				system_table,
				virtual_mappings_count: pages_loaded,
			};
			// let start: extern "C" fn(data: &boot_info::KernelDataHeader) -> ! = unsafe { core::mem::transmute(kernel_entry.as_ptr() as u64 | high_half_base as u64) };
			// start(&header);
			let start: extern "C" fn() -> ! = unsafe { core::mem::transmute(kernel_entry.as_ptr() as u64 | high_half_base as u64) };
			start();
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

	// let max = core::arch::x86_64::__cpuid(0x80000000);
	// if max.eax < 0x80000008 {
	// 	return Status::INVALID_PARAMETER;
	// }
	// println(format_args!("MAX CPUID: {}", max.eax));

	// let vendor_info = unsafe { core::mem::transmute::<core::arch::x86_64::CpuidResult, [u8; size_of::<core::arch::x86_64::CpuidResult>()]>(core::arch::x86_64::__cpuid(0x00000000)) };
	// print_string("Vendor Info: ");
	// println_string_bytes(unsafe { core::slice::from_raw_parts(vendor_info.as_ptr().add(size_of::<u32>()), 3 * size_of::<u32>()) });

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
	// print(format_args!("Flags: "));
	// for (i, string) in flag_strings.iter().enumerate() {
	// 	if test.edx & (1 << i) > 0 {
	// 		print(format_args!("{string}"));
	// 	}
	// }
	// println(format_args!(""));

	// let tlb_info = core::arch::x86_64::__cpuid(0x00000002);
	// println(format_args!("TLB Info: {tlb_info:?}"));

	// let cache_info = core::arch::x86_64::__cpuid(0x00000002);
	// println(format_args!("Cache Info: {cache_info:?}"));

	// let addr_info = core::arch::x86_64::__cpuid(0x80000008);
	// println(format_args!("Address Space Info: {addr_info:?}"));

	// let rflags_flags = [
	// 	"CF,",
	// 	"RESERVED,",
	// 	"PF,",
	// 	"RESERVED,",
	// 	"AF,",
	// 	"RESERVED,",
	// 	"ZF,",
	// 	"SF,",
	// 	"TF,",
	// 	"IF,",
	// 	"DF,",
	// 	"OF,",
	// 	"IOPL,",
	// 	"IOPL,",
	// 	"NT,",
	// 	"RESERVED,",
	// 	"RF,",
	// 	"VM,",
	// 	"AC,", // bit 18
	// 	"VIF,",
	// 	"VIP,",
	// 	"ID,",
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
	// let rflags: u64;
	// unsafe { core::arch::asm!("pushfq","pop {}", out(reg) rflags) };
	// print(format_args!("RFLAGS: "));
	// for (i, string) in rflags_flags.iter().enumerate() {
	// 	if rflags & (1 << i) > 0 {
	// 		print(format_args!("{string}"));
	// 	}
	// }
	// println(format_args!(""));

	// let efer_features = [
	// 	"SCE,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"RESERVED,",
	// 	"LME,", // bit 8
	// 	"RESERVED,",
	// 	"LMA,",
	// 	"NXE,", // bit 11
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
	// print(format_args!("EFER: "));
	// for (i, string) in efer_features.iter().enumerate() {
	// 	if efer & (1 << i) > 0 {
	// 		print(format_args!("{string}"));
	// 	}
	// }
	// println(format_args!(""));

	// Task Priority Register
	// lowest 4 bits for 1-15 task priority where 0 enables and 15 disables all external interrupts
	// let cr8: u64;
	// unsafe { core::arch::asm!("mov {}, cr8", out(reg) cr8) };
	// println(format_args!("CR8: {cr8:X}"));

	// let cr2: u64;
	// unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2) };
	// println(format_args!("CR2: {cr2:X}"));

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
	// 	"WP,", // bit 16
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
	// 	"PG,", // bit 31
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
	// print(format_args!("CR0: "));
	// for (i, string) in control_flag_strings.iter().enumerate() {
	// 	if cr0 & (1 << i) > 0 {
	// 		print(format_args!("{string}"));
	// 	}
	// }
	// println(format_args!(""));
}

#[repr(C, align(4096))]
#[derive(Debug)]
struct Page {
	data: [u8; 4096],
}

#[derive(Debug)]
struct AddressSpace {
	ptr: *mut Page,
	levels: u8,
}

impl AddressSpace {
	fn create(levels: u8) -> Self {
		if let Some(ptr) = unsafe { SYSTEM_TABLE_POINTER } {
			let num_pages = 1;
			let mut paddr = PhysicalAddress::new(0);
			let status = unsafe { ((*ptr.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, num_pages, &mut paddr) };
			if status != Status::SUCCESS {
				panic!("Failed to allocate address space with error: {status}")
			} else {
				println(format_args!("Allocated address space at physical address {paddr:#X}"));
			}
			unsafe {
				paddr.to_ptr::<Page>().write_bytes(0, num_pages);
			}
			Self { ptr: paddr.to_ptr(), levels }
		} else {
			panic!("You didn't initialize the SYSTEM_TABLE_POINTER")
		}
	}

	fn mmap(&mut self, vaddr: VirtualAddress, paddr: PhysicalAddress, flags: u64) {}
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
	let message = info.message();
	let location = info.location().unwrap();
	let file_name = location.file();
	let line_number = location.line();
	let column_number = location.column();
	let format = format_args!("Panicked at: {file_name}\n\tline number: {line_number}\n\tcolumn number: {column_number}\n\tmessage: {message}");
	println(format);
	if let Some(ptr) = unsafe { SYSTEM_TABLE_POINTER } {
		unsafe { ((*ptr.runtime_services).reset_system)(ResetType::SHUTDOWN, Status::SUCCESS, 0, ptr::null()) }
	} else {
		loop {}
	}
}
