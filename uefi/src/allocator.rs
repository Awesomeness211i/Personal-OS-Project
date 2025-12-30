use core::mem::MaybeUninit;

use crate::SystemTablePointer;

pub static mut SYSTEM_TABLE: MaybeUninit<SystemTablePointer> = MaybeUninit::uninit();
