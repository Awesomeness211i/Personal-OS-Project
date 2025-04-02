use crate::{
	Bool,
	Void,
	GUID,
	Event,
	Char16,
	PhysicalAddress,

	status::Status,
	protocols::{
		// Protocol,
		path::DevicePathProtocol,
	},
	memory::{
		MemoryDescriptor,
		MemoryType,
	},
	tables::{
		Time,
		TableHeader,
		CapsuleHeader,
		TimeCapabilities,
	},
};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TaskPriorityLevel(usize);
impl TaskPriorityLevel {
	pub const APPLICATION: Self = Self(0x04);
	pub const CALLBACK: Self = Self(0x08);
	pub const NOTIFY: Self = Self(0x10);
	pub const HIGH_LEVEL: Self = Self(0x1F);
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct AllocateType(u32);
impl AllocateType {
	pub const ANY_PAGES: Self = Self(0x00);
	pub const MAX_ADDRESS: Self = Self(0x01);
	pub const ADDRESS: Self = Self(0x02);
	pub const MAX_TYPE: Self = Self(0x03);
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TimerDelay(u32);
impl TimerDelay {
	pub const CANCEL: Self = Self(0x00);
	pub const PERIODIC: Self = Self(0x01);
	pub const RELATIVE: Self = Self(0x02);
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct InterfaceType(u32);
impl InterfaceType {
	pub const NATIVE_INTERFACE: Self = Self(0x00);
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct LocateSearchType(u32);
impl LocateSearchType {
	pub const ALL_HANDLES: Self = Self(0x00);
	pub const BY_REGISTER_NOTIFY: Self = Self(0x01);
	pub const BY_PROTOCOL: Self = Self(0x02);
}

#[repr(C)]
pub struct BootServices {
	pub header: TableHeader,
	/// newtpl: IN
	pub raise_tpl: unsafe extern "efiapi" fn(newtpl: TaskPriorityLevel) -> TaskPriorityLevel,
	/// oldtpl: IN
	pub restore_tpl: unsafe extern "efiapi" fn(oldtpl: TaskPriorityLevel),
	/// allocatetype: IN, memorytype: IN, pages: IN, memory: IN OUT
	pub allocate_pages: unsafe extern "efiapi" fn(allocatetype: AllocateType, memorytype: MemoryType, pages: usize, memory: *mut PhysicalAddress) -> Status,
	/// memory: IN, pages: IN
	pub free_pages: unsafe extern "efiapi" fn(memory: PhysicalAddress, pages: usize) -> Status,
	/// memorymapsize: IN OUT, memorymap: OUT, mapkey: OUT, descriptorsize: OUT, descriptorversion: OUT
	pub get_memory_map: unsafe extern "efiapi" fn(memorymapsize: *mut usize, memorymap: *mut MemoryDescriptor, mapkey: *mut usize, descriptorsize: *mut usize, descriptorversion: *mut u32) -> Status,
	/// pooltype: IN, size: IN, buffer: OUT
	pub allocate_pool: unsafe extern "efiapi" fn(pooltype: MemoryType, size: usize, buffer: *mut *mut Void) -> Status,
	/// buffer: IN
	pub free_pool: unsafe extern "efiapi" fn(buffer: *const Void) -> Status,
	/// eventtype: IN, notifytpl: IN, notifyfunction: IN, notifycontext: IN, event: OUT
	pub create_event: unsafe extern "efiapi" fn(eventtype: u32, notifytpl: TaskPriorityLevel, notifyfunction: Option<extern "efiapi" fn(event: Event, context: *mut Void)>, notifycontext: *mut Void, event: *mut Event) -> Status,
	/// event: IN, triggertimetype: IN, triggertime: IN
	pub set_timer: unsafe extern "efiapi" fn(event: Event, triggertimetype: TimerDelay, triggertime: u64) -> Status,
	/// numevents: IN, event: IN, index: OUT
	pub wait_for_event: unsafe extern "efiapi" fn(numevents: usize, event: *const Event, index: *mut usize) -> Status,
	/// event: IN
	pub signal_event: unsafe extern "efiapi" fn(event: Event) -> Status,
	/// event: IN
	pub close_event: unsafe extern "efiapi" fn(event: Event) -> Status,
	/// event: IN
	pub check_event: unsafe extern "efiapi" fn(event: Event) -> Status,
	/// handle: IN OUT, protocol: IN, interfacetype: IN, interface: IN
	pub install_protocol_interface: unsafe extern "efiapi" fn(handle: *mut *mut Void, protocol: *const GUID, interfacetype: InterfaceType, interface: *mut Void) -> Status,
	/// handle: IN, protocol: IN, oldinterface: IN, newinterface: IN
	pub reinstall_protocol_interface: unsafe extern "efiapi" fn(handle: *mut Void, protocol: *const GUID, oldinterface: *mut Void, newinterface: *mut Void) -> Status,
	/// handle: IN, protocol: IN, interface: IN
	pub uninstall_protocol_interface: unsafe extern "efiapi" fn(handle: *mut Void, protocol: *const GUID, interface: *mut Void) -> Status,
	/// handle: IN, protocol: IN, interface: OUT
	pub handle_protocol: unsafe extern "efiapi" fn(handle: *mut Void, protocol: *const GUID, interface: *mut *const Void) -> Status,
	/// unused
	reserved: unsafe extern "efiapi" fn() -> Status,
	/// protocol: IN, event: IN, registration: OUT,
	pub register_protocol_notify: unsafe extern "efiapi" fn(protocol: *const GUID, event: Event, registration: *mut *mut Void) -> Status,
	/// searchtype: IN, protocol: IN, searchkey: IN, buffersize: IN OUT, buffer: OUT
	pub locate_handle: unsafe extern "efiapi" fn(searchtype: LocateSearchType, protocol: *const GUID, searchkey: *mut Void, buffersize: *mut usize, buffer: *mut *mut Void) -> Status,
	/// protocol: IN, devicepath: IN OUT, device: OUT
	pub locate_device_path: unsafe extern "efiapi" fn(protocol: *const GUID, devicepath: *mut *const DevicePathProtocol, device: *mut *mut Void) -> Status,
	/// guid: IN, table: IN
	pub install_configuration_table: unsafe extern "efiapi" fn(guid: *const GUID, table: *mut Void) -> Status,
	/// bootpolicy: IN, parentimagehandle: IN, devicepath: IN, sourcebuffer: IN, sourcesize: IN, imagehandle: OUT
	pub load_image: unsafe extern "efiapi" fn(bootpolicy: bool, parentimagehandle: *mut Void, devicepath: *const DevicePathProtocol, sourcebuffer: *mut Void, sourcesize: usize, imagehandle: *mut *mut Void) -> Status,
	/// imagehandle: IN, exitdatasize: OUT, exitdata: OUT
	pub start_image: unsafe extern "efiapi" fn(imagehandle: *mut Void, exitdatasize: *mut usize, exitdata: *mut *const Char16) -> Status,
	/// imagehandle: IN, exitstatus: IN, exitdatasize: IN, exitdata: IN
	pub exit: unsafe extern "efiapi" fn(imagehandle: *mut Void, exitstatus: Status, exitdatasize: usize, exitdata: *const Char16) -> Status,
	/// imagehandle: IN
	pub unload_image: unsafe extern "efiapi" fn(imagehandle: *mut Void) -> Status,
	/// imagehandle: IN, mapkey: IN
	pub exit_boot_services: unsafe extern "efiapi" fn(imagehandle: *mut Void, mapkey: usize) -> Status,
	/// count: OUT
	pub get_next_monotonic_count: unsafe extern "efiapi" fn(count: *mut u64) -> Status,
	/// microseconds: IN
	pub stall: unsafe extern "efiapi" fn(microseconds: usize) -> Status,
	/// timeout: IN, watchdogcode: IN, datasize: IN, watchdogdata: IN
	pub set_watchdog_timer: unsafe extern "efiapi" fn(timeout: usize, watchdogcode: u64, datasize: usize, watchdogdata: Option<&Char16>) -> Status, // EFI 1.0
	/// controllerhandle: IN, driverimagehandle: IN, remainingdevicepath: IN, recursive: IN
	pub connect_controller: unsafe extern "efiapi" fn(controllerhandle: *mut Void, driverimagehandle: *mut Void, remainingdevicepath: *const DevicePathProtocol, recursive: bool) -> Status,
	/// controllerhandle: IN, driverimagehandle: IN, childhandle: IN
	pub disconnect_controller: unsafe extern "efiapi" fn(controllerhandle: *mut Void, driverimagehandle: *mut Void, childhandle: *mut Void) -> Status,
	/// handle: IN, protocol: IN, interface: OUT, agenthandle: IN, controllerhandle: IN, attributes: IN
	pub open_protocol: unsafe extern "efiapi" fn(handle: *const Void, protocol: &GUID, interface: *mut *const Void, agenthandle: *mut Void, controllerhandle: *mut Void, attributes: u32) -> Status,
	/// handle: IN, protocol: IN, agenthandle: IN, controllerhandle: IN
	pub close_protocol: unsafe extern "efiapi" fn(handle: *const Void, protocol: &GUID, agenthandle: *mut Void, controllerhandle: *mut Void) -> Status,
	/// handle: IN, protocol: IN, entrybuffer: OUT, entrycount: OUT
	pub open_protocol_information: unsafe extern "efiapi" fn(handle: *mut Void, protocol: &GUID, entrybuffer: *mut *const OpenProtocolInformationEntry, entrycount: *mut usize) -> Status,
	/// handle: IN, protocolbuffer: OUT, protocolBuffercount: OUT
	pub protocols_per_handle: unsafe extern "efiapi" fn(handle: *mut Void, protocolbuffer: *mut *const *const GUID, protocolbuffercount: *mut usize) -> Status,
	/// searchtype: IN, protocol: IN, searchkey: IN, numhandles: OUT, buffer: OUT
	pub locate_handle_buffer: unsafe extern "efiapi" fn(searchtype: LocateSearchType, protocol: *const GUID, searchkey: *mut Void, numhandles: *mut usize, buffer: *mut *const *mut Void) -> Status,
	/// protocol: IN, registration: IN, interface: OUT
	pub locate_protocol: unsafe extern "efiapi" fn(protocol: *const GUID, registration: *mut Void, interface: *mut *const Void) -> Status,
	/// handle: IN OUT, ...: pairs of protocol GUID and protocol interface
	pub install_multiple_protocol_interfaces: unsafe extern "efiapi" fn(handle: *mut *mut Void, ...),
	/// handle: IN, ...: pairs of protocol GUID and protocol interface
	pub uninstall_multiple_protocol_interfaces: unsafe extern "efiapi" fn(handle: *mut Void, ...),
	/// data: IN, datasize: IN, crc32: OUT
	pub calculate_crc32: unsafe extern "efiapi" fn(data: *mut Void, datasize: usize, crc32: *mut u32) -> Status,
	/// destination: IN, source: IN, length: IN
	pub copy_mem: unsafe extern "efiapi" fn(destination: *mut Void, source: *const Void, length: usize),
	/// buffer: IN, size: IN, value: IN
	pub set_mem: unsafe extern "efiapi" fn(buffer: *mut Void, size: usize, value: u8), // EFI 1.1
	/// eventtype: IN, notifytpl: IN, notifyfuntion: IN, notifycontext: IN CONST, eventgroup: IN CONST, event: OUT
	pub create_event_ex: unsafe extern "efiapi" fn(eventtype: u32, notifytpl: TaskPriorityLevel, notifyfunction: Option<extern "efiapi" fn(event: Event, context: &Void)>, notifycontext: Option<&Void>, eventgroup: Option<&GUID>, event: &mut Event) -> Status, // EFI 2.0
}
impl BootServices {
	pub const SIGNATURE: u64 = 0x56524553544F4F42;
	pub const REVISION: u64 = crate::tables::SystemTable::SPECIFICATION_VERSION;
	/// This function raises the priority of the currently executing task and returns its previous priority level.
	/// Only three task priority levels are exposed outside of the firmware during boot services execution. The first is
	/// [`TaskPriorityLevel::Application`] where all normal execution occurs. That level may be interrupted to perform various asynchronous
	/// interrupt style notifications, which occur at the [`TaskPriorityLevel::Callback`] or [`TaskPriorityLevel::Notify`] level. By raising the
	/// task priority level to [`TaskPriorityLevel::Notify`] such notifications are masked until the task priority level is restored, thereby
	/// synchronizing execution with such notifications. Synchronous blocking I/O functions execute at [`TaskPriorityLevel::Notify`].
	/// [`TaskPriorityLevel::Callback`] is the typically used for application level notification functions. Device drivers will typically
	/// use [`TaskPriorityLevel::Callback`] or [`TaskPriorityLevel::Notify`] for their notification functions. Applications and drivers may also use
	/// [`TaskPriorityLevel::Notify`] to protect data structures in critical sections of code.
	///
	/// The caller must restore the task priority level with [`BootServices::restoretpl`] to the previous level before
	/// returning.
	///
	/// # Safety
	///
	/// If NewTpl is below the current [`TaskPriorityLevel`] level, then the system behavior is indeterminate. Additionally, only
	/// [`TaskPriorityLevel::Application`], [`TaskPriorityLevel::Callback`], [`TaskPriorityLevel::Notify`], and [`TaskPriorityLevel::HighLevel`] may be used. All other values
	/// are reserved for use by the firmware; using them will result in unpredictable behavior. Good coding practice dictates
	/// that all code should execute at its lowest possible [`TaskPriorityLevel`] level, and the use of [`TaskPriorityLevel`] levels above [`TaskPriorityLevel::Application`] must be minimized.
	/// Executing at [`TaskPriorityLevel`] levels above [`TaskPriorityLevel::Application`] for extended periods of time may also result in unpredictable behavior.
	///
	/// # Status Codes Returned
	///
	/// Unlike other UEFI interface functions, this function does not return a status code. Instead, it
	/// returns the previous task priority level, which is to be restored later with a matching call to [`BootServices::restoretpl`].
	pub unsafe fn raise_tpl(&self, newtpl: TaskPriorityLevel) -> TaskPriorityLevel {
		// SAFETY:
		unsafe { (self.raise_tpl)(newtpl) }
	}

	/// This function restores a task’s priority level to its previous value. Calls to this function are matched
	/// with calls to [`BootServices::raisetpl`].
	///
	/// # Safety
	///
	/// If OldTpl is above the current [`TaskPriorityLevel`] level, then the system behavior is indeterminate. Additionally, only
	/// [`TaskPriorityLevel::Application`], [`TaskPriorityLevel::Callback`], [`TaskPriorityLevel::Notify`], and [`TaskPriorityLevel::HighLevel`] *may be used*. All other values
	/// are reserved for use by the firmware; using them will result in unpredictable behavior. Good coding practice dictates
	/// that all code should execute at its lowest possible [`TaskPriorityLevel`] level, and the use of [`TaskPriorityLevel`] levels above [`TaskPriorityLevel::Application`] must be minimized.
	/// Executing at [`TaskPriorityLevel`] levels above [`TaskPriorityLevel::Application`] for extended periods of time may also result in unpredictable behavior.
	///
	/// # Status Codes Returned
	///
	/// None.
	pub unsafe fn restore_tpl(&self, oldtpl: TaskPriorityLevel) {
		// SAFETY:
		unsafe { (self.restore_tpl)(oldtpl) }
	}
	pub fn wait_for_event(&self, events: &[Event]) -> Result<usize, Status> {
		let mut index = 0;
		// SAFETY:
		unsafe { (self.wait_for_event)(events.len(), events.as_ptr(), &mut index) }
			.into_result(index)
	}
	// pub fn handle_protocol<T: Protocol>(&self, handle: *mut Void) -> Result<*mut Void<T>, Status> {
	// 	let mut interface = core::ptr::null();
	// 	// SAFETY:
	// 	// Should be safe because to call this we need a valid implementation of Protocol and even
	// 	// if you implement Protocol on an invalid structure this should be fine because we check
	// 	// the Status returned by handleprotocol
	// 	unsafe { (self.handle_protocol)(handle, &T::GUID, &mut interface) }
	// 		// SAFETY:
	// 		// Should be safe because we got back a valid interface from a valid protocol
	// 		.map(|| unsafe { *mut Void::new_unchecked(interface as *mut T) })
	// }
	// pub fn stall(&self, microseconds: usize) -> Result<(), Status> {
	// 	// SAFETY:
	// 	unsafe { (self.stall)(microseconds) }.into_result(())
	// }
	// pub fn locate_protocol<T: Protocol>(&self, registration: Option<*mut Void>) -> Result<*mut Void<T>, Status> {
	// 	let mut interface = core::ptr::null();
	// 	// SAFETY:
	// 	unsafe { (self.locate_protocol)(&T::GUID, registration, &mut interface) }
	// 		// SAFETY:
	// 		.map(|| unsafe { *mut Void::new_unchecked(interface as *mut T) })
	// }
	// pub fn open_protocol<T: Protocol>(&self, handle: &*mut Void, agenthandle: *mut Void, controllerhandle: *mut Void, attributes: u32) -> Result<*mut Void<T>, Status> {
	// 	let mut interface = core::ptr::null();
	// 	// SAFETY:
	// 	unsafe { (self.open_protocol)(handle.as_ptr(), &T::GUID, Some(&mut interface), agenthandle, controllerhandle, attributes) }
	// 		// SAFETY:
	// 		.map(|| unsafe { *mut Void::new_unchecked(interface as *mut T) })
	// }
	// pub fn close_protocol<T: Protocol>(&self, handle: &*mut Void, agenthandle: *mut Void, controllerhandle: *mut Void) -> Result<(), Status> {
	// 	// SAFETY:
	// 	unsafe { (self.close_protocol)(handle.as_ptr(), &T::GUID, agenthandle, controllerhandle) }
	// 		.into_result(())
	// }
}

#[repr(C)]
pub struct OpenProtocolInformationEntry {
	pub agenthandle: *const (),
	pub controllerhandle: *const (),
	pub attributes: u32,
	pub opencount: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ResetType(u32);
impl ResetType {
	pub const COLD: Self = Self(0x00);
	pub const WARM: Self = Self(0x01);
	pub const SHUTDOWN: Self = Self(0x02);
	pub const PLATFORM_SPECIFIC: Self = Self(0x03);
}

#[repr(C)]
pub struct RuntimeServices {
	pub header: TableHeader,
	/// time: OUT, capabilities: OUT
	pub get_time: unsafe extern "efiapi" fn(time: *mut Time, capabilities: *mut TimeCapabilities) -> Status,
	/// time: IN
	pub set_time: unsafe extern "efiapi" fn(time: *const Time) -> Status,
	/// enabled: OUT, pending: OUT, time: OUT
	pub get_wakeup_time: unsafe extern "efiapi" fn(enabled: *mut Bool, pending: *mut Bool, time: *mut Time) -> Status,
	/// enabled: IN, time: IN
	pub set_wakeup_time: unsafe extern "efiapi" fn(enabled: bool, time: *const Time) -> Status,
	/// memorymapsize: IN, descriptorsize: IN, descriptorversion: IN, virtualmap: IN
	pub set_virtual_address_map: unsafe extern "efiapi" fn(memorymapsize: usize, descriptorsize: usize, descriptorversion: u32, virtualmap: *const MemoryDescriptor) -> Status,
	/// debugdispostition: IN, address: IN
	pub convert_pointer: unsafe extern "efiapi" fn(debugdispostition: usize, address: *mut *const Void) -> Status,
	/// variablename: IN, vendorguid: IN, attributes: OUT, datasize: IN OUT, data: OUT
	pub get_variable: unsafe extern "efiapi" fn(variablename: *const Char16, vendorguid: *const GUID, attributes: *mut u32, datasize: *mut usize, data: *mut Void) -> Status,
	/// variablenamesize: IN OUT, variablename: IN OUT, vendorguid: IN OUT
	pub get_next_variable_name: unsafe extern "efiapi" fn(variablenamesize: *mut usize, variablename: *mut Char16, vendorguid: *mut GUID) -> Status,
	/// variablename: IN, vendorguid: IN, attributes: IN, datasize: IN, data: IN
	pub set_variable: unsafe extern "efiapi" fn(variablename: *const Char16, vendorguid: *const GUID, attributes: u32, datasize: usize, data: *const Void) -> Status,
	/// highcount: OUT
	pub get_next_high_monotonic_count: unsafe extern "efiapi" fn(highcount: *mut u32) -> Status,
	/// resettype: IN, resetstatus: IN, datasize: IN, resetdata: IN
	pub reset_system: unsafe extern "efiapi" fn(resettype: ResetType, resetstatus: Status, datasize: usize, resetdata: *const Void) -> !,
	/// capsuleheaderarray: IN, capsulecount: IN, scattergatherlist: IN
	pub update_capsule: unsafe extern "efiapi" fn(capsuleheaderarray: *const *const CapsuleHeader, capsulecount: usize, scattergatherlist: PhysicalAddress) -> Status,
	/// capsuleheaderarray: IN, capsulecount: IN, maximumcapsulesize: OUT, resettype: OUT
	pub query_capsule_capabilities: unsafe extern "efiapi" fn(capsuleheaderarray: *const *const CapsuleHeader, capsulecount: usize, maximumcapsulesize: *mut u64, resettype: *mut ResetType) -> Status,
	// attributes: IN, maxvariablestoragesize: OUT, remainingvariablestoragesize: OUT, maxvariablesize: OUT
	pub query_variable_info: unsafe extern "efiapi" fn(attributes: u32, maxvariablestoragesize: *mut u64, remainingvariablestoragesize: *mut u64, maxvariablesize: *mut u64) -> Status,
}
impl RuntimeServices {
	pub const SIGNATURE: u64 = 0x56524553544E5552;
	pub const REVISION: u64 = crate::tables::SystemTable::SPECIFICATION_VERSION;
}
