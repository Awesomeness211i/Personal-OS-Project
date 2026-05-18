use core::{
	error::Error,
	fmt::{
		Debug,
		Display,
		Formatter,
	},
	num,
	ops,
	ptr,
	slice,
};

pub type Elf32Half = u16;
pub type Elf32Off = u32;
pub type Elf32Addr = u32;
pub type Elf32Word = u32;
pub type Elf32Sword = i32;

pub type Elf64Half = u16;
pub type Elf64Off = u64;
pub type Elf64Addr = u64;
pub type Elf64Word = u32;
pub type Elf64Sword = i32;
pub type Elf64Xword = u64;
pub type Elf64Sxword = i64;

#[derive(Debug)]
pub enum ELFError {
	Unknown,
	NotELF,
	UnsupportedOSABI(u8),
	UnsupportedOSABIVersion,
	UnsupportedExecutableType,
	UnsupportedMachine,
	UnsupportedELFVersion,
	UnsupportedEndianness,
	UnsupportedArchitectureWidth,
}
impl Display for ELFError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "{self:?}")
	}
}
impl Error for ELFError {}

pub struct Elf<'a>(&'a mut [u8]);
impl Elf<'_> {
	pub fn new(ptr: *mut u8, len: usize) -> Result<Self, ELFError> {
		let Some(ptr) = ptr::NonNull::new(ptr) else {
			return Err(ELFError::NotELF);
		};
		// Safety:
		// should be fine to dereference a NonNull and we are doing this to check the file type
		let header_ptr = unsafe { &*(ptr.as_ptr() as *const ElfHeader) };

		// Testing code
		// return Err(ELFError::UnsupportedOSABI(4));

		if &header_ptr.identifier[..4] != b"\x7FELF"
		/* magic */
		{
			return Err(ELFError::NotELF);
		}
		if header_ptr.identifier[4] != 2
		/* 64 bit */
		{
			return Err(ELFError::UnsupportedArchitectureWidth);
		}
		if header_ptr.identifier[5] != 1
		/* little endian */
		{
			return Err(ELFError::UnsupportedEndianness);
		}
		if header_ptr.identifier[6] != 1
		/* elf version */
		{
			return Err(ELFError::UnsupportedELFVersion);
		}
		if header_ptr.identifier[7] != 0
		/* system v */
		{
			return Err(ELFError::UnsupportedOSABI(header_ptr.identifier[7]));
		}
		if header_ptr.identifier[8] != 0
		/* abi version */
		{
			return Err(ELFError::UnsupportedOSABIVersion);
		}
		if &header_ptr.identifier[9..] != b"\x00\x00\x00\x00\x00\x00\x00"
		/* padding */
		{
			return Err(ELFError::Unknown);
		}
		if header_ptr.executable_type != ExecutableType::DYNAMIC {
			return Err(ELFError::UnsupportedExecutableType);
		}
		if header_ptr.machine != Machine::X86_64 {
			return Err(ELFError::UnsupportedMachine);
		}
		if header_ptr.version != 1 {
			return Err(ELFError::UnsupportedELFVersion);
		}
		if header_ptr.elf_header_size as usize != size_of::<ElfHeader>() {
			return Err(ELFError::UnsupportedELFVersion);
		}

		// Safety:
		// After all the checks we do it should eventually be safe
		Ok(Self(unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), len) }))
	}

	pub fn header(&self) -> &ElfHeader {
		// Safety:
		// Should be safe after the creation of this object succeeds
		unsafe { &*(self.0.as_ptr() as *const ElfHeader) }
	}
}

#[repr(transparent)]
#[derive(Default, PartialEq, Eq)]
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
#[derive(Default, PartialEq, Eq)]
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

#[repr(C)]
#[derive(Debug, Default)]
pub struct ElfHeader {
	/// e_ident:
	pub identifier: [u8; 16],
	/// e_type:
	pub executable_type: ExecutableType,
	/// e_machine:
	pub machine: Machine,
	/// e_version:
	pub version: u32,
	/// e_entry: Virtual address to which the system first transfers control if no entry point it holds 0.
	pub entry: Option<ptr::NonNull<u8>>,
	/// e_phoff: Program header table file offset in bytes if no program header table it holds 0.
	pub program_header_offset: Option<num::NonZeroUsize>,
	/// e_shoff: Section header table file offset in bytes if no section header table it holds 0.
	pub section_header_offset: Option<num::NonZeroUsize>,
	/// e_flags: Processor specific flags associated with file.
	pub flags: u32,
	/// e_ehsize: ELF header size in bytes.
	pub elf_header_size: u16,
	/// e_phentsize: Program header table entry size in bytes.
	pub program_header_entry_size: u16,
	/// e_phnum: Program header table number of entries.
	/// If number of program headers >= PN_XNUM(0xFFFF) then program_header_num is set to 0xFFFF and the actual number is in section_header_info field of the section header at index 0 otherwise section_header_info is 0
	pub program_header_num: Option<num::NonZeroU16>,
	/// e_shentsize: Section header entry size in bytes.
	pub section_header_entry_size: u16,
	/// e_shnum: Section header table number of entries.
	/// If entries >= 0xFF00 then has value of SHN_UNDEF(0) and actual number of header table entries is in size field of section header at index 0.
	pub section_header_num: Option<num::NonZeroU16>,
	/// e_shstrndx: Section header table index of string table.
	/// If no string table then it is SHN_UNDEF(0) and if entries >= SHN_LORESERVE(0xFF00) then it has the value SHN_XINDEX(0xFFFF) and the actual
	/// index is in the link field of section header at index 0 otherwise the link field contains 0.
	pub section_header_string_index: Option<num::NonZeroU16>,
}

impl ElfHeader {
	pub fn is_supported_and_valid(&self) -> bool {
		if self.identifier[..4] != *b"\x7FELF"
		/* magic */
		{
			return false;
		}
		if self.identifier[4] != 2
		/* 64 bit */
		{
			return false;
		}
		if self.identifier[5] != 1
		/* little endian */
		{
			return false;
		}
		if self.identifier[6] != 1
		/* elf version */
		{
			return false;
		}
		if self.identifier[7] != 0
		/* system v */
		{
			return false;
		}
		if self.identifier[8] != 0
		/* abi version */
		{
			return false;
		}
		if self.identifier[9..] != *b"\x00\x00\x00\x00\x00\x00\x00"
		/* padding */
		{
			return false;
		}
		if !(self.executable_type == ExecutableType::EXECUTABLE || self.executable_type == ExecutableType::DYNAMIC) {
			return false;
		}
		if self.machine != Machine::X86_64 {
			return false;
		}
		if self.version != 1 {
			return false;
		}
		if self.elf_header_size as usize != size_of::<ElfHeader>() {
			return false;
		}
		true
	}
}

#[non_exhaustive]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
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
#[derive(Default, Clone, Copy, PartialEq, Eq)]
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

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Elf64ProgramHeader {
	pub p_type: ProgramHeaderType,
	pub p_flags: ProgramHeaderFlags,
	pub p_offset: usize,
	pub p_vaddr: usize,
	pub p_paddr: usize,
	pub p_filesz: usize,
	pub p_memsz: usize,
	pub p_align: usize,
}

impl Elf64ProgramHeader {}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Elf32ProgramHeader {
	pub p_type: ProgramHeaderType,
	pub p_offset: u32,
	pub p_vaddr: u32,
	pub p_paddr: u32,
	pub p_filesz: u32,
	pub p_memsz: u32,
	pub p_flags: ProgramHeaderFlags,
	pub p_align: u32,
}

impl Elf32ProgramHeader {}

#[repr(transparent)]
#[derive(Default, PartialEq, Eq)]
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

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Elf64SectionHeader {
	/// Index into section header string table
	pub name: u32,
	pub section_header_type: SectionHeaderType,
	pub flags: u64,
	/// If appears in memory image of process gives address that it should reside at otherwise 0
	pub address: Option<num::NonZeroU64>,
	/// Gives byte offset from beginning of file to first byte in the section
	pub offset: u64,
	/// Sections size in bytes if it isn't type SHT_NOBITS
	pub size: u64,
	/// Section header index link that interpretation depends on the type of section
	pub link: u32,
	/// Holds extra information that interpretation depends on section type
	pub info: u32,
	pub address_align: u64,
	/// For sections that hold a table of fixed size entries this gives the size in bytes of the
	/// entry and 0 otherwise
	pub entry_size: Option<num::NonZeroU64>,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct Elf32SectionHeader {
	/// Index into section header string table
	pub name: u32,
	pub section_header_type: SectionHeaderType,
	pub flags: u32,
	/// If appears in memory image of process gives address that it should reside at otherwise 0
	pub address: Option<num::NonZeroU32>,
	/// Gives byte offset from beginning of file to first byte in the section
	pub offset: u32,
	/// Sections size in bytes if it isn't type SHT_NOBITS
	pub size: u32,
	/// Section header index link that interpretation depends on the type of section
	pub link: u32,
	/// Holds extra information that interpretation depends on section type
	pub info: u32,
	pub address_align: u32,
	/// For sections that hold a table of fixed size entries this gives the size in bytes of the
	/// entry and 0 otherwise
	pub entry_size: Option<num::NonZeroU32>,
}

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
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Elf64Dynamic {
	#[default]
	Null = 0,
	Needed {
		val: Elf64Xword,
	},
	PLTRELSZ {
		val: Elf64Xword,
	},
	PLTGOT {
		ptr: Elf64Addr,
	},
	HASH {
		ptr: Elf64Addr,
	},
	STRTAB {
		ptr: Elf64Addr,
	},
	SYMTAB {
		ptr: Elf64Addr,
	},
	RELA {
		ptr: Elf64Addr,
	},
	RELASZ {
		val: Elf64Xword,
	},
	RELAENT {
		val: Elf64Xword,
	},
	STRSZ {
		val: Elf64Xword,
	},
	SYMENT {
		val: Elf64Xword,
	},
	INIT {
		ptr: Elf64Addr,
	},
	FINI {
		ptr: Elf64Addr,
	},
	SONAME {
		val: Elf64Xword,
	},
	RPATH {
		val: Elf64Xword,
	},
	SYMBOLIC,
	REL {
		ptr: Elf64Addr,
	},
	RELSZ {
		val: Elf64Xword,
	},
	RELENT {
		val: Elf64Xword,
	},
	PLTREL {
		val: Elf64Xword,
	},
	DEBUG {
		ptr: Elf64Addr,
	},
	TEXTREL,
	JMPREL {
		ptr: Elf64Addr,
	},
	BindNow,
	InitArray {
		ptr: Elf64Addr,
	},
	FiniArray {
		ptr: Elf64Addr,
	},
	InitArraySize {
		val: Elf64Xword,
	},
	FiniArraySize {
		val: Elf64Xword,
	},
	RunPath {
		val: Elf64Xword,
	},
	Flags {
		val: Elf64Xword,
	},
	Encoding {
		val: Elf64Xword,
	},
	PreInitArray {
		ptr: Elf64Addr,
	},
	PreInitArraySize {
		val: Elf64Xword,
	},
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
	GnuHash {
		val: Elf64Xword,
	} = 0x6FFFFEF5,
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
	GnuFlags1 {
		val: Elf64Xword,
	} = 0x6FFFFFFB,
	GnuRelaCount {
		val: Elf64Xword,
	} = 0x6FFFFFF9,
	LOPROC {
		val: Elf64Xword,
	} = 0x70000000,
	HIPROC {
		val: Elf64Xword,
	} = 0x7FFFFFFF,
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
