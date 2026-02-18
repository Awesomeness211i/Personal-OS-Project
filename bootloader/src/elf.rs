use core::{
	error::Error,
	fmt::{
		Debug,
		Display,
		Formatter,
		Pointer,
	},
	num,
	ops,
	ptr,
	slice,
};

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
#[derive(PartialEq, Eq)]
pub struct ExecutableType(u16);
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
#[derive(PartialEq, Eq)]
pub struct Machine(u16);
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

#[non_exhaustive]
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
			_ => write!(f, "Unknown: {}", self.0),
		}
	}
}

#[repr(C)]
#[derive(Debug)]
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

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgramHeaderFlags(u32);

impl ProgramHeaderFlags {
	pub const X: Self = Self(0x1);
	pub const W: Self = Self(0x2);
	pub const R: Self = Self(0x4);
	pub const MASK_OS: Self = Self(0x0FF00000);
	pub const MASK_PROC: Self = Self(0xF0000000);
}

#[repr(C)]
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
pub struct SectionHeaderType(u32);
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
	pub const OS_SPECIFIC: ops::RangeInclusive<Self> = Self(0x60000000)..=Self(0x6fffffff);
	pub const PROCESSOR_SPECIFIC: ops::RangeInclusive<Self> = Self(0x70000000)..=Self(0x7fffffff);
	pub const APPLICATION_SPECIFIC: ops::RangeInclusive<Self> = Self(0x80000000)..=Self(0xffffffff);
}

#[repr(C)]
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
