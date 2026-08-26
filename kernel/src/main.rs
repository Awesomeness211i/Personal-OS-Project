// #![warn(missing_docs)]
#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]
//! # KERNEL
//! Starting executable file for the kernel for my hobby OS project.

use arch::x86_64::paging::Table;
use boot_protocol_structures::debug_print::println;

#[unsafe(no_mangle)]
unsafe extern "C" fn _start(data: &boot_protocol_structures::KernelDataStruct) -> ! {
	println(format_args!("Hello Kernel!"));
	println(format_args!("{data:#X?}"));

	let mut address_space = data.address_space.clone();
	address_space.switch_to_virtual();

	for i in 0..address_space.get_page_count() {
		let table = unsafe { &*address_space.get_ptr::<Table>().add(i) };
		println(format_args!("{table:#X?}"));
	}
	loop {}
}

extern "x86-interrupt" fn handler(a: ()) {}
extern "x86-interrupt" fn handler2(a: (), error_code: u64) {}

#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &core::panic::PanicInfo) -> ! {
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
