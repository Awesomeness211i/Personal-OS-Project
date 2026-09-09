use core::fmt::Debug;

use crate::{
	Elf32Addr,
	Elf32Off,
	Elf32Sword,
	Elf32Word,
	Elf64Addr,
	Elf64Sxword,
	Elf64Xword,
};

#[derive(Debug)]
pub enum ElfDynamic<'a> {
	Elf32Dynamic(&'a Elf32Dynamic),
	Elf64Dynamic(&'a Elf64Dynamic),
}

impl ElfDynamic<'_> {
	/// no field is used
	pub const NULL: Elf64DynamicType = Elf64DynamicType(0);
	/// val field is used
	pub const NEEDED: Elf64DynamicType = Elf64DynamicType(1);
	/// val field is used
	pub const PLTRELSZ: Elf64DynamicType = Elf64DynamicType(2);
	/// ptr field is used
	pub const PLTGOT: Elf64DynamicType = Elf64DynamicType(3);
	/// ptr field is used
	pub const HASH: Elf64DynamicType = Elf64DynamicType(4);
	/// ptr field is used
	pub const STRTAB: Elf64DynamicType = Elf64DynamicType(5);
	/// ptr field is used
	pub const SYMTAB: Elf64DynamicType = Elf64DynamicType(6);
	/// ptr field is used
	pub const RELA: Elf64DynamicType = Elf64DynamicType(7);
	/// val field is used
	pub const RELASZ: Elf64DynamicType = Elf64DynamicType(8);
	/// val field is used
	pub const RELAENT: Elf64DynamicType = Elf64DynamicType(9);
	/// val field is used
	pub const STRSZ: Elf64DynamicType = Elf64DynamicType(10);
	/// val field is used
	pub const SYMENT: Elf64DynamicType = Elf64DynamicType(11);
	/// ptr field is used
	pub const INIT: Elf64DynamicType = Elf64DynamicType(12);
	/// ptr field is used
	pub const FINI: Elf64DynamicType = Elf64DynamicType(13);
	/// val field is used
	pub const SONAME: Elf64DynamicType = Elf64DynamicType(14);
	/// val field is used
	pub const RPATH: Elf64DynamicType = Elf64DynamicType(15);
	/// no field is used
	pub const SYMBOLIC: Elf64DynamicType = Elf64DynamicType(16);
	/// ptr field is used
	pub const REL: Elf64DynamicType = Elf64DynamicType(17);
	/// val field is used
	pub const RELSZ: Elf64DynamicType = Elf64DynamicType(18);
	/// val field is used
	pub const RELENT: Elf64DynamicType = Elf64DynamicType(19);
	/// val field is used
	pub const PLTREL: Elf64DynamicType = Elf64DynamicType(20);
	/// ptr field is used
	pub const DEBUG: Elf64DynamicType = Elf64DynamicType(21);
	/// no field is used
	pub const TEXTREL: Elf64DynamicType = Elf64DynamicType(22);
	/// ptr field is used
	pub const JMPREL: Elf64DynamicType = Elf64DynamicType(23);
	/// no field is used
	pub const BINDNOW: Elf64DynamicType = Elf64DynamicType(24);
	/// ptr field is used
	pub const INITARRAY: Elf64DynamicType = Elf64DynamicType(25);
	/// ptr field is used
	pub const FINIARRAY: Elf64DynamicType = Elf64DynamicType(26);
	/// val field is used
	pub const INITARRAYSIZE: Elf64DynamicType = Elf64DynamicType(27);
	/// val field is used
	pub const FINIARRAYSIZE: Elf64DynamicType = Elf64DynamicType(28);
	/// val field is used
	pub const RUNPATH: Elf64DynamicType = Elf64DynamicType(29);
	/// val field is used
	pub const FLAGS: Elf64DynamicType = Elf64DynamicType(30);
	/// val field is used
	pub const GNUHASH: Elf64DynamicType = Elf64DynamicType(0x6FFFFEF5);
	/// val field is used
	pub const GNURELACOUNT: Elf64DynamicType = Elf64DynamicType(0x6FFFFFF9);
	/// val field is used
	pub const GNUFLAGS1: Elf64DynamicType = Elf64DynamicType(0x6FFFFFFB);

	fn parse(ty: Elf64DynamicType) -> (&'static str, &'static str, bool) {
		match ty {
			ElfDynamic::NULL => ("Null", "", false),
			ElfDynamic::NEEDED => ("Needed", "val", true),
			ElfDynamic::PLTRELSZ => ("PltRelSz", "val", true),
			ElfDynamic::PLTGOT => ("PltGot", "ptr", true),
			ElfDynamic::HASH => ("Hash", "ptr", true),
			ElfDynamic::STRTAB => ("StrTab", "ptr", true),
			ElfDynamic::SYMTAB => ("SymTab", "ptr", true),
			ElfDynamic::RELA => ("Rela", "ptr", true),
			ElfDynamic::RELASZ => ("Rela", "val", true),
			ElfDynamic::RELAENT => ("RelaEnt", "val", true),
			ElfDynamic::STRSZ => ("StrSz", "val", true),
			ElfDynamic::SYMENT => ("SymEnt", "val", true),
			ElfDynamic::INIT => ("Init", "ptr", true),
			ElfDynamic::FINI => ("Fini", "ptr", true),
			ElfDynamic::SONAME => ("SoName", "val", true),
			ElfDynamic::RPATH => ("RPath", "val", true),
			ElfDynamic::SYMBOLIC => ("Symbolic", "", false),
			ElfDynamic::REL => ("Rel", "ptr", true),
			ElfDynamic::RELSZ => ("RelSz", "val", true),
			ElfDynamic::RELENT => ("RelEnt", "val", true),
			ElfDynamic::PLTREL => ("PltRel", "val", true),
			ElfDynamic::DEBUG => ("Debug", "ptr", true),
			ElfDynamic::TEXTREL => ("TextRel", "", false),
			ElfDynamic::JMPREL => ("JmpRel", "ptr", true),
			ElfDynamic::INITARRAY => ("InitArray", "ptr", true),
			ElfDynamic::FINIARRAY => ("FiniArray", "ptr", true),
			ElfDynamic::INITARRAYSIZE => ("InitArraySize", "val", true),
			ElfDynamic::FINIARRAYSIZE => ("FiniArraySize", "val", true),
			ElfDynamic::RUNPATH => ("RunPath", "val", true),
			ElfDynamic::FLAGS => ("Flags", "val", true),

			ElfDynamic::GNUHASH => ("GnuHash", "val", true),
			ElfDynamic::GNURELACOUNT => ("GnuRelaCount", "val", true),
			ElfDynamic::GNUFLAGS1 => ("GnuFlags1", "val", true),
			_ => ("Unknown", "unk", true),
		}
		// 	Encoding { val: Elf64Xword },
		// 	PreInitArray { ptr: Elf64Addr },
		// 	PreInitArraySize { val: Elf64Xword },
		// 	LOOS = 0x6000000D,
		// 	HIOS = 0x6FFFF000,
		// 	GnuPrelinked = 0x6FFFFDF5,
		// 	GnuConflictSize,
		// 	GnuLibraryListSize,
		// 	Checksum,
		// 	PltPadSize,
		// 	MoveEnt,
		// 	MoveSize,
		// 	Feature1,
		// 	PosFlag1,
		// 	SymInSize,
		// 	SymInEnt,
		// 	GnuHash { val: Elf64Xword } = 0x6FFFFEF5,
		// 	GnuReserved1,
		// 	GnuReserved2,
		// 	GnuConflict,
		// 	GnuReserved3,
		// 	GnuReserved4,
		// 	GnuReserved5,
		// 	GnuReserved6,
		// 	GnuReserved7,
		// 	GnuReserved8,
		// 	GnuLibraryList = 0x6FFFFEFF,
		// 	GnuReserved9,
		// 	GnuReserved10,
		// 	GnuReserved11,
		// 	GnuReserved12,
		// 	GnuReserved13,
		// 	GnuReserved14,
		// 	GnuReserved15,
		// 	GnuReserved16,
		// 	GnuReserved17,
		// 	GnuReserved18,
		// 	GnuReserved19,
		// 	GnuReserved20,
		// 	GnuReserved21,
		// 	GnuReserved22,
		// 	GnuReserved23,
		// 	GnuReserved24,
		// 	GnuReserved25,
		// 	GnuReserved26,
		// 	GnuReserved27,
		// 	GnuReserved28,
		// 	GnuReserved29,
		// 	GnuReserved30,
		// 	GnuReserved31,
		// 	GnuReserved32,
		// 	GnuReserved33,
		// 	GnuReserved34,
		// 	GnuReserved35,
		// 	GnuReserved36,
		// 	GnuReserved37,
		// 	GnuReserved38,
		// 	GnuReserved39,
		// 	GnuReserved40,
		// 	GnuReserved41,
		// 	GnuReserved42,
		// 	GnuReserved43,
		// 	GnuReserved44,
		// 	GnuReserved45,
		// 	GnuReserved46,
		// 	GnuReserved47,
		// 	GnuReserved48,
		// 	GnuReserved49,
		// 	GnuFlags1 { val: Elf64Xword } = 0x6FFFFFFB,
		// 	GnuRelaCount { val: Elf64Xword } = 0x6FFFFFF9,
		// 	LOPROC { val: Elf64Xword } = 0x70000000,
		// 	HIPROC { val: Elf64Xword } = 0x7FFFFFFF,
	}
}

#[repr(C)]
union Elf32DynamicUnion {
	val: Elf32Word,
	ptr: Elf32Addr,
	off: Elf32Off,
}

#[repr(C)]
pub struct Elf32Dynamic {
	pub tag: Elf32DynamicType,
	data: Elf32DynamicUnion,
}

impl Debug for Elf32Dynamic {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let (name, field_name, has_field) = ElfDynamic::parse(self.tag.into());
		if has_field {
			f.debug_struct(name).field(field_name, unsafe { &self.data.val }).finish()
		} else {
			f.debug_struct(name).finish()
		}
	}
}

#[repr(C)]
union Elf64DynamicUnion {
	val: Elf64Xword,
	ptr: Elf64Addr,
}

#[repr(C)]
pub struct Elf64Dynamic {
	pub tag: Elf64DynamicType,
	data: Elf64DynamicUnion,
}

impl Elf64Dynamic {
	pub fn field(&self) -> u64 {
		unsafe { self.data.val }
	}
}

impl Debug for Elf64Dynamic {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let (name, field_name, has_field) = ElfDynamic::parse(self.tag);
		if has_field {
			f.debug_struct(name).field(field_name, unsafe { &self.data.val }).finish()
		} else {
			f.debug_struct(name).finish()
		}
	}
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialOrd, PartialEq, Eq)]
pub struct Elf32DynamicType(Elf32Sword);
impl Into<Elf64DynamicType> for Elf32DynamicType {
	fn into(self) -> Elf64DynamicType {
		Elf64DynamicType(self.0 as Elf64Xword)
	}
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialOrd, PartialEq, Eq)]
pub struct Elf64DynamicType(Elf64Xword);

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
