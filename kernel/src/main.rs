// #![warn(missing_docs)]
#![feature(abi_x86_interrupt)]
#![no_main]
#![no_std]
//! # KERNEL
//! Starting executable file for the kernel for my hobby OS project.

use bootloader::{
	boot_info,
	print::{
		print,
		println,
	},
};

#[unsafe(no_mangle)]
extern "C" fn _start(data: boot_info::KernelDataHeader) -> ! {
	// let kernel_table_pointer = unsafe { &raw mut GLOBAL_PAGE_TABLE.kernel };
	// let mut global_page_table = Table::<512>::new();
	// for entry in unsafe { global_page_table.get_entries() } {}
	// unsafe { kernel_table_pointer.write(global_page_table) };
	//
	// println(format_args!("{data:#X?}"));
	// let addr = 0usize.overflowing_sub(data.stack_page_count << 12).0;
	// for i in 0..data.stack_page_count {
	// 	for j in 0..4096 {
	// 		let b = unsafe { *(addr as *const u8).add((i << 12) + j) };
	// 		print(format_args!("{b:02X} "));
	// 	}
	// 	println(format_args!(""));
	// }
	// unsafe {
	// 	(data.trampoline_page.get() as *mut u8).write_bytes(0xFF, 4096);
	// }
	// println(format_args!(""));

	// Model specific extensions register
	// Can use CPUID to query support for feature except for performance counter extensions
	let model_specific_features = [
		"VME,",
		"PVI,",
		"TSD,",
		"DE,",
		"PSE,", // bit 4
		"PAE,", // bit 5
		"MCE,",
		"PGE,", // bit 7
		"PCE,",
		"OSFXSR,",     // bit 9
		"OSXMMEXCPT,", // bit 10
		"UMIP,",
		"LA57,", // bit 12
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"FSGSBASE,",
		"PCIDE,", // bit 17
		"OSXSAVE,",
		"RESERVED,",
		"SMEP,", // bit 20
		"SMAP,", // bit 21
		"PKE,",  // bit 22
		"CET,",  // bit 23
		"PKS,",  // bit 24
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
	];
	let cr4: u64;
	unsafe { core::arch::asm!("mov {}, cr4", out(reg) cr4) };
	print(format_args!("CR4: "));
	for (i, string) in model_specific_features.iter().enumerate() {
		if cr4 & (1 << i) > 0 {
			print(format_args!("{string}"));
		}
	}
	println(format_args!(""));

	// let max = core::arch::x86_64::__cpuid(0x80000000);
	// if max.eax < 0x80000008 {
	// 	return Status::INVALID_PARAMETER;
	// }
	// println(format_args!("MAX CPUID: {}", max.eax));

	// let vendor_info = unsafe { core::mem::transmute::<core::arch::x86_64::CpuidResult, [u8; size_of::<core::arch::x86_64::CpuidResult>()]>(core::arch::x86_64::__cpuid(0x00000000)) };
	// print_string("Vendor Info: ");
	// println_string_bytes(unsafe { core::slice::from_raw_parts(vendor_info.as_ptr().add(size_of::<u32>()), 3 * size_of::<u32>()) });

	let flag_strings = [
		"FPU,",
		"VME,",
		"DE,",
		"PSE,",
		"TSC,",
		"MSR,",
		"PAE,",
		"MCE,",
		"CX8,",
		"APIC,",
		"RESERVED,",
		"SEP,",
		"MTRR,",
		"PGE,",
		"MCA,",
		"CMOV,",
		"PAT,",
		"PSE-36,",
		"PSN,",
		"CLFSH,",
		"RESERVED,",
		"DS,",
		"ACPI,",
		"MMX,",
		"FXSR,",
		"SSE,",
		"SSE2,",
		"SS,",
		"HTT,",
		"TM,",
		"RESERVED,",
		"PBE,",
	];
	let test = core::arch::x86_64::__cpuid(0x00000001);
	print(format_args!("Flags: "));
	for (i, string) in flag_strings.iter().enumerate() {
		if test.edx & (1 << i) > 0 {
			print(format_args!("{string}"));
		}
	}
	println(format_args!(""));

	// let tlb_info = core::arch::x86_64::__cpuid(0x00000002);
	// println(format_args!("TLB Info: {tlb_info:?}"));

	// let cache_info = core::arch::x86_64::__cpuid(0x00000002);
	// println(format_args!("Cache Info: {cache_info:?}"));

	// let addr_info = core::arch::x86_64::__cpuid(0x80000008);
	// println(format_args!("Address Space Info: {addr_info:?}"));

	let rflags_flags = [
		"CF,",
		"RESERVED,",
		"PF,",
		"RESERVED,",
		"AF,",
		"RESERVED,",
		"ZF,",
		"SF,",
		"TF,",
		"IF,",
		"DF,",
		"OF,",
		"IOPL,",
		"IOPL,",
		"NT,",
		"RESERVED,",
		"RF,",
		"VM,",
		"AC,", // bit 18
		"VIF,",
		"VIP,",
		"ID,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
	];
	let rflags: u64;
	unsafe { core::arch::asm!("pushfq","pop {}", out(reg) rflags) };
	print(format_args!("RFLAGS: "));
	for (i, string) in rflags_flags.iter().enumerate() {
		if rflags & (1 << i) > 0 {
			print(format_args!("{string}"));
		}
	}
	println(format_args!(""));

	let efer_features = [
		"SCE,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"LME,", // bit 8
		"RESERVED,",
		"LMA,",
		"NXE,", // bit 11
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
		"RESERVED,",
	];
	let efer = unsafe { arch::x86_64::rdmsr(0xC0000080) };
	print(format_args!("EFER: "));
	for (i, string) in efer_features.iter().enumerate() {
		if efer & (1 << i) > 0 {
			print(format_args!("{string}"));
		}
	}
	println(format_args!(""));

	// Task Priority Register
	// lowest 4 bits for 1-15 task priority where 0 enables and 15 disables all external interrupts
	let cr8: u64;
	unsafe { core::arch::asm!("mov {}, cr8", out(reg) cr8) };
	println(format_args!("CR8: {cr8:X}"));

	let cr2: u64;
	unsafe { core::arch::asm!("mov {}, cr2", out(reg) cr2) };
	println(format_args!("CR2: {cr2:X}"));

	let control_flag_strings = [
		"PE,",
		"MP,",
		"EM,",
		"TS,",
		"ET,",
		"NE,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"WP,", // bit 16
		"Reserved,",
		"AM,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"NW,",
		"CD,",
		"PG,", // bit 31
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
		"Reserved,",
	];
	let cr0: u64;
	unsafe { core::arch::asm!("mov {}, cr0", out(reg) cr0) };
	print(format_args!("CR0: "));
	for (i, string) in control_flag_strings.iter().enumerate() {
		if cr0 & (1 << i) > 0 {
			print(format_args!("{string}"));
		}
	}
	println(format_args!(""));
	loop {}
	// let buffer = unsafe { core::slice::from_raw_parts_mut(data.graphics_ptr, data.graphics_len) };
	// let mask = uefi::protocols::graphics::PixelBitmask::new(0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000);
	// let white = uefi::protocols::graphics::GraphicsOutputProtocol::grapics_color(0xFFFFFFFF, &mask);
	// for pixel in buffer {
	// 	*pixel = white;
	// }
}

extern "x86-interrupt" fn handler(a: ()) {}
extern "x86-interrupt" fn handler2(a: (), error_code: u64) {}

#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &core::panic::PanicInfo) -> ! {
	let message = info.message();
	let location = info.location().unwrap();
	let file_name = location.file();
	let line_number = location.line();
	let column_number = location.column();
	println(format_args!(
		"Panicked at: {file_name}\n\tline number: {line_number}\n\tcolumn number: {column_number}\n\tmessage: {message}"
	));
	loop {}
}
