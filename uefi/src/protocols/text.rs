use crate::{
	Bool,
	Event,
	Char16,
	status::Status,
};

#[repr(C)]
pub struct SimpleTextOutputProtocol {
	pub reset: unsafe extern "efiapi" fn(this: *const Self, extended: bool) -> Status,
	pub output_string: unsafe extern "efiapi" fn(this: *const Self, string: *const Char16) -> Status,
	pub test_string: unsafe extern "efiapi" fn(this: *const Self, string: *const Char16) -> Status,
	pub query_mode: unsafe extern "efiapi" fn(this: *const Self, mode: usize, columns: *mut usize, rows: *mut usize) -> Status,
	pub set_mode: unsafe extern "efiapi" fn(this: *const Self, mode: usize) -> Status,
	pub set_attribute: unsafe extern "efiapi" fn(this: *const Self, attribute: usize) -> Status,
	pub clear_screen: unsafe extern "efiapi" fn(this: *const Self) -> Status,
	pub set_cursor_position: unsafe extern "efiapi" fn(this: *const Self, column: usize, row: usize) -> Status,
	pub enable_cursor: unsafe extern "efiapi" fn(this: *const Self) -> Status,
	pub mode: *const SimpleTextOutputMode,
}

pub enum Color {
	Black = 0x00,
	Blue = 0x01,
	Green = 0x02,
	Cyan = 0x03,
	Red = 0x04,
	Magenta = 0x05,
	Brown = 0x06,
	LightGray = 0x07,
	DarkGray = 0x08,
	LightBlue = 0x09,
	LightGreen = 0x0A,
	LightCyan = 0x0B,
	LightRed = 0x0C,
	LightMagenta = 0x0D,
	Yellow = 0x0E,
	White = 0x0F,
}

pub enum BackgroundColor {
	Black = Color::Black as isize,
	Blue = Color::Blue as isize,
	Green = Color::Green as isize,
	Cyan = Color::Cyan as isize,
	Red = Color::Red as isize,
	Magenta = Color::Magenta as isize,
	Brown = Color::Brown as isize,
	LightGray = Color::LightGray as isize,
}

impl SimpleTextOutputProtocol {
	/// Stand in for the macro EFI_TEXT_ATTR(Foreground,Background)
	pub const fn textattribute(foreground: Color, background: BackgroundColor) -> usize { (foreground as usize) | ((background as usize) << 4) }
}

#[repr(C)]
pub struct SimpleTextOutputMode {
	pub max_mode: i32,
	pub mode: i32,
	pub attribute: i32,
	pub cursorcolumn: i32,
	pub cursor_row: i32,
	pub cursor_visible: Bool,
}

#[repr(C)]
pub struct SimpleTextInputExProtocol {
	// this: IN, extendedverification: IN
	pub reset_ex: unsafe extern "efiapi" fn(this: *const Self, extendedVerification: bool) -> Status,
	// this: IN, key: OUT
	pub read_keystroke_ex: unsafe extern "efiapi" fn(this: *const Self, key: *mut InputKey) -> Status,
	// waitforevent
	pub wait_for_key_ex: Event,
	// this: IN, keytogglestate: IN
	pub set_state: unsafe extern "efiapi" fn(this: *mut Self, keytogglestate: *const KeyToggleState),
	// this: IN, key: IN, notificationfunction: IN, notifyhandle: OUT
	pub register_key_notify: unsafe extern "efiapi" fn(this: *const Self, key: *const KeyData, notificationfunction: unsafe extern "efiapi" fn(data: *const KeyData), notifyhandle: *mut *const ()),
	// this: IN, notificationhandle: IN
	pub unregister_key_notify: unsafe extern "efiapi" fn(this: *const Self, notificationhandle: *const ()),
}

#[repr(C)]
pub struct SimpleTextInputProtocol {
	// this: IN, extendedverification: IN
	pub reset: unsafe extern "efiapi" fn(this: *const Self, extendedVerification: bool) -> Status,
	// this: IN, extendedverification: OUT
	pub read_keystroke: unsafe extern "efiapi" fn(this: *const Self, key: *mut InputKey) -> Status,
	// waitforevent
	pub wait_for_key: Event,
}

#[repr(C)]
#[derive(Default)]
pub struct InputKey {
	pub scancode: u16,
	pub unicodechar: Char16,
}

#[repr(transparent)]
pub struct KeyToggleState(u8);

impl KeyToggleState {
	pub const TOGGLE_STATE_VALID: KeyToggleState = KeyToggleState(0x80);
	pub const KEY_STATE_EXPOSED: KeyToggleState = KeyToggleState(0x40);
	pub const CAPS_LOCK_ACTIVE: KeyToggleState = KeyToggleState(0x04);
	pub const NUM_LOCK_ACTIVE: KeyToggleState = KeyToggleState(0x02);
	pub const SCROLL_LOCK_ACTIVE: KeyToggleState = KeyToggleState(0x01);
}

#[repr(transparent)]
pub struct KeyShiftState(u32);

impl KeyShiftState {
	pub const SHIFT_STATE_VALID: KeyShiftState = KeyShiftState(0x80000000);
	pub const SYS_REQ_PRESSED: KeyShiftState = KeyShiftState(0x00000200);
	pub const MENU_KEY_PRESSED: KeyShiftState = KeyShiftState(0x00000100);
	pub const LEFT_LOGO_PRESSED: KeyShiftState = KeyShiftState(0x00000080);
	pub const RIGHT_LOGO_PRESSED: KeyShiftState = KeyShiftState(0x00000040);
	pub const LEFT_ALT_PRESSED: KeyShiftState = KeyShiftState(0x00000020);
	pub const RIGHT_ALT_PRESSED: KeyShiftState = KeyShiftState(0x00000010);
	pub const LEFT_CONTROL_PRESSED: KeyShiftState = KeyShiftState(0x00000008);
	pub const RIGHT_CONTROL_PRESSED: KeyShiftState = KeyShiftState(0x00000004);
	pub const LEFT_SHIFT_PRESSED: KeyShiftState = KeyShiftState(0x00000002);
	pub const RIGHT_SHIFT_PRESSED: KeyShiftState = KeyShiftState(0x00000001);
}

#[repr(C)]
pub struct KeyState {
	pub keyshiftstate: KeyShiftState,
	pub togglestate: KeyToggleState,
}

#[repr(C)]
pub struct KeyData {
	pub key: InputKey,
	pub keystate: KeyState,
}
