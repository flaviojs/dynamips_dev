//! Generic Hash Tables.

use crate::prelude::*;

/// Key computation function
pub type hash_fcompute = Option<unsafe extern "C" fn(key: *mut c_void) -> c_uint>;

#[no_mangle]
pub extern "C" fn _export(_: hash_fcompute) {}
