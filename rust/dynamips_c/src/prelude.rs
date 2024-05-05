//! Internal shared code to interact with C.
//!
//! cbindgen will ignore the contents of this module.

pub use crate::_ext::cstr;
pub use crate::_ext::str0;
pub use crate::_ext::AsC;
pub use crate::_ext::AsCMut;
pub use crate::_ext::CArray;
pub use libc;
pub use libc::size_t;
pub use libc::ssize_t;
pub use std::ffi::c_char;
pub use std::ffi::c_int;
pub use std::ffi::c_uint;
pub use std::ffi::c_ulonglong;
pub use std::ffi::c_void;
pub use std::marker::PhantomData;
pub use std::mem::size_of;
pub use std::mem::zeroed;
pub use std::ptr::addr_of;
pub use std::ptr::addr_of_mut;
pub use std::ptr::null_mut;

extern "C" {
    // _ext.c
    pub fn c_stderr() -> *mut libc::FILE;
    // libc
    pub fn gethostbyname(name: *const c_char) -> *mut libc::hostent;
    pub fn htons(x: u16) -> u16;
    pub fn inet_addr(cp: *const libc::c_char) -> libc::in_addr_t;
}
