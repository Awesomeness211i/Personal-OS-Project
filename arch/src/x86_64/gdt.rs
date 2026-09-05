#[repr(C)]
#[derive(Debug, Clone)]
pub struct GdtEntry {
	limit: u16,
	base_15_0: u16,
	base23_16: u8,
	entry_type: u8,
	limit19_16_and_flags: u8,
	base31_24: u8,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct GdtTable<const MAX: usize = 512> {
	entries: [GdtEntry; MAX],
}
