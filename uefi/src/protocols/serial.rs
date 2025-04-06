use super::{
	Protocol,
	GUID,
};

#[repr(C)]
pub struct SerialIOProtocol {
	pub revision: u32,
	/// this: IN
	pub reset: unsafe extern "efiapi" fn(this: *const Self),
	/// this: IN, baud_rate: IN, recieve_fifo_depth: IN, timeout: IN, parity: IN, data_bits: IN, stop_bits: IN
	pub set_attributes: unsafe extern "efiapi" fn(this: *const Self, baud_rate: u64, recieve_fifo_depth: u32, timeout: u32, parity: ParityType, data_bits: u8, stop_bits: StopBitsType),
	/// this: IN, control: IN
	pub set_control_bits: unsafe extern "efiapi" fn(this: *const Self, control: ControlBits),
	/// this: IN, control: OUT
	pub get_control_bits: unsafe extern "efiapi" fn(this: *const Self, control: *mut ControlBits),
	/// this: IN, buffer_size: IN OUT, buffer: IN
	pub write: unsafe extern "efiapi" fn(this: *const Self, buffer_size: *mut usize, buffer: *const ()),
	/// this: IN, buffer_size: IN OUT, buffer: IN
	pub read: unsafe extern "efiapi" fn(this: *const Self, buffer_size: *mut usize, buffer: *mut ()),
	pub mode: *mut SerialIOMode,
	pub device_type_guid: *const GUID,
}
impl Protocol for SerialIOProtocol {
	/// GUID: BB25CF6F-F1D4-11D2-9A0C-0090273FC1FD
	const GUID: GUID = GUID::new(0xBB25CF6F, 0xF1D4, 0x11D2, 0x9A0C_0090273FC1FD);
}

impl SerialIOProtocol {
	pub const REVISION: u64 = 0x00010000;
	pub const REVISION1P1: u64 = 0x00010001;

	pub const CLEAR_TO_SEND: ControlBits = ControlBits(0x0010);
	pub const DATA_SET_READY: ControlBits = ControlBits(0x0020);
	pub const RING_INDICATE: ControlBits = ControlBits(0x0040);
	pub const CARRIER_DETECT: ControlBits = ControlBits(0x0080);
	pub const REQUEST_TO_SEND: ControlBits = ControlBits(0x0002);
	pub const DATA_TERMINAL_READY: ControlBits = ControlBits(0x0001);
	pub const INPUT_BUFFER_EMPTY: ControlBits = ControlBits(0x0100);
	pub const OUTPUT_BUFFER_EMPTY: ControlBits = ControlBits(0x0200);
	pub const HARDWARE_LOOPBACK_ENABLE: ControlBits = ControlBits(0x1000);
	pub const SOFTWARE_LOOPBACK_ENABLE: ControlBits = ControlBits(0x2000);
	pub const HARDWARE_FLOW_CONTROL_ENABLE: ControlBits = ControlBits(0x4000);
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ControlBits(u32);

impl core::ops::BitOr for ControlBits {
	type Output = ControlBits;
	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
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
