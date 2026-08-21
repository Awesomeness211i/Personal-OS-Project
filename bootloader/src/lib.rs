#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
// #![warn(missing_docs)]
//! # BOOTLOADER
//! Library for interfacing with bootloader specific data structures for my hobby OS project.

// TODO: Figure out how I want to deal with defining external symbols

// #[unsafe(export_name = "efi_main")]
// pub extern "efiapi" fn efi_main(image_handle: &mut core::ffi::c_void, system_table: SystemTablePointer) -> uefi::status::Status {
// 	__user_uefi_main(image_handle, system_table)
// }
//
// unsafe extern "C" {
// 	safe fn __user_uefi_main(image_handle: &mut core::ffi::c_void, system_table: SystemTablePointer) -> uefi::status::Status;
// }

pub unsafe trait SizeField<StructureType: ?Sized> {
	fn size(&self) -> usize;
}

pub trait RuntimeSizeStructure<HeaderType: Sized + SizeField<Self>> {}
impl<T: ?Sized, H: SizeField<T>> RuntimeSizeStructure<H> for T {}
