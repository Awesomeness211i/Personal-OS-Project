#![feature(lang_items, core_intrinsics, unwrap_infallible)]
#![allow(internal_features)]
// #![warn(missing_docs)]
#![no_main]
#![no_std]
//! # BOOTLOADER
//! Starting executable file for the UEFI bootloader for my hobby OS project.

use core::{
	self,
	ffi::c_void,
	mem::offset_of,
	panic,
	ptr,
	slice,
};

use arch::x86_64::paging::{
	PageGlobalDirectory,
	PageGlobalDirectoryEntry,
	PageMiddleDirectory,
	PageMiddleDirectoryEntry,
	PageTable,
	PageTableEntry,
	PageUpperDirectory,
	PageUpperDirectoryEntry,
	PagingElement,
	PagingStructure,
};
use bootloader::{
	boot_info::{
		self,
	},
	elf::{
		Elf,
		Elf64ProgramHeader,
		ProgramHeaderType,
	},
};
use uefi::{
	Char16,
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

const PAGE_SIZE: usize = 4096;

/// image: IN, system_table: IN
#[unsafe(no_mangle)]
extern "efiapi" fn efi_main(image_handle: &mut c_void, system_table: SystemTablePointer) -> Status {
	match system_table.header.signature {
		SystemTable::SIGNATURE => {},
		_ => return Status::INVALID_PARAMETER,
	}
	unsafe { SYSTEM_TABLE = Some(system_table) };

	unsafe { ((*system_table.console_out).reset)(system_table.console_out, true) };
	unsafe { ((*system_table.std_err).reset)(system_table.std_err, true) };
	unsafe { ((*system_table.console_in).reset)(system_table.console_in, true) };

	let graphics = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*system_table.boot_services).locate_protocol)(&GraphicsOutputProtocol::GUID, ptr::null_mut(), &mut interface_ptr) };
		interface_ptr as *const GraphicsOutputProtocol
	};

	let loaded_image = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*system_table.boot_services).handle_protocol)(image_handle, &LoadedImageProtocol::GUID, &mut interface_ptr) };
		interface_ptr as *const LoadedImageProtocol
	};

	let filesystem = {
		let mut interface_ptr = ptr::null();
		unsafe { ((*system_table.boot_services).handle_protocol)((*loaded_image).device_handle as *mut _, &SimpleFileSystemProtocol::GUID, &mut interface_ptr) };
		interface_ptr as *const SimpleFileSystemProtocol
	};

	let root_filesystem = {
		let mut file_protocol = ptr::null();
		unsafe { ((*filesystem).open_volume)(filesystem, &mut file_protocol) };
		file_protocol
	};

	let kernel_file = {
		let mut file_protocol = ptr::null();
		let path = "k\0e\0r\0n\0e\0l\0\\\0k\0e\0r\0n\0e\0l\0\0\0";
		let result = unsafe { ((*root_filesystem).open)(root_filesystem, &mut file_protocol, path.as_ptr() as *const Char16, FileProtocol::MODE_READ, 0) };
		if result != Status::SUCCESS {
			print_string("Failed to open file: ");
			unsafe { ((*system_table.console_out).output_string)(system_table.console_out, path.as_ptr() as *const Char16) };
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
		file.to_ptr()
	};
	let elf = Elf::new(kernel_file_ptr, file_size)
		.map_err(|e| {
			match e {
				bootloader::elf::ELFError::NotELF => {
					print_string("Pointer isn't an ELF: ");
					println_bytes(&kernel_file_ptr);
				},
				bootloader::elf::ELFError::UnsupportedOSABI(abi) => {
					print_string("Unsupported OS ABI: ");
					println_bytes(&abi);
				},
				_ => println_string("No"),
			}
			unsafe { ((*system_table.runtime_services).reset_system)(ResetType::SHUTDOWN, Status::SUCCESS, 0, ptr::null()) }
		})
		.into_ok();

	let Some(program_header_offset) = elf.header().program_header_offset else {
		println_string("ELF has no program headers");
		return Status::ABORTED;
	};

	let Some(program_header_num) = elf.header().program_header_num else {
		println_string("ELF has no program headers");
		return Status::ABORTED;
	};

	let Some(kernel_entry) = elf.header().entry else {
		println_string("ELF has no entry point");
		return Status::ABORTED;
	};

	print_string("Program Header Offset: ");
	println_bytes(&program_header_offset);
	print_string("ELF Entry Point: ");
	println_bytes(&kernel_entry);

	let rflags_flags = [
		"CF,",
		"RESERVED,",
		"PF,",
		"RESERVED,",
		"AF,",
		"RESERVED,",
		"ZF,",
		"SF,",
		"TF,",
		"IF,",
		"DF,",
		"OF,",
		"IOPL,",
		"IOPL,",
		"NT,",
		"RESERVED,",
		"RF,",
		"VM,",
		"AC,", // bit 18
		"VIF,",
		"VIP,",
		"ID,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
	];
	let rflags: u64;
	unsafe { core::arch::asm!("pushfq","pop {}", out(reg) rflags) };

	print_string("RFLAGS: ");
	for (i, string) in rflags_flags.iter().enumerate() {
		if rflags & (1 << i) > 0 {
			print_string(string);
		}
	}
	println_string("");

	let efer_features = [
		"SCE,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"LME,", // bit 8
		"RESERVED,",
		"LMA,",
		"NXE,", // bit 11
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
	];
	let efer = unsafe { arch::x86_64::rdmsr(0xC0000080) };
	print_string("EFER: ");
	for (i, string) in efer_features.iter().enumerate() {
		if efer & (1 << i) > 0 {
			print_string(string);
		}
	}
	println_string("");

	// Task Priority Register
	// lowest 4 bits for 1-15 task priority where 0 enables and 15 disables all external interrupts
	// let cr8: u64;
	// unsafe { core::arch::asm!("mov {}, cr8", out(reg) cr8) };
	// print_string("CR8: ");
	// println_bytes(&cr8);

	// Model specific extensions register
	// Can use CPUID to query support for feature except for performance counter extensions
	let model_specific_features = [
		"VME,",
		"PVI,",
		"TSD,",
		"DE,",
		"PSE,", // bit 4
		"PAE,", // bit 5
		"MCE,",
		"PGE,", // bit 7
		"PCE,",
		"OSFXSR,",     // bit 9
		"OSXMMEXCPT,", // bit 10
		"UMIP,",
		"LA57,", // bit 12
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"FSGSBASE,",
		"PCIDE,", // bit 17
		"OSXSAVE,",
		"RESERVED,",
		"SMEP,", // bit 20
		"SMAP,", // bit 21
		"PKE,",  // bit 22
		"CET,",  // bit 23
		"PKS,",  // bit 24
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
	];
	let cr4: u64;
	unsafe { core::arch::asm!("mov {}, cr4", out(reg) cr4) };
	print_string("CR4: ");
	for (i, string) in model_specific_features.iter().enumerate() {
		if cr4 & (1 << i) > 0 {
			print_string(string);
		}
	}
	println_string("");

	let cr2: u64;
	unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2) };
	print_string("CR2: ");
	println_bytes(&cr2);

	let control_flag_strings = [
		"PE,",
		"MP,",
		"EM,",
		"TS,",
		"ET,",
		"NE,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"WP,", // bit 16
		"Reserved,",
		"AM,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"NW,",
		"CD,",
		"PG,", // bit 31
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
	];
	let cr0: u64;
	unsafe { core::arch::asm!("mov {}, cr0", out(reg) cr0) };
	print_string("CR0: ");
	for (i, string) in control_flag_strings.iter().enumerate() {
		if cr0 & (1 << i) > 0 {
			print_string(string);
		}
	}
	println_string("");

	let page_table = unsafe { &mut *PageGlobalDirectory::get_mut() };
	print_string("CR3: ");
	println_bytes(&page_table);

	#[repr(C)]
	struct Data {
		pgd: PageGlobalDirectory,
		pud: PageUpperDirectory,
		pmd: PageMiddleDirectory,
		pt: PageTable,
	}
	let page_table_allocation = {
		let mut page_table_allocation = PhysicalAddress::new(0);
		let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, 4, &mut page_table_allocation) };
		if status != Status::SUCCESS {
			println_string("Failed to allocate page");
			match status {
				Status::OUT_OF_RESOURCES => println_string("OUT OF RESOURCES"),
				Status::INVALID_PARAMETER => println_string("INVALID PARAMETER"),
				Status::NOT_FOUND => println_string("NOT FOUND"),
				_ => unreachable!(),
			}
			return status;
		}
		print_string("page table allocation address: ");
		println_bytes(&page_table_allocation);
		unsafe {
			page_table_allocation.to_ptr::<u8>().write_bytes(0, size_of::<Data>());
			&mut *page_table_allocation.to_ptr::<Data>()
		}
	};

	for (i, entry) in page_table.entries().iter().enumerate() {
		unsafe {
			page_table_allocation.pgd.get_entries()[i] = *entry;
		}
	}

	let high_half = 0xFFFF_8000_0000_0000;

	let mut segments_loaded = 0;
	for i in 0..program_header_num.get() {
		let program_header = unsafe { &*(kernel_file_ptr.add(program_header_offset.get() + (i as usize * elf.header().program_header_entry_size as usize)) as *const Elf64ProgramHeader) };
		print_string("Program header: ");

		match program_header.p_type {
			ProgramHeaderType::NULL => println_string("NULL"),
			ProgramHeaderType::LOAD => {
				println_string("LOAD");
				let virtual_address = high_half | program_header.p_vaddr;
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
					return status;
				}

				print_string("Number of Pages: ");
				println_bytes(&segment_pages);
				print_string("Virtual Address: ");
				println_bytes(&virtual_address);
				print_string("Physical Address: ");
				println_bytes(&mem_addr);

				let index_mask = 0x1FF;

				let mut new = virtual_address >> 12;
				// let offset = virtual_address & 0xFFF;

				let page_table_index = new & index_mask;
				new >>= 9;
				let page_middle_directory_index = new & index_mask;
				new >>= 9;
				let page_upper_directory_index = new & index_mask;
				new >>= 9;
				let page_global_directory_index = new & index_mask;

				print_string("\tPage Global Directory Index: ");
				println_bytes(&page_global_directory_index);
				print_string("\tPage Upper Directory Index: ");
				println_bytes(&page_upper_directory_index);
				print_string("\tPage Middle Directory Index: ");
				println_bytes(&page_middle_directory_index);
				print_string("\tPage Table Index: ");
				println_bytes(&page_table_index);

				let pgd_table = unsafe { ptr::read(page_table) };
				let pgd_entry = pgd_table.entries()[page_global_directory_index];
				if !pgd_entry.exists() {
					println_string("Writing Entries");
					unsafe {
						page_table_allocation.pt.get_entries()[page_table_index] = PageTableEntry::new(mem_addr.get() | PageTableEntry::PRESENT);
						page_table_allocation.pmd.get_entries()[page_middle_directory_index] =
							PageMiddleDirectoryEntry::new((&page_table_allocation.pt as *const _ as u64) | PageMiddleDirectoryEntry::PRESENT);
						page_table_allocation.pud.get_entries()[page_upper_directory_index] =
							PageUpperDirectoryEntry::new((&page_table_allocation.pmd as *const _ as u64) | PageUpperDirectoryEntry::PRESENT);
						page_table_allocation.pgd.get_entries()[page_global_directory_index] =
							PageGlobalDirectoryEntry::new((&page_table_allocation.pud as *const _ as u64) | PageGlobalDirectoryEntry::READ_WRITE | PageGlobalDirectoryEntry::PRESENT);
					}
				} else {
					return Status::UNSUPPORTED;
				}

				if program_header.p_vaddr.wrapping_add(mem_size) < program_header.p_vaddr {
					println_string("ELF address structure overflowed");
					return Status::ABORTED;
				}

				if file_size > 0 {
					unsafe { kernel_file_ptr.add(file_offset).copy_to(mem_addr.to_ptr(), file_size) };
				}

				let zero_fill_count = mem_size - file_size;
				if zero_fill_count > 0 {
					unsafe { mem_addr.to_ptr::<u8>().add(file_size).write_bytes(0, zero_fill_count) };
				}

				segments_loaded += 1;
			},
			ProgramHeaderType::DYNAMIC => println_string("DYNAMIC"),
			ProgramHeaderType::INTERP => println_string("INTERP"),
			ProgramHeaderType::NOTE => println_string("NOTE"),
			ProgramHeaderType::SECTION_HEADER_LIB => println_string("SHLIB"),
			ProgramHeaderType::PROGRAM_HEADER => println_string("PROGRAM HEADER"),
			program_header_type => println_bytes(&program_header_type),
		}
	}
	if segments_loaded == 0 {
		return Status::NOT_FOUND;
	}

	unsafe {
		core::arch::asm!("mov cr3, {}", in(reg) page_table_allocation);
		drop(page_table_allocation);
		drop(page_table);
	}

	let x = unsafe { *((kernel_entry.as_ptr() as u64 | high_half as u64) as *const u8) };
	println_bytes(&x);

	// Interesting GUIDs:
	// 00781CA1-5DE3-405F-ABB8-379C3C076984
	// 1E2ED096-30E2-4254-BD89-863BBEF82325
	// 4E28CA50-D582-44AC-A11F-E3D56526DB34
	// C451ED2B-9694-45D3-BABA-ED9F8988A389
	let config_tables = unsafe { core::slice::from_raw_parts(system_table.configuration_tables, system_table.num_table_entries) };
	let mut root_system_description_pointer = None;
	let mut root_system_description_pointer_ex = None;
	for table in config_tables {
		match table.vendor_guid {
			ConfigurationTable::LZMA_FILESYSTEM => println_string("LZMA FILESYSTEM"),
			ConfigurationTable::DXE_SERVICES => println_string("DXE SERVICES"),
			ConfigurationTable::HANDOFF_BLOCK_LIST => println_string("HANDOFF BLOCK LIST"),
			ConfigurationTable::MEMORY_TYPE_INFO_TABLE => println_string("MEMORY TYPE INFO TABLE"),
			DebugImageInfoTable::GUID => println_string("DEBUG IMAGE INFO TABLE"),
			ConfigurationTable::MEMORY_STATUS_CODE_RECORD => println_string("MEMORY STATUS CODE RECORD"),
			ConfigurationTable::EFI_ACPI_20_TABLE => {
				let acpi_table = table.vendor_table as *const acpi::RootSystemDescriptionPointerEx;
				print_string("ACPI 2 Table Pointer: ");
				println_bytes(&acpi_table);
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.rsdp.signature != *b"RSD PTR " {
					println_string("Not an ACPI20 table?");
					return Status::ABORTED;
				}
				println_string("ACPI20");
				// root_system_description_pointer_ex = Some(unsafe { &*(table.vendortable as *const acpi::RootSystemDescriptionPointerEx) });
			},
			ConfigurationTable::ACPI_10_TABLE => {
				let acpi_table = table.vendor_table as *const acpi::RootSystemDescriptionPointer;
				print_string("ACPI 1 Table Pointer: ");
				println_bytes(&acpi_table);
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.signature != *b"RSD PTR " {
					println_string("Not an ACPI10 table?");
					return Status::ABORTED;
				}
				println_string("ACPI10");
				// root_system_description_pointer = Some(unsafe { &*(table.vendortable as *const acpi::RootSystemDescriptionPointer) });
			},
			ConfigurationTable::SAL_SYSTEM_TABLE => println_string("SAL SYSTEM TABLE"),
			ConfigurationTable::SMBIOS_TABLE => {
				// let smbios = table.vendortable as *const SMBIOSTable_64;
				println_string("SMBIOS TABLE");
				// print_bytes(&smbios);
				// print_string(": ");
				// println_bytes(unsafe { &*smbios });
			},
			ConfigurationTable::SMBIOS3_TABLE => {
				// let smbios = table.vendortable as *const SMBIOSTable_64;
				println_string("SMBIOS3 TABLE");
				// print_bytes(&smbios);
				// print_string(": ");
				// println_bytes(unsafe { &*smbios });
			},
			ConfigurationTable::MPS_TABLE => println_string("MPS TABLE"),
			MemoryAttributesTable::GUID => println_string("UEFI MEMORY ATTRIBUTES TABLE"),
			SystemResourceTable::GUID => println_string("EFI SYSTEM RESOURCE TABLE"),
			guid => {
				print_string("UNKNOWN: ");
				println_bytes(&guid);
				print_string("\tptr: ");
				println_bytes(&table.vendor_table);
			},
		}
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
			print_string("UNEXPECTED SUCCESS IN ALLOCATING MEMORY MAP");
			return Status::ABORTED;
		}
		let Status::SUCCESS = (unsafe { ((*system_table.boot_services).allocate_pool)(MemoryType::LOADER_DATA, memory_map_size, &mut memory_map) }) else {
			print_string("FAILED TO ALLOCATE BUFFER FOR MEMORY MAP");
			return Status::ABORTED;
		};
		let Status::SUCCESS =
			(unsafe { ((*system_table.boot_services).get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) })
		else {
			println_string("FAILED TO GET MEMORY MAP");
			return Status::ABORTED;
		};
		(memory_map, map_key, memory_map_size, descriptor_size, descriptor_version)
	};

	println_string("Exiting Boot Services");
	match unsafe { ((*system_table.boot_services).exit_boot_services)(image_handle, map_key) } {
		Status::SUCCESS => {
			// bootloader::font::drawcharacter(screen, pix_per_scan as usize, b'A', 5, 200, 20, &white);
			let start: fn(data: boot_info::KernelDataHeader) -> ! = unsafe { core::mem::transmute(kernel_entry.as_ptr() as u64 | 0xFFFF_8000_0000_0000) };
			let header = boot_info::KernelDataHeader {
				graphics_len,
				graphics_ptr,
				root_system_description_pointer,
				root_system_description_pointer_ex,
				system_table,
				virtual_mappings_count: segments_loaded,
			};
			start(header);
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
				// print_string("Physical Start: ");
				// println_bytes(&unsafe { (*descriptor).physical_start });
				// print_string("Virtual Start: ");
				// println_bytes(&unsafe { (*descriptor).virtual_start });
				// print_string("Number of Pages: ");
				// println_bytes(&unsafe { (*descriptor).num_pages });
				// print_string("Attributes: ");
				// println_bytes(&unsafe { (*descriptor).attribute });
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
			Status::ABORTED
		},
		_ => unreachable!(),
	}

	// let max = core::arch::x86_64::__cpuid(0x80000000);
	// if max.eax < 0x80000008 {
	// 	return Status::INVALID_PARAMETER;
	// }
	// print_string("MAX CPUID: ");
	// println_bytes(&max.eax);

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
			print_string("Panicked at: ");
			println_string(file_name);
			print_string("\tline number: ");
			println_bytes(&line_number);
			print_string("\tcolumn number: ");
			println_bytes(&column_number);
			println_string(message);
			unsafe { ((*system_table.runtime_services).reset_system)(ResetType::SHUTDOWN, Status::SUCCESS, 0, ptr::null()) }
		},
		None => loop {},
	}
}
