use core::fmt::{
	Arguments,
	Error,
	Write,
};

// TODO: Actually make this something that works correctly
#[repr(transparent)]
pub struct Port(u16);

impl Write for Port {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		let str = s.as_bytes();
		let len = str.len();
		// # Safety:
		let result = unsafe { self.out_bytes(str) };
		if result == len { Ok(()) } else { Err(Error) }
	}
}

impl Port {
	pub const COM1: Self = Self(0x3F8);
	pub const COM2: Self = Self(0x2F8);
	pub const COM3: Self = Self(0x3E8);
	pub const COM4: Self = Self(0x2E8);
	pub const COM5: Self = Self(0x5F8);
	pub const COM6: Self = Self(0x4F8);
	pub const COM7: Self = Self(0x5E8);
	pub const COM8: Self = Self(0x4E8);

	const RX_TX_BUFFER: u16 = 0;
	const INT_ENABLE: u16 = 1;
	const FIFO_CTRL: u16 = 2;
	const LINE_CTRL: u16 = 3;
	const MODEM_CTRL: u16 = 4;
	const LINE_STATUS: u16 = 5;
	const MODEM_STATUS: u16 = 6;
	const SCRATCH: u16 = 7;

	pub unsafe fn init(&self) {
		// # Safety:
		unsafe {
			Self(self.0 + Self::INT_ENABLE).out(0x00);
			Self(self.0 + Self::LINE_CTRL).out(0x80);
			Self(self.0 + Self::RX_TX_BUFFER).out(0x03);

			Self(self.0 + Self::INT_ENABLE).out(0x00);
			Self(self.0 + Self::LINE_CTRL).out(0x03);
			Self(self.0 + Self::FIFO_CTRL).out(0x07);
			Self(self.0 + Self::MODEM_CTRL).out(0x1F);

			Self(self.0 + Self::RX_TX_BUFFER).out(0xAA);

			if Self::inb(self) != 0xAA {
				println(format_args!("Error: port not setup correctly"));
			}

			Self(self.0 + Self::MODEM_CTRL).out(0x0F);
		}
	}

	pub fn get(&self) -> u16 {
		self.0
	}

	#[inline(always)]
	pub unsafe fn out(&self, value: u8) {
		// # Safety:
		unsafe {
			core::arch::asm!(
				"out dx, al",
				in("dx") self.0,
				in("al") value,
			)
		}
	}

	#[inline(always)]
	pub unsafe fn inb(&self) -> u8 {
		let value: u8;
		// # Safety:
		unsafe {
			core::arch::asm!(
				"in al, dx",
				in("dx") self.0,
				out("al") value,
			)
		}
		value
	}

	#[inline(always)]
	pub unsafe fn out_bytes(&self, bytes: &[u8]) -> usize {
		let unwritten_bytes: usize;
		// # Safety:
		unsafe {
			core::arch::asm!(
				"rep outsb",
				inout("rcx") bytes.len() => unwritten_bytes,
				in("dx") self.0,
				inout("rsi") bytes.as_ptr() => _,
			);
		}
		bytes.len() - unwritten_bytes
	}
}

pub fn println(args: Arguments<'_>) {
	let mut port = Port::COM1;
	let _ = writeln!(port, "{args}");
}

pub fn print(args: Arguments<'_>) {
	let mut port = Port::COM1;
	let _ = write!(port, "{args}");
}
