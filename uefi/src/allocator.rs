struct UEFIAllocator;

#[global_allocator]
static ALLOCATOR: UEFIAllocator = UEFIAllocator;

// SAFETY:
unsafe impl core::alloc::GlobalAlloc for UEFIAllocator {
	unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
		todo!()
	}
	unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
		todo!()
	}
}
