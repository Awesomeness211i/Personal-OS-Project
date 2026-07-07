#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
// #![warn(missing_docs)]
//! # BOOTLOADER
//! Library for interfacing with bootloader specific data structures for my hobby OS project.

use uefi::SystemTablePointer;

pub mod address_space;
pub mod boot_info;
pub mod elf;
pub mod font;
pub mod print;

pub static mut SYSTEM_TABLE_POINTER: Option<SystemTablePointer> = None;

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
