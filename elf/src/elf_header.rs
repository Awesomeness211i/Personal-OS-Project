use core::{
	fmt::{
		Debug,
		Formatter,
	},
	ops,
};

use crate::{
	Elf32Addr,
	Elf32Half,
	Elf32Off,
	Elf32Word,
	Elf64Addr,
	Elf64Half,
	Elf64Off,
	Elf64Word,
};

pub const EI_NIDENT: usize = size_of::<Identifier>();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfHeader<'a> {
	ElfHeader32(&'a Elf32Header),
	ElfHeader64(&'a Elf64Header),
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf64Header {
	/// e_ident:
	identifier: Identifier,
	/// e_type:
	executable_type: Elf64Half,
	/// e_machine:
	machine: Elf64Half,
	/// e_version:
	version: Elf64Word,
	/// e_entry: Virtual address to which the system first transfers control if no entry point it holds 0.
	entry: Elf64Addr,
	/// e_phoff: Program header table file offset in bytes if no program header table it holds 0.
	program_header_offset: Elf64Off,
	/// e_shoff: Section header table file offset in bytes if no section header table it holds 0.
	section_header_offset: Elf64Off,
	/// e_flags: Processor specific flags associated with file.
	flags: Elf64Word,
	/// e_ehsize: ELF header size in bytes.
	elf_header_size: Elf64Half,
	/// e_phentsize: Program header table entry size in bytes.
	program_header_entry_size: Elf64Half,
	/// e_phnum: Program header table number of entries.
	/// If number of program headers >= PN_XNUM(0xFFFF) then program_header_num is set to 0xFFFF and the actual number is in section_header_info field of the section header at index 0 otherwise section_header_info is 0
	program_header_num: Elf64Half,
	/// e_shentsize: Section header entry size in bytes.
	section_header_entry_size: Elf64Half,
	/// e_shnum: Section header table number of entries.
	/// If entries >= 0xFF00 then has value of SHN_UNDEF(0) and actual number of header table entries is in size field of section header at index 0.
	section_header_num: Elf64Half,
	/// e_shstrndx: Section header table index of string table.
	/// If no string table then it is SHN_UNDEF(0) and if entries >= SHN_LORESERVE(0xFF00) then it has the value SHN_XINDEX(0xFFFF) and the actual
	/// index is in the link field of section header at index 0 otherwise the link field contains 0.
	section_header_string_index: Elf64Half,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf32Header {
	/// e_ident:
	identifier: Identifier,
	/// e_type:
	executable_type: Elf32Half,
	/// e_machine:
	machine: Elf32Half,
	/// e_version:
	version: Elf32Word,
	/// e_entry: Virtual address to which the system first transfers control if no entry point it holds 0.
	entry: Elf32Addr,
	/// e_phoff: Program header table file offset in bytes if no program header table it holds 0.
	program_header_offset: Elf32Off,
	/// e_shoff: Section header table file offset in bytes if no section header table it holds 0.
	section_header_offset: Elf32Off,
	/// e_flags: Processor specific flags associated with file.
	flags: Elf32Word,
	/// e_ehsize: ELF header size in bytes.
	elf_header_size: Elf32Half,
	/// e_phentsize: Program header table entry size in bytes.
	program_header_entry_size: Elf32Half,
	/// e_phnum: Program header table number of entries.
	/// If number of program headers >= PN_XNUM(0xFFFF) then program_header_num is set to 0xFFFF and the actual number is in section_header_info field of the section header at index 0 otherwise section_header_info is 0
	program_header_num: Elf32Half,
	/// e_shentsize: Section header entry size in bytes.
	section_header_entry_size: Elf32Half,
	/// e_shnum: Section header table number of entries.
	/// If entries >= 0xFF00 then has value of SHN_UNDEF(0) and actual number of header table entries is in size field of section header at index 0.
	section_header_num: Elf32Half,
	/// e_shstrndx: Section header table index of string table.
	/// If no string table then it is SHN_UNDEF(0) and if entries >= SHN_LORESERVE(0xFF00) then it has the value SHN_XINDEX(0xFFFF) and the actual
	/// index is in the link field of section header at index 0 otherwise the link field contains 0.
	section_header_string_index: Elf32Half,
}

impl ElfHeader<'_> {
	pub fn executable_type(&self) -> ExecutableType {
		match self {
			ElfHeader::ElfHeader32(header) => ExecutableType(header.executable_type),
			ElfHeader::ElfHeader64(header) => ExecutableType(header.executable_type),
		}
	}

	pub fn machine(&self) -> Machine {
		match self {
			ElfHeader::ElfHeader32(header) => Machine(header.machine),
			ElfHeader::ElfHeader64(header) => Machine(header.machine),
		}
	}

	pub fn version(&self) -> Version {
		match self {
			ElfHeader::ElfHeader32(header) => Version(header.version),
			ElfHeader::ElfHeader64(header) => Version(header.version),
		}
	}

	pub fn entry(&self) -> usize {
		match self {
			ElfHeader::ElfHeader32(header) => header.entry as usize,
			ElfHeader::ElfHeader64(header) => header.entry as usize,
		}
	}

	pub fn program_header_offset(&self) -> usize {
		match self {
			ElfHeader::ElfHeader32(header) => header.program_header_offset as usize,
			ElfHeader::ElfHeader64(header) => header.program_header_offset as usize,
		}
	}

	pub fn section_header_offset(&self) -> usize {
		match self {
			ElfHeader::ElfHeader32(header) => header.section_header_offset as usize,
			ElfHeader::ElfHeader64(header) => header.section_header_offset as usize,
		}
	}

	pub fn flags(&self) -> usize {
		match self {
			ElfHeader::ElfHeader32(header) => header.flags as usize,
			ElfHeader::ElfHeader64(header) => header.flags as usize,
		}
	}

	pub fn size(&self) -> usize {
		match self {
			ElfHeader::ElfHeader32(header) => header.elf_header_size as usize,
			ElfHeader::ElfHeader64(header) => header.elf_header_size as usize,
		}
	}

	pub fn program_header_entry_size(&self) -> usize {
		match self {
			ElfHeader::ElfHeader32(header) => header.program_header_entry_size as usize,
			ElfHeader::ElfHeader64(header) => header.program_header_entry_size as usize,
		}
	}

	pub fn program_header_num(&self) -> usize {
		match self {
			ElfHeader::ElfHeader32(header) => header.program_header_num as usize,
			ElfHeader::ElfHeader64(header) => header.program_header_num as usize,
		}
	}

	pub fn section_header_num(&self) -> usize {
		match self {
			ElfHeader::ElfHeader32(header) => header.section_header_num as usize,
			ElfHeader::ElfHeader64(header) => header.section_header_num as usize,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Magic([u8; 4]);
impl Magic {
	pub(crate) const MAGIC: Self = Self(*b"\x7FELF");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Class(u8);
impl Class {
	pub const ELF_CLASS_NONE: Self = Self(0);
	pub const ELF_CLASS_32: Self = Self(1);
	pub const ELF_CLASS_64: Self = Self(2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Data(u8);
impl Data {
	pub const ELF_DATA_NONE: Self = Self(0);
	pub const ELF_DATA_2LSB: Self = Self(1);
	pub const ELF_DATA_2MSB: Self = Self(2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentifierVersion(u8);
impl IdentifierVersion {
	pub const ELF_VERSION_NONE: Self = Self(0);
	pub const ELF_VERSION_CURRENT: Self = Self(1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsAbi(u8);
impl OsAbi {
	pub const ELF_OS_ABI_NONE: Self = Self(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiVersion(u8);
impl AbiVersion {
	pub const ELF_ABI_VERSION_NONE: Self = Self(0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u32);
impl Version {
	pub const ELF_VERSION_NONE: Self = Self(0);
	pub const ELF_VERSION_CURRENT: Self = Self(1);
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identifier {
	pub(crate) magic: Magic,
	pub(crate) class: Class,
	pub(crate) data: Data,
	pub(crate) version: IdentifierVersion,
	pub(crate) os_abi: OsAbi,
	pub(crate) abi_version: AbiVersion,
	pub(crate) pad: [u8; 7],
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ExecutableType(u16);

impl Debug for ExecutableType {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		match *self {
			Self::NONE => write!(f, "NONE"),
			Self::RELOCATABLE => write!(f, "RELOCATABLE"),
			Self::EXECUTABLE => write!(f, "EXECUTABLE"),
			Self::DYNAMIC => write!(f, "DYNAMIC"),
			Self::CORE => write!(f, "CORE"),
			_ => write!(f, "Unknown: {:#X}", self.0),
		}
	}
}

impl ExecutableType {
	pub const NONE: Self = Self(0x00);
	pub const RELOCATABLE: Self = Self(0x01);
	pub const EXECUTABLE: Self = Self(0x02);
	pub const DYNAMIC: Self = Self(0x03);
	pub const CORE: Self = Self(0x04);
	pub const OS_SPECIFIC: ops::RangeInclusive<Self> = Self(0xFE00)..=Self(0xFEFF);
	pub const PROCESSOR_SPECIFIC: ops::RangeInclusive<Self> = Self(0xFF00)..=Self(0xFFFF);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Machine(u16);

impl Debug for Machine {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		match *self {
			Self::NONE => write!(f, "NONE"),
			Self::M32 => write!(f, "M32"),
			Self::SPARC => write!(f, "SPARC"),
			Self::I386 => write!(f, "I386"),
			Self::M68K => write!(f, "M68K"),
			Self::M88K => write!(f, "M88K"),
			Self::IAMCU => write!(f, "IAMCU"),
			Self::I860 => write!(f, "I860"),
			Self::X86_64 => write!(f, "X86_64"),
			Self::Z80 => write!(f, "Z80"),
			Self::VISIUM => write!(f, "VISIUM"),
			Self::FT32 => write!(f, "FT32"),
			Self::MOXIE => write!(f, "MOXIE"),
			Self::AMDGPU => write!(f, "AMDGPU"),
			Self::RISCV => write!(f, "RISCV"),
			_ => write!(f, "Unknown: {:#X}", self.0),
		}
	}
}

impl Machine {
	pub const NONE: Self = Self(0x00);
	pub const M32: Self = Self(0x01);
	pub const SPARC: Self = Self(0x02);
	pub const I386: Self = Self(0x03);
	pub const M68K: Self = Self(0x04);
	pub const M88K: Self = Self(0x05);
	pub const IAMCU: Self = Self(0x06);
	pub const I860: Self = Self(0x07);

	pub const X86_64: Self = Self(0x3E);

	pub const Z80: Self = Self(0xDC);
	pub const VISIUM: Self = Self(0xDD);
	pub const FT32: Self = Self(0xDE);
	pub const MOXIE: Self = Self(0xDF);
	pub const AMDGPU: Self = Self(0xE0);
	pub const RISCV: Self = Self(0xF3);
}
