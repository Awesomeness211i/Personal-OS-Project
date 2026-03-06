#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Status(usize);

impl core::fmt::Display for Status {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match *self {
			Self::SUCCESS => write!(f, "Success"),
			Self::UNKNOWN_GLYPH => write!(f, "Unknown glyph"),
			Self::DELETE_FAILURE => write!(f, "Delete failure"),
			Self::WARN_BUFFER_TOO_SMALL => write!(f, "Warn Buffer Too Small"),
			Self::STALE_DATA => write!(f, "Stale Data"),
			Self::FILE_SYSTEM => write!(f, "File System"),
			Self::RESET_REQUIRED => write!(f, "Reset Required"),
			Self::LOAD => write!(f, "Load"),
			Self::INVALID_PARAMETER => write!(f, "Invalid Parameter"),
			Self::UNSUPPORTED => write!(f, "Unsupported"),
			Self::BAD_BUFFER_SIZE => write!(f, "Bad Buffer Size"),
			Self::BUFFER_TOO_SMALL => write!(f, "Buffer Too Small"),
			Self::NOT_READY => write!(f, "Not Ready"),
			Self::ERROR_DEVICE => write!(f, "Error Device"),
			Self::WRITE_PROTECTED => write!(f, "Write Protected"),
			Self::OUT_OF_RESOURCES => write!(f, "Out of Resources"),
			Self::VOLUME_CORRUPTED => write!(f, "Volume Corrupted"),
			Self::VOLUME_FULL => write!(f, "Volume Full"),
			Self::NO_MEDIA => write!(f, "No Media"),
			Self::MEDIA_CHANGED => write!(f, "Media Changed"),
			Self::NOT_FOUND => write!(f, "Not Found"),
			Self::ACCESS_DENIED => write!(f, "Access Denied"),
			Self::NO_RESPONSE => write!(f, "No Response"),
			Self::NO_MAPPING => write!(f, "No Mapping"),
			Self::TIMEOUT => write!(f, "Timeout"),
			Self::NOT_STARTED => write!(f, "Not Started"),
			Self::ALREADY_STARTED => write!(f, "Already Started"),
			Self::ABORTED => write!(f, "Aborted"),
			Self::ERROR_ICMP => write!(f, "Error ICMP"),
			Self::ERROR_TFTP => write!(f, "Error TFTP"),
			Self::PROTOCOL => write!(f, "Protocol"),
			Self::INCOMPATIBLE_VERSION => write!(f, "Incompatible Version"),
			Self::SECURITY_VIOLATION => write!(f, "SecurityViolation"),
			Self::ERROR_CRC => write!(f, "Error CRC"),
			Self::END_OF_MEDIA => write!(f, "End of Media"),
			Self::END_OF_FILE => write!(f, "End of File"),
			Self::INVALID_LANGUAGE => write!(f, "Invalid Language"),
			Self::COMPROMISED_DATA => write!(f, "Compromised Data"),
			Self::IP_ADDRESS_CONFLICT => write!(f, "IP Address Conflict"),
			Self::ERROR_HTTP => write!(f, "Error Http"),
			_ => write!(f, "Unknown: {}", self.0),
		}
	}
}

impl core::fmt::LowerHex for Status {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		core::fmt::LowerHex::fmt(&self.0, f)
	}
}

impl core::fmt::UpperHex for Status {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		core::fmt::UpperHex::fmt(&self.0, f)
	}
}

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
	pub const BUFFER_TOO_SMALL: Status = Status(Error::BufferTooSmall as usize);
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
	pub const COMPROMISED_DATA: Status = Status(Error::CompromisedData as usize);
	pub const IP_ADDRESS_CONFLICT: Status = Status(Error::IPAddressConflict as usize);
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
