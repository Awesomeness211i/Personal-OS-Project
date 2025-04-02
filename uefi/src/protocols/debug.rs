use crate::{
	Void,
	GUID,
	status::Status,
};

#[repr(C)]
pub struct SystemContextEBC {
	r0: u64,
	r1: u64,
	r2: u64,
	r3: u64,
	r4: u64,
	r5: u64,
	r6: u64,
	r7: u64,
	flags: u64,
	controlflags: u64,
	ip: u64,
}
impl SystemContextEBC {
	pub const EXCEPT_UNDEFINED: isize = 0;
	pub const EXCEPT_DIVIDE_ERROR: isize = 1;
	pub const EXCEPT_DEBUG: isize = 2;
	pub const EXCEPT_BREAKPOINT: isize = 3;
	pub const EXCEPT_OVERFLOW: isize = 4;
	pub const EXCEPT_INVALID_OPCODE: isize = 5;
	pub const EXCEPT_STACK_FAULT: isize = 6;
	pub const EXCEPT_ALIGNMENT_CHECK: isize = 7;
	pub const EXCEPT_INSTRUCTION_ENCODING: isize = 8;
	pub const EXCEPT_BAD_BREAK: isize = 9;
	pub const EXCEPT_SINGLE_STEP: isize = 10;
}

/// Generic so that it makes it easier to implement register sizes only use u32, u64 and u128
#[repr(C)]
pub struct SystemContextRiscV<T> {
	// integer registers
	zero: T, ra: T, sp: T, gp: T, tp: T, t0: T, t1: T, t2: T,
	s0fp: T, s1: T, a0: T, a1: T, a2: T, a3: T, a4: T, a5: T, a6: T, a7: T,
	s2: T, s3: T, s4: T, s5: T, s6: T, s7: T, s8: T, s9: T, s10: T, s11: T,
	t3: T, t4: T, t5: T, t6: T,

	// Floating registers for F, D and Q Standard Extensions
	ft0: u128, ft1: u128, ft2: u128, ft3: u128, ft4: u128, ft5: u128, ft6: u128, ft7: u128,
	fs0: u128, fs1: u128, fa0: u128, fa1: u128, fa2: u128, fa3: u128, fa4: u128, fa5: u128, fa6: u128, fa7: u128,
	fs2: u128, fs3: u128, fs4: u128, fs5: u128, fs6: u128, fs7: u128, fs8: u128, fs9: u128, fs10: u128, fs11: u128,
	ft8: u128, ft9: u128, ft10: u128, ft11: u128,
}
impl<T> SystemContextRiscV<T> {
	pub const EXCEPT_INST_MISALIGNED: isize = 0;
	pub const EXCEPT_INST_ACCESS_FAULT: isize = 1;
	pub const EXCEPT_ILLEGAL_INST: isize = 2;
	pub const EXCEPT_BREAKPOINT: isize = 3;
	pub const EXCEPT_LOAD_ADDRESS_MISALIGNED: isize = 4;
	pub const EXCEPT_LOAD_ACCESS_FAULT: isize = 5;
	pub const EXCEPT_STORE_AMO_ADDRESS_MISALIGNED: isize = 6;
	pub const EXCEPT_STORE_AMO_ACCESS_FAULT: isize = 7;
	pub const EXCEPT_ENV_CALL_FROM_UMODE: isize = 8;
	pub const EXCEPT_ENV_CALL_FROM_SMODE: isize = 9;
	pub const EXCEPT_ENV_CALL_FROM_MMODE: isize = 11;
	pub const EXCEPT_INST_PAGE_FAULT: isize = 12;
	pub const EXCEPT_LOAD_PAGE_FAULT: isize = 13;
	pub const EXCEPT_STORE_AMO_PAGE_FAULT: isize = 15;

	pub const EXCEPT_SUPERVISOR_SOFTWARE_INT: isize = 1;
	pub const EXCEPT_MACHINE_SOFTWARE_INT: isize = 3;
	pub const EXCEPT_SUPERVISOR_TIMER_INT: isize = 5;
	pub const EXCEPT_MACHINE_TIMER_INT: isize = 7;
	pub const EXCEPT_SUPERVISOR_EXTERNAL_INT: isize = 9;
	pub const EXCEPT_MACHINE_EXTERNAL_INT: isize = 11;
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Ia32 {
	eip: u32,
	cs: u16, reserved1: u16,
	dataoffset: u32,
	ds: u16,
	reserved2: [u8; 10],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct X64 {
	rip: u64,
	dataoffset: u64,
	reserved1: [u8; 8],
}

#[repr(C)]
union State {
	ia32: Ia32,
	x64: X64,
}

#[repr(C)]
pub struct FXSaveState {
	fcw: u16, fsw: u16, ftw: u16, opcode: u16,
	specific: State,
	st0mm0: [u8; 10], reserved3: [u8; 6],
	st1mm1: [u8; 10], reserved4: [u8; 6],
	st2mm2: [u8; 10], reserved5: [u8; 6],
	st3mm3: [u8; 10], reserved6: [u8; 6],
	st4mm4: [u8; 10], reserved7: [u8; 6],
	st5mm5: [u8; 10], reserved8: [u8; 6],
	st6mm6: [u8; 10], reserved9: [u8; 6],
	st7mm7: [u8; 10], reserved10: [u8; 6],
	xmm0: [u8; 16],
	xmm1: [u8; 16],
	xmm2: [u8; 16],
	xmm3: [u8; 16],
	xmm4: [u8; 16],
	xmm5: [u8; 16],
	xmm6: [u8; 16],
	xmm7: [u8; 16],
	reserved11: [u8; 14 * 16],
}

#[repr(C)]
pub struct SystemContextIa32 {
	exceptiondata: u32,
	fxsavestate: FXSaveState,

	dr0: u32, dr1: u32, dr2: u32, dr3: u32, dr4: u32, dr5: u32, dr6: u32, dr7: u32,
	cr0: u32, cr1: u32, cr2: u32, cr3: u32, cr4: u32,
	eflags: u32,
	ldtr: u32, tr: u32,
	gdtr: [u32; 2], idtr: [u32; 2],
	eip: u32,
	gs: u32, fs: u32, es: u32, ds: u32, cs: u32, ss: u32,
	edi: u32, esi: u32, ebp: u32, esp: u32, ebx: u32, edx: u32, ecx: u32, eax: u32,
}

/// exceptiontype: IN, systemcontext: IN OUT
type ExceptionCallback = extern "efiapi" fn(exceptiontype: isize, systemcontext: *const Void) -> Status;

#[repr(C)]
pub struct DebugSupportProtocol {
	isa: InstructionSetArchitecture,
	/// this: IN, maxprocessorindex: OUT
	getmaximumprocessorindex: unsafe extern "efiapi" fn(this: &Self, maxprocessorindex: &mut usize) -> Status,
	/// this: IN, processorindex: IN, periodiccallback: IN OUT
	registerperiodiccallback: unsafe extern "efiapi" fn(this: &Self, processorindex: usize, periodiccallback: *const Void) -> Status,
	/// this: IN, processorindex: IN, exceptioncallback: IN, exceptiontype: IN
	registerexceptioncallback: unsafe extern "efiapi" fn(this: &Self, processorindex: usize, exceptioncallback: ExceptionCallback, exceptiontype: isize) -> Status,
	/// this: IN, processorindex: IN, start: IN, length: IN
	invalidateinstructioncache: unsafe extern "efiapi" fn(this: &Self, processorindex: usize, start: *const Void, length: u64) -> Status,
}
impl super::Protocol for DebugSupportProtocol {
	const GUID: GUID = GUID::new(0x2755590C, 0x6F3C, 0x42FA, 0x9EA4_A3BA543CDA25);
}

#[repr(C)]
pub struct DebugPortProtocol {
	/// this: IN
	reset: unsafe extern "efiapi" fn(this: &Self),
	/// this: IN, timeout: IN, buffersize: IN OUT, buffer: IN
	write: unsafe extern "efiapi" fn(this: &Self, timeout: u32, buffersize: &mut usize, buffer: *const Void),
	/// this: IN, timeout: IN, buffersize: IN OUT, buffer: OUT
	read: unsafe extern "efiapi" fn(this: &Self, timeout: u32, buffersize: &mut usize, buffer: *mut Void),
	/// this: IN
	poll: unsafe extern "efiapi" fn(this: &Self),
}
impl super::Protocol for DebugPortProtocol {
	const GUID: GUID = GUID::new(0xEBA4E8D2, 0x3858, 0x41EC, 0xA281_2647BA9660D0);
}

// #[repr(C, u32)]
// pub enum systemcontext {
// 	/// I386
// 	Ia32 {
// 		exceptiondata: u32,
// 		fxsavestate: FXSaveState,
//
// 		dr0: u32, dr1: u32, dr2: u32, dr3: u32, dr4: u32, dr5: u32, dr6: u32, dr7: u32,
// 		cr0: u32, cr1: u32, cr2: u32, cr3: u32, cr4: u32,
// 		eflags: u32,
// 		ldtr: u32, tr: u32,
// 		gdtr: [u32; 2], idtr: [u32; 2],
// 		eip: u32,
// 		gs: u32, fs: u32, es: u32, ds: u32, cs: u32, ss: u32,
// 		edi: u32, esi: u32, ebp: u32, esp: u32, ebx: u32, edx: u32, ecx: u32, eax: u32,
// 	} = InstructionSetArchitecture::Ia32 as u32,
// 	/// x64
// 	X64 {
// 		exceptiondata: u64,
// 		fxsavestate: FXSaveState,
//
// 		dr0: u64, dr1: u64, dr2: u64, dr3: u64, dr4: u64, dr5: u64, dr6: u64, dr7: u64,
// 		cr0: u64, cr1: u64, cr2: u64, cr3: u64, cr4: u64, cr8: u64,
// 		rflags: u64,
// 		ldtr: u64, tr: u64,
// 		gdtr: [u64; 2], idtr: [u64; 2],
// 		rip: u64,
// 		gs: u64, fs: u64, es: u64, ds: u64, cs: u64, ss: u64,
// 		rdi: u64, rsi: u64, rbp: u64, rsp: u64, rbx: u64, rdx: u64, rcx: u64, rax: u64,
// 		r8: u64, r9: u64, r10: u64, r11: u64, r12: u64, r13: u64, r14: u64, r15: u64,
// 	} = InstructionSetArchitecture::X64 as u32,
// 	/// IA64
// 	Ipf {} = InstructionSetArchitecture::Ipf as u32,
// 	/// EBC
// 	Ebc {
// 		r0: u64,
// 		r1: u64,
// 		r2: u64,
// 		r3: u64,
// 		r4: u64,
// 		r5: u64,
// 		r6: u64,
// 		r7: u64,
// 		flags: u64,
// 		controlflags: u64,
// 		ip: u64,
// 	} = InstructionSetArchitecture::Ebc as u32,
// 	/// ARM thumb mixed
// 	Arm {} = InstructionSetArchitecture::Arm as u32,
// 	AArch64 {} = InstructionSetArchitecture::AArch64 as u32,
// 	RiscV32 {
// 		// integer registers
// 		zero: u32, ra: u32, sp: u32, gp: u32, tp: u32, t0: u32, t1: u32, t2: u32,
// 		s0fp: u32, s1: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32, a7: u32,
// 		s2: u32, s3: u32, s4: u32, s5: u32, s6: u32, s7: u32, s8: u32, s9: u32, s10: u32, s11: u32,
// 		t3: u32, t4: u32, t5: u32, t6: u32,
//
// 		// Floating registers for F, D and Q Standard Extensions
// 		ft0: u128, ft1: u128, ft2: u128, ft3: u128, ft4: u128, ft5: u128, ft6: u128, ft7: u128,
// 		fs0: u128, fs1: u128, fa0: u128, fa1: u128, fa2: u128, fa3: u128, fa4: u128, fa5: u128, fa6: u128, fa7: u128,
// 		fs2: u128, fs3: u128, fs4: u128, fs5: u128, fs6: u128, fs7: u128, fs8: u128, fs9: u128, fs10: u128, fs11: u128,
// 		ft8: u128, ft9: u128, ft10: u128, ft11: u128,
// 	} = InstructionSetArchitecture::RiscV32 as u32,
// 	RiscV64 {
// 		// integer registers
// 		zero: u64, ra: u64, sp: u64, gp: u64, tp: u64, t0: u64, t1: u64, t2: u64,
// 		s0fp: u64, s1: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64, a7: u64,
// 		s2: u64, s3: u64, s4: u64, s5: u64, s6: u64, s7: u64, s8: u64, s9: u64, s10: u64, s11: u64,
// 		t3: u64, t4: u64, t5: u64, t6: u64,
//
// 		// Floating registers for F, D and Q Standard Extensions
// 		ft0: u128, ft1: u128, ft2: u128, ft3: u128, ft4: u128, ft5: u128, ft6: u128, ft7: u128,
// 		fs0: u128, fs1: u128, fa0: u128, fa1: u128, fa2: u128, fa3: u128, fa4: u128, fa5: u128, fa6: u128, fa7: u128,
// 		fs2: u128, fs3: u128, fs4: u128, fs5: u128, fs6: u128, fs7: u128, fs8: u128, fs9: u128, fs10: u128, fs11: u128,
// 		ft8: u128, ft9: u128, ft10: u128, ft11: u128,
// 	} = InstructionSetArchitecture::RiscV64 as u32,
// 	RiscV128 {
// 		// integer registers
// 		zero: u128, ra: u128, sp: u128, gp: u128, tp: u128, t0: u128, t1: u128, t2: u128,
// 		s0fp: u128, s1: u128, a0: u128, a1: u128, a2: u128, a3: u128, a4: u128, a5: u128, a6: u128, a7: u128,
// 		s2: u128, s3: u128, s4: u128, s5: u128, s6: u128, s7: u128, s8: u128, s9: u128, s10: u128, s11: u128,
// 		t3: u128, t4: u128, t5: u128, t6: u128,
//
// 		// Floating registers for F, D and Q Standard Extensions
// 		ft0: u128, ft1: u128, ft2: u128, ft3: u128, ft4: u128, ft5: u128, ft6: u128, ft7: u128,
// 		fs0: u128, fs1: u128, fa0: u128, fa1: u128, fa2: u128, fa3: u128, fa4: u128, fa5: u128, fa6: u128, fa7: u128,
// 		fs2: u128, fs3: u128, fs4: u128, fs5: u128, fs6: u128, fs7: u128, fs8: u128, fs9: u128, fs10: u128, fs11: u128,
// 		ft8: u128, ft9: u128, ft10: u128, ft11: u128,
// 	} = InstructionSetArchitecture::RiscV128 as u32,
// 	LoongArch32 {} = InstructionSetArchitecture::LoongArch32 as u32,
// 	LoongArch64 {} = InstructionSetArchitecture::LoongArch64 as u32,
// }

#[repr(C)]
pub enum InstructionSetArchitecture {
	/// I386
	Ia32 = 0x014C,
	/// x64
	X64 = 0x8664,
	/// IA64
	Ipf = 0x0200,
	/// EBC
	Ebc = 0x0EBC,
	/// ARM thumb mixed
	Arm = 0x01C2,
	AArch64 = 0xAA64,
	RiscV32 = 0x5032,
	RiscV64 = 0x5064,
	RiscV128 = 0x5128,
	LoongArch32 = 0x6232,
	LoongArch64 = 0x6264,
}

// #[repr(C)]
// pub union SystemContext {
// 	ebc: *const SystemContextEBC,
// 	ia32: *const SystemContextIa32,
// 	x64: *const SystemContextX64,
// 	ipf: *const Void,
// 	arm: *const Void,
// 	aarch64: *const Void,
// 	riscv32: *const SystemContextRiscV<u32>,
// 	riscv64: *const SystemContextRiscV<u64>,
// 	riscv128: *const SystemContextRiscV<u128>,
// 	loongarch64: *const Void,
// }
