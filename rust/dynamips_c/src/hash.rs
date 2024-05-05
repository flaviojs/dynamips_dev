//! Generic Hash Tables.

use crate::prelude::*;

/// Key computation function
pub type hash_fcompute = Option<unsafe extern "C" fn(key: *mut c_void) -> c_uint>;

/// Comparison function for 2 keys
pub type hash_fcompare = Option<unsafe extern "C" fn(key1: *mut c_void, key2: *mut c_void) -> c_int>;

/// User function to call when using hash_table_foreach
pub type hash_fforeach = Option<unsafe extern "C" fn(key: *mut c_void, value: *mut c_void, opt_arg: *mut c_void)>;

#[no_mangle]
pub extern "C" fn _export(_: hash_fcompute, _: hash_fcompare, _: hash_fforeach) {}
