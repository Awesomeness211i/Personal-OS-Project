use arch::x86_64::{
	LOW_HALF,
	LogicalAddress,
	paging::{
		Entry,
		EntryFlags,
		Page,
		Table,
	},
};
use uefi::{
	PhysicalAddress,
	VirtualAddress,
};

use crate::debug_print::println;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AddressSpace {
	ptr: LogicalAddress,
	next_allocation_index: usize,
	page_count: usize,
	levels: u8,
}

impl AddressSpace {
	pub fn create(mut laddr: LogicalAddress, page_count: usize, levels: u8) -> Self {
		assert!(page_count > 0);
		assert!(size_of::<Page>() == size_of::<Table>());
		// # Safety:
		// todo
		unsafe {
			laddr.as_mut_ptr::<Table>().write_bytes(0, page_count);
		}
		Self {
			ptr: laddr,
			next_allocation_index: 1,
			page_count,
			levels,
		}
	}

	pub fn switch_to_virtual(&mut self, vaddr: VirtualAddress) {
		self.ptr = LogicalAddress::new(vaddr.get() + (self.ptr.get() & LOW_HALF));
	}

	pub fn as_ptr<T>(&mut self) -> *const T {
		self.ptr.as_ptr::<T>()
	}

	pub fn as_mut_ptr<T>(&mut self) -> *mut T {
		self.ptr.as_mut_ptr::<T>()
	}

	pub fn get_page_count(&self) -> usize {
		self.page_count
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

	fn get_or_create_entry(&mut self, ptr: *mut Table, table_index: usize, flags: EntryFlags) -> Entry {
		match Self::get_entry(ptr, table_index) {
			Ok(entry) => entry,
			Err(_) => {
				if self.next_allocation_index < self.page_count {
					// # Safety:
					// Should be safe because of the check on page count
					let page_table_allocation = unsafe { self.as_mut_ptr::<Table>().add(self.next_allocation_index) };
					println(format_args!("Page Table Allocation Address: {page_table_allocation:#X?}"));

					self.next_allocation_index += 1;
					let entry = Entry::new((page_table_allocation as u64) | flags.get());

					// # Safety:
					// Should be safe because of the check on page count
					unsafe { &mut *ptr }.get_entries()[table_index] = entry.clone();
					entry
				} else {
					panic!("Failed to allocate page table")
				}
			},
		}
	}

	pub fn mmap(&mut self, vaddr: VirtualAddress, paddr: PhysicalAddress, parent_flags: EntryFlags, flags: EntryFlags) {
		let index_mask = 0x1FF;
		let vaddr_no_offset = vaddr.get() as usize >> 12;

		let mut entry = Entry::new(0);
		for i in (0..self.levels).rev() {
			let index = (vaddr_no_offset >> (9 * i)) & index_mask;
			entry = if i == self.levels - 1 {
				let table = self.as_mut_ptr();
				self.get_or_create_entry(table, index, parent_flags)
			} else if i > 0 {
				self.get_or_create_entry(entry.to_mut_addr(), index, parent_flags)
			} else {
				let page_table = entry.to_mut_addr();
				if page_table.entries()[index].exists() {
					panic!("Trying to map a page twice to the same entry seems problematic, physical address: {paddr:#X}, virtual address: {vaddr:#X?}")
				} else {
					page_table.get_entries()[index] = Entry::new(paddr.get() | flags.get())
				}
				break;
			};
		}
	}
}
