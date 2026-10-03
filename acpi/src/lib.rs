#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
// #![warn(missing_docs)]
//! # ACPI
//! Library for interfacing with the ACPI specification

use core::{
	fmt::Debug,
	marker::PhantomData,
	ptr::NonNull,
};

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct ACPITableHeader {
	pub signature: [u8; 4],
	pub length: u32,
	pub revision: u8,
	pub checksum: u8,
	pub oem_id: [u8; 6],
	pub oem_table_id: [u8; 8],
	pub oem_revision: u32,
	pub creator_id: u32,
	pub creator_revision: u32,
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct RootSystemDescriptionPointer {
	pub signature: [u8; 8],
	pub checksum: u8,
	pub oem_id: [u8; 6],
	pub revision: u8,
	pub rsdt_address: u32,
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct RootSystemDescriptionPointerEx {
	pub rsdp: RootSystemDescriptionPointer,
	pub length: u32,
	pub xsdt_address: u64,
	pub extended_checksum: u8,
	reserved: [u8; 3],
}

#[repr(C)]
#[derive(Debug)]
pub struct RootSystemDescriptionTable {
	pub header: ACPITableHeader,
	pub entries: [u32],
}

#[repr(C, packed)]
// #[derive(Debug)]
pub struct RootSystemDescriptionTableEx {
	pub header: ACPITableHeader,
	pub entries: [u64],
}

#[repr(C)]
#[derive(Debug)]
pub struct FixedACPIDescriptionTable {
	pub header: ACPITableHeader,
	pub firmware_ctrl: u32,
	pub dsdt: u32,
	reserved0: u8,
	pub preferred_pm_profile: u8,
	pub sci_int: [u8; 2],
	pub smi_cmd: u32,
	pub acpi_enable: u8,
	pub acpi_disable: u8,
	pub s4bios_req: u8,
	pub pstate_cnt: u8,
	pub pm1a_evt_blk: u32,
	pub pm1b_evt_blk: u32,
	pub pm1a_cnt_blk: u32,
	pub pm1b_cnt_blk: u32,
	pub pm2_cnt_blk: u32,
	pub pm_tmr_blk: u32,
	pub gpe0_blk: u32,
	pub gpe1_blk: u32,
	pub pm1_evt_len: u8,
	pub pm1_cnt_len: u8,
	pub pm2_cnt_len: u8,
	pub pm_tmr_len: u8,
	pub gpe0_blk_len: u8,
	pub gpe1_blk_len: u8,
	pub gpe1_base: u8,
	pub cst_cnt: u8,
	pub p_lvl2_lat: [u8; 2],
	pub p_lvl3_lat: [u8; 2],
	pub flush_size: [u8; 2],
	pub flush_stride: [u8; 2],
	pub duty_offset: u8,
	pub duty_width: u8,
	pub day_alrm: u8,
	pub mon_alrm: u8,
	pub century: u8,
	pub iapc_boot_arch: [u8; 2],
	reserved1: u8,
	pub flags: u32,
	// pub reset_reg: [u8; 12],
	// pub reset_val: u8,
	// pub arm_boot_arch: [u8; 2],
	// pub fadt_minor_version: u8,
	// pub x_firmware_ctrl: [u8; 8],
	// pub x_dsdt: [u8; 8],
	// pub x_pm1a_evt_blk: [u8; 12],
	// pub x_pm1b_evt_blk: [u8; 12],
	// pub x_pm1a_cnt_blk: [u8; 12],
	// pub x_pm1b_cnt_blk: [u8; 12],
	// pub x_pm2_cnt_blk: [u8; 12],
	// pub x_pm_tmr_blk: [u8; 12],
	// pub x_gpe0_blk: [u8; 12],
	// pub x_gpe1_blk: [u8; 12],
	// pub sleep_control_reg: [u8; 12],
	// pub sleep_status_reg: [u8; 12],
	// pub hypervisor_vendor_id: [u8; 8],
}

#[repr(transparent)]
pub struct FADTFlags(u32);

impl Debug for FADTFlags {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		todo!()
	}
}

impl FADTFlags {
	pub const WBINVD: Self = Self(1);
	pub const WBINVD_FLUSH: Self = Self(1 << 1);
	pub const PROC_C1: Self = Self(1 << 2);
	pub const P_LVL2_UP: Self = Self(1 << 3);
	pub const POWER_BUTTON: Self = Self(1 << 4);
	pub const SLEEP_BUTTON: Self = Self(1 << 5);
	pub const FIX_RTC: Self = Self(1 << 6);
	pub const RTC_S4: Self = Self(1 << 7);
	pub const TMR_VAL_EXT: Self = Self(1 << 8);
	pub const DCK_CAP: Self = Self(1 << 9);
	pub const RESET_REG_SUP: Self = Self(1 << 10);
	pub const SEALED_CASE: Self = Self(1 << 11);
	pub const HEADLESS: Self = Self(1 << 12);
	pub const CPU_SW_SLP: Self = Self(1 << 13);
	pub const PCI_EXP_WAK: Self = Self(1 << 14);
	pub const USE_PLATFORM_CLOCK: Self = Self(1 << 15);
	pub const S4_RTC_STS_VALID: Self = Self(1 << 16);
	pub const REMOTE_POWER_ON_CAPABLE: Self = Self(1 << 17);
	pub const FORCE_APIC_CLUSTER_MODEL: Self = Self(1 << 18);
	pub const FORCE_APIC_PHYSICAL_DESTINATION_MODE: Self = Self(1 << 19);
	pub const HARDWARE_REDUCED_ACPI: Self = Self(1 << 20);
	pub const LOW_POWER_S0_IDLE_CAPABLE: Self = Self(1 << 21);

	pub const CPU_CACHE_NOT_REPORTED: Self = Self(0 << 22);
	pub const CPU_CACHE_NOT_PERSISTENT: Self = Self(1 << 22);
	pub const CPU_CACHE_PERSISITENT: Self = Self(2 << 22);
	pub const CPU_CACHE_RESERVED: Self = Self(3 << 22);
}

#[repr(C)]
#[derive(Debug)]
pub struct BootGraphicsResourceTable {
	pub header: ACPITableHeader,
	pub version: u16,
	pub status: u8,
	pub image_type: u8,
	pub image_address: u64,
	pub image_offset_x: u32,
	pub image_offset_y: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct BitMapHeader {
	pub bf_type: [u8; 2],
	pub size: u32,
	reserved0: u16,
	reserved1: u16,
	pub offset: u32,
	pub header_size: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultipleAPICDescriptionTable<T: ?Sized = [u8]> {
	pub header: ACPITableHeader,
	pub local_interrupt_controller_address: u32,
	pub flags: u32,
	interrupt_controller_structure: T,
}

impl MultipleAPICDescriptionTable {
	pub fn interrupt_controller_structure(&self) -> HeterogenousStructIterator<'_> {
		let data = &self.interrupt_controller_structure;
		HeterogenousStructIterator { data, offset: 0 }
	}
}

pub struct HeterogenousStructIterator<'a> {
	data: &'a [u8],
	offset: usize,
}

impl<'a> Iterator for HeterogenousStructIterator<'a> {
	type Item = MadtEntry<'a>;
	fn next(&mut self) -> Option<Self::Item> {
		let Some((header, data)) = self.data[self.offset..].split_first_chunk::<2>() else {
			return None;
		};
		let entry_type = header[0];
		let entry_len = header[1];
		if entry_len as usize - 2 > data.len() {
			return None;
		}
		let entry = match entry_type {
			0 => {
				let Some((structure, _)) = data.split_first_chunk::<6>() else { panic!("LocalApic Too Short?") };
				let acpi_id = structure[0];
				let apic_id = structure[1];
				let flags = u32::from_le_bytes(*unsafe { structure[2..6].as_array().unwrap_unchecked() });
				MadtEntry::LocalApic { acpi_id, apic_id, flags }
			},
			1 => {
				let Some((structure, _)) = data.split_first_chunk::<10>() else { panic!("IoApic Too Short?") };
				let io_apic_id = structure[0];
				let reserved = structure[1];
				let io_apic_address = u32::from_le_bytes(*unsafe { structure[2..6].as_array().unwrap_unchecked() });
				let global_system_interrupt_base = u32::from_le_bytes(*unsafe { structure[6..10].as_array().unwrap_unchecked() });
				MadtEntry::IoApic {
					io_apic_id,
					reserved,
					io_apic_address,
					global_system_interrupt_base,
				}
			},
			2 => {
				let Some((structure, _)) = data.split_first_chunk::<8>() else {
					panic!("InterruptSourceOverride Too Short?")
				};
				let bus = structure[0];
				let source = structure[1];
				let global_system_interrupt = u32::from_le_bytes(*unsafe { structure[2..6].as_array().unwrap_unchecked() });
				let flags = u16::from_le_bytes(*unsafe { structure[6..8].as_array().unwrap_unchecked() });
				MadtEntry::InterruptSourceOverride {
					bus,
					source,
					global_system_interrupt,
					flags,
				}
			},
			3 => {
				let Some((structure, _)) = data.split_first_chunk::<6>() else { panic!("NMISource Too Short?") };
				let flags = u16::from_le_bytes(*unsafe { structure[0..2].as_array().unwrap_unchecked() });
				let global_system_interrupt = u32::from_le_bytes(*unsafe { structure[2..6].as_array().unwrap_unchecked() });
				MadtEntry::NMISource { flags, global_system_interrupt }
			},
			4 => {
				let Some((structure, _)) = data.split_first_chunk::<4>() else { panic!("LocalApicNMI Too Short?") };
				let acpi_id = structure[0];
				let flags = u16::from_le_bytes(*unsafe { structure[1..3].as_array().unwrap_unchecked() });
				let local_apic_lint = structure[3];
				MadtEntry::LocalApicNMI { acpi_id, flags, local_apic_lint }
			},
			_ => MadtEntry::Unknown { entry_type, data },
		};

		self.offset += entry_len as usize;
		Some(entry)
	}
}

#[repr(u8)]
#[derive(Debug)]
pub enum MadtEntry<'a> {
	LocalApic {
		acpi_id: u8,
		apic_id: u8,
		flags: u32,
	},
	IoApic {
		io_apic_id: u8,
		reserved: u8,
		io_apic_address: u32,
		global_system_interrupt_base: u32,
	},
	InterruptSourceOverride {
		bus: u8,
		source: u8,
		global_system_interrupt: u32,
		flags: u16,
	},
	NMISource {
		flags: u16,
		global_system_interrupt: u32,
	},
	LocalApicNMI {
		acpi_id: u8,
		flags: u16,
		local_apic_lint: u8,
	},
	Unknown {
		entry_type: u8,
		data: &'a [u8],
	},
}

#[repr(C, packed)]
pub struct HighPrecisionEventTable {
	pub header: ACPITableHeader,
	pub event_timer_block_id: u32,
	pub base_address_info: u32,
	pub base_address: u64,
	pub hpet_number: u8,
	pub main_counter_min_clock: u16,
	pub page_protection_oem_attribute: u8,
}

impl Debug for HighPrecisionEventTable {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		let header = self.header;
		let event_timer_block_id = self.event_timer_block_id;
		let base_address_info = self.base_address_info;
		let base_address = self.base_address;
		let hpet_number = self.hpet_number;
		let main_counter_min_clock = self.main_counter_min_clock;
		let page_protection_oem_attribute = self.page_protection_oem_attribute;
		f.debug_struct("HighPrecisionEventTable")
			.field("header", &header)
			.field("event_timer_block_id", &event_timer_block_id)
			.field("base_address_info", &base_address_info)
			.field("base_address", &base_address)
			.field("hpet_number", &hpet_number)
			.field("main_counter_min_clock", &main_counter_min_clock)
			.field("page_protection_oem_attribute", &page_protection_oem_attribute)
			.finish()
	}
}

#[repr(C)]
#[derive(Debug)]
pub struct WindowsACPIEmulatedDeviceTable {
	pub header: ACPITableHeader,
	pub emulated_device_flags: u32,
}

#[repr(C)]
pub struct ProcessorLocalAPICStructureHeader {
	pub type_id: u8,
	pub length: u8,
}
