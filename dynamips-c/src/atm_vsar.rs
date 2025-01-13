//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! ATM Virtual Segmentation & Reassembly Engine.

use crate::_extra::*;
use crate::atm::*;
use crate::crc::*;
use crate::dynamips_common::*;
use crate::net_io::*;
use crate::utils::*;
use libc::size_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;
use std::mem::zeroed;
use std::ptr::addr_of;
use std::ptr::addr_of_mut;

pub const ATM_REAS_MAX_SIZE: usize = 16384;

// Reassembly Context
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct atm_reas_context {
    pub buffer: [m_uint8_t; ATM_REAS_MAX_SIZE],
    pub buf_pos: size_t,
    pub len: size_t,
}

// Segmentation Context
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct atm_seg_context {
    pub nio: *mut netio_desc_t,
    pub txfifo_cell: [m_uint8_t; ATM_CELL_SIZE],
    pub txfifo_pos: size_t,
    pub txfifo_avail: size_t,
    pub aal5_len: size_t,
    pub aal5_crc: m_uint32_t,
    pub atm_hdr: m_uint32_t,
    pub buffer: *mut c_char,
    pub buf_len: size_t,
}

// Send the ATM cell in FIFO
unsafe fn atm_send_cell(asc: *mut atm_seg_context) {
    m_hton32((*asc).txfifo_cell.as_mut_ptr(), (*asc).atm_hdr);
    atm_insert_hec((*asc).txfifo_cell.as_mut_ptr());
    netio_send((*asc).nio, (*asc).txfifo_cell.as_mut_ptr().cast::<_>(), ATM_CELL_SIZE);
}

// Clear the TX fifo
unsafe fn atm_clear_tx_fifo(asc: *mut atm_seg_context) {
    (*asc).txfifo_avail = ATM_PAYLOAD_SIZE;
    (*asc).txfifo_pos = ATM_HDR_SIZE;
    libc::memset((*asc).txfifo_cell.as_mut_ptr().cast::<_>(), 0, ATM_CELL_SIZE);
}

// Add padding to the FIFO
unsafe fn atm_add_tx_padding(asc: *mut atm_seg_context, mut len: size_t) {
    if len > (*asc).txfifo_avail {
        len = (*asc).txfifo_avail;
    }

    libc::memset(addr_of_mut!((*asc).txfifo_cell[(*asc).txfifo_pos]).cast::<_>(), 0, len);
    (*asc).txfifo_pos += len;
    (*asc).txfifo_avail -= len;
}

// Send the TX fifo if it is empty
unsafe fn atm_send_fifo(asc: *mut atm_seg_context) {
    if 0 == (*asc).txfifo_avail {
        (*asc).aal5_crc = crc32_compute(!(*asc).aal5_crc, addr_of_mut!((*asc).txfifo_cell[ATM_HDR_SIZE]), ATM_PAYLOAD_SIZE as c_int);
        atm_send_cell(asc);
        atm_clear_tx_fifo(asc);
    }
}

// Store a packet in the TX FIFO
unsafe fn atm_store_fifo(asc: *mut atm_seg_context) -> c_int {
    let len: size_t = m_min!((*asc).buf_len, (*asc).txfifo_avail);

    libc::memcpy(addr_of_mut!((*asc).txfifo_cell[(*asc).txfifo_pos]).cast::<_>(), (*asc).buffer.cast::<_>(), len);
    (*asc).buffer = (*asc).buffer.add(len);
    (*asc).buf_len -= len;
    (*asc).txfifo_pos += len;
    (*asc).txfifo_avail -= len;

    if 0 == (*asc).txfifo_avail {
        atm_send_fifo(asc);
        return TRUE;
    }

    FALSE
}

// Add the AAL5 trailer to the TX FIFO
unsafe fn atm_aal5_add_trailer(asc: *mut atm_seg_context) {
    let trailer: *mut m_uint8_t = addr_of_mut!((*asc).txfifo_cell[ATM_AAL5_TRAILER_POS]);

    // Control field + Length
    m_hton32(trailer, (*asc).aal5_len as m_uint32_t);

    // Final CRC-32 computation
    (*asc).aal5_crc = crc32_compute(!(*asc).aal5_crc, addr_of_mut!((*asc).txfifo_cell[ATM_HDR_SIZE]), ATM_PAYLOAD_SIZE as c_int - 4);

    m_hton32(trailer.add(4), (*asc).aal5_crc);

    // Consider the FIFO as full
    (*asc).txfifo_avail = 0;
}

// Send an AAL5 packet through an NIO (segmentation)
#[no_mangle]
pub unsafe extern "C" fn atm_aal5_send(nio: *mut netio_desc_t, vpi: u_int, vci: u_int, iov: *mut libc::iovec, iovcnt: c_int) -> c_int {
    let mut asc: atm_seg_context = zeroed();

    asc.nio = nio;
    asc.aal5_len = 0;
    asc.aal5_crc = 0; // will be inverted by first CRC update
    atm_clear_tx_fifo(addr_of_mut!(asc));

    // prepare the atm header
    asc.atm_hdr = vpi << ATM_HDR_VPI_SHIFT;
    asc.atm_hdr |= vci << ATM_HDR_VCI_SHIFT;

    for i in 0..iovcnt {
        asc.buffer = (*iov.add(i as usize)).iov_base.cast::<_>();
        asc.buf_len = (*iov.add(i as usize)).iov_len;
        asc.aal5_len += (*iov.add(i as usize)).iov_len;

        while asc.buf_len > 0 {
            atm_store_fifo(addr_of_mut!(asc));
        }
    }

    // Add the PDU trailer. If we have enough room, add it in the last cell,
    // otherwise create a new one.
    if asc.txfifo_avail < ATM_AAL5_TRAILER_SIZE {
        atm_add_tx_padding(addr_of_mut!(asc), asc.txfifo_avail);
        atm_send_fifo(addr_of_mut!(asc));
    }

    // Set AAL5 end of packet in ATM header (PTI field)
    asc.atm_hdr |= ATM_PTI_EOP;

    atm_add_tx_padding(addr_of_mut!(asc), asc.txfifo_avail - ATM_AAL5_TRAILER_SIZE);
    atm_aal5_add_trailer(addr_of_mut!(asc));
    atm_send_cell(addr_of_mut!(asc));
    0
}

// Reset a receive context
#[no_mangle]
pub unsafe extern "C" fn atm_aal5_recv_reset(arc: *mut atm_reas_context) {
    (*arc).buf_pos = 0;
    (*arc).len = 0;
}

// Receive an ATM cell and process reassembly
#[no_mangle]
pub unsafe extern "C" fn atm_aal5_recv(arc: *mut atm_reas_context, cell: *mut m_uint8_t) -> c_int {
    // Check buffer boundary
    if ((*arc).buf_pos + ATM_PAYLOAD_SIZE) > ATM_REAS_MAX_SIZE {
        atm_aal5_recv_reset(arc);
        return -1;
    }

    // Get the PTI field: we cannot handle "network" traffic
    let atm_hdr: m_uint32_t = m_ntoh32(cell);

    if (atm_hdr & ATM_PTI_NETWORK) != 0 {
        return 2;
    }

    // Copy the payload
    libc::memcpy(addr_of_mut!((*arc).buffer[(*arc).buf_pos]).cast::<_>(), addr_of!(*cell.add(ATM_HDR_SIZE)).cast::<_>(), ATM_PAYLOAD_SIZE);
    (*arc).buf_pos += ATM_PAYLOAD_SIZE;

    // If this is the last cell of the packet, get the real length (the
    // trailer is at the end).
    if (atm_hdr & ATM_PTI_EOP) != 0 {
        (*arc).len = m_ntoh16(addr_of_mut!(*cell.add(ATM_AAL5_TRAILER_POS + 2))) as size_t;
        return if (*arc).len <= (*arc).buf_pos { 1 } else { -2 };
    }

    0
}

// Send a packet through a rfc1483 bridge encap
#[no_mangle]
pub unsafe extern "C" fn atm_aal5_send_rfc1483b(nio: *mut netio_desc_t, vpi: u_int, vci: u_int, pkt: *mut c_void, len: size_t) -> c_int {
    let mut vec: [libc::iovec; 2] = [zeroed(); 2];

    #[allow(static_mut_refs)]
    {
        vec[0].iov_base = atm_rfc1483b_header.as_mut_ptr().cast::<_>();
    }
    vec[0].iov_len = ATM_RFC1483B_HLEN;
    vec[1].iov_base = pkt;
    vec[1].iov_len = len;

    atm_aal5_send(nio, vpi, vci, vec.as_mut_ptr(), 2)
}
