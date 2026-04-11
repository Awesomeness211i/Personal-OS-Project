use core::{
	fmt::{
		LowerHex,
		UpperHex,
	},
	ops::{
		BitAnd,
		BitOr,
	},
};

pub enum EntrySize {
	FourKiB,
	TwoMiB,
	OneGiB,
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EntryFlags(u64);

impl EntryFlags {
	pub const fn get(&self) -> u64 {
		self.0
	}
	pub const NONE: Self = Self(0x00);

	/// Present in all entry types
	pub const PRESENT: Self = Self(0x1);
	/// Present in all entry types
	pub const READ_WRITE: Self = Self(0x2);
	/// Present in all entry types
	pub const USER_ACCESSIBLE: Self = Self(0x4);
	/// Present in all entry types
	pub const WRITE_THROUGH: Self = Self(0x8);
	/// Present in all entry types
	pub const PAGE_CACHE_DISABLE: Self = Self(0x10);
	/// Present in all entry types
	pub const ACCESSED: Self = Self(0x20);

	pub const DIRTY: Self = Self(0x40);

	/// This bit is only in the PDPE and PDE
	pub const PAGE_SIZE: Self = Self(0x80);

	pub const PAGE_ATTRIBUTE_TABLE: Self = Self(0x80);
	pub const GLOBAL: Self = Self(0x100);
	pub const MEMORY_PROTECTION_KEY: Self = Self(0x7800_0000_0000_0000);
	pub const NOT_EXECUTABLE: Self = Self(0x8000_0000_0000_0000);
}

impl LowerHex for EntryFlags {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:x}", self.0)
	}
}

impl UpperHex for EntryFlags {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{:X}", self.0)
	}
}

impl BitAnd for EntryFlags {
	type Output = Self;
	fn bitand(self, rhs: Self) -> Self::Output {
		Self(self.0 & rhs.0)
	}
}

impl BitOr for EntryFlags {
	type Output = Self;
	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Entry(u64);

impl Entry {
	pub unsafe fn new(entry: u64) -> Self {
		Self(entry)
	}

	pub fn exists(&self) -> bool {
		self.0 & EntryFlags::PRESENT.0 != 0
	}

	pub fn get_flags(&self) -> EntryFlags {
		EntryFlags(self.0 & !Self::PHYSICAL_ADDRESS)
	}

	pub const fn to_addr(&self) -> &Table {
		// # Safety:
		unsafe { &*((self.0 & Self::PHYSICAL_ADDRESS) as *const Table) }
	}

	pub const fn to_mut_addr(&mut self) -> &mut Table {
		// # Safety:
		unsafe { &mut *((self.0 & Self::PHYSICAL_ADDRESS) as *mut Table) }
	}

	const PHYSICAL_ADDRESS: u64 = 0x000FFFFFFFFFF000;
}

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct Table<const ENTRY_NUM: usize = 512> {
	entries: [Entry; ENTRY_NUM],
}

impl<const ENTRY_NUM: usize> Table<ENTRY_NUM> {
	pub fn entries(&self) -> &[Entry] {
		&self.entries
	}

	pub unsafe fn get_entries(&mut self) -> &mut [Entry] {
		&mut self.entries
	}
}
