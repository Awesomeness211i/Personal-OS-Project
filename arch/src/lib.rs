#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
// #![warn(missing_docs)]
//! # ARCH
//! Library for interfacing with the cpu architecture specific features that I want to support for
//! my hobby OS project.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;
