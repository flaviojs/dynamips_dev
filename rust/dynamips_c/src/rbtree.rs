//! Red/Black Trees.

use crate::prelude::*;

/// Comparison function for 2 keys
pub type tree_fcompare = Option<unsafe extern "C" fn(key1: *mut c_void, key2: *mut c_void, opt: *mut c_void) -> c_int>;

/// User function to call when using rbtree_foreach
pub type tree_fforeach = Option<unsafe extern "C" fn(key: *mut c_void, value: *mut c_void, opt: *mut c_void)>;

// Node colors // TODO enum
pub const RBTREE_RED: c_short = 0;
pub const RBTREE_BLACK: c_short = 1;

#[no_mangle]
pub extern "C" fn _export(_: tree_fcompare, _: tree_fforeach) {}
