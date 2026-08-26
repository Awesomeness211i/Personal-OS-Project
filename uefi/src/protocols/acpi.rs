use crate::{
	GUID,
	protocols::{
		HasGUID,
		Protocol,
	},
	status::Status,
};

#[repr(C)]
pub struct AcpiTableProtocol {
	pub installacpitable: unsafe extern "efiapi" fn(this: *mut Self, acpitablebuffer: *mut (), acpitablebuffersize: usize, tablekey: *mut usize) -> Status,
	pub uninstallacpitable: unsafe extern "efiapi" fn(this: *mut Self, tablekey: usize) -> Status,
}

unsafe impl Protocol for AcpiTableProtocol {}
impl HasGUID for AcpiTableProtocol {
	/// GUID: FFE06BBD-6107-46A6-7BB2-5A9C7EC5275C
	const GUID: GUID = GUID::new(0xFFE06BDD, 0x6107, 0x46A6, 0x7BB2_5A9C7EC5275C);
}
