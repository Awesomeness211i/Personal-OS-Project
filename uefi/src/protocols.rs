use crate::{
	Void,
	GUID,
	status::Status,
};

pub mod graphics;
pub mod string;
pub mod image;
pub mod debug;
pub mod acpi;
pub mod file;
pub mod path;
pub mod text;

pub trait Protocol {
	const GUID: GUID;
}

#[repr(C)]
pub struct DecompressProtocol {
	/// self: IN, source: IN, sourcesize: IN, destinationsize: OUT, scratchsize: OUT
	pub get_info: unsafe extern "efiapi" fn(&Self, source: *const Void, sourcesize: u32, destinationsize: &mut u32, scratchsize: &mut u32) -> Status,
	/// self: IN, source: IN, sourcesize: IN, destination: IN OUT, destinationsize: IN, scratch: IN OUT, scratchsize: IN
	pub decompress: unsafe extern "efiapi" fn(&Self,  source: *const Void, sourcesize: u32, destination: *mut Void, destinationsize: u32, scratch: *mut Void, scratchsize: u32) -> Status,
}
impl Protocol for DecompressProtocol {
	/// GUID: D8117CFE-94A6-11D4-9A3A-0090273FC14D
	const GUID: GUID = GUID::new(0xD8117CFE, 0x94A6, 0x11D4, 0x9A3A_0090273FC14D);
}

#[repr(C)]
pub struct BootManagerPolicyProtocol {
	pub revision: u64,
	/// this: IN, devicepath: IN, recursive: IN
	pub connectdevicepath: unsafe extern "efiapi" fn(this: &Self, devicepath: *const Void, recursive: bool) -> Status,
	/// this: IN, class: IN
	pub connectdeviceclass: unsafe extern "efiapi" fn(this: &Self, class: &GUID) -> Status,
}
impl Protocol for BootManagerPolicyProtocol {
	/// GUID: FEDF8E0C-E147-11E3-9903-B8E8562CBAFA
	const GUID: GUID = GUID::new(0xFEDF8E0C, 0xE147, 0x11E3, 0x9903_B8E8562CBAFA);
}
