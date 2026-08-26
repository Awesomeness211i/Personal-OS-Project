#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
// #![warn(missing_docs)]
//! # BOOTLOADER
//! Library for interfacing with bootloader specific data structures for my hobby OS project.

// TODO: Figure out how I want to deal with defining external symbols

// #[unsafe(export_name = "efi_main")]
// pub extern "efiapi" fn efi_main(image_handle: &mut core::ffi::c_void, system_table: SystemTablePointer) -> uefi::status::Status {
// 	__user_uefi_main(image_handle, system_table)
// }
//
// unsafe extern "C" {
// 	safe fn __user_uefi_main(image_handle: &mut core::ffi::c_void, system_table: SystemTablePointer) -> uefi::status::Status;
// }

pub unsafe trait SizeField<StructureType: ?Sized> {
	fn size(&self) -> usize;
}

pub trait RuntimeSizeStructure<HeaderType: Sized + SizeField<Self>> {}
impl<T: ?Sized, H: SizeField<T>> RuntimeSizeStructure<H> for T {}

// trait StateType<F: ?Sized> {}
// impl<T, F: StateFamily<T>> StateType<F> for T {}
//
// unsafe trait StateFamily<T: StateType<Self>> {
// 	type Data;
// 	type Next: StateType<Self>;
// 	type NextData;
//
// 	fn transition(data: Self::Data) -> Self::NextData;
// }
//
// struct Machine<S: StateFamily<T>, T> {
// 	data: S::Data,
// 	_phantom: PhantomData<T>,
// }
//
// impl<S: StateFamily<T>, T: StateType<S>> Machine<S, T> {
// 	fn advance(self) -> Machine<S::NextData, S::Next>
// 	where
// 		S::NextData: StateFamily<<S as StateFamily<T>>::Next>,
// 	{
// 		Machine {
// 			data: S::transition(self.data),
// 			_phantom: PhantomData,
// 		}
// 	}
// }

// trait Monad {
// 	type M<A>: Monad;
//
// 	fn pure<A>(val: A) -> Self::M<A>;
//
// 	fn bind<A, B, F: FnOnce(A) -> Self::M<B>>(ma: Self::M<A>, f: F) -> Self::M<B>;
//
// 	fn map<A, B, F: FnOnce(A) -> B>(ma: Self::M<A>, f: F) -> Self::M<B> {
// 		Self::bind(ma, |a| { Self::pure(f(a)) })
// 	}
// }
//
// impl<T> Monad for Option<T> {
// 	type M<B> = Option<B>;
//
// 	fn pure<A>(val: A) -> Self::M<A> {
// 		Some(val)
// 	}
//
// 	fn bind<A, B, F: FnOnce(A) -> Self::M<B>>(ma: Self::M<A>, f: F) -> Self::M<B> {
// 		ma.and_then(f)
// 	}
// }
//
// impl<T, E> Monad for Result<T, E> {
// 	type M<A> = Result<A, E>;
//
// 	fn pure<A>(val: A) -> Self::M<A> {
// 		Ok(val)
// 	}
//
// 	fn bind<A, B, F: FnOnce(A) -> Self::M<B>>(ma: Self::M<A>, f: F) -> Self::M<B> {
// 		ma.and_then(f)
// 	}
// }
