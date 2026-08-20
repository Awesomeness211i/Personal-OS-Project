#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
// #![warn(missing_docs)]
//! # BOOTLOADER
//! Library for interfacing with bootloader specific data structures for my hobby OS project.

pub mod address_space;
pub mod boot_info;
pub mod elf;
pub mod font;
pub mod print;

pub const PAGE_SIZE: usize = 4096;

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

struct RuntimeSizeStructureIterator<'a, H: SizeField<T>, T: RuntimeSizeStructure<H>> {
	header_pointer: &'a H,
	data_pointer: *const u8,
	_phantom: core::marker::PhantomData<&'a T>,
	size_bytes: usize,
	index: usize,
}

impl<'a, H: SizeField<T>, T: RuntimeSizeStructure<H>> Iterator for RuntimeSizeStructureIterator<'a, H, T> {
	type Item = &'a T;
	fn next(&mut self) -> Option<Self::Item> {
		let size = self.header_pointer.size();
		let end = unsafe { self.data_pointer.add(self.size_bytes) };
		let ptr = unsafe { self.data_pointer.add(self.index * size) };
		if ptr < end {
			self.index += 1;
			Some(unsafe { &*(ptr as *const T) })
		} else {
			None
		}
	}
}
