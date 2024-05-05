//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//!
//! Generic Hash Tables.

use crate::_private::*;
use crate::dynamips_common::*;

pub type hash_node_t = hash_node;
pub type hash_table_t = hash_table;

/// Key computation function
pub type hash_fcompute = Option<unsafe extern "C" fn(key: *mut c_void) -> u_int>;

/// Comparison function for 2 keys
pub type hash_fcompare = Option<unsafe extern "C" fn(key1: *mut c_void, key2: *mut c_void) -> c_int>;

/// User function to call when using hash_table_foreach
pub type hash_fforeach = Option<unsafe extern "C" fn(key: *mut c_void, value: *mut c_void, opt_arg: *mut c_void)>;

/// Hash element (pair key,value)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hash_node {
    pub key: *mut c_void,
    pub value: *mut c_void,
    pub next: *mut hash_node_t,
}

/// Hash Table definition
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
pub unsafe extern "C" fn hash_string_create(hash_size: c_int) -> *mut hash_table_t {
    hash_table_create(Some(str_hash), Some(str_equal), hash_size)
}

#[no_mangle]
pub unsafe extern "C" fn hash_int_create(hash_size: c_int) -> *mut hash_table_t {
    hash_table_create(Some(int_hash), Some(int_equal), hash_size)
}

#[no_mangle]
pub unsafe extern "C" fn hash_u64_create(hash_size: c_int) -> *mut hash_table_t {
    hash_table_create(Some(u64_hash), Some(u64_equal), hash_size)
}

#[no_mangle]
pub unsafe extern "C" fn hash_ptr_create(hash_size: c_int) -> *mut hash_table_t {
    hash_table_create(Some(ptr_hash), Some(ptr_equal), hash_size)
}

#[macro_export]
macro_rules! HASH_TABLE_FOREACH {
    ($i:ident, $ht:expr, $hn:ident, $($tt:tt)*) => {
        $i = 0;
        while $i < (*$ht).size {
            $hn = *(*$ht).nodes.add($i as usize);
            while !$hn.is_null() {
                $($tt)*;
                $hn = (*$hn).next;
            }
            $i += 1;
        }
    };
}
pub use HASH_TABLE_FOREACH;

/// Compare two strings
#[no_mangle]
pub unsafe extern "C" fn str_equal(s1: *mut c_void, s2: *mut c_void) -> c_int {
    (libc::strcmp(s1.cast::<_>(), s2.cast::<_>()) == 0) as c_int
}

/// Hash function for a string
#[no_mangle]
pub unsafe extern "C" fn str_hash(str_: *mut c_void) -> u_int {
    let s: *mut c_char = str_.cast::<_>();

    let mut h: u_int = 0;
    let mut p = s;
    while *p != b'\0' as c_char {
        h = (h << 4) + *p as c_int as u_int; // i8->i32->u32
        let g: u_int = h & 0xf0000000;
        if g != 0 {
            h ^= g >> 24;
            h ^= g;
        }
        p = p.add(1);
    }

    h
}

/// Compare two integers (yes, it's stupid)
#[no_mangle]
pub unsafe extern "C" fn int_equal(i1: *mut c_void, i2: *mut c_void) -> c_int {
    (i1 as c_long as c_int == i2 as c_long as c_int) as c_int
}

/// Hash function for an integer (see above)
#[no_mangle]
pub unsafe extern "C" fn int_hash(i: *mut c_void) -> u_int {
    let val: u_int = i as c_long as u_int;
    val ^ (val >> 16)
}

/// Compare two u64 (yes, it's stupid)
#[no_mangle]
pub unsafe extern "C" fn u64_equal(i1: *mut c_void, i2: *mut c_void) -> c_int {
    (*i1.cast::<m_uint64_t>() == *i2.cast::<m_uint64_t>()) as c_int
}

/// Hash function for an u64 (see above)
#[no_mangle]
pub unsafe extern "C" fn u64_hash(i: *mut c_void) -> u_int {
    let val: m_uint64_t = *i.cast::<m_uint64_t>();
    (val ^ (val >> 32)) as u_int
}

/// Compare 2 pointers
#[no_mangle]
pub unsafe extern "C" fn ptr_equal(i1: *mut c_void, i2: *mut c_void) -> c_int {
    (i1 == i2) as c_int
}

/// Hash function for a pointer (see above)
#[no_mangle]
pub unsafe extern "C" fn ptr_hash(i: *mut c_void) -> u_int {
    let val: m_uint64_t = i as m_iptr_t as m_uint64_t;
    ((val & 0xFFFF) ^ ((val >> 24) & 0xFFFF) ^ ((val >> 48) & 0xFFFF)) as u_int
}

/// Free memory used by a node
#[inline]
unsafe fn hash_node_free(node: *mut hash_node_t) {
    libc::free(node.cast::<_>());
}

/// Allocate memory for a new node
unsafe fn hash_node_alloc(_ht: *mut hash_table_t, key: *mut c_void, value: *mut c_void) -> *mut hash_node_t {
    let node: *mut hash_node_t = libc::malloc(size_of::<hash_node_t>()).cast::<_>();
    assert!(!node.is_null());
    (*node).key = key;
    (*node).value = value;
    (*node).next = null_mut();
    node
}

/// Create a new hash table
#[no_mangle]
pub unsafe extern "C" fn hash_table_create(hash_func: hash_fcompute, key_cmp: hash_fcompare, hash_size: c_int) -> *mut hash_table_t {
    if hash_func.is_none() || hash_size <= 0 {
        return null_mut();
    }

    let ht: *mut hash_table_t = libc::malloc(size_of::<hash_table_t>()).cast::<_>();
    assert!(!ht.is_null());

    libc::memset(ht.cast::<_>(), 0, size_of::<hash_table_t>());
    (*ht).hash_func = hash_func;
    (*ht).key_cmp = key_cmp;
    (*ht).size = hash_size;
    (*ht).nodes = libc::calloc((*ht).size as usize, size_of::<*mut hash_node_t>()).cast::<_>();
    assert!(!(*ht).nodes.is_null());
    ht
}

/// Delete an existing Hash Table
#[no_mangle]
pub unsafe extern "C" fn hash_table_delete(ht: *mut hash_table_t) {
    if ht.is_null() {
        return;
    }

    for hash_val in 0..(*ht).size as u_int {
        let mut node: *mut hash_node_t = *(*ht).nodes.add(hash_val as usize);
        while !node.is_null() {
            let node_next: *mut hash_node_t = (*node).next;
            hash_node_free(node);
            node = node_next;
        }
        *(*ht).nodes.add(hash_val as usize) = null_mut();
    }
    libc::free((*ht).nodes.cast::<_>());
    libc::free(ht.cast::<_>());
}

/// Insert a new (key,value). If key already exists in table, replace value
#[no_mangle]
pub unsafe extern "C" fn hash_table_insert(ht: *mut hash_table_t, key: *mut c_void, value: *mut c_void) -> c_int {
    assert!(!ht.is_null());

    let hash_val: usize = (*ht).hash_func.unwrap()(key) as usize % (*ht).size as usize;

    let mut node: *mut hash_node_t = *(*ht).nodes.add(hash_val);
    while !node.is_null() {
        if (*ht).key_cmp.unwrap()((*node).key, key) != 0 {
            (*node).value = value;
            return 0;
        }
        node = (*node).next;
    }

    node = hash_node_alloc(ht, key, value);
    (*node).next = *(*ht).nodes.add(hash_val);
    *(*ht).nodes.add(hash_val) = node;
    (*ht).nnodes += 1;
    0
}

/// Remove a pair (key,value) from an hash table
#[no_mangle]
pub unsafe extern "C" fn hash_table_remove(ht: *mut hash_table_t, key: *mut c_void) -> *mut c_void {
    assert!(!ht.is_null());

    let hash_val: u_int = (*ht).hash_func.unwrap()(key) % (*ht).size as u_int;

    let mut node: *mut *mut hash_node_t = (*ht).nodes.add(hash_val as usize);
    while !(*node).is_null() {
        if (*ht).key_cmp.unwrap()((*(*node)).key, key) != 0 {
            let tmp: *mut hash_node_t = *node;
            let value: *mut c_void = (*tmp).value;
            *node = (*tmp).next;

            hash_node_free(tmp);
            return value;
        }
        node = addr_of_mut!((*(*node)).next);
    }

    null_mut()
}

/// Hash Table Lookup
#[no_mangle]
pub unsafe extern "C" fn hash_table_lookup(ht: *mut hash_table_t, key: *mut c_void) -> *mut c_void {
    assert!(!ht.is_null());

    let hash_val: u_int = (*ht).hash_func.unwrap()(key) % (*ht).size as u_int;

    let mut node: *mut hash_node_t = *(*ht).nodes.add(hash_val as usize);
    while !node.is_null() {
        if (*ht).key_cmp.unwrap()((*node).key, key) != 0 {
            return (*node).value;
        }
        node = (*node).next;
    }

    null_mut()
}

/// Hash Table Lookup - key direct comparison
#[no_mangle]
pub unsafe extern "C" fn hash_table_lookup_dcmp(ht: *mut hash_table_t, key: *mut c_void) -> *mut c_void {
    assert!(!ht.is_null());

    let hash_val: u_int = (*ht).hash_func.unwrap()(key) % (*ht).size as u_int;

    let mut node: *mut hash_node_t = *(*ht).nodes.add(hash_val as usize);
    while !node.is_null() {
        if (*node).key == key {
            return (*node).value;
        }
        node = (*node).next;
    }

    null_mut()
}

/// Call the specified function for each node found in hash table
#[no_mangle]
pub unsafe extern "C" fn hash_table_foreach(ht: *mut hash_table_t, user_fn: hash_fforeach, opt_arg: *mut c_void) -> c_int {
    assert!(!ht.is_null());

    for i in 0..(*ht).size as u_int {
        let mut node: *mut hash_node_t = *(*ht).nodes.add(i as usize);
        while !node.is_null() {
            user_fn.unwrap()((*node).key, (*node).value, opt_arg);
            node = (*node).next;
        }
    }

    0
}
