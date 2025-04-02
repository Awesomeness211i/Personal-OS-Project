#![no_std]
#![feature(adt_const_params, extended_varargs_abi_support)]
#![deny(clippy::undocumented_unsafe_blocks)]
//! # UEFI
//! Library for interfacing with the UEFI specification

//! IN = Passed into function
//! OUT = Returned from function
//! OPTIONAL = Passing field to function is optional and NULL may be passed
//! CONST = Read only
//! EFIAPI = UEFI calling convention

#[cfg(feature = "allocator")]
extern crate alloc;

#[cfg(feature = "allocator")]
pub mod allocator;

mod strings;
pub mod memory;
pub mod protocols;
pub mod services;
pub mod status;
pub mod tables;
pub mod chars;

pub use chars::{
	Char8,
	Char16,
};
pub use strings::{
	CStr8,
	CStr16,
};
type Void = ();

/// Type just to interpret C like boolean this is needed only because hardware vendors tend to use
/// C like conventions for implementing booleans despite the standard saying booleans should only
/// be 0 or 1
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Bool {
	#[default]
	False,
	True(core::num::NonZeroU8),
}
impl Bool {
	pub const fn new(bool: bool) -> Self {
		// SAFETY:
		// This shouldn't cause any undefined behavior because bool is a byte long and the enum
		// should take care of the interpretation of the data inside of the bool if it is 0 then it
		// is False because True contains a NonZeroU8
		unsafe { core::mem::transmute(bool) }
	}
	pub const fn to_bool(&self) -> bool {
		match self {
			Bool::True(_) => true,
			Bool::False => false,
		}
	}
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Event(*mut ());
impl Event {
	pub const TIMER: u32 = 0x80000000;
	pub const RUNTIME: u32 = 0x40000000;
	pub const NOTIFY_WAIT: u32 = 0x00000100;
	pub const NOTIFY_SIGNAL: u32 = 0x00000200;
	pub const SIGNAL_EXIT_BOOT_SERVICES: u32 = 0x00000201;
	pub const SIGNAL_VIRTUAL_ADDRESS_CHANGE: u32 = 0x60000202;
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LogicalBlockAddress(u64);

#[repr(C)]
pub struct MasterBootRecordPartitionRecord {
	bootindicator: u8,
	starthead: u8,
	startsector: u8,
	starttrack: u8,
	osindicator: u8,
	endhead: u8,
	endsector: u8,
	endtrack: u8,
	startinglba: [u8; 4],
	sizeinlba: [u8; 4],
}

#[repr(C)]
pub struct MasterBootRecord {
	bootstrapcode: [u8; 440],
	uniquembrsignature: [u8; 4],
	unknown: [u8; 2],
	partition: [MasterBootRecordPartitionRecord; 4],
	signature: u16,
}

#[repr(C)]
pub struct EFIPartitionEntry {
	partitiontypeguid: GUID,
	uniquepartitionguid: GUID,
	startinglba: u64,
	endinglba: u64,
	attributes: u64,
	partitionname: [Char16; 36],
}

#[repr(C)]
pub struct EFIBlockTranslationTableInfoBlock {
	sig: [Char8; 16],
	uuid: GUID,
	parentuuid: GUID,
	flags: u32,
	major: u16,
	minor: u16,
	externallbasize: u32,
	externalnlba: u32,
	internallbasize: u32,
	internalnlba: u32,
	nfree: u32,
	infosize: u32,
	nextoff: u64,
	dataoff: u64,
	mapoff: u64,
	flogoff: u64,
	infooff: u64,
	unused: [Char8; 3968],
	checksum: u64,
}

#[repr(C)]
pub struct EFIBlockTranslationTableMapEntry {
	// #[bitfield(30)] postmaplba: u32,
	// #[bitfield(1)] error: u32,
	// #[bitfield(1)] zero: u32,
	fields: u32,
}

#[repr(C)]
pub struct EFIBlockTranslationTableFlog {
	lba0: u32,
	oldmap0: u32,
	newmap0: u32,
	seq0: u32,
	lba1: u32,
	oldmap1: u32,
	newmap1: u32,
	seq1: u32,
}

// #[repr(C)]
// pub struct LoadOption {
// 	attributes: u32,
// 	filepathlistlength: u16,
// 	description: [Char16],
// 	filepathlist: [protocols::path::DevicePathProtocol],
// 	optionaldata: [u8],
// }
// impl LoadOption {
// 	// All values 0x00000200-0x00001F00 are reserved
// 	pub const ACTIVE: u32 = 0x00000001;
// 	pub const FORCE_RECONNECT: u32 = 0x00000002;
// 	pub const HIDDEN: u32 = 0x00000008;
// 	pub const CATEGORY: u32 = 0x00001F00;
// 	pub const CATEGORY_BOOT: u32 = 0x00000000;
// 	pub const CATEGORY_APP: u32 = 0x00000100;
// }

// #[repr(C)]
// pub struct KeyOption {
// 	keydata: BootKeyData,
// 	bootoptioncrc: u32,
// 	bootoption: u16,
// 	keys: [protocols::text::InputKey],
// }
// #[repr(C)]
// pub union BootKeyData {
// 	// #[bitfield(8)] revision: u32,
// 	// #[bitfield(1)] shiftpressed: u32,
// 	// #[bitfield(1)] controlpressed: u32,
// 	// #[bitfield(1)] altpressed: u32,
// 	// #[bitfield(1)] logopressed: u32,
// 	// #[bitfield(1)] menupressed: u32,
// 	// #[bitfield(1)] sysreqpressed: u32,
// 	// #[bitfield(16)] reserved: u32,
// 	// #[bitfield(2)] inputkeycount: u32,
// 	options: u32,
// 	packedvalue: u32,
// }

/// GUIDs have the first 3 fields of data as little endian byte sequences
/// and the last 2 fields as big endian byte sequences there are practically
/// only 4 fields because of the last 2 big endian fields being combined
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct GUID {
	/// little endian byte sequence
	data1: u32,
	/// little endian byte sequence
	data2: u16,
	/// little endian byte sequence
	data3: u16,
	/// big endian byte sequence combining last 2 fields (2 bytes then 6 bytes)
	data4: u64,
}
impl GUID {
	const fn new(data1: u32, data2: u16, data3: u16, data4: u64) -> Self {
		Self {
			data1: data1.to_le(),
			data2: data2.to_le(),
			data3: data3.to_le(),
			data4: data4.to_be(),
		}
	}
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PhysicalAddress(u64);
impl PhysicalAddress {
	pub fn get(&self) -> u64 { self.0 }
	pub fn to_ptr<T>(&self) -> *mut T { self.0 as *mut T }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct VirtualAddress(u64);
impl VirtualAddress {
	pub fn get(&self) -> u64 { self.0 }
}

pub type IpV4Address = core::net::Ipv4Addr;
pub type IpV6Address = core::net::Ipv6Addr;

#[repr(C)]
#[derive(Clone, Copy)]
pub union IpAddress {
	pub addr: [u32; 4],
	pub v4: IpV4Address,
	pub v6: IpV6Address,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct MacAddress([u8; 32]);
