#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct ExecutableType(u16);
impl ExecutableType {
	pub const NONE: Self = Self(0x00);
	pub const RELOCATABLE: Self = Self(0x01);
	pub const EXECUTABLE: Self = Self(0x02);
	pub const DYNAMIC: Self = Self(0x03);
	pub const CORE: Self = Self(0x04);
	pub const OS_SPECIFIC: core::ops::RangeInclusive<Self> = Self(0xFE00)..=Self(0xFEFF);
	pub const PROCESSOR_SPECIFIC: core::ops::RangeInclusive<Self> = Self(0xFF00)..=Self(0xFFFF);
}

#[repr(C)]
pub struct ElfHeader {
	pub identifier: [u8; 16],
	pub executable_type: ExecutableType,
	pub machine: u16,
	pub version: u32,
	pub entry: usize,
	pub phoff: usize,
	pub shoff: usize,
	pub flags: u32,
	pub ehsize: u16,
	pub phentsize: u16,
	pub phnum: u16,
	pub shentsize: u16,
	pub shnum: u16,
	pub shstrndx: u16,
}

#[repr(C)]
pub struct ElfProgramHeader {
	pub p_type: u32,
	#[cfg(target_arch = "x86_64")]
	pub p_flags: u32,
	pub p_offset: usize,
	pub p_vaddr: usize,
	pub p_paddr: usize,
	pub p_filesz: usize,
	pub p_memsz: usize,
	#[cfg(target_arch = "x86")]
	pub p_flags: u32,
	pub p_align: usize,
}

#[repr(C)]
pub struct ElfSectionHeader {
	pub sh_name: u32,
	pub sh_type: u32,
	pub sh_flags: usize,
	pub sh_addr: usize,
	pub sh_offset: usize,
	pub sh_size: usize,
	pub sh_link: u32,
	pub sh_info: u32,
	pub sh_addralign: usize,
	pub sh_entsize: usize,
}
