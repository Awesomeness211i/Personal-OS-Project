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
