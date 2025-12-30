use super::Char16;

#[repr(transparent)]
#[derive(Hash)]
pub struct CStr16([Char16]);
impl CStr16 {
	/// # Safety
	/// You need to make sure that there is a null byte at the end of this otherwise this is
	/// completely unsafe
	pub const unsafe fn from_ptr<'ptr>(ptr: *const Char16) -> &'ptr Self {
		let ptr = ptr.cast::<u16>();
		let mut len = 0;
		// SAFETY:
		// todo
		unsafe {
			while *(ptr.add(len)) != 0 {
				len += 1;
			}
			Self::from_u16_with_nul_unchecked(core::slice::from_raw_parts(ptr, len + 1))
		}
	}

	/// # Safety
	/// You need to make sure that there is a null byte at the end of this otherwise this is
	/// completely unsafe
	pub const unsafe fn from_u16_with_nul_unchecked(chars: &[u16]) -> &Self {
		// SAFETY:
		// todo
		unsafe { &*(chars as *const [u16] as *const Self) }
	}

	pub const fn as_ptr(&self) -> *const Char16 {
		self.0.as_ptr()
	}

	pub const fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0[0] == '\0'
	}

	pub const fn as_bytes(&self) -> &[u8] {
		// SAFETY:
		// todo
		unsafe { core::slice::from_raw_parts(self.as_ptr().cast(), self.len() * 2) }
	}
}
