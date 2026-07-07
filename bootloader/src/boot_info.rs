use acpi::{
	RootSystemDescriptionPointer,
	RootSystemDescriptionPointerEx,
};

use crate::address_space::AddressSpace;

#[repr(C)]
pub struct SMBIOSTable_64 {
	pub anchor_string: [u8; 5],
	pub checksum: u8,
	pub entrypoint_length: u8,
	pub version: [u8; 4],
	reserved: u8,
	pub maximum_size: u32,
	pub structure_table_address: u64,
}

#[repr(C)]
pub struct MappingInfo {
	pub physical_address: uefi::PhysicalAddress,
	pub virtual_address: uefi::VirtualAddress,
	pub len: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct KernelDataHeader {
	/// size of structure in bytes
	pub graphics_len: usize,
	pub graphics_ptr: *mut uefi::protocols::graphics::GraphicsPixel,
	pub graphics_format: uefi::protocols::graphics::GraphicsPixelFormat,
	pub root_system_description_pointer: RootSystemDescriptionPointer,
	pub root_system_description_pointer_ex: RootSystemDescriptionPointerEx,
	pub system_table: uefi::tables::SystemTable,
	pub address_space: AddressSpace,
	pub virtual_mappings_count: usize,
	// pub memorymap: uefi::memory::MemoryMap,
	// pub imagehandle: uefi::Handle,
}

#[repr(C)]
pub struct KernelData {
	pub header: KernelDataHeader,
	pub virtual_mappings: [MappingInfo],
}
