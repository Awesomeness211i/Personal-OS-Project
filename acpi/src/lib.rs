#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]

#[repr(C)]
pub struct RootSystemDescriptionPointerEx {
	pub rsdp: RootSystemDescriptionPointer,
	pub length: u32,
	pub xsdt_address: u64,
	pub extended_checksum: u8,
	reserved: [u8; 3],
}

#[repr(C)]
pub struct RootSystemDescriptionPointer {
	pub signature: [u8; 8],
	pub checksum: u8,
	pub oem_id: [u8; 6],
	pub revision: u8,
	pub rsdt_address: u32,
}
