//! Code that was not converted from C.

use crate::prelude::*;
use libc_alloc::LibcAlloc;
use std::ffi::CString;
use std::ptr::read_volatile;
use std::ptr::write_volatile;

/// cbindgen:no-export
#[cfg(feature = "ENABLE_GEN_ETH")]
pub type pcap_t = pcap;

/// cbindgen:no-export
#[cfg(feature = "ENABLE_GEN_ETH")]
#[repr(C)]
pub struct pcap {
    _todo: u8,
}

/// Make rust memory compatible with C malloc/free/...
#[global_allocator]
static GLOBAL_ALLOCATOR: LibcAlloc = LibcAlloc;

/// Macro that mimics `__func__`.
/// Returns the function name as a C string (*mut c_char).
/// The function must have `#[named]` or `function_name!`.
#[macro_export]
macro_rules! cfunc {
    () => {
        concat!(function_name!(), "\0").as_ptr().cast::<c_char>().cast_mut()
    };
}
pub use cfunc;

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

/// Non-standard unsigned char
pub type u_char = std::ffi::c_uchar;

/// Non-standard unsigned int
pub type u_int = std::ffi::c_uint;

/// Non-standard unsigned long
pub type u_long = std::ffi::c_ulong;

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
impl AsC<*const c_char, *const c_void> for CStr {
    fn as_c(&self) -> *const c_char {
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
impl AsC<*const c_char, *const c_void> for &str {
    fn as_c(&self) -> *const c_char {
        self.as_ptr().cast::<_>()
    }
    fn as_c_void(&self) -> *const c_void {
        self.as_c().cast::<_>()
    }
}
impl AsC<*const c_char, *const c_void> for String {
    fn as_c(&self) -> *const c_char {
        self.as_str().as_c()
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

/// Wrapper to allow formatting of std::ffi types by the sprintf crate.
pub struct Printf<T>(pub T);
macro_rules! impl_Printf {
    ($($T:ident),*) => {
        $(
            impl sprintf::Printf for Printf<$T> {
                fn format(&self, x: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
                    self.0.format(x)
                }
                fn as_int(&self) -> Option<i32> {
                    self.0.as_int()
                }
            }
        )*
    };
    ($(&$T:ident),*) => {
        $(
            impl sprintf::Printf for Printf<&$T> {
                fn format(&self, x: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
                    self.0.format(x)
                }
                fn as_int(&self) -> Option<i32> {
                    self.0.as_int()
                }
            }
        )*
    };
}
impl_Printf!(u64, i64, u32, i32, u16, i16, u8, i8, usize, isize, f64, f32, char, String, CString);
impl_Printf!(&str, &CStr);
impl sprintf::Printf for Printf<*const c_char> {
    fn format(&self, x: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
        Printf(self.0.cast_mut()).format(x)
    }
    fn as_int(&self) -> Option<i32> {
        Printf(self.0.cast_mut()).as_int()
    }
}
impl sprintf::Printf for Printf<*mut c_char> {
    fn format(&self, x: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
        if self.0.is_null() {
            Err(sprintf::PrintfError::WrongType) // null
        } else if let Ok(s) = unsafe { CStr::from_ptr(self.0).to_str() } {
            s.format(x)
        } else {
            Err(sprintf::PrintfError::WrongType) // not utf8
        }
    }
    fn as_int(&self) -> Option<i32> {
        None
    }
}
impl sprintf::Printf for Printf<&[c_char]> {
    fn format(&self, spec: &sprintf::ConversionSpecifier) -> Result<String, sprintf::PrintfError> {
        let p: *const c_char = self.0.as_c();
        Printf(p).format(spec)
    }
    fn as_int(&self) -> Option<i32> {
        None
    }
}

/// Wrapper around a volatile type.
/// cbindgen:no-export
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Volatile<T>(pub T);
impl<T> Volatile<T> {
    pub fn get(&self) -> T {
        unsafe { read_volatile(addr_of!(self.0)) }
    }
    pub fn set(&mut self, x: T) {
        unsafe { write_volatile(addr_of_mut!(self.0), x) }
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
