use super::{
	VirtualAddress,
	PhysicalAddress,
};

#[repr(transparent)]
pub struct Attribute(u64);
impl Attribute {
	/// Supports memory region being configured to not cacheable.
	pub const UNCACHEABLE: Self = Self(0x0000000000000001);
	/// Supports memory region being configured as write combining.
	pub const WRITE_COMBINING: Self = Self(0x0000000000000002);
	/// Supports memory region being configured as cacheable with a "write through" policy.
	/// Writes that hit in cache will also be written to main memory
	pub const WRITE_THROUGH: Self = Self(0x0000000000000004);
	/// Supports memory region being configured as cacheable with a "write back" policy.
	/// Reads and writes that hit the cache do not propagate to main memory. Dirty data is
	/// written back to main memory when a new cache line is allocated.
	pub const WRITE_BACK: Self = Self(0x0000000000000008);
	/// Supports memory region being configured as not cacheable and exported.
	/// Supports the "fetch and add" semaphore mechanism
	pub const UNCACHEABLE_EXPORT: Self = Self(0x0000000000000010);
	/// Supports memory region being configured as write-protected by system hardware.
	/// Supports being configured as cacheable with a "write protected" policy.
	/// Reads come from cache lines when possible and read misses cause cache fills.
	/// Writes probagated to system bus and cause corresponding cache lines on all processors
	/// on the bus to be invalidated
	pub const WRITE_PROTECT: Self = Self(0x0000000000001000);
	/// Supports memory region being configured as read-protected by system hardware.
	pub const READ_PROTECT: Self = Self(0x0000000000002000);
	/// Supports memory region being configured to be protected by system hardware from
	/// executing code.
	pub const EXECUTE_PROTECT: Self = Self(0x0000000000004000);
	/// Refers to persistent memory
	pub const NONVOLATILE: Self = Self(0x0000000000008000);
	/// Memory region provides higher reliability relative to other memory in the system.
	/// If all memory has the same reliability then this isn't used.
	pub const MORE_RELIABLE: Self = Self(0x0000000000010000);
	/// Supports memory region being configured as read-only by system hardware.
	pub const READ_ONLY: Self = Self(0x0000000000020000);
	/// Memory is earmarked for specific purposes such as for device specific drivers or
	/// applications. Serves as a hint to OS to aviod allocating this memory for core OS data or
	/// code that can not be relocated. Prolonged use of this memory for purposes other than the
	/// intended purpose may result in suboptimal platform performance
	pub const SPECIFIC_PURPOSE: Self = Self(0x0000000000040000);
	/// If this flag is set the memory region is capable of being protected with CPU's memory
	/// cryptographic capabilities. If this flag is clear the memory region is not capable of being
	/// protected with the cpu's memory cryptographic capabilities.
	pub const CPU_CRYPTO: Self = Self(0x0000000000080000);
	/// Memory region needs to be given a virtual mapping by OS when SetVirtualAddressMap() is
	/// called.
	pub const RUNTIME: Self = Self(0x8000000000000000);
	/// If this flag is set the memory region is described with additional ISA-specific memory
	/// attributes as specified in ISA_MASK
	pub const ISA_VALID: Self = Self(0x4000000000000000);
	/// Defines the bits reserved for describing optional ISA-specific cacheability attributes that
	/// are not covered by the standard  UEFI Memory Attributes cacheability bits.
	pub const ISA_MASK: Self = Self(0x0FFFF00000000000);
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct MemoryType(u32);
impl MemoryType {
	pub const RESERVED: Self = Self(0x00);
	pub const LOADER_CODE: Self = Self(0x01);
	pub const LOADER_DATA: Self = Self(0x02);
	pub const BOOT_SERVICES_CODE: Self = Self(0x03);
	pub const BOOT_SERVICES_DATA: Self = Self(0x04);
	pub const RUNTIME_SERVICES_CODE: Self = Self(0x05);
	pub const RUNTIME_SERVICES_DATA: Self = Self(0x06);
	pub const CONVENTIONAL_MEMORY: Self = Self(0x07);
	pub const UNUSABLE_MEMORY: Self = Self(0x08);
	pub const ACPI_RECLAIM_MEMORY: Self = Self(0x09);
	pub const ACPI_MEMORY_NVS: Self = Self(0x0A);
	pub const MEMORY_MAPPED_IO: Self = Self(0x0B);
	pub const MEMORY_MAPPED_IO_PORT_SPACE: Self = Self(0x0C);
	pub const PAL_CODE: Self = Self(0x0D);
	pub const PERSISTENT_MEMORY: Self = Self(0x0E);
	pub const UNACCEPTED: Self = Self(0x0F);
	pub const MAX_TYPE: Self = Self(0x10);

	pub const OEM_RESERVED: core::ops::RangeInclusive<Self> = Self(0x7000_0000)..=Self(0x7FFF_FFFF);
	pub const OS_RESERVED: core::ops::RangeInclusive<Self> = Self(0x8000_0000)..=Self(0xFFFF_FFFF);
	pub const fn custom(value: u32) -> Self {
		let result = Self(value);
		// assert!(Self::OS_RESERVED.contains(&result));
		assert!(result.get() >= Self::OS_RESERVED.start().get());
		result
	}
	pub const fn get(&self) -> u32 { self.0 }
}

/// Can't rely on the static size of this type, you need to query for the descriptor size
/// using memorymapdata that under the hood uses getmemorymap
#[repr(C)]
pub struct MemoryDescriptor {
	/// Type of memory occupying this range.
	pub regiontype: MemoryType,
	pub physicalstart: PhysicalAddress,
	pub virtualstart: VirtualAddress,
	/// Number of 4 KiB pages contained in this range.
	pub numofpages: u64,
	pub attribute: Attribute,
}

impl MemoryDescriptor {
	pub const VERSION: u32 = 1;
}
