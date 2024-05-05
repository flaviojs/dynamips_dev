//! Generic Hash Tables.

use crate::prelude::*;

/// Key computation function
pub type hash_fcompute = Option<unsafe extern "C" fn(key: *mut c_void) -> c_uint>;

/// Comparison function for 2 keys
pub type hash_fcompare = Option<unsafe extern "C" fn(key1: *mut c_void, key2: *mut c_void) -> c_int>;

/// User function to call when using hash_table_foreach
pub type hash_fforeach = Option<unsafe extern "C" fn(key: *mut c_void, value: *mut c_void, opt_arg: *mut c_void)>;

/// Hash element (pair key,value)
pub type hash_node_t = hash_node;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hash_node {
    pub key: *mut c_void,
    pub value: *mut c_void,
    pub next: *mut hash_node_t,
}

/// Hash Table definition
pub type hash_table_t = hash_table;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hash_table {
    pub size: c_int,
    pub nnodes: c_int,
    pub nodes: *mut *mut hash_node_t,
    pub hash_func: hash_fcompute,
    pub key_cmp: hash_fcompare,
}

#[no_mangle]
pub extern "C" fn _export(_: hash_fcompute, _: hash_fcompare, _: hash_fforeach, _: *mut hash_node_t, _: *mut hash_table_t) {}
