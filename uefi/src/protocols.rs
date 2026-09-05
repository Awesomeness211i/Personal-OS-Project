use crate::GUID;

pub mod acpi;
pub mod file;
pub mod graphics;
pub mod image;
pub mod path;
pub mod string;
pub mod text;

pub trait HasGUID {
	const GUID: GUID;
}

pub unsafe trait Protocol: HasGUID {}
