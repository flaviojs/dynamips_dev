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

/// Compare two integers (yes, it's stupid)
#[no_mangle]
pub unsafe extern "C" fn int_equal(i1: *mut c_void, i2: *mut c_void) -> c_int {
    (i1 as c_long as c_int == i2 as c_long as c_int).into()
}

/// Hash function for an integer (see above)
#[no_mangle]
pub unsafe extern "C" fn int_hash(i: *mut c_void) -> c_uint {
    let val: c_uint = i as c_long as c_uint;
    val ^ (val >> 16)
}

/// Compare two u64 (yes, it's stupid)
#[no_mangle]
pub unsafe extern "C" fn u64_equal(i1: *mut c_void, i2: *mut c_void) -> c_int {
    (*i1.cast::<u64>() == *i2.cast::<u64>()).into()
}

/// Hash function for an u64 (see above)
#[no_mangle]
pub unsafe extern "C" fn u64_hash(i: *mut c_void) -> c_uint {
    let val: u64 = *i.cast::<u64>();
    (val ^ (val >> 32)) as c_uint
}

/// Compare 2 pointers
#[no_mangle]
pub unsafe extern "C" fn ptr_equal(i1: *mut c_void, i2: *mut c_void) -> c_int {
    (i1 == i2).into()
}

/// Hash function for a pointer (see above)
#[no_mangle]
pub unsafe extern "C" fn ptr_hash(i: *mut c_void) -> c_uint {
    let val: u64 = i as usize as u64;
    ((val & 0xFFFF) ^ ((val >> 24) & 0xFFFF) ^ ((val >> 48) & 0xFFFF)) as c_uint
}

/// Free memory used by a node
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

    for hash_val in 0..(*ht).size as isize {
        let mut node: *mut hash_node_t = *(*ht).nodes.offset(hash_val);
        while !node.is_null() {
            let node_next: *mut hash_node_t = (*node).next;
            hash_node_free(node);
            node = node_next;
        }
        *(*ht).nodes.offset(hash_val) = null_mut();
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

    let hash_val: usize = (*ht).hash_func.unwrap()(key) as usize % (*ht).size as usize;

    let mut node: *mut *mut hash_node_t = (*ht).nodes.add(hash_val);
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

#[no_mangle]
pub extern "C" fn _export(_: hash_fcompute, _: hash_fcompare, _: hash_fforeach, _: *mut hash_node_t, _: *mut hash_table_t) {}
