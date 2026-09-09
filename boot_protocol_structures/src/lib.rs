#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
// #![warn(missing_docs)]

pub mod address_space;
pub mod debug_print;

use core::ffi::c_void;

use acpi::{
	RootSystemDescriptionPointer,
	RootSystemDescriptionPointerEx,
};

use crate::address_space::AddressSpace;

#[repr(C)]
#[derive(Debug)]
pub struct KernelData {
	pub version_tag: usize,
	/// Size of this structure in bytes
	pub size: usize,
	pub memory_map: *mut c_void,
	pub descriptor_size: usize,
	pub memory_map_size: usize,
	pub map_key: usize,
	pub descriptor_version: u32,
	pub stack_page_count: usize,
	pub trampoline_page: uefi::PhysicalAddress,
	pub address_space: AddressSpace,
	pub system_table: uefi::SystemTablePointer<uefi::RuntimeServices>,
	pub root_system_description_pointer: RootSystemDescriptionPointer,
	pub root_system_description_pointer_ex: RootSystemDescriptionPointerEx,
}

impl KernelData {
	pub const CURRENT_VERSION: usize = 1;
}

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

// #[repr(C)]
// pub struct MappingInfo {
// 	pub physical_address: uefi::PhysicalAddress,
// 	pub virtual_address: uefi::VirtualAddress,
// 	pub len: usize,
// }

// #[repr(C)]
// pub struct KernelData {
// 	pub header: KernelDataHeader,
// 	pub virtual_mappings: [MappingInfo],
// }
