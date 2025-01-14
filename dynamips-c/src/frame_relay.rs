//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//!
//! Frame-Relay definitions.
//! Frame-Relay switch.

use crate::_extra::*;
use crate::dynamips_common::*;
use crate::mempool::*;
use crate::net_io::*;
use crate::registry::*;
use crate::utils::*;
use libc::size_t;
use libc::ssize_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;
use std::ptr::addr_of_mut;
use std::ptr::null_mut;

// DLCIs used for LMI
pub const FR_DLCI_LMI_ANSI: m_uint32_t = 0; // ANSI LMI
pub const FR_DLCI_LMI_CISCO: m_uint32_t = 1023; // Cisco LMI

pub const FR_LMI_ANSI_STATUS_OFFSET: usize = 5;
pub const FR_LMI_ANSI_STATUS_ENQUIRY: c_int = 0x75; // sent by user
pub const FR_LMI_ANSI_STATUS: m_uint8_t = 0x7d; // sent by network

// Maximum packet size
pub const FR_MAX_PKT_SIZE: usize = 2048;

// Frame-Relay switch table
pub type frsw_conn_t = frsw_conn;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct frsw_conn {
    pub hash_next: *mut frsw_conn_t,
    pub next: *mut frsw_conn_t,
    pub pprev: *mut *mut frsw_conn_t,
    pub input: *mut netio_desc_t,
    pub output: *mut netio_desc_t,
    pub dlci_in: u_int,
    pub dlci_out: u_int,
    pub count: m_uint64_t,
}

// Virtual Frame-Relay switch table
pub const FRSW_HASH_SIZE: usize = 256;

pub type frsw_table_t = frsw_table;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct frsw_table {
    pub name: *mut ::std::os::raw::c_char,
    pub lock: libc::pthread_mutex_t,
    pub mp: mempool_t,
    pub drop: m_uint64_t,
    pub dlci_table: [*mut frsw_conn_t; FRSW_HASH_SIZE],
}

macro_rules! FRSW_LOCK {
    ($t:expr) => {
        libc::pthread_mutex_lock(addr_of_mut!((*$t).lock))
    };
}
macro_rules! FRSW_UNLOCK {
    ($t:expr) => {
        libc::pthread_mutex_unlock(addr_of_mut!((*$t).lock))
    };
}

const DEBUG_FRSW: c_int = 0;

// ANSI LMI packet header
#[rustfmt::skip]
static lmi_ansi_hdr: [m_uint8_t; 7] = [
    0x00, 0x01, 0x03, 0x08, 0x00, 0x75, 0x95,
];

// DLCI hash function
#[inline]
unsafe fn frsw_dlci_hash(dlci: u_int) -> u_int {
    (dlci ^ (dlci >> 8)) & (FRSW_HASH_SIZE as u_int - 1)
}

// DLCI lookup
#[no_mangle]
pub unsafe extern "C" fn frsw_dlci_lookup(t: *mut frsw_table_t, input: *mut netio_desc_t, dlci: u_int) -> *mut frsw_conn_t {
    let mut vc: *mut frsw_conn_t;

    vc = (*t).dlci_table[frsw_dlci_hash(dlci) as usize];
    while !vc.is_null() {
        if ((*vc).input == input) && ((*vc).dlci_in == dlci) {
            return vc;
        }
        vc = (*vc).hash_next;
    }

    null_mut()
}

// Handle a ANSI LMI packet
#[no_mangle]
pub unsafe extern "C" fn frsw_handle_lmi_ansi_pkt(_t: *mut frsw_table_t, input: *mut netio_desc_t, pkt: *mut m_uint8_t, #[allow(unused_assignments)] mut len: ssize_t) -> ssize_t {
    let mut resp: [m_uint8_t; FR_MAX_PKT_SIZE] = [0; FR_MAX_PKT_SIZE];
    let mut pres: *mut m_uint8_t;
    let mut preq: *mut m_uint8_t;
    let mut itype: m_uint8_t;
    let mut isize_: m_uint8_t;
    let mut msg_type: c_int;
    let mut seq_ok: c_int;
    let mut sc: *mut frsw_conn_t;
    let mut dlci: u_int;

    len = lmi_ansi_hdr.len() as ssize_t;
    if len != 0 || libc::memcmp(pkt.cast::<_>(), lmi_ansi_hdr.as_ptr().cast::<_>(), lmi_ansi_hdr.len()) != 0 {
        return -1;
    }

    if DEBUG_FRSW != 0 {
        m_log!((*input).name, c"received an ANSI LMI packet:\n".as_ptr());
        mem_dump(log_file, pkt, len as u_int);
    }

    // Prepare response packet
    libc::memcpy(resp.as_mut_ptr().cast::<_>(), lmi_ansi_hdr.as_ptr().cast::<_>(), lmi_ansi_hdr.len());
    resp[FR_LMI_ANSI_STATUS_OFFSET] = FR_LMI_ANSI_STATUS;

    preq = pkt.add(lmi_ansi_hdr.len());
    pres = resp.as_mut_ptr().add(lmi_ansi_hdr.len());

    msg_type = -1;
    seq_ok = FALSE;

    'not_done: while preq.add(2) < pkt.offset(len) {
        // get item type and size
        itype = *preq.add(0);
        isize_ = *preq.add(1);

        // check packet boundary
        if preq.add(isize_ as usize + 2) > pkt.offset(len) {
            m_log!((*input).name, c"invalid LMI packet:\n".as_ptr());
            mem_dump(log_file, pkt, len as u_int);
            return -1;
        }

        match itype {
            0x01 => {
                // report information element
                if isize_ != 1 {
                    m_log!((*input).name, c"invalid LMI item size.\n".as_ptr());
                    return -1;
                }

                msg_type = *preq.add(2) as c_int;
                if msg_type > 1 {
                    m_log!((*input).name, c"unknown LMI report type 0x%x.\n".as_ptr(), msg_type);
                    return -1;
                }

                *pres.add(0) = 0x01;
                *pres.add(1) = 0x01;
                *pres.add(2) = msg_type as m_uint8_t;
                pres = pres.add(3);
            }

            0x03 => {
                // sequences
                if isize_ != 2 {
                    m_log!((*input).name, c"invalid LMI item size.\n".as_ptr());
                    return -1;
                }

                *pres.add(0) = 0x03;
                *pres.add(1) = 0x02;

                if (*input).fr_lmi_seq != *preq.add(3) {
                    m_log!((*input).name, c"resynchronization with LMI sequence...\n".as_ptr());
                    (*input).fr_lmi_seq = *preq.add(3);
                }

                (*input).fr_lmi_seq += 1;
                if 0 == (*input).fr_lmi_seq {
                    (*input).fr_lmi_seq += 1;
                }
                *pres.add(2) = (*input).fr_lmi_seq;
                *pres.add(3) = *preq.add(2);

                if DEBUG_FRSW != 0 {
                    m_log!((*input).name, c"iSSN=0x%x, iRSN=0x%x, oSSN=0x%x, oRSN=0x%x\n".as_ptr(), *preq.add(2), *preq.add(3), *pres.add(2), *pres.add(3));
                }
                pres = pres.add(4);
                seq_ok = TRUE;
            }

            _ => {
                m_log!((*input).name, c"unknown LMI item type %u\n".as_ptr(), itype);
                break 'not_done;
            }
        }

        // proceed next item
        preq = preq.add(isize_ as usize + 2);
    }

    // done:
    if (msg_type == -1) || 0 == seq_ok {
        m_log!((*input).name, c"incomplete LMI packet.\n".as_ptr());
        return -1;
    }

    // full status, send DLCI info
    if msg_type == 0 {
        if DEBUG_FRSW != 0 {
            m_log!((*input).name, c"LMI full status, advertising DLCIs\n".as_ptr());
        }
        sc = (*input).fr_conn_list.cast::<_>();
        while !sc.is_null() {
            dlci = (*sc).dlci_in;
            if DEBUG_FRSW != 0 {
                m_log!((*input).name, c"sending LMI adv for DLCI %u\n".as_ptr(), dlci);
            }
            *pres.add(0) = 0x07;
            *pres.add(1) = 0x03;
            *pres.add(2) = (dlci >> 4) as m_uint8_t;
            *pres.add(3) = 0x80 | (((dlci & 0x0f) << 3) as m_uint8_t);
            *pres.add(4) = 0x82;
            pres = pres.add(5);
            sc = (*sc).next;
        }
    }

    let rlen: ssize_t = pres.offset_from(resp.as_ptr());

    if DEBUG_FRSW != 0 {
        m_log!((*input).name, c"sending ANSI LMI packet:\n".as_ptr());
        mem_dump(log_file, resp.as_mut_ptr(), rlen as u_int);
    }

    netio_send(input, resp.as_mut_ptr().cast::<_>(), rlen as size_t);
    0
}

// DLCI switching
#[no_mangle]
pub unsafe extern "C" fn frsw_dlci_switch(vc: *mut frsw_conn_t, pkt: *mut m_uint8_t) {
    *pkt.add(0) = (*pkt.add(0) & 0x03) | (((*vc).dlci_out >> 4) << 2) as m_uint8_t;
    *pkt.add(1) = (*pkt.add(1) & 0x0f) | (((*vc).dlci_out & 0x0f) << 4) as m_uint8_t;

    // update the statistics counter
    (*vc).count += 1;
}

// Handle a Frame-Relay packet
#[no_mangle]
pub unsafe extern "C" fn frsw_handle_pkt(t: *mut frsw_table_t, input: *mut netio_desc_t, pkt: *mut m_uint8_t, len: ssize_t) -> ssize_t {
    let mut output: *mut netio_desc_t = null_mut();
    let mut dlci: m_uint32_t;

    // Extract DLCI information
    dlci = (((*pkt.add(0) & 0xfc) >> 2) << 4) as m_uint32_t;
    dlci |= ((*pkt.add(1) & 0xf0) >> 4) as m_uint32_t;

    if DEBUG_FRSW != 0 {
        m_log!((*input).name, c"Trying to switch packet with input DLCI %u.\n".as_ptr(), dlci);
        mem_dump(log_file, pkt, len as u_int);
    }

    // LMI ?
    if dlci == FR_DLCI_LMI_ANSI {
        return frsw_handle_lmi_ansi_pkt(t, input, pkt, len);
    }

    // DLCI switching
    let vc: *mut frsw_conn_t = frsw_dlci_lookup(t, input, dlci);
    if !vc.is_null() {
        frsw_dlci_switch(vc, pkt);
        output = (*vc).output;
    }

    if DEBUG_FRSW != 0 {
        if !output.is_null() {
            m_log!((*input).name, c"Switching packet to interface %s.\n".as_ptr(), (*output).name);
        } else {
            m_log!((*input).name, c"Unable to switch packet.\n".as_ptr());
        }
    }

    // Send the packet on output interface
    let slen: ssize_t = netio_send(output, pkt.cast::<_>(), len as size_t);

    if len != slen {
        (*t).drop += 1;
        return -1;
    }

    0
}

// Receive a Frame-Relay packet
unsafe extern "C" fn frsw_recv_pkt(nio: *mut netio_desc_t, pkt: *mut u_char, pkt_len: ssize_t, t: *mut c_void, _: *mut c_void) -> c_int {
    let t: *mut frsw_table_t = t.cast::<_>();
    FRSW_LOCK!(t);
    let res: c_int = frsw_handle_pkt(t, nio, pkt, pkt_len) as c_int;
    FRSW_UNLOCK!(t);
    res
}

// Acquire a reference to a Frame-Relay switch (increment reference count)
#[no_mangle]
pub unsafe extern "C" fn frsw_acquire(name: *mut c_char) -> *mut frsw_table_t {
    registry_find(name, OBJ_TYPE_FRSW).cast::<_>()
}

// Release a Frame-Relay switch (decrement reference count)
#[no_mangle]
pub unsafe extern "C" fn frsw_release(name: *mut c_char) -> c_int {
    registry_unref(name, OBJ_TYPE_FRSW)
}

// Create a virtual switch table
#[no_mangle]
pub unsafe extern "C" fn frsw_create_table(name: *mut c_char) -> *mut frsw_table_t {
    // Allocate a new switch structure
    let t: *mut frsw_table_t = libc::malloc(size_of::<frsw_table_t>()).cast::<_>();
    if t.is_null() {
        return null_mut();
    }

    libc::memset(t.cast::<_>(), 0, size_of::<frsw_table_t>());
    libc::pthread_mutex_init(addr_of_mut!((*t).lock), null_mut());
    mp_create_fixed_pool(addr_of_mut!((*t).mp), c"Frame-Relay Switch".as_ptr().cast_mut());

    (*t).name = mp_strdup(addr_of_mut!((*t).mp), name);
    if (*t).name.is_null() {
        mp_free_pool(addr_of_mut!((*t).mp));
        libc::free(t.cast::<_>());
        return null_mut();
    }

    // Record this object in registry
    if registry_add((*t).name, OBJ_TYPE_FRSW, t.cast::<_>()) == -1 {
        libc::fprintf(c_stderr(), c"frsw_create_table: unable to create switch '%s'\n".as_ptr(), name);
        mp_free_pool(addr_of_mut!((*t).mp));
        libc::free(t.cast::<_>());
        return null_mut();
    }

    t
}

// Unlink a VC
unsafe fn frsw_unlink_vc(vc: *mut frsw_conn_t) {
    if !vc.is_null() {
        if !(*vc).next.is_null() {
            (*(*vc).next).pprev = (*vc).pprev;
        }

        if !(*vc).pprev.is_null() {
            *((*vc).pprev) = (*vc).next;
        }
    }
}

// Free resources used by a VC
unsafe fn frsw_release_vc(vc: *mut frsw_conn_t) {
    if !vc.is_null() {
        // release input NIO
        if !(*vc).input.is_null() {
            netio_rxl_remove((*vc).input);
            netio_release((*(*vc).input).name);
        }

        // release output NIO
        if !(*vc).output.is_null() {
            netio_release((*(*vc).output).name);
        }
    }
}

// Free resources used by a Frame-Relay switch
unsafe extern "C" fn frsw_free(data: *mut c_void, _arg: *mut c_void) -> c_int {
    let t: *mut frsw_table_t = data.cast::<_>();
    let mut vc: *mut frsw_conn_t;

    for i in 0..FRSW_HASH_SIZE as c_int {
        vc = (*t).dlci_table[i as usize];
        while !vc.is_null() {
            frsw_release_vc(vc);
            vc = (*vc).hash_next;
        }
    }

    mp_free_pool(addr_of_mut!((*t).mp));
    libc::free(t.cast::<_>());
    TRUE
}

// Delete a Frame-Relay switch
#[no_mangle]
pub unsafe extern "C" fn frsw_delete(name: *mut c_char) -> c_int {
    registry_delete_if_unused(name, OBJ_TYPE_FRSW, Some(frsw_free), null_mut())
}

// Delete all Frame-Relay switches
#[no_mangle]
pub unsafe extern "C" fn frsw_delete_all() -> c_int {
    registry_delete_type(OBJ_TYPE_FRSW, Some(frsw_free), null_mut())
}

// Create a switch connection
#[no_mangle]
pub unsafe extern "C" fn frsw_create_vc(t: *mut frsw_table_t, nio_input: *mut c_char, dlci_in: u_int, nio_output: *mut c_char, dlci_out: u_int) -> c_int {
    let mut p: *mut *mut frsw_conn_t;

    FRSW_LOCK!(t);

    // Allocate a new VC
    let vc: *mut frsw_conn_t = mp_alloc(addr_of_mut!((*t).mp), size_of::<frsw_table_t>()).cast::<_>();
    if vc.is_null() {
        FRSW_UNLOCK!(t);
        return -1;
    }

    (*vc).input = netio_acquire(nio_input);
    (*vc).output = netio_acquire(nio_output);
    (*vc).dlci_in = dlci_in;
    (*vc).dlci_out = dlci_out;

    // Check these NIOs are valid and the input VC does not exists
    if (*vc).input.is_null() || (*vc).output.is_null() {
        FRSW_UNLOCK!(t);
        frsw_release_vc(vc);
        mp_free(vc.cast::<_>());
        return -1;
    }

    if !frsw_dlci_lookup(t, (*vc).input, dlci_in).is_null() {
        libc::fprintf(c_stderr(), c"FRSW %s: switching for VC %u on IF %s already defined.\n".as_ptr(), (*t).name, dlci_in, (*(*vc).input).name);
        FRSW_UNLOCK!(t);
        frsw_release_vc(vc);
        mp_free(vc.cast::<_>());
        return -1;
    }

    // Add as a RX listener
    if netio_rxl_add((*vc).input, Some(frsw_recv_pkt), t.cast::<_>(), null_mut()) == -1 {
        FRSW_UNLOCK!(t);
        frsw_release_vc(vc);
        mp_free(vc.cast::<_>());
        return -1;
    }

    let hbucket: u_int = frsw_dlci_hash(dlci_in);
    (*vc).hash_next = (*t).dlci_table[hbucket as usize];
    (*t).dlci_table[hbucket as usize] = vc;

    p = addr_of_mut!((*(*vc).input).fr_conn_list).cast::<*mut frsw_conn_t>();
    while !p.is_null() {
        if (*(*p)).dlci_in > dlci_in {
            break;
        }
        p = addr_of_mut!((*(*p)).next);
    }

    (*vc).next = *p;
    if !(*p).is_null() {
        (*(*p)).pprev = addr_of_mut!((*vc).next);
    }
    (*vc).pprev = p;
    *p = vc;

    FRSW_UNLOCK!(t);
    0
}

// Remove a switch connection
#[no_mangle]
pub unsafe extern "C" fn frsw_delete_vc(t: *mut frsw_table_t, nio_input: *mut c_char, dlci_in: u_int, nio_output: *mut c_char, dlci_out: u_int) -> c_int {
    let mut vc: *mut *mut frsw_conn_t;
    let mut p: *mut frsw_conn_t;

    FRSW_LOCK!(t);

    let input: *mut netio_desc_t = registry_exists(nio_input, OBJ_TYPE_NIO).cast::<_>();
    let output: *mut netio_desc_t = registry_exists(nio_output, OBJ_TYPE_NIO).cast::<_>();

    if input.is_null() || output.is_null() {
        FRSW_UNLOCK!(t);
        return -1;
    }

    let hbucket: u_int = frsw_dlci_hash(dlci_in);
    vc = addr_of_mut!((*t).dlci_table[hbucket as usize]);
    while !vc.is_null() {
        p = *vc;

        if ((*p).input == input) && ((*p).output == output) && ((*p).dlci_in == dlci_in) && ((*p).dlci_out == dlci_out) {
            // Found a matching VC, remove it
            *vc = (*(*vc)).hash_next;
            frsw_unlink_vc(p);
            FRSW_UNLOCK!(t);

            // Release NIOs
            frsw_release_vc(p);
            mp_free(p.cast::<_>());
            return 0;
        }
        vc = addr_of_mut!((*(*vc)).hash_next);
    }

    FRSW_UNLOCK!(t);
    -1
}

// Save the configuration of a Frame-Relay switch
#[no_mangle]
pub unsafe extern "C" fn frsw_save_config(t: *mut frsw_table_t, fd: *mut libc::FILE) {
    let mut vc: *mut frsw_conn_t;

    libc::fprintf(fd, c"frsw create %s\n".as_ptr(), (*t).name);

    FRSW_LOCK!(t);

    for i in 0..FRSW_HASH_SIZE as c_int {
        vc = (*t).dlci_table[i as usize];
        while !vc.is_null() {
            libc::fprintf(fd, c"frsw create_vc %s %s %u %s %u\n".as_ptr(), (*t).name, (*(*vc).input).name, (*vc).dlci_in, (*(*vc).output).name, (*vc).dlci_out);
            vc = (*vc).next;
        }
    }

    FRSW_UNLOCK!(t);

    libc::fprintf(fd, c"\n".as_ptr());
}

// Save configurations of all Frame-Relay switches
unsafe extern "C" fn frsw_reg_save_config(entry: *mut registry_entry_t, opt: *mut c_void, _err: *mut c_int) {
    frsw_save_config((*entry).data.cast::<frsw_table_t>(), opt.cast::<libc::FILE>());
}

#[no_mangle]
pub unsafe extern "C" fn frsw_save_config_all(fd: *mut libc::FILE) {
    registry_foreach_type(OBJ_TYPE_FRSW, Some(frsw_reg_save_config), fd.cast::<_>(), null_mut());
}

// Create a new interface
#[no_mangle]
pub unsafe extern "C" fn frsw_cfg_create_if(_t: *mut frsw_table_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    let mut nio: *mut netio_desc_t = null_mut();

    // at least: IF, interface name, NetIO type
    if count < 3 {
        libc::fprintf(c_stderr(), c"frsw_cfg_create_if: invalid interface description\n".as_ptr());
        return -1;
    }

    let nio_type: c_int = netio_get_type(*tokens.add(2));
    match nio_type as u_int {
        NETIO_TYPE_UNIX => 'block: {
            if count != 5 {
                libc::fprintf(c_stderr(), c"FRSW: invalid number of arguments for UNIX NIO '%s'\n".as_ptr(), *tokens.add(1));
                break 'block;
            }

            nio = netio_desc_create_unix(*tokens.add(1), *tokens.add(3), *tokens.add(4));
        }

        NETIO_TYPE_UDP => 'block: {
            if count != 6 {
                libc::fprintf(c_stderr(), c"FRSW: invalid number of arguments for UDP NIO '%s'\n".as_ptr(), *tokens.add(1));
                break 'block;
            }

            nio = netio_desc_create_udp(*tokens.add(1), libc::atoi(*tokens.add(3)), *tokens.add(4), libc::atoi(*tokens.add(5)));
        }

        NETIO_TYPE_TCP_CLI => 'block: {
            if count != 5 {
                libc::fprintf(c_stderr(), c"FRSW: invalid number of arguments for TCP CLI NIO '%s'\n".as_ptr(), *tokens.add(1));
                break 'block;
            }

            nio = netio_desc_create_tcp_cli(*tokens.add(1), *tokens.add(3), *tokens.add(4));
        }

        NETIO_TYPE_TCP_SER => 'block: {
            if count != 4 {
                libc::fprintf(c_stderr(), c"FRSW: invalid number of arguments for TCP SER NIO '%s'\n".as_ptr(), *tokens.add(1));
                break 'block;
            }

            nio = netio_desc_create_tcp_ser(*tokens.add(1), *tokens.add(3));
        }

        _ => {
            libc::fprintf(c_stderr(), c"FRSW: unknown/invalid NETIO type '%s'\n".as_ptr(), *tokens.add(2));
        }
    }

    if nio.is_null() {
        libc::fprintf(c_stderr(), c"FRSW: unable to create NETIO descriptor of interface %s\n".as_ptr(), *tokens.add(1));
        return -1;
    }

    netio_release((*nio).name);
    0
}

// Create a new virtual circuit
#[no_mangle]
pub unsafe extern "C" fn frsw_cfg_create_vc(t: *mut frsw_table_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    // 5 parameters: "VC", InputIF, InDLCI, OutputIF, OutDLCI
    if count != 5 {
        libc::fprintf(c_stderr(), c"FRSW: invalid VPC descriptor.\n".as_ptr());
        return -1;
    }

    frsw_create_vc(t, *tokens.add(1), libc::atoi(*tokens.add(2)) as u_int, *tokens.add(3), libc::atoi(*tokens.add(4)) as u_int)
}

const FRSW_MAX_TOKENS: usize = 16;

// Handle a FRSW configuration line
#[no_mangle]
pub unsafe extern "C" fn frsw_handle_cfg_line(t: *mut frsw_table_t, str_: *mut c_char) -> c_int {
    let mut tokens: [*mut c_char; FRSW_MAX_TOKENS] = [null_mut(); FRSW_MAX_TOKENS];

    let count: c_int = m_strsplit(str_, b':' as c_char, tokens.as_mut_ptr(), FRSW_MAX_TOKENS as c_int);
    if count <= 1 {
        return -1;
    }

    if 0 == libc::strcmp(tokens[0], c"IF".as_ptr()) {
        return frsw_cfg_create_if(t, tokens.as_mut_ptr(), count);
    } else if 0 == libc::strcmp(tokens[0], c"VC".as_ptr()) {
        return frsw_cfg_create_vc(t, tokens.as_mut_ptr(), count);
    }

    libc::fprintf(c_stderr(), c"FRSW: Unknown statement \"%s\" (allowed: IF,VC)\n".as_ptr(), tokens[0]);
    -1
}

// Read a FRSW configuration file
#[no_mangle]
pub unsafe extern "C" fn frsw_read_cfg_file(t: *mut frsw_table_t, filename: *mut c_char) -> c_int {
    let mut buffer: [c_char; 1024] = [0; 1024];
    let mut ptr: *mut c_char;

    let fd: *mut libc::FILE = libc::fopen(filename, c"r".as_ptr());
    if fd.is_null() {
        libc::perror(c"fopen".as_ptr());
        return -1;
    }

    while 0 == libc::feof(fd) {
        if !libc::fgets(buffer.as_mut_ptr(), buffer.len() as c_int, fd).is_null() {
            break;
        }

        // skip comments and end of line
        ptr = libc::strpbrk(buffer.as_ptr(), c"#\r\n".as_ptr());
        if !ptr.is_null() {
            *ptr = 0;
        }

        // analyze non-empty lines
        if !libc::strchr(buffer.as_ptr(), b':' as c_int).is_null() {
            frsw_handle_cfg_line(t, buffer.as_mut_ptr());
        }
    }

    libc::fclose(fd);
    0
}

// Start a virtual Frame-Relay switch
#[no_mangle]
pub unsafe extern "C" fn frsw_start(filename: *mut c_char) -> c_int {
    let t: *mut frsw_table_t = frsw_create_table(c"default".as_ptr().cast_mut());
    if t.is_null() {
        libc::fprintf(c_stderr(), c"FRSW: unable to create virtual fabric table.\n".as_ptr());
        return -1;
    }

    if frsw_read_cfg_file(t, filename) == -1 {
        libc::fprintf(c_stderr(), c"FRSW: unable to parse configuration file.\n".as_ptr());
        return -1;
    }

    frsw_release(c"default".as_ptr().cast_mut());
    0
}
