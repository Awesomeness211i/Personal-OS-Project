use core::ops::{
	Index,
	IndexMut,
};

use arch::x86_64::paging::{
	Entry,
	EntryFlags,
	Table,
};
use uefi::{
	PhysicalAddress,
	VirtualAddress,
	memory::MemoryType,
	services::AllocateType,
	status::Status,
};

use crate::{
	PAGE_SIZE,
	SYSTEM_TABLE_POINTER,
	print::{
		print,
		println,
	},
};

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct Page<const SIZE: usize = PAGE_SIZE> {
	data: [u8; SIZE],
}

impl<const SIZE: usize> Page<SIZE> {
	pub const fn as_ref(&self) -> &[u8] {
		&self.data
	}
	pub const fn as_mut(&mut self) -> &mut [u8] {
		&mut self.data
	}
}

pub struct PageIterator<'a, const SIZE: usize> {
	page: &'a Page<SIZE>,
	index: usize,
}

impl<'a, const SIZE: usize> From<&'a Page<SIZE>> for PageIterator<'a, SIZE> {
	fn from(value: &'a Page<SIZE>) -> Self {
		Self { page: value, index: 0 }
	}
}

impl<'a, const SIZE: usize> Iterator for PageIterator<'a, SIZE> {
	type Item = u8;
	fn next(&mut self) -> Option<Self::Item> {
		if let Some(result) = self.page.data.get(self.index) {
			self.index += 1;
			Some(*result)
		} else {
			None
		}
	}
}

impl<const SIZE: usize> Index<usize> for Page<SIZE> {
	type Output = u8;
	fn index(&self, index: usize) -> &Self::Output {
		&self.data[index]
	}
}

impl<const SIZE: usize> IndexMut<usize> for Page<SIZE> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.data[index]
	}
}

#[repr(C)]
#[derive(Debug)]
pub struct AddressSpace {
	ptr: PhysicalAddress,
	// TODO: Actually use this field to map things properly
	levels: u8,
}

impl AddressSpace {
	pub const ADDRESS_SPACE: MemoryType = MemoryType::custom_os(0x80000003);
	pub fn create(levels: u8) -> Self {
		// # Safety:
		let ptr = unsafe { SYSTEM_TABLE_POINTER.expect("System Table Pointer uninitialized").boot_services };
		let num_pages = 1;
		let mut paddr = PhysicalAddress::new(0);
		// # Safety:
		let status = unsafe { ((*ptr).allocate_pages)(AllocateType::ANY_PAGES, MemoryType::LOADER_DATA, num_pages, &mut paddr) };
		if status != Status::SUCCESS {
			panic!("Failed to allocate address space with error: {status}")
		} else {
			println(format_args!("Allocated address space at physical address {paddr:X}"));
		}
		// # Safety:
		unsafe {
			paddr.to_ptr::<Page>().write_bytes(0, num_pages);
		}
		Self { ptr: paddr, levels }
	}

	pub fn get_ptr(&self) -> *mut Page {
		self.ptr.to_ptr::<Page>()
	}

	pub fn get_physical_address(&self, vaddr: VirtualAddress) -> PhysicalAddress {
		assert!(self.levels == 4);
		let index_mask = 0x1FF;
		let vaddr_offset = vaddr.get() & 0xFFF;
		let vaddr_no_offset = vaddr.get() as usize >> 12;

		let page_table_index = (vaddr_no_offset >> (9 * 0)) & index_mask;
		let page_middle_directory_index = (vaddr_no_offset >> (9 * 1)) & index_mask;
		let page_upper_directory_index = (vaddr_no_offset >> (9 * 2)) & index_mask;
		let page_global_directory_index = (vaddr_no_offset >> (9 * 3)) & index_mask;

		let table = unsafe { &mut *self.ptr.to_ptr::<Table>() };

		let page_global_directory_entry = Self::get_entry(table, page_global_directory_index).expect("Until I figure out how to deal with less than exactly 4 tables this is expected");
		let page_upper_directory_entry =
			Self::get_entry(page_global_directory_entry.to_addr(), page_upper_directory_index).expect("Until I figure out how to deal with less than exactly 4 tables this is expected");
		let page_middle_directory_entry =
			Self::get_entry(page_upper_directory_entry.to_addr(), page_middle_directory_index).expect("Until I figure out how to deal with less than exactly 4 tables this is expected");

		let page_table = Self::get_entry(page_middle_directory_entry.to_addr(), page_table_index).expect("Until I figure out how to deal with less than exactly 4 tables this is expected");
		PhysicalAddress::new((page_table.to_addr() as *const _ as u64) | vaddr_offset)
	}

	fn get_entry(ptr: *const Table, table_index: usize) -> Result<Entry, ()> {
		let table = unsafe { &*ptr };
		let entry = table.entries()[table_index].clone();
		if entry.exists() {
			// TODO: Make it so that this isn't necessary
			if entry.get_flags() & EntryFlags::PAGE_SIZE == EntryFlags::NONE {
				Ok(entry)
			} else {
				panic!("Expanded page size bit set")
			}
		} else {
			Err(())
		}
	}

	fn get_or_create_entry(ptr: *mut Table, table_index: usize, flags: EntryFlags) -> Entry {
		match Self::get_entry(ptr, table_index) {
			Ok(entry) => entry,
			Err(_) => {
				let mut page_table_allocation = PhysicalAddress::new(0);
				// # Safety:
				let status =
					unsafe { ((*SYSTEM_TABLE_POINTER.expect("SYSTEM_TABLE_POINTER").boot_services).allocate_pages)(AllocateType::ANY_PAGES, Self::ADDRESS_SPACE, 1, &mut page_table_allocation) };
				if status != Status::SUCCESS {
					panic!("Failed to allocate page table allocation: {status}")
				}
				println(format_args!("Page Table Allocation Address: {page_table_allocation:#X?}"));
				// # Safety:
				unsafe {
					let entry = Entry::new((page_table_allocation.to_ptr::<Page>() as u64) | flags.get());
					page_table_allocation.to_ptr::<u8>().write_bytes(0, PAGE_SIZE);
					(&mut *ptr).get_entries()[table_index] = entry.clone();
					entry
				}
			},
		}
	}

	pub fn mmap(&mut self, vaddr: VirtualAddress, paddr: PhysicalAddress, flags: EntryFlags) {
		// TODO: Figure out how I would do stuff with more than just PML4 4KiB pages
		assert!(self.levels == 4);
		let permissive = EntryFlags::PRESENT | EntryFlags::READ_WRITE | EntryFlags::USER_ACCESSIBLE;
		let index_mask = 0x1FF;

		let vaddr_no_offset = vaddr.get() as usize >> 12;
		let page_table_index = (vaddr_no_offset >> (9 * 0)) & index_mask;
		let page_middle_directory_index = (vaddr_no_offset >> (9 * 1)) & index_mask;
		let page_upper_directory_index = (vaddr_no_offset >> (9 * 2)) & index_mask;
		let page_global_directory_index = (vaddr_no_offset >> (9 * 3)) & index_mask;

		// for i in 0..self.levels {}
		let mut page_global_directory_entry = Self::get_or_create_entry(self.ptr.to_ptr::<Table>(), page_global_directory_index, permissive);
		let mut page_upper_directory_entry = Self::get_or_create_entry(page_global_directory_entry.to_mut_addr(), page_upper_directory_index, permissive);
		let mut page_middle_directory_entry = Self::get_or_create_entry(page_upper_directory_entry.to_mut_addr(), page_middle_directory_index, permissive);

		let page_table = page_middle_directory_entry.to_mut_addr();

		if page_table.entries()[page_table_index].exists() {
			panic!("Trying to map a page twice to the same entry seems problematic")
		} else {
			println(format_args!("{paddr:#X} -> {:#X}", vaddr.get() & !0xFFF));
			// for i in 0..PAGE_SIZE {
			// 	// # Safety:
			// 	let b = unsafe { *paddr.to_ptr::<u8>().add(i) };
			// 	print(format_args!("{b:02X} "));
			// }
			// println(format_args!(""));

			// # Safety:
			unsafe { page_table.get_entries()[page_table_index] = Entry::new(paddr.get() | flags.get()) }
		}
	}
}
