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
	fmt::{
		Error,
		Write,
	},
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
		SystemTable,
	},
};

fn println(args: core::fmt::Arguments<'_>) {
	let mut port = Port::COM1;
	writeln!(port, "{args}");
}

fn print(args: core::fmt::Arguments<'_>) {
	let mut port = Port::COM1;
	write!(port, "{args}");
}

const PAGE_SIZE: usize = 4096;

#[repr(transparent)]
struct Port(u16);

impl core::fmt::Write for Port {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		let str = s.as_bytes();
		let len = str.len();
		let result = unsafe { self.out_bytes(str) };
		if result == len { Ok(()) } else { Err(Error::default()) }
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

/// image: IN, system_table: IN
#[unsafe(export_name = "efi_main")]
pub extern "efiapi" fn main(image_handle: &mut c_void, system_table: SystemTablePointer) -> Status {
	unsafe {
		Port::COM1.init();
	};
	match system_table.header.signature {
		SystemTable::SIGNATURE => {},
		_ => return Status::INVALID_PARAMETER,
	}

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
			println(format_args!("Failed to open file: {path}"));
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
					println(format_args!("Pointer isn't an ELF: {kernel_file_ptr:?}"));
				},
				bootloader::elf::ELFError::UnsupportedOSABI(abi) => {
					println(format_args!("Unsupported OS ABI: {abi}"));
				},
				other => println(format_args!("{other}")),
			}
			unsafe { ((*system_table.runtime_services).reset_system)(ResetType::SHUTDOWN, Status::SUCCESS, 0, ptr::null()) }
		})
		.into_ok();

	let Some(program_header_offset) = elf.header().program_header_offset else {
		println(format_args!("ELF has no program headers"));
		return Status::ABORTED;
	};

	let Some(program_header_num) = elf.header().program_header_num else {
		println(format_args!("ELF has no program headers"));
		return Status::ABORTED;
	};

	let Some(kernel_entry) = elf.header().entry else {
		println(format_args!("ELF has no entry point"));
		return Status::ABORTED;
	};

	println(format_args!("Program Header Offset: {program_header_offset}"));
	println(format_args!("ELF Entry Point: {kernel_entry:?}"));

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
			println(format_args!("Failed to allocate page"));
			match status {
				Status::OUT_OF_RESOURCES => println(format_args!("OUT OF RESOURCES")),
				Status::INVALID_PARAMETER => println(format_args!("INVALID PARAMETER")),
				Status::NOT_FOUND => println(format_args!("NOT FOUND")),
				_ => unreachable!(),
			}
			return status;
		}
		println(format_args!("Page Table Allocation Address: {page_table_allocation:?}"));
		unsafe {
			page_table_allocation.to_ptr::<u8>().write_bytes(0, size_of::<Data>());
			&mut *page_table_allocation.to_ptr::<Data>()
		}
	};

	let page_table = PageGlobalDirectory::get();
	println(format_args!("CR3: {:X?}", page_table as *const PageGlobalDirectory));

	for (i, entry) in page_table.entries().iter().enumerate() {
		unsafe {
			page_table_allocation.pgd.get_entries()[i] = *entry;
		}
	}

	let high_half = 0xFFFF_8000_0000_0000;
	let index_mask = 0x1FF;
	let mut new = high_half >> 12;
	new >>= 9;
	let page_middle_directory_index = new & index_mask;
	new >>= 9;
	let page_upper_directory_index = new & index_mask;
	new >>= 9;
	let page_global_directory_index = new & index_mask;
	unsafe {
		page_table_allocation.pmd.get_entries()[page_middle_directory_index] = PageMiddleDirectoryEntry::new((&page_table_allocation.pt as *const _ as u64) | PageMiddleDirectoryEntry::PRESENT);
		page_table_allocation.pud.get_entries()[page_upper_directory_index] = PageUpperDirectoryEntry::new((&page_table_allocation.pmd as *const _ as u64) | PageUpperDirectoryEntry::PRESENT);
		page_table_allocation.pgd.get_entries()[page_global_directory_index] = PageGlobalDirectoryEntry::new((&page_table_allocation.pud as *const _ as u64) | PageGlobalDirectoryEntry::PRESENT);
	}
	println(format_args!("Page Global Directory Index: {page_global_directory_index}"));
	println(format_args!("Page Upper Directory Index: {page_upper_directory_index}"));
	println(format_args!("Page Middle Directory Index: {page_middle_directory_index}"));

	let mut segments_loaded = 0;
	for i in 0..program_header_num.get() {
		let program_header = unsafe { &*(kernel_file_ptr.add(program_header_offset.get() + (i as usize * elf.header().program_header_entry_size as usize)) as *const Elf64ProgramHeader) };

		let virtual_address = high_half | program_header.p_vaddr;
		let file_offset = program_header.p_offset;
		let file_size = program_header.p_filesz;
		let mem_size = program_header.p_memsz;
		// let align = program_header.p_align;
		// let flags = program_header.p_flags;
		let segment_pages = mem_size.div_ceil(PAGE_SIZE);
		println(format_args!("{program_header:?}"));
		println(format_args!("VIRTUAL ADDRESS LOADING TO: {virtual_address}"));

		match program_header.p_type {
			ProgramHeaderType::LOAD => {
				let allocated_address = {
					let mut address = PhysicalAddress::new(0);
					let status = unsafe { ((*system_table.boot_services).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, segment_pages, &mut address) };
					if status != Status::SUCCESS {
						println(format_args!("{status}"));
						return status;
					}
					address
				};
				println(format_args!("Allocated Address: {allocated_address:X}"));

				// let offset = virtual_address & 0xFFF;
				let page_table_index = (virtual_address >> 12) & index_mask;
				println(format_args!("Page Table Index: {page_table_index}"));

				if program_header.p_vaddr.wrapping_add(mem_size) < program_header.p_vaddr {
					println(format_args!("ELF address structure overflowed"));
					return Status::ABORTED;
				}

				// unsafe { allocated_address.to_ptr::<u8>().write_bytes(0, segment_pages * 4096) };

				if file_size > 0 {
					unsafe { kernel_file_ptr.add(file_offset).copy_to(allocated_address.to_ptr::<u8>().add(file_offset), file_size) };
				}

				let zero_fill_count = mem_size - file_size;
				if zero_fill_count > 0 {
					println(format_args!("Filling zeros: {zero_fill_count}"));
					unsafe { allocated_address.to_ptr::<u8>().add(file_size).write_bytes(0, zero_fill_count) };
				}

				unsafe {
					page_table_allocation.pt.get_entries()[page_table_index] = PageTableEntry::new(allocated_address.get() | PageTableEntry::PRESENT);
				}

				segments_loaded += 1;
			},
			_ => continue,
		}
	}
	if segments_loaded == 0 {
		return Status::NOT_FOUND;
	}

	unsafe {
		core::arch::asm!("mov cr3, {}", in(reg) page_table_allocation);
	}

	println(format_args!("Segments Loaded: {segments_loaded}"));

	let entry = 0x1280 as u64 | high_half as u64;
	println(format_args!("Entry: {entry:#X}"));

	println(format_args!("Kernel Space:"));
	for i in 0..segments_loaded * 4096 {
		let x = unsafe { *(high_half as *const u8).add(i) };
		print(format_args!("{x:02X} "));
	}
	println(format_args!(""));

	println(format_args!("Entry Start: "));
	for i in 0..segments_loaded * 4096 {
		let x = unsafe { *kernel_entry.as_ptr().add(i) };
		print(format_args!("{x:02X} "));
	}
	println(format_args!(""));

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
				// root_system_description_pointer_ex = Some(unsafe { &*(table.vendortable as *const acpi::RootSystemDescriptionPointerEx) });
			},
			ConfigurationTable::ACPI_10_TABLE => {
				let acpi_table = table.vendor_table as *const acpi::RootSystemDescriptionPointer;
				let t = unsafe { core::ptr::read_unaligned(acpi_table) };
				if t.signature != *b"RSD PTR " {
					println(format_args!("Not an ACPI10 table?"));
					return Status::ABORTED;
				}
				// root_system_description_pointer = Some(unsafe { &*(table.vendortable as *const acpi::RootSystemDescriptionPointer) });
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
			println(format_args!("UNEXPECTED SUCCESS IN ALLOCATING MEMORY MAP"));
			return Status::ABORTED;
		}
		let Status::SUCCESS = (unsafe { ((*system_table.boot_services).allocate_pool)(MemoryType::LOADER_DATA, memory_map_size, &mut memory_map) }) else {
			println(format_args!("FAILED TO ALLOCATE BUFFER FOR MEMORY MAP"));
			return Status::ABORTED;
		};
		let Status::SUCCESS =
			(unsafe { ((*system_table.boot_services).get_memory_map)(&mut memory_map_size, memory_map as *mut MemoryDescriptor, &mut map_key, &mut descriptor_size, &mut descriptor_version) })
		else {
			println(format_args!("FAILED TO GET MEMORY MAP"));
			return Status::ABORTED;
		};
		(memory_map, map_key, memory_map_size, descriptor_size, descriptor_version)
	};

	match unsafe { ((*system_table.boot_services).exit_boot_services)(image_handle, map_key) } {
		Status::SUCCESS => {
			println(format_args!("Exit Boot Services Succeeded!"));
			let header = boot_info::KernelDataHeader {
				graphics_len,
				graphics_ptr,
				root_system_description_pointer,
				root_system_description_pointer_ex,
				system_table,
				virtual_mappings_count: segments_loaded,
			};
			let start: fn(data: boot_info::KernelDataHeader) -> ! = unsafe { core::mem::transmute(kernel_entry.as_ptr() as u64 | 0xFFFF_8000_0000_0000) };
			start(header);
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
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
	let message = info.message().as_str().unwrap_or("");
	let location = info.location().unwrap();
	let file_name = location.file();
	let line_number = location.line();
	let column_number = location.column();
	let format = format_args!("Panicked at: {file_name}\n\tline number: {line_number}\n\tcolumn number: {column_number}\n\tmessage: {message}");
	println(format);
	loop {}
}
