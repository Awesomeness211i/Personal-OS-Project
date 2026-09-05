use core::fmt::{
	Debug,
	Formatter,
};

use crate::{
	Elf,
	Elf32Addr,
	Elf32Off,
	Elf32Word,
	Elf64Addr,
	Elf64Off,
	Elf64Word,
	Elf64Xword,
	elf_header::ElfHeader,
};

pub enum ProgramHeader<'a> {
	ProgramHeader32(&'a Elf32ProgramHeader),
	ProgramHeader64(&'a Elf64ProgramHeader),
}

impl ProgramHeader<'_> {
	pub fn header_type(&self) -> ProgramHeaderType {
		match self {
			ProgramHeader::ProgramHeader32(header) => ProgramHeaderType(header.p_type),
			ProgramHeader::ProgramHeader64(header) => ProgramHeaderType(header.p_type),
		}
	}

	pub fn flags(&self) -> ProgramHeaderFlags {
		match self {
			ProgramHeader::ProgramHeader32(header) => ProgramHeaderFlags(header.p_flags),
			ProgramHeader::ProgramHeader64(header) => ProgramHeaderFlags(header.p_flags),
		}
	}

	pub fn offset(&self) -> usize {
		match self {
			ProgramHeader::ProgramHeader32(header) => header.p_offset as usize,
			ProgramHeader::ProgramHeader64(header) => header.p_offset as usize,
		}
	}

	pub fn virtual_address(&self) -> usize {
		match self {
			ProgramHeader::ProgramHeader32(header) => header.p_vaddr as usize,
			ProgramHeader::ProgramHeader64(header) => header.p_vaddr as usize,
		}
	}

	pub fn physical_address(&self) -> usize {
		match self {
			ProgramHeader::ProgramHeader32(header) => header.p_paddr as usize,
			ProgramHeader::ProgramHeader64(header) => header.p_paddr as usize,
		}
	}

	pub fn file_size(&self) -> usize {
		match self {
			ProgramHeader::ProgramHeader32(header) => header.p_filesz as usize,
			ProgramHeader::ProgramHeader64(header) => header.p_filesz as usize,
		}
	}

	pub fn mem_size(&self) -> usize {
		match self {
			ProgramHeader::ProgramHeader32(header) => header.p_memsz as usize,
			ProgramHeader::ProgramHeader64(header) => header.p_memsz as usize,
		}
	}

	pub fn align(&self) -> usize {
		match self {
			ProgramHeader::ProgramHeader32(header) => header.p_align as usize,
			ProgramHeader::ProgramHeader64(header) => header.p_align as usize,
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf32ProgramHeader {
	p_type: Elf32Word,
	p_offset: Elf32Off,
	p_vaddr: Elf32Addr,
	p_paddr: Elf32Addr,
	p_filesz: Elf32Word,
	p_memsz: Elf32Word,
	p_flags: Elf32Word,
	p_align: Elf32Word,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf64ProgramHeader {
	p_type: Elf64Word,
	p_flags: Elf64Word,
	p_offset: Elf64Off,
	p_vaddr: Elf64Addr,
	p_paddr: Elf64Addr,
	p_filesz: Elf64Xword,
	p_memsz: Elf64Xword,
	p_align: Elf64Xword,
}

pub struct ProgramHeaderIterator<'a> {
	ptr: &'a [u8],
	elf: &'a Elf<'a>,
	i: usize,
}

impl<'a> ProgramHeaderIterator<'a> {
	pub(crate) fn new(elf: &'a Elf) -> Self {
		Self {
			ptr: &elf.file[elf.header().program_header_offset()..elf.header().program_header_offset() + elf.header().program_header_entry_size() * elf.header().program_header_num()],
			elf,
			i: 0,
		}
	}
}

impl<'a> Iterator for ProgramHeaderIterator<'a> {
	type Item = ProgramHeader<'a>;
	fn next(&mut self) -> Option<Self::Item> {
		let header = self.elf.header();
		let size = header.program_header_entry_size();

		if self.i < self.ptr.len() / size {
			let result = match header {
				// Safety:
				// Should be safe because of bounds checking on buffer and the calculation given for
				// the program header range
				ElfHeader::ElfHeader32(_) => ProgramHeader::ProgramHeader32(unsafe { &*(self.ptr[self.i * size..(self.i + 1) * size].as_ptr() as *const Elf32ProgramHeader) }),
				// Safety:
				// Should be safe because of bounds checking on buffer and the calculation given for
				// the program header range
				ElfHeader::ElfHeader64(_) => ProgramHeader::ProgramHeader64(unsafe { &*(self.ptr[self.i * size..(self.i + 1) * size].as_ptr() as *const Elf64ProgramHeader) }),
			};
			self.i += 1;
			Some(result)
		} else {
			None
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProgramHeaderType(u32);
impl ProgramHeaderType {
	pub const NULL: Self = Self(0);
	pub const LOAD: Self = Self(1);
	pub const DYNAMIC: Self = Self(2);
	pub const INTERP: Self = Self(3);
	pub const NOTE: Self = Self(4);
	pub const SECTION_HEADER_LIB: Self = Self(5);
	pub const PROGRAM_HEADER: Self = Self(6);
	pub const THREAD_LOCAL_STORAGE: Self = Self(7);
	pub const GNU_EH_FRAME: Self = Self(0x6474e550);
	pub const GNU_STACK: Self = Self(0x6474e551);
	pub const GNU_RELRO: Self = Self(0x6474e552);
	pub const GNU_PROPERTY: Self = Self(0x6474e553);
	pub const GNU_SFRAME: Self = Self(0x6474e554);
}

impl Debug for ProgramHeaderType {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		match *self {
			Self::NULL => write!(f, "NULL"),
			Self::LOAD => write!(f, "LOAD"),
			Self::DYNAMIC => write!(f, "DYNAMIC"),
			Self::INTERP => write!(f, "INTERP"),
			Self::NOTE => write!(f, "NOTE"),
			Self::SECTION_HEADER_LIB => write!(f, "SECTION HEADER LIBRARY"),
			Self::PROGRAM_HEADER => write!(f, "PROGRAM HEADER"),
			Self::THREAD_LOCAL_STORAGE => write!(f, "THREAD LOCAL STORAGE"),
			Self::GNU_EH_FRAME => write!(f, "GNU EH FRAME"),
			Self::GNU_STACK => write!(f, "GNU STACK"),
			Self::GNU_RELRO => write!(f, "GNU RELRO"),
			Self::GNU_PROPERTY => write!(f, "GNU PROPERTY"),
			Self::GNU_SFRAME => write!(f, "GNU SFRAME"),
			_ => write!(f, "Unknown: {:#X}", self.0),
		}
	}
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProgramHeaderFlags(u32);

impl Debug for ProgramHeaderFlags {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		let none = '-';
		let r = if self.0 & Self::R.0 != 0 { 'r' } else { none };
		let w = if self.0 & Self::W.0 != 0 { 'w' } else { none };
		let x = if self.0 & Self::X.0 != 0 { 'x' } else { none };
		write!(f, "{r}{w}{x}: {:#X}", self.0)
	}
}

impl ProgramHeaderFlags {
	pub const X: Self = Self(0x1);
	pub const W: Self = Self(0x2);
	pub const R: Self = Self(0x4);
	pub const MASK_OS: Self = Self(0x0FF00000);
	pub const MASK_PROC: Self = Self(0xF0000000);
	pub fn get(&self) -> u32 {
		self.0
	}
}
