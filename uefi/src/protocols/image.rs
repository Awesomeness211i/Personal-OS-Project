use crate::{
	Void,
	GUID,
	status::Status,
	tables::SystemTable,
	memory::MemoryType,
};

#[repr(C)]
pub struct LoadedImageProtocol {
	pub revision: u32,
	parent_handle: *const (),
	system_table: *const SystemTable,
	pub device_handle: *const (),
	file_path: *const super::path::DevicePathProtocol,
	reserved: *const (),
	load_options_size: u32,
	load_options: *const (),
	image_base: *const (),
	image_size: u64,
	image_code_type: MemoryType,
	image_data_type: MemoryType,
	/// handle: IN
	unload: Option<unsafe extern "efiapi" fn(handle: &Void) -> Status>,
}
impl super::Protocol for LoadedImageProtocol {
	/// GUID: 5B1B31A1-9562-11D2-8E3F-00A0C969723B
	const GUID: GUID = GUID::new(0x5B1B31A1, 0x9562, 0x11D2, 0x8E3F_00A0C969723B);
}

/// Not sure if this should exist yet
#[repr(C)]
pub struct LoadedImageDevicePathProtocol(pub super::path::PathType);
impl super::Protocol for LoadedImageDevicePathProtocol {
	/// GUID: BC62157E-3E33-4FEC-9920-2D3B36D750DF
	const GUID: GUID = GUID::new(0xBC62157E, 0x3E33, 0x4FEC, 0x9920_2D3B36D750DF);
}
