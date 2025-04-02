#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]

#[cfg(target_arch = "x86_64")]
pub mod x86_64;
