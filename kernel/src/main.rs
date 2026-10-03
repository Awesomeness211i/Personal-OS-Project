// #![warn(missing_docs)]
#![feature(abi_x86_interrupt, ptr_metadata)]
#![no_main]
#![no_std]
// #![feature(custom_test_frameworks)]
// #![test_runner(tests)]
// #![reexport_test_harness_main = "test_main"]
//! # KERNEL
//! Starting executable file for the kernel for my hobby OS project.

use core::panic;

use arch::x86_64::{
	DescriptorTable,
	DescriptorTablePointer,
};
use boot_protocol_structures::debug_print::println;
// use uefi::protocols::graphics::GraphicsPixel;

#[unsafe(no_mangle)]
unsafe extern "C" fn _start(data: &boot_protocol_structures::KernelData) -> ! {
	println(format_args!("Hello Kernel!"));
	// let mut address_space = data.address_space;
	// address_space.switch_to_virtual(uefi::VirtualAddress::new(data.base_address));

	// let graphics = unsafe { core::slice::from_raw_parts_mut(data.graphics_ptr, data.graphics_len / size_of::<GraphicsPixel>()) };

	let table_ptr = unsafe { &*((data.base_address + data.root_system_description_pointer_ex.xsdt_address) as *const acpi::ACPITableHeader) };

	let length = (table_ptr.length as usize - size_of::<acpi::ACPITableHeader>()) / 8;

	let table = unsafe { &*core::ptr::from_raw_parts::<acpi::RootSystemDescriptionTableEx>(table_ptr, length) };
	for i in unsafe { &*(&raw const (*table).entries) } {
		let addr = *i + data.base_address;
		let ptr = unsafe { &*(addr as *const [u8; 4]) };
		match ptr {
			b"FACP" => {
				let facp = unsafe { &*(addr as *const acpi::FixedACPIDescriptionTable) };
				println(format_args!("{facp:#X?}"));
			},
			b"APIC" => {
				let header = unsafe { &*(addr as *const acpi::ACPITableHeader) };
				let length = header.length as usize - core::mem::size_of::<acpi::MultipleAPICDescriptionTable<[u8; 0]>>();
				let madt = unsafe { &*core::ptr::from_raw_parts::<acpi::MultipleAPICDescriptionTable>(header as *const _, length) };
				let structures = madt.interrupt_controller_structure();
				for structure in structures {
					println(format_args!("{structure:#X?}"));
				}
				// println(format_args!("{madt:#X?}"));
			},
			b"HPET" => {
				let hpet = unsafe { &*(addr as *const acpi::HighPrecisionEventTable) };
				println(format_args!("{hpet:#X?}"));
			},
			b"WAET" => {
				let waet = unsafe { &*(addr as *const acpi::WindowsACPIEmulatedDeviceTable) };
				println(format_args!("{waet:#X?}"));
			},
			b"BGRT" => {
				let bgrt = unsafe { &*(addr as *const acpi::BootGraphicsResourceTable) };
				let bitmap_header = unsafe { &*(bgrt.image_address as *const acpi::BitMapHeader) };
				// let image_ptr = (bgrt.image_address + bitmap_header.offset as u64) as *const u8;
				// let size = (bitmap_header.size - bitmap_header.offset) as usize;
				// let data = unsafe { core::slice::from_raw_parts(image_ptr, size) };
				println(format_args!("{bgrt:#X?}, {bitmap_header:#X?}"));
			},
			other => println(format_args!("{}{}{}{}", other[0] as char, other[1] as char, other[2] as char, other[3] as char)),
		}
	}

	// println(format_args!("table: {table:#X?}"));

	let mut gdt_ptr = DescriptorTablePointer { limit: 0, address: 0 };
	let mut ldt_ptr = DescriptorTablePointer { limit: 0, address: 0 };
	let mut idt_ptr = DescriptorTablePointer { limit: 0, address: 0 };

	unsafe {
		core::arch::asm!(
			"sgdt [{gdt}]",
			"sldt [{ldt}]",
			"sidt [{idt}]",
			gdt = in(reg) &mut gdt_ptr,
			ldt = in(reg) &mut ldt_ptr,
			idt = in(reg) &mut idt_ptr,
		)
	};

	println(format_args!("GDT pointer: {gdt_ptr:#X?}, LDT pointer: {ldt_ptr:#X?}, IDT pointer: {idt_ptr:#X?}"));

	let gdt = unsafe { &*core::ptr::from_raw_parts::<DescriptorTable>(gdt_ptr.address as *const (), (gdt_ptr.limit as usize + 1) / 8) };
	let ldt = unsafe { &*core::ptr::from_raw_parts::<DescriptorTable>(ldt_ptr.address as *const (), (ldt_ptr.limit as usize + 1) / 8) };
	let idt = unsafe { &*core::ptr::from_raw_parts::<DescriptorTable>(idt_ptr.address as *const (), (idt_ptr.limit as usize + 1) / 8) };

	println(format_args!("GDT: {gdt:#X?}, LDT: {ldt:#X?}, IDT: {idt:#X?}"));

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
