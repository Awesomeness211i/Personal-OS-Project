use crate::{
	Void,
	GUID,
	Event,
	CStr16,
	Char16,
	tables::Time,
	status::Status,
	protocols::Protocol,
};

#[repr(C)]
pub struct SimpleFileSystemProtocol {
	pub revision: u32,
	pub open_volume: unsafe extern "efiapi" fn(this: *const Self, root: *mut *const FileProtocol) -> Status,
}
impl Protocol for SimpleFileSystemProtocol {
	/// GUID: 964E5B22-6459-11D2-8E39-00A0C969723B
	const GUID: GUID = GUID::new(0x964E5B22, 0x6459, 0x11D2, 0x8E39_00A0C969723B);
}

#[repr(C)]
pub struct FileSystemInfo {
	pub size: u64,
	pub read_only: bool,
	pub volume_size: u64,
	pub free_space: u64,
	pub block_size: u32,
	pub volume_label: CStr16,
}
impl FileSystemInfo {
	pub const GUID: GUID = GUID::new(0x09576E93, 0x6D3F, 0x11D2, 0x8E39_00A0C969723B);
}

#[repr(C)]
pub struct FileSystemVolumeLabel {
	pub volumelabel: CStr16,
}
impl FileSystemVolumeLabel {
	pub const GUID: GUID = GUID::new(0xDB47D7D3, 0xFE81, 0x11D3, 0x9A35_0090273FC14D);
}

#[repr(C)]
pub struct FileInfo {
	pub size: u64,
	pub file_size: u64,
	pub physical_size: u64,
	pub create_time: Time,
	pub last_access_time: Time,
	pub modification_time: Time,
	pub attribute: u64,
	pub file_name: CStr16,
}
impl FileInfo {
	pub const GUID: GUID = GUID::new(0x09576E92, 0x6D3F, 0x11D2, 0x8E39_00A0C969723B);
	pub const READ_ONLY: u64 = 0x0000000000000001;
	pub const HIDDEN: u64 = 0x0000000000000002;
	pub const SYSTEM: u64 = 0x0000000000000004;
	pub const RESERVED: u64 = 0x0000000000000008;
	pub const DIRECTORY: u64 = 0x0000000000000010;
	pub const ARCHIVE: u64 = 0x0000000000000020;
	pub const VALID_ATTR: u64 = 0x0000000000000037;
}

#[repr(C)]
pub struct FileIOToken {
	pub event: Event,
	pub status: Status,
	pub buffersize: usize,
	pub buffer: *const Void,
}

#[repr(C)]
pub struct FileProtocol {
	pub revision: u32,
	/// this: IN, newhandle: OUT, filename: IN, openmode: IN, attributes: IN
	pub open: unsafe extern "efiapi" fn(this: *const Self, newhandle: *mut *const Self, filename: *const Char16, openmode: u64, attributes: u64) -> Status,
	/// this: IN
	pub close: unsafe extern "efiapi" fn(this: *const Self) -> Status,
	/// this: IN
	pub delete: unsafe extern "efiapi" fn(this: *const Self) -> Status,
	/// this: IN, buffersize: IN OUT, buffer: OUT
	pub read: unsafe extern "efiapi" fn(this: *const Self, buffersize: *mut usize, buffer: *mut ()) -> Status,
	/// this: IN, buffersize: IN OUT, buffer: IN
	pub write: unsafe extern "efiapi" fn(this: *const Self, buffersize: *mut usize, buffer: *const ()) -> Status,
	/// this: IN, position: OUT
	pub get_position: unsafe extern "efiapi" fn(this: *const Self, position: *mut u64) -> Status,
	/// this: IN, position: IN
	pub set_position: unsafe extern "efiapi" fn(this: *const Self, position: u64) -> Status,
	/// this: IN, infotype: IN, buffersize: IN OUT, buffer: OUT
	pub get_info: unsafe extern "efiapi" fn(this: *const Self, infotype: *const GUID, buffersize: *mut usize, buffer: *mut ()) -> Status,
	/// this: IN, infotype: IN, buffersize: IN, buffer: IN
	pub set_info: unsafe extern "efiapi" fn(this: *const Self, infotype: *const GUID, buffersize: usize, buffer: *const ()) -> Status,
	/// this: IN
	pub flush: unsafe extern "efiapi" fn(this: *const Self) -> Status,
	/// this: IN, newhandle: OUT, filename: IN, openmode: IN, attributes: IN, token: IN OUT
	pub open_ex: unsafe extern "efiapi" fn(this: *const Self, newhandle: *mut *const Self, filename: *const Char16, openmode: u64, attributes: u64, token: *mut FileIOToken) -> Status,
	/// this: IN, token: IN OUT
	pub read_ex: unsafe extern "efiapi" fn(this: *const Self, token: *mut FileIOToken) -> Status,
	/// this: IN, token: IN OUT
	pub write_ex: unsafe extern "efiapi" fn(this: *const Self, token: *mut FileIOToken) -> Status,
	/// this: IN, token: IN OUT
	pub flush_ex: unsafe extern "efiapi" fn(this: *const Self, token: *mut FileIOToken) -> Status,
}
impl FileProtocol {
	pub const REVISION: u32 = 0x00010000;
	pub const REVISION2: u32 = 0x00020000;
	pub const LATEST_REVISION: u32 = Self::REVISION2;
	pub const MODE_READ: u64 = 0x0000000000000001;
	pub const MODE_WRITE: u64 = 0x0000000000000002;
	pub const MODE_CREATE: u64 = 0x8000000000000000;
}
