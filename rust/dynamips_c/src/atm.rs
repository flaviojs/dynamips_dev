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
