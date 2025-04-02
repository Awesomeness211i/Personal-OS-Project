use super::{
	Bool,
	GUID,
	Void,
	Char16,
	services,
	protocols::text,
};

// #[derive(core::marker::ConstParamTy, PartialEq, Eq)]
// enum TableStates {
// 	Boot,
// 	Runtime,
// }
// #[derive(core::marker::ConstParamTy, PartialEq, Eq)]
// pub struct TableState(TableStates);
#[allow(non_snake_case)]
pub mod TableStates {
	pub trait TableState {}
	pub struct Boot;
	impl TableState for Boot {}
	pub struct Runtime;
	impl TableState for Runtime {}
}

#[repr(C)]
pub struct TableHeader {
	pub signature: u64,
	pub revision: u32,
	pub headersize: u32,
	pub crc32: u32,
	reserved: u32,
}

#[repr(C)]
pub struct ConfigurationTable {
	vendorguid: GUID,
	vendortable: *const Void,
}
impl ConfigurationTable {
	/// GUID: 8868E871-E4F1-11D3-BC22-0080C73C8881
	pub const EFI_ACPI_20_TABLE: GUID = GUID::new(0x8868E871, 0xE4F1, 0x11D3, 0xBC22_0080C73C8881);
	/// GUID: EB9D2D30-2D88-11D3-9A16-0090273FC14D
	pub const ACPI_10_TABLE: GUID = GUID::new(0xEB9D2D30, 0x2D88, 0x11D3, 0x9A16_0090273FC14D);
	/// GUID: EB9D2D32-2D88-11D3-9A16-0090273FC14D
	pub const SAL_SYSTEM_TABLE: GUID = GUID::new(0xEB9D2D32, 0x2D88, 0x11D3, 0x9A16_0090273FC14D);
	/// GUID: EB9D2D31-2D88-11D3-9A16-0090273FC14D
	pub const SMBIOS_TABLE: GUID = GUID::new(0xEB9D2D31, 0x2D88, 0x11D3, 0x9A16_0090273FC14D);
	/// GUID: F2FD1544-9794-4A2C-992E-E5BBCF20E394
	pub const SMBIOS3_TABLE: GUID = GUID::new(0xF2FD1544, 0x9794, 0x4A2C, 0x992E_E5BBCF20E394);
	/// GUID: EB9D2D2F-2D88-11D3-9A16-0090273FC14D
	pub const MPS_TABLE: GUID = GUID::new(0xEB9D2D2F, 0x2D88, 0x11D3, 0x9A16_0090273FC14D);
	/// GUID: EB9D2D30-2D88-11D3-9A16-0090273FC14D
	pub const EFI_ACPI_TABLE: GUID = GUID::new(0xEB9D2D30, 0x2D88, 0x11D3, 0x9A16_0090273FC14D);
	/// GUID: EB9D2D30-2D88-11D3-9A16-0090273FC14D
	pub const ACPI_TABLE: GUID = GUID::new(0xEB9D2D30, 0x2D88, 0x11D3, 0x9A16_0090273FC14D);
}

#[repr(C)]
pub struct CapsuleHeader {
	pub capsuleguid: GUID,
	pub headersize: u32,
	pub flags: u32,
	pub capsuleimagesize: u32,
}

#[repr(C)]
pub struct RTPropertiesTable {
	version: u16,
	length: u16,
	runtimeservicessupported: u32,
}
impl RTPropertiesTable {
	pub const VERSION: u8 = 0x1;
	pub const GUID: GUID = GUID::new(0xEB66918A, 0x7EEF, 0x402A, 0x842E_931D21C38AE9);
	pub const SUPPORTED_GET_TIME: u32 = 0x0001;
	pub const SUPPORTED_SET_TIME: u32 = 0x0002;
	pub const SUPPORTED_GET_WAKEUP_TIME: u32 = 0x0004;
	pub const SUPPORTED_SET_WAKEUP_TIME: u32 = 0x0008;
	pub const SUPPORTED_GET_VARIABLE: u32 = 0x0010;
	pub const SUPPORTED_GET_NEXT_VARIABLE_NAME: u32 = 0x0020;
	pub const SUPPORTED_SET_VARIABLE: u32 = 0x0040;
	pub const SUPPORTED_SET_VIRTUAL_ADDRESS_MAP: u32 = 0x0080;
	pub const SUPPORTED_CONVERT_POINTER: u32 = 0x0100;
	pub const SUPPORTED_GET_NEXT_HIGH_MONOTONIC_COUNT: u32 = 0x0200;
	pub const SUPPORTED_RESET_SYSTEM: u32 = 0x0400;
	pub const SUPPORTED_UPDATE_CAPSULE: u32 = 0x0800;
	pub const SUPPORTED_QUERY_CAPSULE_CAPABILITIES: u32 = 0x1000;
	pub const SUPPORTED_QUERY_VARIABLE_INFO: u32 = 0x2000;
}
#[repr(C)]
#[deprecated]
pub struct PropertiesTable {
	version: u32,
	length: u32,
	memoryprotectionattribute: u64,
}
#[repr(C)]
pub struct MemoryAttributesTable {
	version: u32,
	numentries: u32,
	descriptorsize: u32,
	flags: u32,
}
impl MemoryAttributesTable {
	pub const GUID: GUID = GUID::new(0xDCFA911D, 0x26EB, 0x469F, 0xA220_38B7DC461220);
}
#[repr(C)]
pub struct ConformanceProfilesTable {
	version: u16,
	numprofiles: u16,
}
impl ConformanceProfilesTable {
	pub const GUID: GUID = GUID::new(0x36122546, 0xF7E7, 0x4C8F, 0xBD9B_EB8525B50C0B);
}

#[repr(C)]
pub struct SystemTable {
	pub header: TableHeader,
	pub firmware_vendor: *const Char16,
	pub firmware_revision: u32,
	pub console_in_handle: *const (),
	pub console_in: *const text::SimpleTextInputProtocol,
	pub console_out_handle: *const (),
	pub console_out: *const text::SimpleTextOutputProtocol,
	pub std_err_handle: *const (),
	pub std_err: *const text::SimpleTextOutputProtocol,
	pub runtime_services: *const services::RuntimeServices,
	pub boot_services: *const services::BootServices,
	pub num_table_entries: usize,
	pub configuration_tables: *const ConfigurationTable,
}
impl SystemTable {
	pub const SIGNATURE: u64 = 0x5453595320494249;
	pub const REVISION: u64 = Self::REVISION_2_100;
	pub const REVISION_2_100: u64 = ((2 << 16) | 100);
	pub const REVISION_2_90: u64 = ((2 << 16) | 90);
	pub const REVISION_2_80: u64 = ((2 << 16) | 80);
	pub const REVISION_2_70: u64 = ((2 << 16) | 70);
	pub const REVISION_2_60: u64 = ((2 << 16) | 60);
	pub const REVISION_2_50: u64 = ((2 << 16) | 50);
	pub const REVISION_2_40: u64 = ((2 << 16) | 40);
	pub const REVISION_2_31: u64 = ((2 << 16) | 31);
	pub const REVISION_2_30: u64 = ((2 << 16) | 30);
	pub const REVISION_2_20: u64 = ((2 << 16) | 20);
	pub const REVISION_2_10: u64 = ((2 << 16) | 10);
	#[allow(clippy::identity_op)]
	pub const REVISION_2_00: u64 = ((2 << 16) | 00);
	pub const REVISION_1_10: u64 = ((1 << 16) | 10);
	pub const REVISION_1_02: u64 = ((1 << 16) | 2);
	pub const SPECIFICATION_VERSION: u64 = Self::REVISION;
}

#[repr(C)]
#[derive(Default)]
pub struct Time {
	pub year: u16,
	pub month: u8,
	pub day: u8,
	pub hour: u8,
	pub minute: u8,
	pub second: u8,
	pad1: u8,
	pub nanosecond: u32,
	pub timezone: i16,
	pub daylight: u8,
	pad2: u8,
}

#[repr(C)]
#[derive(Default)]
pub struct TimeCapabilities {
	pub resolution: u32,
	pub accuracy: u32,
	pub setstozero: Bool,
}
