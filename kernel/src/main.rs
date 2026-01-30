// #![warn(missing_docs)]
#![no_main]
#![no_std]
//! # KERNEL
//! Starting executable file for the kernel for my hobby OS project.

use bootloader::boot_info;

#[unsafe(no_mangle)]
fn _start(data: boot_info::KernelDataHeader) -> ! {
	loop {}
	// let buffer = unsafe { core::slice::from_raw_parts_mut(data.graphics_ptr, data.graphics_len) };
	// let mask = uefi::protocols::graphics::PixelBitmask::new(0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000);
	// let white = uefi::protocols::graphics::GraphicsOutputProtocol::grapics_color(0xFFFFFFFF, &mask);
	// for pixel in buffer {
	// 	*pixel = white;
	// }
}

#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &core::panic::PanicInfo) -> ! {
	loop {}
}
