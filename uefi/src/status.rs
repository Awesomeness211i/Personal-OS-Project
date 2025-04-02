#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Status(usize);
impl Status {
	pub fn into_result<T>(self, result: T) -> Result<T, Self> {
		match self {
			Self::SUCCESS => Ok(result),
			_ => Err(self),
		}
	}
	pub fn map<T, F: FnOnce() -> T>(self, op: F) -> Result<T, Self> {
		match self {
			Self::SUCCESS => Ok(op()),
			_ => Err(self),
		}
	}
	pub const SUCCESS: Status = Status(0);

	pub const UNKNOWN_GLYPH: Status = Status(Warn::UnknownGlyph as usize);
	pub const DELETE_FAILURE: Status = Status(Warn::DeleteFailure as usize);
	pub const WRITE_FAILURE: Status = Status(Warn::WriteFailure as usize);
	pub const WARN_BUFFER_TOO_SMALL: Status = Status(Warn::BufferTooSmall as usize);
	pub const STALE_DATA: Status = Status(Warn::StaleData as usize);
	pub const FILE_SYSTEM: Status = Status(Warn::FileSystem as usize);
	pub const RESET_REQUIRED: Status = Status(Warn::ResetRequired as usize);

	pub const LOAD: Status = Status(Error::Load as usize);
	pub const INVALID_PARAMETER: Status = Status(Error::InvalidParameter as usize);
	pub const UNSUPPORTED: Status = Status(Error::Unsupported as usize);
	pub const BAD_BUFFER_SIZE: Status = Status(Error::BadBufferSize as usize);
	pub const ERROR_BUFFER_TOO_SMALL: Status = Status(Error::BufferTooSmall as usize);
	pub const NOT_READY: Status = Status(Error::NotReady as usize);
	pub const ERROR_DEVICE: Status = Status(Error::Device as usize);
	pub const WRITE_PROTECTED: Status = Status(Error::WriteProtected as usize);
	pub const OUT_OF_RESOURCES: Status = Status(Error::OutOfResources as usize);
	pub const VOLUME_CORRUPTED: Status = Status(Error::VolumeCorrupted as usize);
	pub const VOLUME_FULL: Status = Status(Error::VolumeFull as usize);
	pub const NO_MEDIA: Status = Status(Error::NoMedia as usize);
	pub const MEDIA_CHANGED: Status = Status(Error::MediaChanged as usize);
	pub const NOT_FOUND: Status = Status(Error::NotFound as usize);
	pub const ACCESS_DENIED: Status = Status(Error::AccessDenied as usize);
	pub const NO_RESPONSE: Status = Status(Error::NoResponse as usize);
	pub const NO_MAPPING: Status = Status(Error::NoMapping as usize);
	pub const TIMEOUT: Status = Status(Error::Timeout as usize);
	pub const NOT_STARTED: Status = Status(Error::NotStarted as usize);
	pub const ALREADY_STARTED: Status = Status(Error::AlreadyStarted as usize);
	pub const ABORTED: Status = Status(Error::Aborted as usize);
	pub const ERROR_ICMP: Status = Status(Error::Icmp as usize);
	pub const ERROR_TFTP: Status = Status(Error::Tftp as usize);
	pub const PROTOCOL: Status = Status(Error::Protocol as usize);
	pub const INCOMPATIBLE_VERSION: Status = Status(Error::IncompatibleVersion as usize);
	pub const SECURITY_VIOLATION: Status = Status(Error::SecurityViolation as usize);
	pub const ERROR_CRC: Status = Status(Error::Crc as usize);
	pub const END_OF_MEDIA: Status = Status(Error::EndOfMedia as usize);
	pub const END_OF_FILE: Status = Status(Error::EndOfFile as usize);
	pub const INVALID_LANGUAGE: Status = Status(Error::InvalidLanguage as usize);
	pub const COMPROMISED_DATA: Status = Status(Error::CompromisedData as usize); pub const IP_ADDRESS_CONFLICT: Status = Status(Error::IPAddressConflict as usize);
	pub const ERROR_HTTP: Status = Status(Error::Http as usize);
}
#[repr(C)]
enum Warn {
	UnknownGlyph = 0x1,
	DeleteFailure = 0x2,
	WriteFailure = 0x3,
	BufferTooSmall = 0x4,
	StaleData = 0x5,
	FileSystem = 0x6,
	ResetRequired = 0x7,
}
#[repr(usize)]
#[allow(clippy::enum_clike_unportable_variant)]
enum Error {
	Load = Self::ERROR | 0x1,
	InvalidParameter = Self::ERROR | 0x2,
	Unsupported = Self::ERROR | 0x3,
	BadBufferSize = Self::ERROR | 0x4,
	BufferTooSmall = Self::ERROR | 0x5,
	NotReady = Self::ERROR | 0x6,
	Device = Self::ERROR | 0x7,
	WriteProtected = Self::ERROR | 0x8,
	OutOfResources = Self::ERROR | 0x9,
	VolumeCorrupted = Self::ERROR | 0xA,
	VolumeFull = Self::ERROR | 0xB,
	NoMedia = Self::ERROR | 0xC,
	MediaChanged = Self::ERROR | 0xD,
	NotFound = Self::ERROR | 0xE,
	AccessDenied = Self::ERROR | 0xF,
	NoResponse = Self::ERROR | 0x10,
	NoMapping = Self::ERROR | 0x11,
	Timeout = Self::ERROR | 0x12,
	NotStarted = Self::ERROR | 0x13,
	AlreadyStarted = Self::ERROR | 0x14,
	Aborted = Self::ERROR | 0x15,
	Icmp = Self::ERROR | 0x16,
	Tftp = Self::ERROR | 0x17,
	Protocol = Self::ERROR | 0x18,
	IncompatibleVersion = Self::ERROR | 0x19,
	SecurityViolation = Self::ERROR | 0x1A,
	Crc = Self::ERROR | 0x1B,
	EndOfMedia = Self::ERROR | 0x1C,
	EndOfFile = Self::ERROR | 0x1D,
	InvalidLanguage = Self::ERROR | 0x1E,
	CompromisedData = Self::ERROR | 0x1F,
	IPAddressConflict = Self::ERROR | 0x20,
	Http = Self::ERROR | 0x21,
}
impl Error {
	const ERROR: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1); // 8 represents bits in byte
}
