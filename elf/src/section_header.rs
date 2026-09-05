use core::{
	fmt::{
		Debug,
		Formatter,
	},
	ops,
};

use crate::{
	Elf32Addr,
	Elf32Off,
	Elf32Word,
	Elf64Addr,
	Elf64Off,
	Elf64Word,
	Elf64Xword,
};

pub enum SectionHeader<'a> {
	SectionHeader32(&'a Elf32SectionHeader),
	SectionHeader64(&'a Elf64SectionHeader),
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf64SectionHeader {
	/// Index into section header string table
	pub name: Elf64Word,
	pub section_header_type: Elf64Word,
	pub flags: Elf64Xword,
	/// If appears in memory image of process gives address that it should reside at otherwise 0
	pub address: Elf64Addr,
	/// Gives byte offset from beginning of file to first byte in the section
	pub offset: Elf64Off,
	/// Sections size in bytes if it isn't type SHT_NOBITS
	pub size: Elf64Xword,
	/// Section header index link that interpretation depends on the type of section
	pub link: Elf64Word,
	/// Holds extra information that interpretation depends on section type
	pub info: Elf64Word,
	pub address_align: Elf64Xword,
	/// For sections that hold a table of fixed size entries this gives the size in bytes of the
	/// entry and 0 otherwise
	pub entry_size: Elf64Xword,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf32SectionHeader {
	/// Index into section header string table
	pub name: Elf32Word,
	pub section_header_type: Elf32Word,
	pub flags: Elf32Word,
	/// If appears in memory image of process gives address that it should reside at otherwise 0
	pub address: Elf32Addr,
	/// Gives byte offset from beginning of file to first byte in the section
	pub offset: Elf32Off,
	/// Sections size in bytes if it isn't type SHT_NOBITS
	pub size: Elf32Word,
	/// Section header index link that interpretation depends on the type of section
	pub link: Elf32Word,
	/// Holds extra information that interpretation depends on section type
	pub info: Elf32Word,
	pub address_align: Elf32Word,
	/// For sections that hold a table of fixed size entries this gives the size in bytes of the
	/// entry and 0 otherwise
	pub entry_size: Elf32Word,
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SectionHeaderType(u32);

impl Debug for SectionHeaderType {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		match *self {
			Self::NULL => write!(f, "NULL"),
			Self::PROGRAM_BITS => write!(f, "PROGRAM BITS"),
			Self::SYMBOL_TABLE => write!(f, "SYMBOL TABLE"),
			Self::STRING_TABLE => write!(f, "STRING TABLE"),
			Self::RELOCATION_ADDENDS => write!(f, "RELOCATION ADDENDS"),
			Self::HASH => write!(f, "HASH"),
			Self::DYNAMIC => write!(f, "DYNAMIC"),
			Self::NOTE => write!(f, "NOTE"),
			Self::NO_BITS => write!(f, "NO BITS"),
			Self::RELOCATION => write!(f, "RELOCATION"),
			Self::LIB => write!(f, "LIB"),
			Self::DYNAMIC_SYMBOLS => write!(f, "DYNAMIC SYMBOLS"),
			Self::INITIZATION_ARRAY => write!(f, "INITIZATION ARRAY"),
			Self::PREINITITIALIZATION_ARRAY => write!(f, "PREINITITIALIZATION ARRAY"),
			Self::GROUP => write!(f, "GROUP"),
			Self::SYMBOL_TABLE_SECTION_HEADER_INDEX => write!(f, "SYMBOL TABLE SECTION HEADER INDEX"),
			Self::SHT_GNU_HASH => write!(f, "SHT GNU HASH"),
			_ => write!(f, "Unknown: {:#X}", self.0),
		}
	}
}

impl SectionHeaderType {
	pub const NULL: Self = Self(0x00000000);
	pub const PROGRAM_BITS: Self = Self(0x00000001);
	pub const SYMBOL_TABLE: Self = Self(0x00000002);
	pub const STRING_TABLE: Self = Self(0x00000003);
	pub const RELOCATION_ADDENDS: Self = Self(0x00000004);
	pub const HASH: Self = Self(0x00000005);
	pub const DYNAMIC: Self = Self(0x00000006);
	pub const NOTE: Self = Self(0x00000007);
	pub const NO_BITS: Self = Self(0x00000008);
	pub const RELOCATION: Self = Self(0x00000009);
	pub const LIB: Self = Self(0x0000000A);
	pub const DYNAMIC_SYMBOLS: Self = Self(0x0000000B);
	pub const INITIZATION_ARRAY: Self = Self(0x0000000E);
	pub const FINISH_ARRAY: Self = Self(0x0000000F);
	pub const PREINITITIALIZATION_ARRAY: Self = Self(0x00000010);
	pub const GROUP: Self = Self(0x00000011);
	pub const SYMBOL_TABLE_SECTION_HEADER_INDEX: Self = Self(0x00000012);
	pub const SHT_GNU_HASH: Self = Self(0x6FFFFFF6);
	pub const OS_SPECIFIC: ops::RangeInclusive<Self> = Self(0x60000000)..=Self(0x6fffffff);
	pub const PROCESSOR_SPECIFIC: ops::RangeInclusive<Self> = Self(0x70000000)..=Self(0x7fffffff);
	pub const APPLICATION_SPECIFIC: ops::RangeInclusive<Self> = Self(0x80000000)..=Self(0xffffffff);
}
