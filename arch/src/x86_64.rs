pub mod gdt;
pub mod idt;
pub mod paging;

#[repr(C, packed)]
#[derive(Debug)]
pub struct DescriptorTablePointer {
	pub limit: u16,
	pub address: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct DescriptorTable {
	entries: [DescriptorEntry],
}

#[repr(transparent)]
#[derive(Debug)]
pub struct DescriptorEntry(u64);

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct LogicalAddress(u64);

impl LogicalAddress {
	pub fn new(addr: u64) -> Self {
		Self(addr)
	}

	pub fn get(&self) -> u64 {
		self.0
	}

	pub fn as_ptr<T>(&self) -> *const T {
		self.0 as *const T
	}

	pub fn as_mut_ptr<T>(&mut self) -> *mut T {
		self.0 as *mut T
	}
}

pub const PAGE_SIZE: usize = 4096;

pub const HIGH_HALF: u64 = 0xFFFF_8000_0000_0000;
pub const LOW_HALF: u64 = 0x0000_7FFF_FFFF_F000;

enum DescriptorTableEntry {
	CodeSegment {
		limit_0: u16,
		base_0: u16,
		base_2: u8,
		descriptor_priviledge_and_flags: u8,
		limit_2: u8,
		base_3: u8,
	},
	DataSegment {
		limit_0: u16,
		base_0: u16,
		base_2: u8,
		descriptor_priviledge_and_flags: u8,
		limit_2: u8,
		base_3: u8,
	},
	SystemSegment {
		limit_0: u16,
		base_0: u16,
		base_2: u8,
		descriptor_priviledge_and_flags: u8,
		limit_2: u8,
		base_3: u8,
		base_4: u32,
		reserved: u32,
	},
	CallGateSegment {
		target_offset: u16,
		target_selector: u16,
		reserved: u8,
		descriptor_priviledge_and_flags: u8,
		target_offset_2: u16,
		target_offset_4: u32,
		reserved_2: u32,
	},
	TrapGateSegment {
		target_offset: u16,
		target_selector: u16,
		ist: u8,
		descriptor_priviledge_and_flags: u8,
		target_offset_2: u16,
		target_offset_4: u32,
		reserved: u32,
	},
}

/// This is a thin wrapper function around the cli x86_64 instruction and the only difference
///
/// # Safety
/// todo
#[inline(always)]
pub unsafe fn disable_interrupts() {
	// Safety:
	// unsafe
	unsafe { core::arch::asm!("cli") };
}

/// This is a thin wrapper function around the sti x86_64 instruction and the only difference
///
/// # Safety
/// todo
#[inline(always)]
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
#[inline(always)]
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
