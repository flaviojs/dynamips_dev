//! ATM definitions, ATM utility functions and Virtual ATM switch.
//!
//! HEC and AAL5 CRC computation functions are from Charles Michael Heard
//! and can be found at (no licence specified, this is to check!):
//!
//!    http://cell-relay.indiana.edu/cell-relay/publications/software/CRC/

use crate::dynamips_common::*;
use crate::mempool::*;
use crate::net_io::*;
use crate::prelude::*;
use crate::registry::*;
use crate::utils::*;

pub type atmsw_vp_conn_t = atmsw_vp_conn;
pub type atmsw_vc_conn_t = atmsw_vc_conn;
pub type atmsw_table_t = atmsw_table;

/// ATM payload size
pub const ATM_HDR_SIZE: usize = 5;
pub const ATM_PAYLOAD_SIZE: usize = 48;
pub const ATM_CELL_SIZE: usize = ATM_HDR_SIZE + ATM_PAYLOAD_SIZE;
pub const ATM_AAL5_TRAILER_SIZE: usize = 8;
pub const ATM_AAL5_TRAILER_POS: usize = ATM_CELL_SIZE - ATM_AAL5_TRAILER_SIZE;

/// ATM header structure
pub const ATM_HDR_VPI_MASK: m_uint32_t = 0xFFF00000;
pub const ATM_HDR_VPI_SHIFT: c_int = 20;
pub const ATM_HDR_VCI_MASK: m_uint32_t = 0x000FFFF0;
pub const ATM_HDR_VCI_SHIFT: c_int = 4;
pub const ATM_HDR_PTI_MASK: m_uint32_t = 0x0000000E;
pub const ATM_HDR_PTI_SHIFT: c_int = 1;

/// PTI bits
pub const ATM_PTI_EOP: m_uint32_t = 0x00000002; // End of packet
pub const ATM_PTI_CONGESTION: m_uint32_t = 0x00000004; // Congestion detected
pub const ATM_PTI_NETWORK: m_uint32_t = 0x00000008; // Network traffic

/// VP-level switch table
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct atmsw_vp_conn {
    pub next: *mut atmsw_vp_conn_t,
    pub input: *mut netio_desc_t,
    pub output: *mut netio_desc_t,
    pub vpi_in: u_int,
    pub vpi_out: u_int,
    pub cell_cnt: m_uint64_t,
}

/// VC-level switch table
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct atmsw_vc_conn {
    pub next: *mut atmsw_vc_conn_t,
    pub input: *mut netio_desc_t,
    pub output: *mut netio_desc_t,
    pub vpi_in: u_int,
    pub vci_in: u_int,
    pub vpi_out: u_int,
    pub vci_out: u_int,
    pub cell_cnt: m_uint64_t,
}

/// Virtual ATM switch table
pub const ATMSW_NIO_MAX: usize = 32;
pub const ATMSW_VP_HASH_SIZE: usize = 256;
pub const ATMSW_VC_HASH_SIZE: usize = 1024;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct atmsw_table {
    pub name: *mut c_char,
    pub lock: libc::pthread_mutex_t,
    pub mp: mempool_t,
    pub cell_drop: m_uint64_t,
    pub vp_table: [*mut atmsw_vp_conn_t; ATMSW_VP_HASH_SIZE],
    pub vc_table: [*mut atmsw_vc_conn_t; ATMSW_VC_HASH_SIZE],
}

pub const ATM_RFC1483B_HLEN: usize = 10;
/// RFC1483 bridged mode header
#[no_mangle]
pub static mut atm_rfc1483b_header: [m_uint8_t; ATM_RFC1483B_HLEN] = [0xaa, 0xaa, 0x03, 0x00, 0x80, 0xc2, 0x00, 0x07, 0x00, 0x00];

#[no_mangle]
pub extern "C" fn _export_atm(_: *mut atmsw_vp_conn_t, _: *mut atmsw_vc_conn_t, _: *mut atmsw_table_t) {}

unsafe fn ATMSW_LOCK(t: *mut atmsw_table_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*t).lock));
}
unsafe fn ATMSW_UNLOCK(t: *mut atmsw_table_t) {
    libc::pthread_mutex_unlock(addr_of_mut!((*t).lock));
}

// ******************************************************************
pub const HEC_GENERATOR: c_int = 0x107; //  x^8 + x^2 +  x  + 1
pub const COSET_LEADER: m_uint8_t = 0x055; // x^6 + x^4 + x^2 + 1

static mut hec_syndrome_table: [m_uint8_t; 256] = [0; 256];

/// Generate a table of CRC-8 syndromes for all possible input bytes
unsafe fn gen_syndrome_table() {
    for i in 0..=255 {
        let mut syndrome: c_int = i;

        for _ in 0..8 {
            if (syndrome & 0x80) != 0 {
                syndrome = (syndrome << 1) ^ HEC_GENERATOR;
            } else {
                syndrome <<= 1;
            }
        }
        hec_syndrome_table[i as usize] = syndrome as m_uint8_t;
    }
}

/// Compute HEC field for ATM header */
#[no_mangle]
pub unsafe extern "C" fn atm_compute_hec(cell_header: *mut m_uint8_t) -> m_uint8_t {
    let mut hec_accum: m_uint8_t = 0;

    // calculate CRC-8 remainder over first four bytes of cell header.
    // exclusive-or with coset leader & insert into fifth header byte.
    for i in 0..4 {
        hec_accum = hec_syndrome_table[(hec_accum ^ *cell_header.add(i)) as usize];
    }

    hec_accum ^ COSET_LEADER
}

/// Insert HEC field into an ATM header
#[no_mangle]
pub unsafe extern "C" fn atm_insert_hec(cell_header: *mut m_uint8_t) {
    *cell_header.add(4) = atm_compute_hec(cell_header);
}

/// Initialize ATM code (for HEC checksums)
#[no_mangle]
pub unsafe extern "C" fn atm_init() {
    gen_syndrome_table();
}

/// VPC hash function
#[no_mangle] // TODO private
#[inline]
pub unsafe extern "C" fn atmsw_vpc_hash(vpi: u_int) -> u_int {
    (vpi ^ (vpi >> 8)) & (ATMSW_VP_HASH_SIZE as u_int - 1)
}

/// VCC hash function
#[no_mangle] // TODO private
#[inline]
pub unsafe extern "C" fn atmsw_vcc_hash(vpi: u_int, vci: u_int) -> u_int {
    (vpi ^ vci) & (ATMSW_VC_HASH_SIZE as u_int - 1)
}

/// VP lookup
#[no_mangle]
pub unsafe extern "C" fn atmsw_vp_lookup(t: *mut atmsw_table_t, input: *mut netio_desc_t, vpi: u_int) -> *mut atmsw_vp_conn_t {
    let mut swc: *mut atmsw_vp_conn_t;

    swc = (*t).vp_table[atmsw_vpc_hash(vpi) as usize];
    while !swc.is_null() {
        if ((*swc).input == input) && ((*swc).vpi_in == vpi) {
            return swc;
        }
        swc = (*swc).next
    }

    null_mut()
}

/// VC lookup
#[no_mangle]
pub unsafe extern "C" fn atmsw_vc_lookup(t: *mut atmsw_table_t, input: *mut netio_desc_t, vpi: u_int, vci: u_int) -> *mut atmsw_vc_conn_t {
    let mut swc: *mut atmsw_vc_conn_t;

    swc = (*t).vc_table[atmsw_vcc_hash(vpi, vci) as usize];
    while !swc.is_null() {
        if ((*swc).input == input) && ((*swc).vpi_in == vpi) && ((*swc).vci_in == vci) {
            return swc;
        }
        swc = (*swc).next;
    }

    null_mut()
}

/// VP switching
#[no_mangle]
pub unsafe extern "C" fn atmsw_vp_switch(vpc: *mut atmsw_vp_conn_t, cell: *mut m_uint8_t) {
    let mut atm_hdr: m_uint32_t;

    // rewrite the atm header with new vpi
    atm_hdr = m_ntoh32(cell);
    atm_hdr &= !ATM_HDR_VPI_MASK;
    atm_hdr |= (*vpc).vpi_out << ATM_HDR_VPI_SHIFT;
    m_hton32(cell, atm_hdr);

    // recompute HEC field
    atm_insert_hec(cell);

    // update the statistics counter
    (*vpc).cell_cnt += 1;
}

/// VC switching
#[no_mangle]
pub unsafe extern "C" fn atmsw_vc_switch(vcc: *mut atmsw_vc_conn_t, cell: *mut m_uint8_t) {
    let mut atm_hdr: m_uint32_t;

    // rewrite the atm header with new vpi/vci
    atm_hdr = m_ntoh32(cell);

    atm_hdr &= !(ATM_HDR_VPI_MASK | ATM_HDR_VCI_MASK);
    atm_hdr |= (*vcc).vpi_out << ATM_HDR_VPI_SHIFT;
    atm_hdr |= (*vcc).vci_out << ATM_HDR_VCI_SHIFT;
    m_hton32(cell, atm_hdr);

    // recompute HEC field
    atm_insert_hec(cell);

    // update the statistics counter
    (*vcc).cell_cnt += 1;
}

/// Handle an ATM cell
#[no_mangle]
pub unsafe extern "C" fn atmsw_handle_cell(t: *mut atmsw_table_t, input: *mut netio_desc_t, cell: *mut m_uint8_t) -> ssize_t {
    let mut output: *mut netio_desc_t = null_mut();

    // Extract VPI/VCI information
    let atm_hdr: m_uint32_t = m_ntoh32(cell);

    let vpi: m_uint32_t = (atm_hdr & ATM_HDR_VPI_MASK) >> ATM_HDR_VPI_SHIFT;
    let vci: m_uint32_t = (atm_hdr & ATM_HDR_VCI_MASK) >> ATM_HDR_VCI_SHIFT;

    // VP switching */
    let vpc: *mut atmsw_vp_conn_t = atmsw_vp_lookup(t, input, vpi);
    if !vpc.is_null() {
        atmsw_vp_switch(vpc, cell);
        output = (*vpc).output;
    } else {
        // VC switching
        let vcc: *mut atmsw_vc_conn_t = atmsw_vc_lookup(t, input, vpi, vci);
        if !vcc.is_null() {
            atmsw_vc_switch(vcc, cell);
            output = (*vcc).output;
        }
    }

    let len: ssize_t = netio_send(output, cell.cast::<_>(), ATM_CELL_SIZE);

    if len != ATM_CELL_SIZE as ssize_t {
        (*t).cell_drop += 1;
        return -1;
    }

    0
}

/// Acquire a reference to an ATM switch (increment reference count)
#[no_mangle]
pub unsafe extern "C" fn atmsw_acquire(name: *mut c_char) -> *mut atmsw_table_t {
    registry_find(name, OBJ_TYPE_ATMSW).cast::<_>()
}

/// Release an ATM switch (decrement reference count)
#[no_mangle]
pub unsafe extern "C" fn atmsw_release(name: *mut c_char) -> c_int {
    registry_unref(name, OBJ_TYPE_ATMSW)
}

/// Create a virtual switch table
#[no_mangle]
pub unsafe extern "C" fn atmsw_create_table(name: *mut c_char) -> *mut atmsw_table_t {
    // Allocate a new switch structure
    let t: *mut atmsw_table_t = libc::malloc(size_of::<atmsw_table_t>()).cast::<_>();
    if t.is_null() {
        return null_mut();
    }

    libc::memset(t.cast::<_>(), 0, size_of::<atmsw_table_t>());
    libc::pthread_mutex_init(addr_of_mut!((*t).lock), null_mut());
    mp_create_fixed_pool(addr_of_mut!((*t).mp), cstr!("ATM Switch"));

    (*t).name = mp_strdup(addr_of_mut!((*t).mp), name);
    if (*t).name.is_null() {
        mp_free_pool(addr_of_mut!((*t).mp));
        libc::free(t.cast::<_>());
        return null_mut();
    }

    // Record this object in registry
    if registry_add((*t).name, OBJ_TYPE_ATMSW, t.cast::<_>()) == -1 {
        libc::fprintf(c_stderr(), cstr!("atmsw_create_table: unable to create switch '%s'\n"), name);
        mp_free_pool(addr_of_mut!((*t).mp));
        libc::free(t.cast::<_>());
        return null_mut();
    }

    t
}

/// Receive an ATM cell
#[no_mangle] // TODO private
pub unsafe extern "C" fn atmsw_recv_cell(nio: *mut netio_desc_t, atm_cell: *mut u_char, cell_len: ssize_t, t: *mut c_void, _: *mut c_void) -> c_int {
    let t: *mut atmsw_table_t = t.cast::<_>();

    if cell_len != ATM_CELL_SIZE as ssize_t {
        return -1;
    }

    ATMSW_LOCK(t);
    let res: c_int = atmsw_handle_cell(t, nio, atm_cell) as c_int;
    ATMSW_UNLOCK(t);
    res
}

/// Free resources used by a VPC
#[no_mangle] // TODO private
pub unsafe extern "C" fn atmsw_release_vpc(swc: *mut atmsw_vp_conn_t) {
    if !swc.is_null() {
        // release input NIO
        if !(*swc).input.is_null() {
            netio_rxl_remove((*swc).input);
            netio_release((*(*swc).input).name);
        }

        // release output NIO
        if !(*swc).output.is_null() {
            netio_release((*(*swc).output).name);
        }
    }
}

/// Create a VP switch connection
#[no_mangle]
pub unsafe extern "C" fn atmsw_create_vpc(t: *mut atmsw_table_t, nio_input: *mut c_char, vpi_in: u_int, nio_output: *mut c_char, vpi_out: u_int) -> c_int {
    ATMSW_LOCK(t);

    // Allocate a new switch connection
    let swc: *mut atmsw_vp_conn_t = mp_alloc(addr_of_mut!((*t).mp), size_of::<atmsw_vp_conn_t>()).cast::<_>();
    if swc.is_null() {
        ATMSW_UNLOCK(t);
        return -1;
    }

    (*swc).input = netio_acquire(nio_input);
    (*swc).output = netio_acquire(nio_output);
    (*swc).vpi_in = vpi_in;
    (*swc).vpi_out = vpi_out;

    // Check these NIOs are valid and the input VPI does not exists
    if (*swc).input.is_null() || (*swc).output.is_null() || !atmsw_vp_lookup(t, (*swc).input, vpi_in).is_null() {
        ATMSW_UNLOCK(t);
        atmsw_release_vpc(swc);
        mp_free(swc.cast::<_>());
        return -1;
    }

    // Add as a RX listener
    if netio_rxl_add((*swc).input, Some(atmsw_recv_cell), t.cast::<_>(), null_mut()) == -1 {
        ATMSW_UNLOCK(t);
        atmsw_release_vpc(swc.cast::<_>());
        mp_free(swc.cast::<_>());
        return -1;
    }

    let hbucket: u_int = atmsw_vpc_hash(vpi_in);
    (*swc).next = (*t).vp_table[hbucket as usize];
    (*t).vp_table[hbucket as usize] = swc;
    ATMSW_UNLOCK(t);
    0
}

/// Delete a VP switch connection
#[no_mangle]
pub unsafe extern "C" fn atmsw_delete_vpc(t: *mut atmsw_table_t, nio_input: *mut c_char, vpi_in: u_int, nio_output: *mut c_char, vpi_out: u_int) -> c_int {
    let mut swc: *mut *mut atmsw_vp_conn_t;
    let mut p: *mut atmsw_vp_conn_t;

    ATMSW_LOCK(t);

    let input: *mut netio_desc_t = registry_exists(nio_input, OBJ_TYPE_NIO).cast::<_>();
    let output: *mut netio_desc_t = registry_exists(nio_output, OBJ_TYPE_NIO).cast::<_>();

    if input.is_null() || output.is_null() {
        ATMSW_UNLOCK(t);
        return -1;
    }

    let hbucket: u_int = atmsw_vpc_hash(vpi_in);
    swc = addr_of_mut!((*t).vp_table[hbucket as usize]);
    while !(*swc).is_null() {
        p = *swc;

        if ((*p).input == input) && ((*p).output == output) && ((*p).vpi_in == vpi_in) && ((*p).vpi_out == vpi_out) {
            // found a matching VP, remove it
            *swc = (*(*swc)).next;
            ATMSW_UNLOCK(t);

            atmsw_release_vpc(p);
            mp_free(p.cast::<_>());
            return 0;
        }
        swc = addr_of_mut!((*(*swc)).next);
    }

    ATMSW_UNLOCK(t);
    -1
}
