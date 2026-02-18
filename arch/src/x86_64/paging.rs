pub trait PagingElement {
	fn exists(&self) -> bool;
}

pub trait PagingStructure {
	type EntryType: PagingElement;
	fn entries(&self) -> &[Self::EntryType];
	unsafe fn get_entries(&mut self) -> &mut [Self::EntryType];
}

const PHYSICAL_ADDRESS: u64 = 0x000FFFFFFFFFF000;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PageGlobalDirectoryEntry(u64);
impl PagingElement for PageGlobalDirectoryEntry {
	fn exists(&self) -> bool {
		self.0 & Self::PRESENT != 0
	}
}

impl PageGlobalDirectoryEntry {
	pub unsafe fn new(entry: u64) -> Self {
		Self(entry)
	}

	pub const fn to_addr(&self) -> *mut PageUpperDirectory {
		(self.0 & PHYSICAL_ADDRESS) as *mut PageUpperDirectory
	}
	pub const PRESENT: u64 = 0x1;
	pub const READ_WRITE: u64 = 0x2;
	pub const USER_ACCESSIBLE: u64 = 0x4;
	pub const WRITE_THROUGH: u64 = 0x8;
	pub const PAGE_CACHE_DISABLE: u64 = 0x10;
	pub const ACCESSED: u64 = 0x20;
	pub const NOT_EXECUTABLE: u64 = 0x8000000000000000;
}

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct PageGlobalDirectory {
	entries: [PageGlobalDirectoryEntry; 512],
}
impl PagingStructure for PageGlobalDirectory {
	type EntryType = PageGlobalDirectoryEntry;
	fn entries(&self) -> &[Self::EntryType] {
		&self.entries
	}
	unsafe fn get_entries(&mut self) -> &mut [Self::EntryType] {
		&mut self.entries
	}
}

impl PageGlobalDirectory {
	pub fn get() -> &'static Self {
		let uefi_cr3: *mut PageGlobalDirectory;
		// Safety:
		// should be safe to get the physical address of PageGlobalDirectory?
		unsafe {
			core::arch::asm!("mov {}, cr3", out(reg) uefi_cr3);
			&*uefi_cr3
		}
	}
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageUpperDirectoryEntry(u64);
impl PagingElement for PageUpperDirectoryEntry {
	fn exists(&self) -> bool {
		self.0 & Self::PRESENT != 0
	}
}

impl PageUpperDirectoryEntry {
	pub unsafe fn new(entry: u64) -> Self {
		Self(entry)
	}
	pub const fn to_addr(&self) -> *mut PageMiddleDirectory {
		(self.0 & PHYSICAL_ADDRESS) as *mut PageMiddleDirectory
	}
	pub const PRESENT: u64 = 0x1;
	pub const READ_WRITE: u64 = 0x2;
	pub const USER_ACCESSIBLE: u64 = 0x4;
	pub const WRITE_THROUGH: u64 = 0x8;
	pub const PAGE_CACHE_DISABLE: u64 = 0x10;
	pub const ACCESSED: u64 = 0x20;
	pub const PAGE_SIZE: u64 = 0x80;
	pub const NOT_EXECUTABLE: u64 = 0x8000000000000000;
}

#[repr(C, align(4096))]
#[derive(Clone)]
pub struct PageUpperDirectory {
	entries: [PageUpperDirectoryEntry; 512],
}
impl PagingStructure for PageUpperDirectory {
	type EntryType = PageUpperDirectoryEntry;
	fn entries(&self) -> &[Self::EntryType] {
		&self.entries
	}
	unsafe fn get_entries(&mut self) -> &mut [Self::EntryType] {
		&mut self.entries
	}
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageMiddleDirectoryEntry(u64);
impl PagingElement for PageMiddleDirectoryEntry {
	fn exists(&self) -> bool {
		self.0 & Self::PRESENT != 0
	}
}

impl PageMiddleDirectoryEntry {
	pub unsafe fn new(entry: u64) -> Self {
		Self(entry)
	}
	pub const fn to_addr(&self) -> *mut PageTable {
		(self.0 & PHYSICAL_ADDRESS) as *mut PageTable
	}
	pub const PRESENT: u64 = 0x1;
	pub const READ_WRITE: u64 = 0x2;
	pub const USER_ACCESSIBLE: u64 = 0x4;
	pub const WRITE_THROUGH: u64 = 0x8;
	pub const PAGE_CACHE_DISABLE: u64 = 0x10;
	pub const ACCESSED: u64 = 0x20;
	pub const PAGE_SIZE: u64 = 0x80;
	pub const NOT_EXECUTABLE: u64 = 0x8000000000000000;
}

#[repr(C, align(4096))]
#[derive(Clone)]
pub struct PageMiddleDirectory {
	entries: [PageMiddleDirectoryEntry; 512],
}
impl PagingStructure for PageMiddleDirectory {
	type EntryType = PageMiddleDirectoryEntry;
	fn entries(&self) -> &[Self::EntryType] {
		&self.entries
	}
	unsafe fn get_entries(&mut self) -> &mut [Self::EntryType] {
		&mut self.entries
	}
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);
impl PagingElement for PageTableEntry {
	fn exists(&self) -> bool {
		self.0 & Self::PRESENT != 0
	}
}

impl PageTableEntry {
	pub unsafe fn new(entry: u64) -> Self {
		Self(entry)
	}
	pub const fn to_addr(&self) -> u64 {
		self.0 & PHYSICAL_ADDRESS
	}
	pub const PRESENT: u64 = 0x1;
	pub const READ_WRITE: u64 = 0x2;
	pub const USER_ACCESSIBLE: u64 = 0x4;
	pub const WRITE_THROUGH: u64 = 0x8;
	pub const PAGE_CACHE_DISABLE: u64 = 0x10;
	pub const ACCESSED: u64 = 0x20;
	pub const DIRTY: u64 = 0x40;
	pub const PAGE_ATTRIBUTE_TABLE: u64 = 0x80;
	pub const GLOBAL: u64 = 0x100;
	pub const MEMORY_PROTECTION_KEY: u64 = 0x7800000000000000;
	pub const NOT_EXECUTABLE: u64 = 0x8000000000000000;
}

#[repr(C, align(4096))]
pub struct PageTable {
	entries: [PageTableEntry; 512],
}
impl PagingStructure for PageTable {
	type EntryType = PageTableEntry;
	fn entries(&self) -> &[Self::EntryType] {
		&self.entries
	}
	unsafe fn get_entries(&mut self) -> &mut [Self::EntryType] {
		&mut self.entries
	}
}
