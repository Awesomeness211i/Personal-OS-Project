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

#[non_exhaustive]
#[repr(C, usize)]
#[derive(Debug)]
pub enum KernelData {
	Unknown = 0,
	V1 {
		/// Size of this structure in bytes
		size: usize,
		memory_map: *mut c_void,
		stack_page_count: usize,
		trampoline_page: uefi::PhysicalAddress,
		address_space: AddressSpace,
		system_table: uefi::SystemTablePointer<uefi::RuntimeServices>,
		root_system_description_pointer: RootSystemDescriptionPointer,
		root_system_description_pointer_ex: RootSystemDescriptionPointerEx,
	} = 1,
	MaxSupportedVersion,
}

#[repr(C)]
#[derive(Debug)]
pub struct KernelDataStruct {
	pub version_tag: usize,
	pub size: usize,
	pub memory_map: *mut c_void,
	pub stack_page_count: usize,
	pub trampoline_page: uefi::PhysicalAddress,
	pub address_space: AddressSpace,
	pub system_table: uefi::SystemTablePointer<uefi::RuntimeServices>,
	pub root_system_description_pointer: RootSystemDescriptionPointer,
	pub root_system_description_pointer_ex: RootSystemDescriptionPointerEx,
}

// #[repr(C)]
// #[derive(Debug)]
// pub struct KernelDataHeader {
// 	pub graphics_format: uefi::protocols::graphics::GraphicsPixelFormat,
// 	pub graphics_ptr: *mut uefi::protocols::graphics::GraphicsPixel,
// 	/// size of structure in bytes
// 	pub graphics_len: usize,
// 	pub root_system_description_pointer: RootSystemDescriptionPointer,
// 	pub root_system_description_pointer_ex: RootSystemDescriptionPointerEx,
// 	pub system_table: uefi::tables::SystemTable,
// 	pub address_space: address_space::AddressSpace,
// 	pub stack_page_count: usize,
// 	pub virtual_mappings_count: usize,
// 	pub trampoline_page: uefi::PhysicalAddress,
// }

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
