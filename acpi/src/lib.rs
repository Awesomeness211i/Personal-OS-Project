#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]

#[repr(C)]
pub struct SystemDescriptionTableHeader {
	pub signature: [u8; 4],
	pub length: u32,
	pub revision: u8,
	pub checksum: u8,
	pub oem_id: [u8; 6],
	pub oem_table_id: [u8; 8],
	pub oem_revision: u32,
	pub creator_id: u32,
	pub creator_revision: u32,
}

#[repr(C)]
pub struct RootSystemDescriptionPointer {
	pub signature: [u8; 8],
	pub checksum: u8,
	pub oem_id: [u8; 6],
	pub revision: u8,
	rsdt_address: u32,
}

#[repr(C)]
pub struct RootSystemDescriptionPointerEx {
	pub rsdp: RootSystemDescriptionPointer,
	pub length: u32,
	xsdt_address: u64,
	pub extended_checksum: u8,
	reserved: [u8; 3],
}

#[repr(C)]
pub struct RootSystemDescriptionTable {
	header: SystemDescriptionTableHeader,
	entries: [u32],
}

#[repr(C)]
pub struct RootSystemDescriptionTableEx {
	header: SystemDescriptionTableHeader,
	entries: [u64],
}
