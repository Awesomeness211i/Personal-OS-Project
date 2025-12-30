#[repr(transparent)]
#[derive(Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Char16(u16);
impl TryFrom<Char16> for char {
	type Error = core::char::CharTryFromError;
	fn try_from(char: Char16) -> Result<Self, Self::Error> {
		u32::from(char.0).try_into()
	}
}

impl From<u16> for Char16 {
	fn from(value: u16) -> Self {
		Self(value)
	}
}

impl From<Char16> for u16 {
	fn from(char: Char16) -> Self {
		char.0
	}
}

impl PartialEq<char> for Char16 {
	fn eq(&self, other: &char) -> bool {
		u32::from(self.0) == u32::from(*other)
	}
}
