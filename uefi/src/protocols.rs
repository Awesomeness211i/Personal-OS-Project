use crate::{
	GUID,
	status::Status,
};

pub mod acpi;
pub mod debug;
pub mod file;
pub mod graphics;
pub mod image;
pub mod path;
pub mod serial;
pub mod string;
pub mod text;

pub trait HasGUID {
	const GUID: GUID;
}

pub unsafe trait Protocol: HasGUID {}

#[repr(C)]
pub struct DecompressProtocol {
	/// this: IN, source: IN, sourcesize: IN, destinationsize: OUT, scratchsize: OUT
	pub get_info: unsafe extern "efiapi" fn(this: *const Self, source: *const (), sourcesize: u32, destinationsize: *mut u32, scratchsize: *mut u32) -> Status,
	/// this: IN, source: IN, sourcesize: IN, destination: IN OUT, destinationsize: IN, scratch: IN OUT, scratchsize: IN
	pub decompress: unsafe extern "efiapi" fn(this: *const Self, source: *const (), sourcesize: u32, destination: *mut (), destinationsize: u32, scratch: *mut (), scratchsize: u32) -> Status,
}
impl HasGUID for DecompressProtocol {
	/// GUID: D8117CFE-94A6-11D4-9A3A-0090273FC14D
	const GUID: GUID = GUID::new(0xD8117CFE, 0x94A6, 0x11D4, 0x9A3A_0090273FC14D);
}
unsafe impl Protocol for DecompressProtocol {}

#[repr(C)]
pub struct BootManagerPolicyProtocol {
	pub revision: u64,
	/// this: IN, devicepath: IN, recursive: IN
	pub connectdevicepath: unsafe extern "efiapi" fn(this: *const Self, devicepath: *const (), recursive: bool) -> Status,
	/// this: IN, class: IN
	pub connectdeviceclass: unsafe extern "efiapi" fn(this: *const Self, class: *const GUID) -> Status,
}
impl HasGUID for BootManagerPolicyProtocol {
	/// GUID: FEDF8E0C-E147-11E3-9903-B8E8562CBAFA
	const GUID: GUID = GUID::new(0xFEDF8E0C, 0xE147, 0x11E3, 0x9903_B8E8562CBAFA);
}
unsafe impl Protocol for BootManagerPolicyProtocol {}
