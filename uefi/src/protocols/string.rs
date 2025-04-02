use crate::{
	GUID,
	Bool,
	Char8,
	Char16,
	status::Status,
};

#[repr(C)]
pub struct UnicodeCollationProtocol {
	/// this: IN, s1: IN, s2: IN
	/// case insensitive comparison where if s1 == s2 then 0 and if s1 lexically < s2 then negative and
	/// if s1 lexically > s2 then positive
	string_collation: unsafe extern "efiapi" fn(this: *const Self, s1: *const Char16, s2: *const Char16) -> isize,
	/// this: IN, string: IN, pattern: IN
	/// case insensitive comparison where if the pattern match succeeds it returns TRUE else FALSE
	meta_insensitive_match: unsafe extern "efiapi" fn(this: *const Self, string: *const Char16, pattern: *const Char16) -> Bool,
	/// this: IN, string: IN OUT
	/// walks through all characters in string and converts each one to lowercase equivalent if it exists
	string_to_lowercase: unsafe extern "efiapi" fn(this: *const Self, string: *mut Char16),
	/// this: IN, string: IN OUT
	/// walks through all characters in string and converts each one to uppercase equivalent if it exists
	string_to_uppercase: unsafe extern "efiapi" fn(this: *const Self, string: *mut Char16),
	/// this: IN, fat_size: IN, fat: IN, string: OUT
	fat_to_string: unsafe extern "efiapi" fn(this: *const Self, fat_size: usize, fat: *const Char8, string: *mut Char16),
	/// this: IN, string: IN, fat_size: IN, fat: OUT
	/// characters that map to illegal fat characters or have no valid mapping are replaced with '_' if any
	/// character conversions are substituted then this returns TRUE else FALSE
	string_to_fat: unsafe extern "efiapi" fn(this: *const Self, string: *const Char16, fat_size: usize, fat: *mut Char8) -> Bool,
	supported_languages: *const Char8,
}
impl UnicodeCollationProtocol {
}
impl super::Protocol for UnicodeCollationProtocol {
	/// GUID: A4C751FC-23AE-4C3E-92E9-4964CF63F349
	const GUID: GUID = GUID::new(0xA4C751FC, 0x23AE, 0x4C3E, 0x92E9_4964CF63F349);
}

#[repr(C)]
pub struct RegularExpressionProtocol {
	match_string: unsafe extern "efiapi" fn(this: *const Self) -> Status,
	get_info: unsafe extern "efiapi" fn(this: *const Self) -> Status,
}
impl RegularExpressionProtocol {
}
impl super::Protocol for RegularExpressionProtocol {
	/// GUID: B3F79D9A-436C-DC11-B052-CD85DF524CE6
	const GUID: GUID = GUID::new(0xB3F79D9A, 0x436C, 0xDC11, 0xB052_CD85DF524CE6);
}
