//! Code that was not converted from C.

use crate::prelude::*;
use libc_alloc::LibcAlloc;
use std::ffi::CString;

/// Make rust memory compatible with C malloc/free/...
#[global_allocator]
static GLOBAL_ALLOCATOR: LibcAlloc = LibcAlloc;

/// Macro that concatenates expressions and a nul terminator.
#[macro_export]
macro_rules! str0 {
    () => {
        "\0"
    };
    ($($e:expr),*) => {
        concat!($($e),*, "\0")
    };
}
pub use str0;

/// Macro that converts a static string (&'static str) to a C string (*mut c_char).
#[macro_export]
macro_rules! cstr {
    ($($e:expr),*) => {
        str0!($($e),*).as_ptr().cast::<c_char>().cast_mut()
    };
}
pub use cstr;

/// Trait that converts a rust type to a C representation.
pub trait AsC<T, V> {
    fn as_c(&self) -> T;
    fn as_c_void(&self) -> V;
}
impl<T, const N: usize> AsC<*const T, *const c_void> for [T; N] {
    fn as_c(&self) -> *const T {
        self.as_ptr()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}
impl<T> AsC<*const T, *const c_void> for &[T] {
    fn as_c(&self) -> *const T {
        self.as_ptr()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}
impl AsC<*const c_char, *const c_void> for CString {
    fn as_c(&self) -> *const c_char {
        self.as_c_str().as_ptr()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}

/// Trait that converts a mutable rust type to a C representation.
pub trait AsCMut<T, V> {
    fn as_c_mut(&mut self) -> T;
    fn as_c_void_mut(&mut self) -> V;
}
impl<T, const N: usize> AsCMut<*mut T, *mut c_void> for [T; N] {
    fn as_c_mut(&mut self) -> *mut T {
        self.as_ptr().cast_mut()
    }
    fn as_c_void_mut(&mut self) -> *mut c_void {
        self.as_c_mut().cast::<_>()
    }
}

/// Wrapper around a C array of unknown size.
#[derive(Debug)]
#[repr(transparent)]
pub struct CArray<T>(pub *mut T);
impl<T> std::ops::Index<c_int> for CArray<T> {
    type Output = T;
    fn index(&self, index: c_int) -> &T {
        unsafe { &*self.0.offset(index.try_into().expect("c_int->isize")) }
    }
}
impl<T> std::ops::IndexMut<c_int> for CArray<T> {
    fn index_mut(&mut self, index: c_int) -> &mut T {
        unsafe { &mut *self.0.offset(index.try_into().expect("c_int->isize")) }
    }
}
impl<T> std::convert::From<*mut T> for CArray<T> {
    fn from(x: *mut T) -> Self {
        Self(x)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_str0_cstr() {
        unsafe {
            let want = "test\0";
            let have = str0!("test");
            assert_eq!(want, have);
            let want = want.as_ptr().cast::<c_char>().cast_mut();
            let have = cstr!("test");
            assert_eq!(libc::strcmp(want, have), 0);
        }
    }
}
