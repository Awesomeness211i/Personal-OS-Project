use crate::{
	GUID,
	PhysicalAddress,

	status::Status,
};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct GraphicsOutputBLTOperation(u32);
impl GraphicsOutputBLTOperation {
	pub const VIDEO_FILL: Self = Self(0x00);
	pub const VIDEO_TO_BUFFER: Self = Self(0x01);
	pub const BUFFER_TO_VIDEO: Self = Self(0x02);
	pub const VIDEO_TO_VIDEO: Self = Self(0x03);
	pub const GRAPICS_OUTPUT_BLT_OPERATION_MAX: Self = Self(0x04);
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct GraphicsPixelFormat(u32);
impl GraphicsPixelFormat {
	pub const RED_GREEN_BLUE_RESERVED_8BIT_PER_COLOR: Self = Self(0x00);
	pub const BLUE_GREEN_RED_RESERVED_8BIT_PER_COLOR: Self = Self(0x01);
	pub const BIT_MASK: Self = Self(0x02);
	pub const BLT_ONLY: Self = Self(0x03);
}

#[repr(C)]
pub struct GraphicsOutputProtocol {
	/// this: IN, modenumber: IN, sizeofinfo: OUT, info: OUT
	pub query_mode: unsafe extern "efiapi" fn(this: *const Self, modenumber: u32, sizeofinfo: *mut usize, info: *const *mut GraphicsOutputModeInformation) -> Status,
	pub set_mode: unsafe extern "efiapi" fn(*const Self, modenumber: u32) -> Status,
	/// this: IN, buffer: IN OUT, operation: IN, sourcex: IN, sourcey: IN, destx: IN, desty: IN, width: IN, height: IN, delta: IN
	pub blt: unsafe extern "efiapi" fn(this: *const Self, buffer: *mut GraphicsPixel, operation: GraphicsOutputBLTOperation, sourcex: usize, sourcey: usize, destx: usize, desty: usize, width: usize, height: usize, delta: Option<core::num::NonZeroUsize>) -> Status,
	pub mode: *mut GraphicsOutputProtocolMode,
}
impl GraphicsOutputProtocol {
	pub fn grapics_color(color: u32, mask: &PixelBitmask) -> GraphicsPixel {
		GraphicsPixel {
			blue: (color >> mask.bluemask.trailing_zeros()) as u8,
			green: (color >> mask.greenmask.trailing_zeros()) as u8,
			red: (color >> mask.redmask.trailing_zeros()) as u8,
			reserved: (color >> mask.reservedmask.trailing_zeros()) as u8,
		}
	}
}
impl super::Protocol for GraphicsOutputProtocol {
	/// GUID: 9042A9DE-23DC-4A38-96FB-7ADED080516A
	const GUID: GUID = GUID::new(0x9042A9DE, 0x23DC, 0x4A38, 0x96FB_7ADED080516A);
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GraphicsOutputProtocolMode {
	pub max_mode: u32,
	pub mode: u32,
	pub info: *const GraphicsOutputModeInformation,
	pub sizeofinfo: usize,
	pub framebuffer_base: PhysicalAddress,
	pub framebuffer_size: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GraphicsOutputModeInformation {
	pub version: u32,
	pub horizontalresolution: u32,
	pub verticalresolution: u32,
	pub pixelformat: GraphicsPixelFormat,
	pub pixelinfo: PixelBitmask,
	pub pixelsperscanline: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct PixelBitmask {
	redmask: u32,
	greenmask: u32,
	bluemask: u32,
	reservedmask: u32,
}
impl PixelBitmask {
	pub fn new(red: u32, green: u32, blue: u32, reserved: u32) -> Self {
		Self {
			redmask: red,
			greenmask: green,
			bluemask: blue,
			reservedmask: reserved,
		}
	}
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct GraphicsPixel {
	pub blue: u8,
	pub green: u8,
	pub red: u8,
	reserved: u8,
}

impl GraphicsPixel {
	pub const fn new(red: u8, green: u8, blue: u8, reserved: u8) -> Self {
		Self {
			blue,
			green,
			red,
			reserved
		}
	}
}
