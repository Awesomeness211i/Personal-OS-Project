// #![warn(missing_docs)]
#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]
// #![feature(custom_test_frameworks)]
// #![test_runner(tests)]
// #![reexport_test_harness_main = "test_main"]
//! # KERNEL
//! Starting executable file for the kernel for my hobby OS project.

use core::panic;

use arch::x86_64::DescriptorTablePointer;
use boot_protocol_structures::debug_print::println;

// #[unsafe(link_section = "booby")]
// static IDT: idk = core::cell::LazyCell::new(|| {});

// extern crate alloc;

#[unsafe(no_mangle)]
unsafe extern "C" fn _start(data: &boot_protocol_structures::KernelDataStruct) -> ! {
	println(format_args!("Hello Kernel!"));
	println(format_args!("{data:#X?}"));

	let mut address_space = data.address_space.clone();
	address_space.switch_to_virtual();

	let gdt = DescriptorTablePointer { limit: 0, address: 0 };

	// unsafe {
	// 	core::arch::asm!(
	// 		"sgdt [{gdt}]",
	// 		"sidt [{idt}]",
	// 		gdt = in(reg) &mut gdt,
	// 		idt = in(reg) &mut idt,
	// 	)
	// };

	loop {}
}

extern "x86-interrupt" fn handler(a: ()) {}
extern "x86-interrupt" fn handler2(a: (), error_code: u64) {}

#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &panic::PanicInfo) -> ! {
	let message = info.message();
	if let Some(location) = info.location() {
		let file_name = location.file();
		let line_number = location.line();
		let column_number = location.column();
		println(format_args!(
			"Panicked at: {file_name}\n\tline number: {line_number}\n\tcolumn number: {column_number}\n\tmessage: {message}"
		));
	} else {
		println(format_args!("Panicked: {message}"));
	}
	loop {}
}
