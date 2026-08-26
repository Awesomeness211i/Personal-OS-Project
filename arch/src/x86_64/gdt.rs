#[repr(C)]
pub struct GDTEntry {
	limit: u16,
	base_15_0: u16,
	base23_16: u8,
	entry_type: u8,
	limit19_16_and_flags: u8,
	base31_24: u8,
}

#[repr(C, align(4096))]
pub struct GDTTable {
	null: GDTEntry,
	kernel_code: GDTEntry,
	kernel_data: GDTEntry,
	null2: GDTEntry,
	user_data: GDTEntry,
	user_code: GDTEntry,
	ovmf_data: GDTEntry,
	ovmf_code: GDTEntry,
	tss_low: GDTEntry,
	tss_high: GDTEntry,
}
