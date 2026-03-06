#![feature(custom_test_frameworks)]
// #![test_runner(crate::test_runner)]
// #![warn(missing_docs)]
#![no_main]
#![no_std]
//! # KERNEL
//! Starting executable file for the kernel for my hobby OS project.

use core::fmt::{
	Error,
	Write,
};

// use bootloader::boot_info;

#[inline(always)]
fn println(args: core::fmt::Arguments<'_>) {
	let mut port = Port::COM1;
	let _ = writeln!(port, "{args}");
}

#[inline(always)]
fn print(args: core::fmt::Arguments<'_>) {
	let mut port = Port::COM1;
	let _ = write!(port, "{args}");
}

#[repr(transparent)]
struct Port(u16);

impl Write for Port {
	#[inline(always)]
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		let str = s.as_bytes();
		let len = str.len();
		let result = unsafe { self.out_bytes(str) };
		if result == len { Ok(()) } else { Err(Error) }
	}
}

impl Port {
	pub const COM1: Self = Self(0x3F8);
	pub const COM2: Self = Self(0x2F8);
	pub const COM3: Self = Self(0x3E8);
	pub const COM4: Self = Self(0x2E8);
	pub const COM5: Self = Self(0x5F8);
	pub const COM6: Self = Self(0x4F8);
	pub const COM7: Self = Self(0x5E8);
	pub const COM8: Self = Self(0x4E8);

	const RX_TX_BUFFER: u16 = 0;
	const INT_ENABLE: u16 = 1;
	const FIFO_CTRL: u16 = 2;
	const LINE_CTRL: u16 = 3;
	const MODEM_CTRL: u16 = 4;
	const LINE_STATUS: u16 = 5;
	const MODEM_STATUS: u16 = 6;
	const SCRATCH: u16 = 7;

	#[inline(always)]
	pub unsafe fn init(&self) {
		unsafe {
			Self(self.0 + Self::INT_ENABLE).out(0x00);
			Self(self.0 + Self::LINE_CTRL).out(0x80);
			Self(self.0 + Self::RX_TX_BUFFER).out(0x03);

			Self(self.0 + Self::INT_ENABLE).out(0x00);
			Self(self.0 + Self::LINE_CTRL).out(0x03);
			Self(self.0 + Self::FIFO_CTRL).out(0x07);
			Self(self.0 + Self::MODEM_CTRL).out(0x1F);

			Self(self.0 + Self::RX_TX_BUFFER).out(0xAA);

			if Self::inb(self) != 0xAA {
				println(format_args!("Error: port not setup correctly"));
			}

			Self(self.0 + Self::MODEM_CTRL).out(0x0F);
		}
	}

	#[inline(always)]
	unsafe fn out(&self, value: u8) {
		unsafe {
			core::arch::asm!(
				"out dx, al",
				in("dx") self.0,
				in("al") value,
			)
		}
	}

	#[inline(always)]
	unsafe fn inb(&self) -> u8 {
		let value: u8;
		unsafe {
			core::arch::asm!(
				"in al, dx",
				in("dx") self.0,
				out("al") value,
			)
		}
		value
	}

	#[inline(always)]
	unsafe fn out_bytes(&self, bytes: &[u8]) -> usize {
		let unwritten_bytes: usize;
		unsafe {
			core::arch::asm!(
				"rep outsb",
				inout("rcx") bytes.len() => unwritten_bytes,
				in("dx") self.0,
				inout("rsi") bytes.as_ptr() => _,
			);
		}
		bytes.len() - unwritten_bytes
	}
}

#[unsafe(no_mangle)]
extern "C" fn _start(/* data: &boot_info::KernelDataHeader */) -> ! {
	let port = Port::COM1;
	unsafe { port.out_bytes(b"Hello Kernel!\n") };

	// let buffer = unsafe { core::slice::from_raw_parts_mut(data.graphics_ptr, data.graphics_len) };
	// let mask = uefi::protocols::graphics::PixelBitmask::new(0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000);
	// let white = uefi::protocols::graphics::GraphicsOutputProtocol::grapics_color(0xFFFFFFFF, &mask);
	// for pixel in buffer {
	// 	*pixel = white;
	// }
	println(format_args!("Nope"));
	loop {}
}

#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &core::panic::PanicInfo) -> ! {
	let port = Port::COM1;
	unsafe { port.out_bytes(b"Hello Panic!\n") };
	loop {}
}
