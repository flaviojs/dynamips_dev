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

/// Compare two strings
#[no_mangle]
pub unsafe extern "C" fn str_equal(s1: *mut c_void, s2: *mut c_void) -> c_int {
    (libc::strcmp(s1.cast::<_>(), s2.cast::<_>()) == 0).into()
}

/// Hash function for a string
#[no_mangle]
pub unsafe extern "C" fn str_hash(str_: *mut c_void) -> c_uint {
    let s: *mut c_char = str_.cast::<_>();

    let mut h: c_uint = 0;
    let mut p = s;
    while *p != b'\0' as c_char {
        h = (h << 4) + *p as c_int as c_uint; // i8->i32->u32
        let g: c_uint = h & 0xf0000000;
        if g != 0 {
            h ^= g >> 24;
            h ^= g;
        }
        p = p.add(1);
    }

    h
}

#[no_mangle]
pub extern "C" fn _export(_: hash_fcompute, _: hash_fcompare, _: hash_fforeach, _: *mut hash_node_t, _: *mut hash_table_t) {}
