//! Instruction Lookup Tables.

use crate::dynamips::*;
use crate::hash::*;
use crate::prelude::*;
use crate::utils::*;

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

/// RFC Chunk preprocessing: phase 0
#[no_mangle] // TODO private
pub unsafe extern "C" fn rfc_phase_0(ilt: *mut insn_lookup_t, pcheck: ilt_check_cbk_t) -> *mut rfc_array_t {
    // allocate a temporary class bitmap
    let bmp: *mut cbm_array_t = cbm_create(ilt);
    assert!(!bmp.is_null());

    // Allocate a new RFC array of 16-bits entries
    let rfct: *mut rfc_array_t = rfc_alloc_array(RFC_ARRAY_MAXSIZE as c_int);
    assert!(!rfct.is_null());

    for i in 0..RFC_ARRAY_MAXSIZE {
        // determine all instructions that match this value
        rfc_check_insn(ilt, bmp, pcheck, i as c_int);

        // get equivalent class for this bitmap */
        let eqcl: *mut rfc_eqclass_t = cbm_get_eqclass(rfct, bmp);
        assert!(!eqcl.is_null());

        // fill the RFC table
        *(*rfct).eqID.as_ptr().cast_mut().add(i) = (*eqcl).eqID;
    }

    libc::free(bmp.cast::<_>());
    rfct
}

/// RFC Chunk preprocessing: phase j (j > 0)
#[no_mangle] // TODO private
pub unsafe extern "C" fn rfc_phase_j(ilt: *mut insn_lookup_t, p0: *mut rfc_array_t, p1: *mut rfc_array_t) -> *mut rfc_array_t {
    let mut index: isize = 0;

    // allocate a temporary class bitmap
    let bmp: *mut cbm_array_t = cbm_create(ilt);
    assert!(!bmp.is_null());

    // compute number of elements
    let nr_elements: c_int = (*p0).nr_eqid * (*p1).nr_eqid;

    // allocate a new RFC array
    let rfct: *mut rfc_array_t = rfc_alloc_array(nr_elements);
    assert!(!rfct.is_null());
    (*rfct).parent0 = p0;
    (*rfct).parent1 = p1;

    // make a cross product between p0 and p1
    for i in 0..(*p0).nr_eqid as isize {
        for j in 0..(*p1).nr_eqid as isize {
            // compute bitwise AND
            cbm_bitwise_and(bmp, *(*p0).id2cbm.offset(i), *(*p1).id2cbm.offset(j));

            // get equivalent class for this bitmap
            let eqcl: *mut rfc_eqclass_t = cbm_get_eqclass(rfct, bmp);
            assert!(!eqcl.is_null());

            // fill RFC table
            *(*rfct).eqID.as_ptr().cast_mut().offset(index) = (*eqcl).eqID;
            index += 1;
        }
    }

    libc::free(bmp.cast::<_>());
    rfct
}

/// Compute RFC phase 0
#[no_mangle] // TODO private
pub unsafe extern "C" fn ilt_phase_0(ilt: *mut insn_lookup_t, idx: c_int, pcheck: ilt_check_cbk_t) {
    let rfct: *mut rfc_array_t = rfc_phase_0(ilt, pcheck);
    assert!(!rfct.is_null());
    (*ilt).rfct[idx as usize] = rfct;
}

/// Compute RFC phase j
#[no_mangle] // TODO private
pub unsafe extern "C" fn ilt_phase_j(ilt: *mut insn_lookup_t, p0: c_int, p1: c_int, res: c_int) {
    let rfct: *mut rfc_array_t = rfc_phase_j(ilt, (*ilt).rfct[p0 as usize], (*ilt).rfct[p1 as usize]);
    assert!(!rfct.is_null());
    (*ilt).rfct[res as usize] = rfct;
}

/// Postprocessing
#[no_mangle] // TODO private
pub unsafe extern "C" fn ilt_postprocessing(ilt: *mut insn_lookup_t) {
    let rfct: *mut rfc_array_t = (*ilt).rfct[2];

    for i in 0..(*rfct).nr_elements as isize {
        *(*rfct).eqID.as_ptr().cast_mut().offset(i) = cbm_first_match(ilt, *(*rfct).id2cbm.offset(*(*rfct).eqID.as_ptr().offset(i) as isize));
    }
}

/// Instruction lookup table compilation
#[no_mangle] // TODO private
pub unsafe extern "C" fn ilt_compile(ilt: *mut insn_lookup_t) {
    ilt_phase_0(ilt, 0, (*ilt).chk_hi);
    ilt_phase_0(ilt, 1, (*ilt).chk_lo);
    ilt_phase_j(ilt, 0, 1, 2);
    ilt_postprocessing(ilt);
}

/// Dump an instruction lookup table
unsafe fn ilt_dump(table_name: *mut c_char, ilt: *mut insn_lookup_t) -> c_int {
    let filename: *mut c_char = dyn_sprintf!(cstr!("ilt_dump_%s_%s.txt"), sw_version_tag, table_name);
    assert!(!filename.is_null());

    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("w"));
    assert!(!fd.is_null());

    libc::fprintf(fd, cstr!("ILT %p: nr_insn=%d, cbm_size=%d\n"), ilt, (*ilt).nr_insn, (*ilt).cbm_size);

    for i in 0..RFC_ARRAY_NUMBER {
        let rfct: *mut rfc_array_t = (*ilt).rfct[i];

        libc::fprintf(fd, cstr!("RFCT %d: nr_elements=%d, nr_eqid=%d\n"), i as c_int, (*rfct).nr_elements, (*rfct).nr_eqid);

        for j in 0..(*rfct).nr_elements {
            libc::fprintf(fd, cstr!("  (0x%4.4x,0x%4.4x) = 0x%4.4x\n"), i as c_int, j, *(*rfct).eqID.as_ptr().offset(j as isize));
        }
    }

    libc::fclose(fd);
    libc::free(filename.cast::<_>());
    0
}

/// Write the specified RFC array to disk
#[no_mangle] // TODO private
pub unsafe extern "C" fn ilt_store_rfct(fd: *mut libc::FILE, id: c_int, rfct: *mut rfc_array_t) {
    // Store RFC array ID + number of elements
    libc::fwrite(addr_of!(id).cast::<_>(), size_of::<c_int>(), 1, fd);
    libc::fwrite(addr_of!((*rfct).nr_elements).cast::<_>(), size_of::<c_int>(), 1, fd);
    libc::fwrite(addr_of!((*rfct).nr_eqid).cast::<_>(), size_of::<c_int>(), 1, fd);

    libc::fwrite((*rfct).eqID.as_ptr().cast::<_>(), size_of::<c_int>(), (*rfct).nr_elements as size_t, fd);
}

#[no_mangle]
pub extern "C" fn _export(_: *mut cbm_array_t) {}
