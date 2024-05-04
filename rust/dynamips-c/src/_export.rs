//! Rust code that is available in C.
//!
//! cbindgen will parse this module.

use crate::_private::*;
use std::ptr::read_volatile;
use std::ptr::write_volatile;

/// Make sure cbindgen exports the types it needs.
#[rustfmt::skip]
#[no_mangle]
pub extern "C" fn _export(
) {}

// Non-standard unsigned integers
pub type u_char = c_uchar;
pub type u_int = c_uint;
pub type u_long = c_ulong;
pub type u_short = c_ushort;

/// Wrapper around a volatile type.
/// cbindgen:no-export
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Volatile<T>(pub T);
impl<T> Volatile<T> {
    pub fn get(&self) -> T {
        // SAFETY the pointer is valid if self is valid
        unsafe { read_volatile(addr_of!(self.0)) }
    }
    pub fn set(&mut self, x: T) {
        // SAFETY the pointer is valid if self is valid
        unsafe { write_volatile(addr_of_mut!(self.0), x) }
    }
}
