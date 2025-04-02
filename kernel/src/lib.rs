#![no_std]

pub struct KernelData {
	pub graphics: *mut uefi::protocols::graphics::GraphicsPixel,
	pub graphicslen: usize,
	// pub systemtable: uefi::Handle<uefi::tables::SystemTable>,
	// pub memorymap: uefi::memory::MemoryMap,
	// pub imagehandle: uefi::Handle,
}

impl KernelData {
	pub fn new(graphics: *mut uefi::protocols::graphics::GraphicsPixel, graphicslen: usize, /* systemtable: uefi::Handle<uefi::tables::SystemTable>, imagehandle: uefi::Handle, memorymap: uefi::memory::MemoryMap */) -> KernelData {
		KernelData {
			graphics,
			graphicslen,
			// systemtable,
			// imagehandle,
			// memorymap,
		}
	}
}
