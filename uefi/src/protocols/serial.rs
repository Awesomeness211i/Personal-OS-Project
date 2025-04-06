use super::{Protocol, GUID};

#[repr(C)]
pub struct SerialIOProtocol {
	pub revision: u32,
	pub reset: unsafe extern "efiapi" fn(),
	pub set_attributes: unsafe extern "efiapi" fn(),
	pub set_control: unsafe extern "efiapi" fn(),
	pub get_control: unsafe extern "efiapi" fn(),
	pub write: unsafe extern "efiapi" fn(),
	pub read: unsafe extern "efiapi" fn(),
	pub mode: *mut SerialIOMode,
	pub device_type_guid: *const GUID,
}
impl Protocol for SerialIOProtocol {
	/// GUID: BB25CF6F-F1D4-11D2-9A0C-0090273FC1FD
	const GUID: GUID = GUID::new(0xBB25CF6F, 0xF1D4, 0x11D2, 0x9A0C_0090273FC1FD);
}

impl SerialIOProtocol {

}

#[repr(C)]
pub struct SerialIOMode {
	pub constrol_mask: u32,
	pub timeout: u32,
	pub baud_rate: u64,
	pub recieve_fifo_depth: u32,
	pub data_bits: u32,
	pub parity: ParityType,
	pub stop_bits: StopBitsType,
}

#[repr(C)]
pub enum ParityType {
	DefaultParity,
	NoParity,
	EvenParity,
	OddParity,
	MarkParity,
	SpaceParity,
}

#[repr(C)]
pub enum StopBitsType {
	DefaultStopBits,
	OneStopBit, // 1 stop bit
	OneFiveStopBits, // 1.5 stop bits
	TwoStopBits, // 2 stop bits
}
