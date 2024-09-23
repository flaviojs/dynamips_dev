//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//!
//! Instruction Lookup Tables.

use crate::_private::*;
use crate::dynamips::*;
use crate::dynamips_common::*;
use crate::hash::*;
use crate::utils::*;

pub type cbm_array_t = cbm_array;
pub type rfc_array_t = rfc_array;
pub type rfc_eqclass_t = rfc_eqclass;
pub type insn_lookup_t = insn_lookup;

/// CBM (Class BitMap) array
pub const CBM_SHIFT: size_t = 5; // log2(32)
pub const CBM_SIZE: size_t = 1 << CBM_SHIFT; // Arrays of 32-bits Integers
pub const CBM_HASH_SIZE: size_t = 256; // Size for Hash Tables

/// CBM (Class BitMap) array
#[repr(C)]
#[derive(Debug)]
pub struct cbm_array {
    nr_entries: c_int, // Number of entries
    tab: [c_int; 0],   // Values... // XXX length determined by nr_entries
}

macro_rules! CBM_ARRAY {
    ($array:expr, $i:expr) => {
        *(*$array).tab.as_c_mut().offset($i as isize)
    };
}
macro_rules! CBM_CSIZE {
    ($count:expr) => {
        $count * size_of::<c_int>() as c_int + size_of::<cbm_array_t>() as c_int
    };
}

// callback function prototype for instruction checking
pub type ilt_check_cbk_t = Option<unsafe extern "C" fn(arg1: *mut c_void, value: c_int) -> c_int>;
pub type ilt_get_insn_cbk_t = Option<unsafe extern "C" fn(index: c_int) -> *mut c_void>;

/// RFC (Recursive Flow Classification) arrays
pub const RFC_ARRAY_MAXSIZE: c_int = 65536;
pub const RFC_ARRAY_MAXBITS: size_t = 16;
pub const RFC_ARRAY_NUMBER: usize = 3;

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
    pub eqID: [c_int; 0], // XXX length determined by nr_eqid
}

/// Equivalent Classes
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rfc_eqclass {
    pub cbm: *mut cbm_array_t, // Class Bitmap
    pub eqID: c_int,           // Index associated to this class
}

/// Instruction lookup table
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct insn_lookup {
    pub nr_insn: c_int,  // Number of instructions
    pub cbm_size: c_int, // Size of Class Bitmaps

    pub get_insn: ilt_get_insn_cbk_t,
    pub chk_lo: ilt_check_cbk_t,
    pub chk_hi: ilt_check_cbk_t,

    /// RFC tables
    pub rfct: [*mut rfc_array_t; RFC_ARRAY_NUMBER],
}

/// Instruction lookup
#[inline(always)]
unsafe fn ilt_get_index(a1: *mut rfc_array_t, a2: *mut rfc_array_t, i1: c_int, i2: c_int) -> c_int {
    (*(*a1).eqID.as_ptr().offset(i1 as isize) * (*a2).nr_eqid) + *(*a2).eqID.as_ptr().offset(i2 as isize)
}

#[inline(always)]
unsafe fn ilt_get_idx(ilt: *mut insn_lookup_t, a1: c_int, a2: c_int, i1: c_int, i2: c_int) -> c_int {
    ilt_get_index((*ilt).rfct[a1 as usize], (*ilt).rfct[a2 as usize], i1, i2)
}

#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ilt_lookup(ilt: *mut insn_lookup_t, insn: mips_insn_t) -> c_int {
    let id_i: c_int = ilt_get_idx(ilt, 0, 1, (insn >> 16) as c_int, (insn & 0xFFFF) as c_int);
    *(*(*ilt).rfct[2]).eqID.as_c().offset(id_i as isize)
}

/// Hash function for a CBM
#[inline]
unsafe extern "C" fn cbm_hash_f(ccbm: *mut c_void) -> u_int {
    let cbm: *mut cbm_array_t = ccbm.cast::<_>();
    let mut p: *mut c_char;
    let s: *mut c_char = (*cbm).tab.as_c_mut().cast::<c_char>();
    let mut h: u_int;
    let mut g: u_int;

    h = 0;
    p = s;
    for _ in 0..((*cbm).nr_entries * size_of::<c_int>() as c_int) as u_int {
        h = (h << 4) + *p as c_int as u_int;
        g = h & 0xf0000000;
        if g != 0 {
            h ^= g >> 24;
            h ^= g;
        }
        p = p.add(1);
    }

    h
}

/// Comparison function for 2 CBM
#[inline]
unsafe extern "C" fn cbm_cmp_f(b1: *mut c_void, b2: *mut c_void) -> c_int {
    let cbm1: *mut cbm_array_t = b1.cast::<_>();
    let cbm2: *mut cbm_array_t = b2.cast::<_>();

    for i in 0..(*cbm1).nr_entries as c_int {
        if *(*cbm1).tab.as_c().offset(i as isize) != *(*cbm2).tab.as_c().offset(i as isize) {
            return FALSE;
        }
    }

    TRUE
}

/// Set bit corresponding to a rule number in a CBM
#[inline]
unsafe fn cbm_set_rule(cbm: *mut cbm_array_t, rule_id: c_int) {
    CBM_ARRAY!(cbm, rule_id >> CBM_SHIFT) |= 1 << (rule_id & (CBM_SIZE as c_int - 1));
}

/// Clear bit corresponding to a rule number in a CBM
#[inline]
unsafe fn cbm_unset_rule(cbm: *mut cbm_array_t, rule_id: c_int) {
    CBM_ARRAY!(cbm, rule_id >> CBM_SHIFT) &= !(1 << (rule_id & (CBM_SIZE as c_int - 1)));
}

/// Returns TRUE if  bit corresponding to a rule number in a CBM is set
#[inline]
unsafe fn cbm_check_rule(cbm: *mut cbm_array_t, rule_id: c_int) -> c_int {
    CBM_ARRAY!(cbm, rule_id >> CBM_SHIFT) & (1 << (rule_id & (CBM_SIZE as c_int - 1)))
}

/// Compute bitwise ANDing of two CBM
#[inline]
unsafe fn cbm_bitwise_and(result: *mut cbm_array_t, a1: *mut cbm_array_t, a2: *mut cbm_array_t) {
    // Compute bitwise ANDing
    for i in 0..(*a1).nr_entries as c_int {
        CBM_ARRAY!(result, i) = CBM_ARRAY!(a1, i) & CBM_ARRAY!(a2, i);
    }
}

/// Get first matching rule number
#[inline]
unsafe fn cbm_first_match(ilt: *mut insn_lookup_t, cbm: *mut cbm_array_t) -> c_int {
    for i in 0..(*ilt).nr_insn as c_int {
        if cbm_check_rule(cbm, i) != 0 {
            return i;
        }
    }

    -1
}

/// Create a class bitmap (CBM)
unsafe fn cbm_create(ilt: *mut insn_lookup_t) -> *mut cbm_array_t {
    let size: c_int = CBM_CSIZE!((*ilt).cbm_size);

    // CBM are simply bit arrays
    let array: *mut cbm_array_t = libc::malloc(size as size_t).cast::<_>();
    assert!(!array.is_null());

    libc::memset(array.cast::<_>(), 0, size as size_t);
    (*array).nr_entries = (*ilt).cbm_size;
    array
}

/// Duplicate a class bitmap
unsafe fn cbm_duplicate(cbm: *mut cbm_array_t) -> *mut cbm_array_t {
    let size: c_int = CBM_CSIZE!((*cbm).nr_entries);

    let array: *mut cbm_array_t = libc::malloc(size as size_t).cast::<_>();
    assert!(!array.is_null());
    libc::memcpy(array.cast::<_>(), cbm.cast::<_>(), size as size_t);
    array
}

/// Get equivalent class corresponding to a class bitmap. Create eqclass
/// structure if needed (CBM not previously seen).
unsafe fn cbm_get_eqclass(rfct: *mut rfc_array_t, cbm: *mut cbm_array_t) -> *mut rfc_eqclass_t {
    let mut eqcl: *mut rfc_eqclass_t;

    // Lookup for CBM into hash table
    eqcl = hash_table_lookup((*rfct).cbm_hash, cbm.cast::<_>()).cast::<_>();
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
unsafe fn rfc_alloc_array(nr_elements: c_int) -> *mut rfc_array_t {
    // Compute size of memory chunk needed to store the array
    let total_size: c_int = (nr_elements * size_of::<c_int>() as c_int) + size_of::<rfc_array_t>() as c_int;
    let array: *mut rfc_array_t = libc::malloc(total_size as size_t).cast::<_>();
    assert!(!array.is_null());
    libc::memset(array.cast::<_>(), 0, total_size as size_t);
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
unsafe fn rfc_free_array(array: *mut rfc_array_t) {
    assert!(!array.is_null());

    // Free hash table for Class Bitmaps
    if !(*array).cbm_hash.is_null() {
        hash_table_foreach((*array).cbm_hash, Some(rfc_free_array_cbm_hash_value), array.cast::<_>());
        hash_table_delete((*array).cbm_hash);
        (*array).cbm_hash = null_mut();
    }

    // Free table for converting ID to CBM
    if !(*array).id2cbm.is_null() {
        for i in 0..(*array).nr_elements as c_int {
            if !(*(*array).id2cbm.offset(i as isize)).is_null() {
                libc::free((*(*array).id2cbm.offset(i as isize)).cast::<_>());
            }
        }
        libc::free((*array).id2cbm.cast::<_>());
        (*array).id2cbm = null_mut();
    }

    // Free array
    libc::free(array.cast::<_>());
}

/// Check an instruction with specified parameter
unsafe fn rfc_check_insn(ilt: *mut insn_lookup_t, cbm: *mut cbm_array_t, pcheck: ilt_check_cbk_t, value: c_int) {
    for i in 0..(*ilt).nr_insn as c_int {
        let p: *mut c_void = (*ilt).get_insn.unwrap()(i);

        if pcheck.unwrap()(p, value) != 0 {
            cbm_set_rule(cbm, i);
        } else {
            cbm_unset_rule(cbm, i);
        }
    }
}

/// RFC Chunk preprocessing: phase 0
unsafe fn rfc_phase_0(ilt: *mut insn_lookup_t, pcheck: ilt_check_cbk_t) -> *mut rfc_array_t {
    let mut eqcl: *mut rfc_eqclass_t;

    // allocate a temporary class bitmap
    let bmp: *mut cbm_array_t = cbm_create(ilt);
    assert!(!bmp.is_null());

    // Allocate a new RFC array of 16-bits entries
    let rfct: *mut rfc_array_t = rfc_alloc_array(RFC_ARRAY_MAXSIZE);
    assert!(!rfct.is_null());

    for i in 0..RFC_ARRAY_MAXSIZE as c_int {
        // determine all instructions that match this value
        rfc_check_insn(ilt, bmp, pcheck, i);

        // get equivalent class for this bitmap
        eqcl = cbm_get_eqclass(rfct, bmp);
        assert!(!eqcl.is_null());

        // fill the RFC table
        *(*rfct).eqID.as_c_mut().offset(i as isize) = (*eqcl).eqID;
    }

    libc::free(bmp.cast::<_>());
    rfct
}

/// RFC Chunk preprocessing: phase j (j > 0)
unsafe fn rfc_phase_j(ilt: *mut insn_lookup_t, p0: *mut rfc_array_t, p1: *mut rfc_array_t) -> *mut rfc_array_t {
    let mut eqcl: *mut rfc_eqclass_t;
    let mut index: c_int = 0;

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
    for i in 0..(*p0).nr_eqid as c_int {
        for j in 0..(*p1).nr_eqid as c_int {
            // compute bitwise AND
            cbm_bitwise_and(bmp, *(*p0).id2cbm.offset(i as isize), *(*p1).id2cbm.offset(j as isize));

            // get equivalent class for this bitmap
            eqcl = cbm_get_eqclass(rfct, bmp);
            assert!(!eqcl.is_null());

            // fill RFC table
            *(*rfct).eqID.as_c_mut().offset(index as isize) = (*eqcl).eqID;
            index += 1;
        }
    }

    libc::free(bmp.cast::<_>());
    rfct
}

/// Compute RFC phase 0
unsafe fn ilt_phase_0(ilt: *mut insn_lookup_t, idx: c_int, pcheck: ilt_check_cbk_t) {
    let rfct: *mut rfc_array_t = rfc_phase_0(ilt, pcheck);
    assert!(!rfct.is_null());
    (*ilt).rfct[idx as usize] = rfct;
}

/// Compute RFC phase j
unsafe fn ilt_phase_j(ilt: *mut insn_lookup_t, p0: c_int, p1: c_int, res: c_int) {
    let rfct: *mut rfc_array_t = rfc_phase_j(ilt, (*ilt).rfct[p0 as usize], (*ilt).rfct[p1 as usize]);
    assert!(!rfct.is_null());
    (*ilt).rfct[res as usize] = rfct;
}

/// Postprocessing
unsafe fn ilt_postprocessing(ilt: *mut insn_lookup_t) {
    let rfct: *mut rfc_array_t = (*ilt).rfct[2];

    for i in 0..(*rfct).nr_elements as c_int {
        *(*rfct).eqID.as_c_mut().offset(i as isize) = cbm_first_match(ilt, *(*rfct).id2cbm.offset(*(*rfct).eqID.as_c().offset(i as isize) as isize));
    }
}

/// Instruction lookup table compilation
unsafe fn ilt_compile(ilt: *mut insn_lookup_t) {
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

    for i in 0..RFC_ARRAY_NUMBER as c_int {
        let rfct: *mut rfc_array_t = (*ilt).rfct[i as usize];

        libc::fprintf(fd, cstr!("RFCT %d: nr_elements=%d, nr_eqid=%d\n"), i, (*rfct).nr_elements, (*rfct).nr_eqid);

        for j in 0..(*rfct).nr_elements as c_int {
            libc::fprintf(fd, cstr!("  (0x%4.4x,0x%4.4x) = 0x%4.4x\n"), i, j, *(*rfct).eqID.as_c().offset(j as isize));
        }
    }

    libc::fclose(fd);
    libc::free(filename.cast::<_>());
    0
}

/// Write the specified RFC array to disk
unsafe fn ilt_store_rfct(fd: *mut libc::FILE, id: c_int, rfct: *mut rfc_array_t) {
    // Store RFC array ID + number of elements
    libc::fwrite(addr_of!(id).cast::<_>(), size_of::<c_int>(), 1, fd);
    libc::fwrite(addr_of!((*rfct).nr_elements).cast::<_>(), size_of::<c_int>(), 1, fd);
    libc::fwrite(addr_of!((*rfct).nr_eqid).cast::<_>(), size_of::<c_int>(), 1, fd);

    libc::fwrite((*rfct).eqID.as_ptr().cast::<_>(), size_of::<c_int>(), (*rfct).nr_elements as size_t, fd);
}

/// Write the full instruction lookup table
unsafe fn ilt_store_table(fd: *mut libc::FILE, ilt: *mut insn_lookup_t) {
    for i in 0..RFC_ARRAY_NUMBER as c_int {
        if !(*ilt).rfct[i as usize].is_null() {
            ilt_store_rfct(fd, i, (*ilt).rfct[i as usize]);
        }
    }
}

/// Load an RFC array from disk
unsafe fn ilt_load_rfct(fd: *mut libc::FILE, ilt: *mut insn_lookup_t) -> c_int {
    let mut id: u_int = 0;
    let mut nr_elements: u_int = 0;
    let mut nr_eqid: u_int = 0;

    // Read ID and number of elements
    if libc::fread(addr_of_mut!(id).cast::<_>(), size_of::<u_int>(), 1, fd) != 1 || libc::fread(addr_of_mut!(nr_elements).cast::<_>(), size_of::<u_int>(), 1, fd) != 1 || libc::fread(addr_of_mut!(nr_eqid).cast::<_>(), size_of::<u_int>(), 1, fd) != 1 {
        return -1;
    }

    if id >= RFC_ARRAY_NUMBER as u_int || nr_elements > RFC_ARRAY_MAXSIZE as u_int {
        return -1;
    }

    // Allocate the RFC array with the eqID table
    let len: size_t = size_of::<rfc_array_t>() + (nr_elements as size_t * size_of::<c_int>());

    let rfct: *mut rfc_array_t = libc::malloc(len).cast::<_>();
    if !rfct.is_null() {
        return -1;
    }

    libc::memset(rfct.cast::<_>(), 0, size_of::<rfc_array_t>());
    (*rfct).nr_elements = nr_elements as c_int;
    (*rfct).nr_eqid = nr_eqid as c_int;

    // Read the equivalent ID array
    if libc::fread((*rfct).eqID.as_c_mut().cast::<_>(), size_of::<c_int>(), nr_elements as size_t, fd) != nr_elements as size_t {
        libc::free(rfct.cast::<_>());
        return -1;
    }

    (*ilt).rfct[id as usize] = rfct;
    0
}

/// Check an instruction table loaded from disk
unsafe fn ilt_check_cached_table(ilt: *mut insn_lookup_t) -> c_int {
    // All arrays must have been loaded
    for i in 0..RFC_ARRAY_NUMBER as c_int {
        if (*ilt).rfct[i as usize].is_null() {
            return -1;
        }
    }

    0
}

/// Load a full instruction table from disk
unsafe fn ilt_load_table(fd: *mut libc::FILE) -> *mut insn_lookup_t {
    let ilt: *mut insn_lookup_t = libc::malloc(size_of::<insn_lookup_t>()).cast::<_>();
    if ilt.is_null() {
        return null_mut();
    }

    libc::memset(ilt.cast::<_>(), 0, size_of::<insn_lookup_t>());
    libc::fseek(fd, 0, libc::SEEK_SET);

    for _ in 0..RFC_ARRAY_NUMBER {
        if ilt_load_rfct(fd, ilt) == -1 {
            ilt_destroy(ilt);
            return null_mut();
        }
    }

    if ilt_check_cached_table(ilt) == -1 {
        ilt_destroy(ilt);
        return null_mut();
    }

    ilt
}

/// Build a filename for a cached ILT table on disk
unsafe fn ilt_build_filename(table_name: *mut c_char) -> *mut c_char {
    dyn_sprintf!(cstr!("ilt_%s_%s"), sw_version_tag, table_name)
}

/// Try to load a cached ILT table from disk
unsafe fn ilt_cache_load(table_name: *mut c_char) -> *mut insn_lookup_t {
    let filename: *mut c_char = ilt_build_filename(table_name);
    if filename.is_null() {
        return null_mut();
    }

    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("rb"));
    if fd.is_null() {
        libc::free(filename.cast::<_>());
        return null_mut();
    }

    let ilt: *mut insn_lookup_t = ilt_load_table(fd);
    libc::fclose(fd);
    libc::free(filename.cast::<_>());
    ilt
}

/// Store the specified ILT table on disk for future use (cache)
unsafe fn ilt_cache_store(table_name: *mut c_char, ilt: *mut insn_lookup_t) -> c_int {
    let filename: *mut c_char = ilt_build_filename(table_name);
    if filename.is_null() {
        return -1;
    }

    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("wb"));
    if fd.is_null() {
        libc::free(filename.cast::<_>());
        return -1;
    }

    ilt_store_table(fd, ilt);
    libc::fclose(fd);
    libc::free(filename.cast::<_>());
    0
}

/// Create an instruction lookup table
#[no_mangle]
pub unsafe extern "C" fn ilt_create(table_name: *mut c_char, nr_insn: c_int, get_insn: ilt_get_insn_cbk_t, chk_lo: ilt_check_cbk_t, chk_hi: ilt_check_cbk_t) -> *mut insn_lookup_t {
    // Try to load a cached table from disk
    let mut ilt: *mut insn_lookup_t = ilt_cache_load(table_name);
    if !ilt.is_null() {
        libc::printf(cstr!("ILT: loaded table \"%s\" from cache.\n"), table_name);
        return ilt;
    }

    // We have to build the full table...
    ilt = libc::malloc(size_of::<insn_lookup_t>()).cast::<_>();
    assert!(!ilt.is_null());
    libc::memset(ilt.cast::<_>(), 0, size_of::<insn_lookup_t>());

    (*ilt).cbm_size = normalize_size(nr_insn as c_uint, CBM_SIZE as c_uint, CBM_SHIFT as c_int) as c_int;
    (*ilt).nr_insn = nr_insn;
    (*ilt).get_insn = get_insn;
    (*ilt).chk_lo = chk_lo;
    (*ilt).chk_hi = chk_hi;

    // Compile the instruction opcodes
    ilt_compile(ilt);

    // Store the result on disk for future exec
    ilt_cache_store(table_name, ilt);
    ilt
}

/// Destroy an instruction lookup table
#[no_mangle]
pub unsafe extern "C" fn ilt_destroy(ilt: *mut insn_lookup_t) {
    assert!(!ilt.is_null());

    // Free instruction opcodes
    for i in 0..RFC_ARRAY_NUMBER as c_int {
        if !(*ilt).rfct[i as usize].is_null() {
            rfc_free_array((*ilt).rfct[i as usize]);
        }
    }

    // Free instruction lookup table
    libc::free(ilt.cast::<_>());
}
