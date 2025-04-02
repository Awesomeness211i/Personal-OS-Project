use crate::{
	GUID,
	Bool,
	MacAddress,
	IpV4Address,
	IpV6Address,
	memory::MemoryType,
};

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Data<T> {
	pub len: u16,
	pub data: T,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pci {
	func: u8,
	device: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PcCard {
	func: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MemoryMapped {
	memtype: MemoryType,
	start: u64,
	end: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HardwareVendor {
	guid: GUID,
	/// variable length
	data: [u8; 0]
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Controller(u32);

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BaseboardManagementController {
	interface: u8,
	base: u64,
}

#[repr(C, u8)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HardwareType {
	Pci(Data<Pci>) = 0x01,
	PcCard(Data<PcCard>) = 0x02,
	MemoryMapped(Data<MemoryMapped>) = 0x03,
	Vendor(Data<HardwareVendor>) = 0x04,
	Controller(Data<Controller>) = 0x05,
	BaseboardManagementController(Data<BaseboardManagementController>) = 0x06,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Acpi {
	_hid: u32,
	_uid: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AcpiExpanded {
	acpi: Acpi,
	_cid: u32,
	// _hidstr: ,
	// _uidstr: ,
	// _cidstr: ,
	/// variable length
	_strs: [u8; 0],
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Adr {
	pub _adr: u32,
	_adrs: [u32; 0],
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Nvdimm {
	nfit: u32,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AcpiType {
	Acpi(Data<Acpi>) = 0x01,
	AcpiExpanded(Data<AcpiExpanded>) = 0x02,
	Adr(Data<Adr>) = 0x03,
	Nvdimm(Data<Nvdimm>) = 0x04,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Atapi {
	primarysecondary: u8,
	slavemaster: u8,
	logicalunit: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Scsi {
	target: u16,
	logicalunit: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FiberChannel {
	reserved: u32,
	worldwidename: u64,
	logicalunit: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DevicePath1394 {
	reserved: u32,
	guid: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Usb {
	parentport: u8,
	interface: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct I2ORandomBlockStorageClass {
	tid: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Infiniband {
	// Bit 0: 0 = IOC, 1 = Service
	// Bit 1: Extend Boot Environment
	// Bit 2: Console Protocol
	// Bit 3: Storage Protocol
	// Bit 4: Network Protocol
	// Bits 5-31 = reserved
	resourceflags: u32,
	portgid: u128,
	iocguidserviceid: u64,
	targetport: u64,
	deviceid: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MessagingVendor {
	guid: GUID,
	/// variable length
	data: [u8; 0]
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MacAddressNetworkInterface {
	address: MacAddress,
	interface: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ipv4 {
	local: IpV4Address,
	remote: IpV4Address,
	localport: u16,
	remoteport: u16,
	protocol: u16,
	// 0x00 = DHCP
	// 0x01 = Static
	staticip: u8,
	gateway: IpV4Address,
	subnetmask: IpV4Address,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ipv6 {
	local: IpV6Address,
	remote: IpV6Address,
	localport: u16,
	remoteport: u16,
	protocol: u16,
	// 0x00 = local ip manual
	// 0x01 = stateless auto
	// 0x02 = stateful config
	origin: u8,
	prefixlength: u8,
	gateway: IpV6Address,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Uart {
	reserved: u32,
	baudrate: u64,
	databits: u8,
	parity: u8,
	stop: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UsbClass {
	vendor: u16,
	product: u16,
	class: u8,
	subclass: u8,
	protocol: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UsbWwid {
	interface: u16,
	vendor: u16,
	product: u16,
	/// variable length
	serial: [u8; 0],
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LogicalUnit {
	logicalunit: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Sata {
	hbaport: u16,
	portmultiplier: u16,
	logicalunit: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Vlan {
	vlanid: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Iscsi {
	protocol: u16,
	options: u16,
	logicalunit: u64,
	targetportal: u16,
	/// variable length
	targetname: [u8; 0],
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SasEx {
	address: u64,
	logicalunit: u64,
	deviceinfo: u16,
	targetport: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NvmExpress {
	namespaceid: u32,
	extendeduid: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ufs {
	targetid: u8,
	lun: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Sd {
	slotnumber: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bluetooth {
	address: [u8; 6],
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WiFi {
	ssid: [u8; 32],
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Emmc {
	slotnumber: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BluetoothLE {
	address: Bluetooth,
	// 0x00 = Public Address
	// 0x01 = Random Address
	addresstype: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Dns {
	// 0x00 = IPv4
	// 0x01 = IPv6
	is_ipv6: u8,
	/// variable length
	dns: [u8; 0],
}

#[repr(C, u8)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MessagingType {
	Atapi(Data<Atapi>) = 0x01,
	Scsi(Data<Scsi>) = 0x02,
	FiberChannel(Data<FiberChannel>) = 0x03,
	DevicePath1394(Data<DevicePath1394>) = 0x04,
	Usb(Data<Usb>) = 0x05,
	I2ORandomBlockStorageClass(Data<I2ORandomBlockStorageClass>) = 0x06,
	Infiniband(Data<Infiniband>) = 0x09,
	Vendor(Data<MessagingVendor>) = 0x0A,
	MacAddressNetworkInterface(Data<MacAddressNetworkInterface>) = 0x0B,
	Ipv4(Data<Ipv4>) = 0x0C,
	Ipv6(Data<Ipv6>) = 0x0D,
	Uart(Data<Uart>) = 0x0E,
	UsbClass(Data<UsbClass>) = 0x0F,
	UsbWwid(Data<UsbWwid>) = 0x10,
	LogicalUnit(Data<LogicalUnit>) = 0x11,
	Sata(Data<Sata>) = 0x12,
	Iscsi(Data<Iscsi>) = 0x13,
	Vlan(Data<Vlan>) = 0x14,
	FiberChannelEx(Data<FiberChannel>) = 0x15,
	SasEx(Data<SasEx>) = 0x16,
	NvmExpress(Data<NvmExpress>) = 0x17,
	UniformResourceIdentifier(Data<()>) = 0x18,
	Ufs(Data<Ufs>) = 0x19,
	Sd(Data<Sd>) = 0x1A,
	Bluetooth(Data<Bluetooth>) = 0x1B,
	WiFi(Data<WiFi>) = 0x1C,
	Emmc(Data<Emmc>) = 0x1D,
	BluetoothLE(Data<BluetoothLE>) = 0x1E,
	Dns(Data<Dns>) = 0x1F,
	Nvdimm(Data<()>) = 0x20,
	Rest(Data<()>) = 0x21,
	Nvme(Data<()>) = 0x22,
}
impl MessagingType {
	pub const PC_ANSI_GUID: GUID = GUID::new(0xE0C14753, 0xF9BE, 0x11D2, 0x9A0C_0090273FC14D);
	pub const VT_100_GUID: GUID = GUID::new(0xDFA66065, 0xB419, 0x11D3, 0x9A2D_0090273FC14D);
	pub const VT_100_PLUS_GUID: GUID = GUID::new(0x7BAEC70B, 0x57E0, 0x4C76, 0x8E87_2F9E28088343);
	pub const VT_UTF8_GUID: GUID = GUID::new(0xAD15A0D6,0x8BEC,0x4ACF,0xA073_D01DE77E2D88);
	pub const UART_FLOW_CONTROL_GUID: GUID = GUID::new(0x37499A9D,0x542F,0x4C89,0xA026_35DA142094E4);
}

#[repr(C, u8)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
	HardDrive(Data<()>) = 0x01,
	CdRom(Data<()>) = 0x02,
	Vendor(Data<()>) = 0x03,
	Filepath(Data<()>) = 0x04,
	MediaProtocol(Data<()>) = 0x05,
	PiwgFirmwareFile(Data<()>) = 0x06,
	PiwgFirmwareVolume(Data<()>) = 0x07,
	RelativeOffsetRange(Data<()>) = 0x08,
	RamDisk(Data<()>) = 0x09,
}

#[repr(C, u8)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BiosBootSpecType {
	Version(Data<()>) = 0x01,
}

#[repr(C, u8)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EndHardwareType {
	StartNew(Data<()>) = 0x01,
	EndEntire(Data<()>) = 0xFF,
}

#[repr(C, u8)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PathType {
	Hardware(HardwareType) = 0x01,
	Acpi(AcpiType) = 0x02,
	Messaging(MessagingType) = 0x03,
	Media(MediaType) = 0x04,
	BiosBootSpec(BiosBootSpecType) = 0x05,
	EndHardware(EndHardwareType) = 0x7F,
}

/// Not sure if this should exist yet
#[repr(C)]
pub struct DevicePathProtocol(pub PathType);
impl super::Protocol for DevicePathProtocol {
	/// GUID: 09576E91-6D3F-11D2-8E39-00A0C969723B
	const GUID: GUID = GUID::new(0x09576E91, 0x6D3F, 0x11D2, 0x8E39_00A0C969723B);
}

#[repr(C)]
pub struct DevicePathUtilitiesProtocol {
	/// devicepath: IN CONST
	get_device_path_size: unsafe extern "efiapi" fn(device_path: *const DevicePathProtocol) -> usize,
	/// devicepath: IN CONST
	duplicate_device_path: unsafe extern "efiapi" fn(device_path: *const DevicePathProtocol) -> *const DevicePathProtocol,
	/// src1: IN CONST, src2: IN CONST
	append_device_path: unsafe extern "efiapi" fn(src1: *const DevicePathProtocol, src2: *const DevicePathProtocol) -> *const DevicePathProtocol,
	/// devicepath: IN CONST, devicenode: IN CONST
	append_device_node: unsafe extern "efiapi" fn(device_path: *const DevicePathProtocol, device_node: *const DevicePathProtocol) -> *const DevicePathProtocol,
	/// devicepath: IN CONST, devicepathinstance: IN CONST
	append_device_path_instance: unsafe extern "efiapi" fn(device_path: *const DevicePathProtocol, device_path_instance: *const DevicePathProtocol) -> *const DevicePathProtocol,
	/// devicepathinstance: IN OUT, devicepathinstancesize: OUT
	get_next_device_path_instance: unsafe extern "efiapi" fn(device_path_instance: *mut *const DevicePathProtocol, device_path_instance_size: *mut usize) -> *const DevicePathProtocol,
	/// devicepath: IN CONST
	is_device_path_multi_instance: unsafe extern "efiapi" fn(device_path: *const DevicePathProtocol) -> Bool,
	/// nodetype: IN, nodesubtype: IN, nodelength: IN
	create_device_node: unsafe extern "efiapi" fn(node_type: u8, node_subtype: u8, node_length: u16) -> *const DevicePathProtocol,
}

impl super::Protocol for DevicePathUtilitiesProtocol {
	/// GUID: 0379BE4E-D706-437D-B037-EDB82FB772A4
	const GUID: GUID = GUID::new(0x0379BE4E, 0xD706, 0x437D, 0xB037_EDB82FB772A4);
}
