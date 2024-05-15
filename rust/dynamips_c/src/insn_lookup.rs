//! Instruction Lookup Tables.

use crate::hash::*;
use crate::prelude::*;

pub type cbm_array_t = cbm_array;
pub type rfc_array_t = rfc_array;
pub type rfc_eqclass_t = rfc_eqclass;
pub type insn_lookup_t = insn_lookup;

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

// callback function prototype for instruction checking
pub type ilt_check_cbk_t = Option<unsafe extern "C" fn(arg1: *mut c_void, value: c_int) -> c_int>;
pub type ilt_get_insn_cbk_t = Option<unsafe extern "C" fn(index: c_int) -> *mut c_void>;

pub const RFC_ARRAY_MAXSIZE: size_t = 65536;
pub const RFC_ARRAY_MAXBITS: size_t = 16;
pub const RFC_ARRAY_NUMBER: size_t = 3;

/// RFC (Recursive Flow Classification) arrays
#[repr(C)]
#[derive(Debug)]
pub struct rfc_array {
    pub parent0: *mut rfc_array_t,
    pub parent1: *mut rfc_array_t,
    pub nr_elements: c_int,

    /// Number of Equivalent ID
    pub nr_eqid: c_int,

    /// Hash Table for Class Bitmaps
    pub cbm_hash: *mut hash_table_t,

    /// Array to get Class Bitmaps from IDs
    pub id2cbm: *mut *mut cbm_array_t,

    /// Equivalent ID (eqID) array
    pub eqID: [c_int; 0],
}

/// Equivalent Classes
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rfc_eqclass {
    /// Class Bitmap
    pub cbm: *mut cbm_array_t,
    /// Index associated to this class
    pub eqID: c_int,
}

/// Instruction lookup table
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct insn_lookup {
    /// Number of instructions
    pub nr_insn: c_int,
    /// Size of Class Bitmaps
    pub cbm_size: c_int,

    pub get_insn: ilt_get_insn_cbk_t,
    pub chk_lo: ilt_check_cbk_t,
    pub chk_hi: ilt_check_cbk_t,

    /// RFC tables
    pub rfct: [*mut rfc_array_t; RFC_ARRAY_NUMBER],
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

/// Returns TRUE if  bit corresponding to a rule number in a CBM is set
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_check_rule(cbm: *mut cbm_array_t, rule_id: c_int) -> c_int {
    *CBM_ARRAY(cbm, rule_id >> CBM_SHIFT) & (1 << (rule_id & (CBM_SIZE - 1) as c_int))
}

/// Compute bitwise ANDing of two CBM
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_bitwise_and(result: *mut cbm_array_t, a1: *mut cbm_array_t, a2: *mut cbm_array_t) {
    // Compute bitwise ANDing
    for i in 0..(*a1).nr_entries {
        *CBM_ARRAY(result, i) = *CBM_ARRAY(a1, i) & *CBM_ARRAY(a2, i);
    }
}

/// Get first matching rule number
#[no_mangle] // TODO ptivate
pub unsafe extern "C" fn cbm_first_match(ilt: *mut insn_lookup_t, cbm: *mut cbm_array_t) -> c_int {
    for i in 0..(*ilt).nr_insn {
        if cbm_check_rule(cbm, i) != 0 {
            return i;
        }
    }

    -1
}

/// Create a class bitmap (CBM)
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_create(ilt: *mut insn_lookup_t) -> *mut cbm_array_t {
    let size: size_t = CBM_CSIZE((*ilt).cbm_size) as size_t;

    // CBM are simply bit arrays
    let array: *mut cbm_array_t = libc::malloc(size).cast::<_>();
    assert!(!array.is_null());

    libc::memset(array.cast::<_>(), 0, size);
    (*array).nr_entries = (*ilt).cbm_size;
    array
}

/// Duplicate a class bitmap
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_duplicate(cbm: *mut cbm_array_t) -> *mut cbm_array_t {
    let size: size_t = CBM_CSIZE((*cbm).nr_entries) as size_t;

    let array: *mut cbm_array_t = libc::malloc(size).cast::<_>();
    assert!(!array.is_null());
    libc::memcpy(array.cast::<_>(), cbm.cast::<_>(), size);
    array
}

/// Get equivalent class corresponding to a class bitmap. Create eqclass
/// structure if needed (CBM not previously seen).
#[no_mangle] // TODO private
pub unsafe extern "C" fn cbm_get_eqclass(rfct: *mut rfc_array_t, cbm: *mut cbm_array_t) -> *mut rfc_eqclass_t {
    // Lookup for CBM into hash table
    let mut eqcl: *mut rfc_eqclass_t = hash_table_lookup((*rfct).cbm_hash, cbm.cast::<_>()).cast::<_>();
    if eqcl.is_null() {
        // Duplicate CBM
        let bmp: *mut cbm_array_t = cbm_duplicate(cbm);
        assert!(!bmp.is_null());

        // CBM is not already known
        eqcl = libc::malloc(size_of::<rfc_eqclass_t>()).cast::<_>();
        assert!(!eqcl.is_null());

        assert!((*rfct).nr_eqid < (*rfct).nr_elements);

        // Get a new equivalent ID
        (*eqcl).eqID = (*rfct).nr_eqid;
        (*rfct).nr_eqid += 1;
        (*eqcl).cbm = bmp;
        *(*rfct).id2cbm.offset((*eqcl).eqID as isize) = bmp;

        // Insert it in hash table
        if hash_table_insert((*rfct).cbm_hash, bmp.cast::<_>(), eqcl.cast::<_>()) == -1 {
            return null_mut();
        }
    }

    eqcl
}

/// Allocate an array for Recursive Flow Classification
#[no_mangle] // TODO private
pub unsafe extern "C" fn rfc_alloc_array(nr_elements: c_int) -> *mut rfc_array_t {
    // Compute size of memory chunk needed to store the array
    let total_size: size_t = (nr_elements as size_t * size_of::<c_int>()) + size_of::<rfc_array_t>();
    let array: *mut rfc_array_t = libc::malloc(total_size).cast::<_>();
    assert!(!array.is_null());
    libc::memset(array.cast::<_>(), 0, total_size);
    (*array).nr_elements = nr_elements;

    // Initialize hash table for Class Bitmaps
    (*array).cbm_hash = hash_table_create(Some(cbm_hash_f), Some(cbm_cmp_f), CBM_HASH_SIZE as c_int);
    assert!(!(*array).cbm_hash.is_null());

    // Initialize table for converting ID to CBM
    (*array).id2cbm = libc::calloc(nr_elements as size_t, size_of::<*mut cbm_array_t>()).cast::<_>();
    assert!(!(*array).id2cbm.is_null());

    array
}

/// Free value of cbm_hash
unsafe extern "C" fn rfc_free_array_cbm_hash_value(_key: *mut c_void, value: *mut c_void, _opt_arg: *mut c_void) {
    libc::free(value); // rfc_eqclass_t *
}

/// Free an array for Recursive Flow Classification
#[no_mangle] // TODO private
pub unsafe extern "C" fn rfc_free_array(array: *mut rfc_array_t) {
    assert!(!array.is_null());

    // Free hash table for Class Bitmaps
    if !(*array).cbm_hash.is_null() {
        hash_table_foreach((*array).cbm_hash, Some(rfc_free_array_cbm_hash_value), array.cast::<_>());
        hash_table_delete((*array).cbm_hash);
        (*array).cbm_hash = null_mut();
    }

    // Free table for converting ID to CBM
    if !(*array).id2cbm.is_null() {
        for i in 0..(*array).nr_elements as isize {
            if !(*(*array).id2cbm.offset(i)).is_null() {
                libc::free((*(*array).id2cbm.offset(i)).cast::<_>());
            }
        }
        libc::free((*array).id2cbm.cast::<_>());
        (*array).id2cbm = null_mut();
    }

    // Free array
    libc::free(array.cast::<_>());
}

/// Check an instruction with specified parameter
#[no_mangle] // TODO private
pub unsafe extern "C" fn rfc_check_insn(ilt: *mut insn_lookup_t, cbm: *mut cbm_array_t, pcheck: ilt_check_cbk_t, value: c_int) {
    for i in 0..(*ilt).nr_insn {
        let p: *mut c_void = (*ilt).get_insn.unwrap()(i);

        if pcheck.unwrap()(p, value) != 0 {
            cbm_set_rule(cbm, i);
        } else {
            cbm_unset_rule(cbm, i);
        }
    }
}

#[no_mangle]
pub extern "C" fn _export(_: *mut cbm_array_t) {}
