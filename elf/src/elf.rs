use core::fmt::Debug;

use crate::{
	Elf32Addr,
	Elf32Sword,
	Elf32Word,
	Elf64Addr,
	Elf64Sxword,
	Elf64Xword,
};

#[non_exhaustive]
#[repr(C, i32)]
#[derive(Debug, PartialEq, Eq)]
pub enum Elf32Dynamic {
	Null = 0,
	Needed { val: Elf32Word },
	PLTRELSZ { val: Elf32Word },
	PLTGOT { ptr: Elf32Addr },
	HASH { ptr: Elf32Addr },
	STRTAB { ptr: Elf32Addr },
	SYMTAB { ptr: Elf32Addr },
	RELA { ptr: Elf32Addr },
	RELASZ { val: Elf32Word },
	RELAENT { val: Elf32Word },
	STRSZ { val: Elf32Word },
	SYMENT { val: Elf32Word },
	INIT { ptr: Elf32Addr },
	FINI { ptr: Elf32Addr },
	SONAME { val: Elf32Word },
	RPATH { val: Elf32Word },
	SYMBOLIC,
	REL { ptr: Elf32Addr },
	RELSZ { val: Elf32Word },
	RELENT { val: Elf32Word },
	PLTREL { val: Elf32Word },
	DEBUG { ptr: Elf32Addr },
	TEXTREL,
	JMPREL { ptr: Elf32Addr },
	LOPROC = 0x70000000,
	HIPROC = 0x7FFFFFFF,
}

#[non_exhaustive]
#[repr(C, u64)]
#[derive(Debug, PartialEq, Eq)]
pub enum Elf64Dynamic {
	Null = 0,
	Needed { val: Elf64Xword },
	PLTRELSZ { val: Elf64Xword },
	PLTGOT { ptr: Elf64Addr },
	HASH { ptr: Elf64Addr },
	STRTAB { ptr: Elf64Addr },
	SYMTAB { ptr: Elf64Addr },
	RELA { ptr: Elf64Addr },
	RELASZ { val: Elf64Xword },
	RELAENT { val: Elf64Xword },
	STRSZ { val: Elf64Xword },
	SYMENT { val: Elf64Xword },
	INIT { ptr: Elf64Addr },
	FINI { ptr: Elf64Addr },
	SONAME { val: Elf64Xword },
	RPATH { val: Elf64Xword },
	SYMBOLIC,
	REL { ptr: Elf64Addr },
	RELSZ { val: Elf64Xword },
	RELENT { val: Elf64Xword },
	PLTREL { val: Elf64Xword },
	DEBUG { ptr: Elf64Addr },
	TEXTREL,
	JMPREL { ptr: Elf64Addr },
	BindNow,
	InitArray { ptr: Elf64Addr },
	FiniArray { ptr: Elf64Addr },
	InitArraySize { val: Elf64Xword },
	FiniArraySize { val: Elf64Xword },
	RunPath { val: Elf64Xword },
	Flags { val: Elf64Xword },
	Encoding { val: Elf64Xword },
	PreInitArray { ptr: Elf64Addr },
	PreInitArraySize { val: Elf64Xword },
	LOOS = 0x6000000D,
	HIOS = 0x6FFFF000,
	GnuPrelinked = 0x6FFFFDF5,
	GnuConflictSize,
	GnuLibraryListSize,
	Checksum,
	PltPadSize,
	MoveEnt,
	MoveSize,
	Feature1,
	PosFlag1,
	SymInSize,
	SymInEnt,
	GnuHash { val: Elf64Xword } = 0x6FFFFEF5,
	GnuReserved1,
	GnuReserved2,
	GnuConflict,
	GnuReserved3,
	GnuReserved4,
	GnuReserved5,
	GnuReserved6,
	GnuReserved7,
	GnuReserved8,
	GnuLibraryList = 0x6FFFFEFF,
	GnuReserved9,
	GnuReserved10,
	GnuReserved11,
	GnuReserved12,
	GnuReserved13,
	GnuReserved14,
	GnuReserved15,
	GnuReserved16,
	GnuReserved17,
	GnuReserved18,
	GnuReserved19,
	GnuReserved20,
	GnuReserved21,
	GnuReserved22,
	GnuReserved23,
	GnuReserved24,
	GnuReserved25,
	GnuReserved26,
	GnuReserved27,
	GnuReserved28,
	GnuReserved29,
	GnuReserved30,
	GnuReserved31,
	GnuReserved32,
	GnuReserved33,
	GnuReserved34,
	GnuReserved35,
	GnuReserved36,
	GnuReserved37,
	GnuReserved38,
	GnuReserved39,
	GnuReserved40,
	GnuReserved41,
	GnuReserved42,
	GnuReserved43,
	GnuReserved44,
	GnuReserved45,
	GnuReserved46,
	GnuReserved47,
	GnuReserved48,
	GnuReserved49,
	GnuFlags1 { val: Elf64Xword } = 0x6FFFFFFB,
	GnuRelaCount { val: Elf64Xword } = 0x6FFFFFF9,
	LOPROC { val: Elf64Xword } = 0x70000000,
	HIPROC { val: Elf64Xword } = 0x7FFFFFFF,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Elf32Rel {
	pub r_offset: Elf32Addr,
	pub r_info: Elf32Word,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Elf32Rela {
	pub r_offset: Elf32Addr,
	pub r_info: Elf32Word,
	pub r_addend: Elf32Sword,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Elf64Rel {
	pub r_offset: Elf64Addr,
	pub r_info: Elf64Xword,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Elf64Rela {
	pub r_offset: Elf64Addr,
	pub r_info: Elf64Xword,
	pub r_addend: Elf64Sxword,
}

#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Elf64RTypeX86_64(Elf64Xword);
impl Elf64RTypeX86_64 {
	pub const R_AMD64_NONE: Self = Self(0);
	pub const R_AMD64_RELATIVE: Self = Self(8);

	pub const fn new(info: u64) -> Self {
		Self(info)
	}

	pub const fn get(&self) -> u64 {
		self.0
	}
}
