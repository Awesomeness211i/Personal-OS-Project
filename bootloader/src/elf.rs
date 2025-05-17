use core::{
	num,
	ops,
	ptr,
	slice,
};

pub enum TestError {
	Unknown,
	NotELF,
	UnsupportedOSABI,
	UnsupportedOSABIVersion,
	UnsupportedExecutableType,
	UnsupportedMachine,
	UnsupportedELFVersion,
	UnsupportedEndianness,
	UnsupportedArchitectureWidth,
}

pub struct Elf<'a>(&'a mut [u8]);
impl Elf<'_> {
	pub fn new(ptr: *mut u8, len: usize) -> Result<Self, TestError> {
		let Some(ptr) = ptr::NonNull::new(ptr) else {
			return Err(TestError::NotELF);
		};
		// Safety:
		// should be fine to dereference a NonNull and we are doing this to check the file type
		let header_ptr = unsafe { &*(ptr.as_ptr() as *const ElfHeader) };

		if &header_ptr.identifier[0..4] != b"\x7FELF"
		/* magic */
		{
			return Err(TestError::NotELF);
		}
		if header_ptr.identifier[4] != 2
		/* 64 bit */
		{
			return Err(TestError::UnsupportedArchitectureWidth);
		}
		if header_ptr.identifier[5] != 1
		/* little endian */
		{
			return Err(TestError::UnsupportedEndianness);
		}
		if header_ptr.identifier[6] != 1
		/* elf version */
		{
			return Err(TestError::UnsupportedELFVersion);
		}
		if header_ptr.identifier[7] != 0
		/* system v */
		{
			return Err(TestError::UnsupportedOSABI);
		}
		if header_ptr.identifier[8] != 0
		/* abi version */
		{
			return Err(TestError::UnsupportedOSABIVersion);
		}
		if &header_ptr.identifier[9..] != b"\x00\x00\x00\x00\x00\x00\x00"
		/* padding */
		{
			return Err(TestError::Unknown);
		}
		if header_ptr.executable_type != ExecutableType::DYNAMIC {
			return Err(TestError::UnsupportedExecutableType);
		}
		if header_ptr.machine != Machine::X86_64 {
			return Err(TestError::UnsupportedMachine);
		}
		if header_ptr.version != 1 {
			return Err(TestError::UnsupportedELFVersion);
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
	pub entry: usize,
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
	pub program_header_num: u16,
	/// e_shentsize: Section header entry size in bytes.
	pub section_header_entry_size: u16,
	/// e_shnum: Section header table number of entries.
	/// If entries >= 0xFF00 then has value of SHN_UNDEF(0) and actual number of header table entries is in sh_size field of section header at index 0.
	pub section_header_num: u16,
	/// e_shstrndx: Section header table index of string table if no string table then it is 0.
	pub section_header_string_index: Option<num::NonZeroU16>,
}

#[repr(C)]
pub struct Elf64ProgramHeader {
	pub p_type: u32,
	pub p_flags: u32,
	pub p_offset: usize,
	pub p_vaddr: usize,
	pub p_paddr: usize,
	pub p_filesz: usize,
	pub p_memsz: usize,
	pub p_align: usize,
}

#[repr(C)]
pub struct Elf32ProgramHeader {
	pub p_type: u32,
	pub p_offset: usize,
	pub p_vaddr: usize,
	pub p_paddr: usize,
	pub p_filesz: usize,
	pub p_memsz: usize,
	pub p_flags: u32,
	pub p_align: usize,
}

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
pub struct ElfSectionHeader {
	pub sh_name: u32,
	pub sh_type: SectionHeaderType,
	pub sh_flags: usize,
	pub sh_addr: usize,
	pub sh_offset: usize,
	pub sh_size: usize,
	pub sh_link: u32,
	pub sh_info: u32,
	pub sh_addralign: usize,
	pub sh_entsize: usize,
}
