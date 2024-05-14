//! Instruction Lookup Tables.

use crate::prelude::*;

pub type cbm_array_t = cbm_array;

/// log2(32)
pub const CBM_SHIFT: size_t = 5;
/// Arrays of 32-bits Integers
pub const CBM_SIZE: size_t = 1 << CBM_SHIFT;
/// Size for Hash Tables
pub const CBM_HASH_SIZE: size_t = 256;

/// CBM (Class BitMap) array
#[repr(C)]
#[derive(Debug)]
pub struct cbm_array {
    /// Number of entries
    nr_entries: c_int,
    /// Values...
    tab: [c_int; 0],
}

unsafe fn CBM_ARRAY<'a>(array: *mut cbm_array, i: c_int) -> &'a mut c_int {
    (*array).tab.as_ptr().cast_mut().offset(i as isize).as_mut().unwrap()
}
unsafe fn CBM_CSIZE(count: c_int) -> c_int {
    count * size_of::<c_int>() as c_int + size_of::<cbm_array_t>() as c_int
}

/// Hash function for a CBM
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_hash_f(ccbm: *mut c_void) -> c_uint {
    let cbm: *mut cbm_array_t = ccbm.cast::<_>();
    let s: *mut c_char = (*cbm).tab.as_ptr().cast_mut().cast::<_>();

    let mut h: c_uint = 0;
    let mut p: *mut c_char = s;
    for _ in 0..(*cbm).nr_entries * size_of::<c_int>() as c_int {
        h = (h << 4) + *p as c_int as c_uint;
        let g: c_uint = h & 0xf0000000;
        if g != 0 {
            h ^= g >> 24;
            h ^= g;
        }
        p = p.add(1);
    }

    h
}

/// Comparison function for 2 CBM
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_cmp_f(b1: *mut c_void, b2: *mut c_void) -> c_int {
    let cbm1: *mut cbm_array_t = b1.cast::<_>();
    let cbm2: *mut cbm_array_t = b2.cast::<_>();

    for i in 0..(*cbm1).nr_entries as isize {
        if *(*cbm1).tab.as_ptr().offset(i) != *(*cbm2).tab.as_ptr().offset(i) {
            return 0;
        }
    }

    1
}

/// Set bit corresponding to a rule number in a CBM
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_set_rule(cbm: *mut cbm_array_t, rule_id: c_int) {
    *CBM_ARRAY(cbm, rule_id >> CBM_SHIFT) |= 1 << (rule_id & (CBM_SIZE - 1) as c_int);
}

/// Clear bit corresponding to a rule number in a CBM
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_unset_rule(cbm: *mut cbm_array_t, rule_id: c_int) {
    *CBM_ARRAY(cbm, rule_id >> CBM_SHIFT) &= !(1 << (rule_id & (CBM_SIZE - 1) as c_int));
}

#[no_mangle]
pub extern "C" fn _export(_: *mut cbm_array_t) {}
