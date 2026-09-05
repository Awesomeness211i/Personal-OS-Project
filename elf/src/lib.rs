#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]

use core::{
	error::Error,
	fmt::{
		Display,
		Formatter,
	},
};

use crate::{
	elf_header::{
		AbiVersion,
		Class,
		Data,
		EI_NIDENT,
		Elf32Header,
		Elf64Header,
		ElfHeader::{
			self,
		},
		ExecutableType,
		Identifier,
		IdentifierVersion,
		Machine,
		Magic,
		OsAbi,
		Version,
	},
	program_header::ProgramHeaderIterator,
};

pub mod elf;
pub mod elf_header;
pub mod program_header;
pub mod section_header;

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
	UnsupportedOsAbi(OsAbi),
	UnsupportedOSABIVersion(AbiVersion),
	UnsupportedExecutableType(ExecutableType),
	UnsupportedMachine(Machine),
	UnsupportedELFIdentifierVersion(IdentifierVersion),
	UnsupportedELFVersion(Version),
	UnsupportedEndianness(Data),
	UnsupportedArchitectureWidth(Class),
	ElfHeaderTooSmall(usize),
	ElfTooSmall(usize),
}
impl Display for ELFError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "{self:?}")
	}
}
impl Error for ELFError {}

#[derive(Debug)]
pub struct Elf<'a> {
	header: ElfHeader<'a>,
	file: &'a [u8],
}

impl<'elf> Elf<'elf> {
	pub fn new(file: &'elf [u8]) -> Result<Self, ELFError> {
		let Some((identifier, _)) = file.split_first_chunk::<EI_NIDENT>() else {
			return Err(ELFError::ElfTooSmall(file.len()));
		};
		// Safety:
		// should be fine to convert this into the Identifier type to do checking now
		let identifier = unsafe { &*(identifier as *const _ as *const Identifier) };

		/* magic */
		if identifier.magic != Magic::MAGIC {
			return Err(ELFError::NotELF);
		}

		let header = match identifier.class {
			Class::ELF_CLASS_32 => {
				const SIZE: usize = size_of::<Elf32Header>();
				let Some((header, _)) = file.split_first_chunk::<SIZE>() else {
					return Err(ELFError::ElfTooSmall(file.len()));
				};
				ElfHeader::ElfHeader32(unsafe { &*(header as *const _ as *const Elf32Header) })
			},
			Class::ELF_CLASS_64 => {
				const SIZE: usize = size_of::<Elf64Header>();
				let Some((header, _)) = file.split_first_chunk::<SIZE>() else {
					return Err(ELFError::ElfTooSmall(file.len()));
				};
				ElfHeader::ElfHeader64(unsafe { &*(header as *const _ as *const Elf64Header) })
			},
			_ => return Err(ELFError::UnsupportedArchitectureWidth(identifier.class)),
		};

		/* little endian */
		if identifier.data != Data::ELF_DATA_2LSB {
			return Err(ELFError::UnsupportedEndianness(identifier.data));
		}

		if identifier.version != IdentifierVersion::ELF_VERSION_CURRENT {
			return Err(ELFError::UnsupportedELFIdentifierVersion(identifier.version));
		}

		if header.version() != Version::ELF_VERSION_CURRENT {
			return Err(ELFError::UnsupportedELFVersion(header.version()));
		}

		/* system v */
		if identifier.os_abi != OsAbi::ELF_OS_ABI_NONE {
			return Err(ELFError::UnsupportedOsAbi(identifier.os_abi));
		}

		/* abi version */
		if identifier.abi_version != AbiVersion::ELF_ABI_VERSION_NONE {
			return Err(ELFError::UnsupportedOSABIVersion(identifier.abi_version));
		}

		/* padding */
		if &identifier.pad != b"\x00\x00\x00\x00\x00\x00\x00" {
			return Err(ELFError::Unknown);
		}

		if !(header.executable_type() == ExecutableType::DYNAMIC || header.executable_type() == ExecutableType::EXECUTABLE) {
			return Err(ELFError::UnsupportedExecutableType(header.executable_type()));
		}

		if header.machine() != Machine::X86_64 {
			return Err(ELFError::UnsupportedMachine(header.machine()));
		}

		match header {
			ElfHeader::ElfHeader32(_) => {
				if header.size() < size_of::<Elf32Header>() {
					return Err(ELFError::ElfHeaderTooSmall(header.size()));
				}
			},
			ElfHeader::ElfHeader64(_) => {
				if header.size() < size_of::<Elf64Header>() {
					return Err(ELFError::ElfHeaderTooSmall(header.size()));
				}
			},
		}

		Ok(Self { header, file })
	}

	pub fn ptr<T>(&self) -> *const T {
		self.file.as_ptr() as *const T
	}

	pub unsafe fn offset<T>(&self, offset: usize) -> &T {
		match self.file.get(offset) {
			Some(data) => {
				if size_of::<T>() + offset - 1 <= self.file.len() {
					unsafe { &*(data as *const _ as *const T) }
				} else {
					panic!("Out of bounds dummy: offset: {offset}, len: {}", self.file.len())
				}
			},
			None => panic!("Out of bounds dummy: offset: {offset}, len: {}", self.file.len()),
		}
	}

	pub fn header<'a>(&'a self) -> ElfHeader<'a> {
		self.header
	}

	pub fn program_headers<'a>(&'a self) -> ProgramHeaderIterator<'a> {
		ProgramHeaderIterator::new(self)
	}
}
