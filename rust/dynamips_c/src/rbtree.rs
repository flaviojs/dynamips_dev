//! Red/Black Trees.

use crate::prelude::*;

/// Comparison function for 2 keys
pub type tree_fcompare = Option<unsafe extern "C" fn(key1: *mut c_void, key2: *mut c_void, opt: *mut c_void) -> c_int>;

#[no_mangle]
pub extern "C" fn _export(_: tree_fcompare) {}
