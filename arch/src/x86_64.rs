pub mod gdt;
pub mod idt;
pub mod paging;

#[repr(C, packed)]
pub struct DescriptorTablePointer {
	pub limit: u16,
	pub address: u64,
}

pub const PAGE_SIZE: usize = 4096;

/// This is a thin wrapper function around the cli x86_64 instruction and the only difference
///
/// # Safety
/// todo
pub unsafe fn disable_interrupts() {
	// Safety:
	// unsafe
	unsafe { core::arch::asm!("cli") };
}

/// This is a thin wrapper function around the sti x86_64 instruction and the only difference
///
/// # Safety
/// todo
pub unsafe fn enable_interrupts() {
	// Safety:
	// unsafe
	unsafe { core::arch::asm!("sti") };
}

/// This is a thin wrapper function around the rdmsr x86_64 instruction and the only difference is
/// that I output a u64 from the combined u32 values to make reading the register value easier
/// instead of just outputing 2 u32 values.
///
/// # Safety
/// todo
pub unsafe fn rdmsr(ecx: u32) -> u64 {
	let eax: u32;
	let edx: u32;
	// Safety:
	// unsafe
	unsafe {
		core::arch::asm!(
			"rdmsr",
			in("ecx") ecx,
			out("eax") eax,
			out("edx") edx,
		)
	};
	((edx as u64) << 32) | (eax as u64)
}
