//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//!
//! Virtual Ethernet switch definitions.
//! Virtual Ethernet switch with VLAN/Trunk support.

use crate::_extra::*;
use crate::dynamips_common::*;
use crate::net::*;
use crate::net_io::*;
use crate::registry::*;
use crate::utils::*;
use libc::size_t;
use libc::ssize_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_uint;
use std::ffi::c_void;
use std::mem::transmute;
use std::mem::zeroed;
use std::ptr::addr_of;
use std::ptr::addr_of_mut;
use std::ptr::null_mut;

// Hash entries for the MAC address table
pub const ETHSW_HASH_SIZE: usize = 4096;

// Maximum port number
pub const ETHSW_MAX_NIO: usize = 64;

// Maximum packet size
pub const ETHSW_MAX_PKT_SIZE: usize = 2048;

// Port types: access, 802.1Q, 802.1Q tunnel (QinQ)
pub type _ETHSW_PORT_TYPE_ENUM = u_int; // TODO enum
pub const ETHSW_PORT_TYPE_ACCESS: _ETHSW_PORT_TYPE_ENUM = 1;
pub const ETHSW_PORT_TYPE_DOT1Q: _ETHSW_PORT_TYPE_ENUM = 2;
pub const ETHSW_PORT_TYPE_QINQ: _ETHSW_PORT_TYPE_ENUM = 3;
pub const ETHSW_PORT_TYPE_ISL: _ETHSW_PORT_TYPE_ENUM = 4;

// Received packet
pub type ethsw_packet_t = ethsw_packet;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ethsw_packet {
    pub pkt: *mut u_char,
    pub pkt_len: ssize_t,
    pub input_port: *mut netio_desc_t,
    pub input_vlan: u_int,
    pub input_tag: c_int,
}

// MAC address table entry
pub type ethsw_mac_entry_t = ethsw_mac_entry;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ethsw_mac_entry {
    pub nio: *mut netio_desc_t,
    pub mac_addr: n_eth_addr_t,
    pub vlan_id: m_uint16_t,
}

// Virtual Ethernet switch
pub type ethsw_table_t = ethsw_table;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ethsw_table {
    pub name: *mut c_char,
    pub lock: libc::pthread_mutex_t,
    pub debug: c_int,

    // Virtual Ports
    pub nio: [*mut netio_desc_t; ETHSW_MAX_NIO],

    // MAC address table
    pub mac_addr_table: [ethsw_mac_entry_t; ETHSW_HASH_SIZE],
}

// Packet input vector
pub type ethsw_input_vector_t = Option<unsafe extern "C" fn(t: *mut ethsw_table_t, sp: *mut ethsw_packet_t, output_port: *mut netio_desc_t)>;

// "foreach" vector
pub type ethsw_foreach_entry_t = Option<unsafe extern "C" fn(t: *mut ethsw_table_t, entry: *mut ethsw_mac_entry_t, opt: *mut c_void)>;

macro_rules! ETHSW_LOCK {
    ($t:expr) => {
        libc::pthread_mutex_lock(addr_of_mut!((*$t).lock))
    };
}
macro_rules! ETHSW_UNLOCK {
    ($t:expr) => {
        libc::pthread_mutex_unlock(addr_of_mut!((*$t).lock))
    };
}
macro_rules! ETHSW_TRYLOCK {
    ($t:expr) => {
        libc::pthread_mutex_trylock(addr_of_mut!((*$t).lock))
    };
}

// Debbuging message
macro_rules! ethsw_debug {
    ($t:expr, $fmt:expr$(, $arg:expr)*) => {{
        let t: *mut ethsw_table_t = $t;
        let fmt: *const c_char = $fmt;
        let args: &[&dyn sprintf::Printf] = &[$(crate::_extra::Printf($arg)),*];
        let mut module: [c_char; 128] = [0; 128];

        if (*t).debug != 0 {
            libc::snprintf(module.as_mut_ptr(), module.len(), c"ETHSW %s".as_ptr(), (*t).name);
            m_flog(log_file, module.as_ptr(), fmt, args);
        }
    }};
}

// Compute hash index on the specified MAC address and VLAN
#[inline]
unsafe fn ethsw_hash_index(addr: *mut n_eth_addr_t, vlan_id: u_int) -> u_int {
    let mut h_index: u_int;

    h_index = (((*addr).eth_addr_byte[0] as u_int) << 8) | (*addr).eth_addr_byte[1] as u_int;
    h_index ^= (((*addr).eth_addr_byte[2] as u_int) << 8) | (*addr).eth_addr_byte[3] as u_int;
    h_index ^= (((*addr).eth_addr_byte[4] as u_int) << 8) | (*addr).eth_addr_byte[5] as u_int;
    h_index ^= vlan_id;
    h_index & (ETHSW_HASH_SIZE as u_int - 1)
}

// Invalidate the whole MAC address table
unsafe fn ethsw_invalidate(t: *mut ethsw_table_t) {
    libc::memset((*t).mac_addr_table.as_mut_ptr().cast::<_>(), 0, size_of_val(&(*t).mac_addr_table));
}

// Invalidate entry of the MAC address table referring to the specified NIO
unsafe fn ethsw_invalidate_port(t: *mut ethsw_table_t, nio: *mut netio_desc_t) {
    let mut entry: *mut ethsw_mac_entry_t;

    for i in 0..ETHSW_HASH_SIZE as c_int {
        entry = addr_of_mut!((*t).mac_addr_table[i as usize]);
        if (*entry).nio == nio {
            (*entry).nio = null_mut();
            (*entry).vlan_id = 0;
        }
    }
}

// Push a 802.1Q tag
unsafe fn dot1q_push_tag(pkt: *mut m_uint8_t, sp: *mut ethsw_packet_t, _vlan: u_int, ethertype: m_uint16_t) {
    libc::memcpy(pkt.cast::<_>(), (*sp).pkt.cast::<_>(), N_ETH_HLEN - 2);

    let hdr: *mut n_eth_dot1q_hdr_t = pkt.cast::<_>();
    (*hdr).r#type = libc::htons(ethertype);
    (*hdr).vlan_id = libc::htons((*sp).input_vlan as m_uint16_t);

    libc::memcpy(pkt.add(size_of::<n_eth_dot1q_hdr_t>()).cast::<_>(), (*sp).pkt.add(N_ETH_HLEN - 2).cast::<_>(), (*sp).pkt_len as size_t - (N_ETH_HLEN - 2));
}

// Pop a 802.1Q tag
unsafe fn dot1q_pop_tag(pkt: *mut m_uint8_t, sp: *mut ethsw_packet_t) {
    libc::memcpy(pkt.cast::<_>(), (*sp).pkt.cast::<_>(), N_ETH_HLEN - 2);

    libc::memcpy(pkt.add(N_ETH_HLEN - 2).cast::<_>(), (*sp).pkt.add(size_of::<n_eth_dot1q_hdr_t>()).cast::<_>(), (*sp).pkt_len as size_t - size_of::<n_eth_dot1q_hdr_t>());
}

// Input vector for ACCESS ports
unsafe extern "C" fn ethsw_iv_access(_t: *mut ethsw_table_t, sp: *mut ethsw_packet_t, op: *mut netio_desc_t) {
    let pkt: *mut m_uint8_t;

    match (*op).vlan_port_type {
        // Access -> Access: no special treatment
        ETHSW_PORT_TYPE_ACCESS => {
            netio_send(op, (*sp).pkt.cast::<_>(), (*sp).pkt_len as size_t);
        }

        // Access -> 802.1Q: push tag
        ETHSW_PORT_TYPE_DOT1Q => 'block: {
            // If the native VLAN of output port is the same as input,
            // forward the packet without adding the tag.
            if (*op).vlan_id as u_int == (*sp).input_vlan {
                netio_send(op, (*sp).pkt.cast::<_>(), (*sp).pkt_len as size_t);
            } else {
                pkt = libc::malloc((*sp).pkt_len as size_t + 4).cast::<_>();
                if pkt.is_null() {
                    libc::perror(c"ethsw_iv_access: dot1q".as_ptr());
                    break 'block;
                }
                libc::memset(pkt.cast::<_>(), 0, (*sp).pkt_len as size_t + 4);
                dot1q_push_tag(pkt, sp, (*op).vlan_id as u_int, (*(*sp).input_port).ethertype);
                netio_send(op, pkt.cast::<_>(), (*sp).pkt_len as size_t + 4);
                libc::free(pkt.cast::<_>());
            }
        }

        _ => {
            libc::fprintf(c_stderr(), c"ethsw_iv_access: unknown port type %u\n".as_ptr(), (*op).vlan_port_type);
        }
    }
}

// Input vector for 802.1Q ports
unsafe extern "C" fn ethsw_iv_dot1q(t: *mut ethsw_table_t, sp: *mut ethsw_packet_t, op: *mut netio_desc_t) {
    let pkt: *mut m_uint8_t;

    // If we don't have an input tag, we work temporarily as an access port
    if 0 == (*sp).input_tag {
        ethsw_iv_access(t, sp, op);
        return;
    }

    match (*op).vlan_port_type {
        // 802.1Q -> Access: pop tag
        ETHSW_PORT_TYPE_ACCESS => 'block: {
            pkt = libc::malloc((*sp).pkt_len as size_t - 4).cast::<_>();
            if pkt.is_null() {
                libc::perror(c"ethsw_iv_dot1q: access".as_ptr());
                break 'block;
            }
            libc::memset(pkt.cast::<_>(), 0, (*sp).pkt_len as size_t - 4);
            dot1q_pop_tag(pkt, sp);
            netio_send(op, pkt.cast::<_>(), (*sp).pkt_len as size_t - 4);
            libc::free(pkt.cast::<_>());
        }

        // 802.1Q -> 802.1Q: pop tag if native VLAN in output otherwise no-op
        ETHSW_PORT_TYPE_DOT1Q => 'block: {
            if (*op).vlan_id as u_int == (*sp).input_vlan {
                pkt = libc::malloc((*sp).pkt_len as size_t - 4).cast::<_>();
                if pkt.is_null() {
                    libc::perror(c"ethsw_iv_dot1q: dot1q".as_ptr());
                    break 'block;
                }
                libc::memset(pkt.cast::<_>(), 0, (*sp).pkt_len as size_t - 4);
                dot1q_pop_tag(pkt, sp);
                netio_send(op, pkt.cast::<_>(), (*sp).pkt_len as size_t - 4);
                libc::free(pkt.cast::<_>());
            } else {
                netio_send(op, (*sp).pkt.cast::<_>(), (*sp).pkt_len as size_t);
            }
        }

        // 802.1Q -> QinQ: pop outer tag if native VLAN in the one specified
        // tunnel port.
        ETHSW_PORT_TYPE_QINQ => 'block: {
            if (*op).vlan_id as u_int == (*sp).input_vlan {
                pkt = libc::malloc((*sp).pkt_len as size_t - 4).cast::<_>();
                if pkt.is_null() {
                    libc::perror(c"ethsw_iv_dot1q: qinq".as_ptr());
                    break 'block;
                }
                libc::memset(pkt.cast::<_>(), 0, (*sp).pkt_len as size_t - 4);
                dot1q_pop_tag(pkt, sp);
                netio_send(op, pkt.cast::<_>(), (*sp).pkt_len as size_t - 4);
                libc::free(pkt.cast::<_>());
            }
        }

        _ => {
            libc::fprintf(c_stderr(), c"ethsw_iv_dot1q: unknown port type %u\n".as_ptr(), (*op).vlan_port_type);
        }
    }
}

// Input vector for QinQ ports
unsafe extern "C" fn ethsw_iv_qinq(_t: *mut ethsw_table_t, sp: *mut ethsw_packet_t, op: *mut netio_desc_t) {
    let pkt: *mut m_uint8_t;

    match (*op).vlan_port_type {
        // QinQ -> 802.1Q: push outer tag
        ETHSW_PORT_TYPE_DOT1Q => 'block: {
            pkt = libc::malloc((*sp).pkt_len as size_t + 4).cast::<_>();
            if pkt.is_null() {
                libc::perror(c"ethsw_iv_qinq: dot1q".as_ptr());
                break 'block;
            }
            libc::memset(pkt.cast::<_>(), 0, (*sp).pkt_len as size_t + 4);
            dot1q_push_tag(pkt, sp, (*(*sp).input_port).vlan_id as u_int, (*(*sp).input_port).ethertype);
            netio_send(op, pkt.cast::<_>(), (*sp).pkt_len as size_t + 4);
            libc::free(pkt.cast::<_>());
        }

        // QinQ -> QinQ: valid situation if we have the same customer connected
        // on two ports (so with identical VLAN id on tunnel ports).
        ETHSW_PORT_TYPE_QINQ => 'block: {
            if (*(*sp).input_port).vlan_id == (*op).vlan_id {
                pkt = libc::malloc((*sp).pkt_len as size_t - 4).cast::<_>();
                if pkt.is_null() {
                    libc::perror(c"ethsw_iv_qinq: qinq".as_ptr());
                    break 'block;
                }
                libc::memset(pkt.cast::<_>(), 0, (*sp).pkt_len as size_t - 4);
                dot1q_pop_tag(pkt, sp);
                netio_send(op, pkt.cast::<_>(), (*sp).pkt_len as size_t - 4);
                libc::free(pkt.cast::<_>());
            }
        }

        _ => {
            libc::fprintf(c_stderr(), c"ethsw_iv_dot1q: unknown port type %u\n".as_ptr(), (*op).vlan_port_type);
        }
    }
}

// Flood a packet
unsafe fn ethsw_flood(t: *mut ethsw_table_t, sp: *mut ethsw_packet_t) {
    let mut op: *mut netio_desc_t;

    let input_vector: ethsw_input_vector_t = transmute::<*mut c_void, ethsw_input_vector_t>((*(*sp).input_port).vlan_input_vector);
    assert!(input_vector.is_some());

    for i in 0..ETHSW_MAX_NIO as c_int {
        op = (*t).nio[i as usize];

        if op.is_null() || (op == (*sp).input_port) {
            continue;
        }

        // skip output port configured in access mode with a different vlan
        if ((*op).vlan_port_type == ETHSW_PORT_TYPE_ACCESS) && ((*op).vlan_id as u_int != (*sp).input_vlan) {
            continue;
        }

        // send the packet to the output port
        input_vector.unwrap()(t, sp, op);
    }
}

// Forward a packet
unsafe fn ethsw_forward(t: *mut ethsw_table_t, sp: *mut ethsw_packet_t) {
    let hdr: *mut n_eth_hdr_t = (*sp).pkt.cast::<n_eth_hdr_t>();
    let input_vector: ethsw_input_vector_t;
    let mut entry: *mut ethsw_mac_entry_t;
    let mut h_index: u_int;

    // Learn the source MAC address
    h_index = ethsw_hash_index(addr_of_mut!((*hdr).saddr), (*sp).input_vlan);
    entry = addr_of_mut!((*t).mac_addr_table[h_index as usize]);

    (*entry).nio = (*sp).input_port;
    (*entry).vlan_id = (*sp).input_vlan as m_uint16_t;
    (*entry).mac_addr = (*hdr).saddr;

    // If we have a broadcast/multicast packet, flood it
    if eth_addr_is_mcast(addr_of_mut!((*hdr).daddr)) != 0 {
        ethsw_debug!(t, c"multicast dest, flooding packet.\n".as_ptr());
        ethsw_flood(t, sp);
        return;
    }

    // Lookup on the destination MAC address (unicast)
    h_index = ethsw_hash_index(addr_of_mut!((*hdr).daddr), (*sp).input_vlan);
    entry = addr_of_mut!((*t).mac_addr_table[h_index as usize]);

    // If the dest MAC is unknown, flood the packet
    if libc::memcmp(addr_of!((*entry).mac_addr).cast::<_>(), addr_of!((*hdr).daddr).cast::<_>(), N_ETH_ALEN) != 0 || ((*entry).vlan_id as u_int != (*sp).input_vlan) {
        ethsw_debug!(t, c"unknown dest, flooding packet.\n".as_ptr());
        ethsw_flood(t, sp);
        return;
    }

    // Forward the packet to the output port only
    if (*entry).nio != (*sp).input_port {
        input_vector = transmute::<*mut libc::c_void, ethsw_input_vector_t>((*(*sp).input_port).vlan_input_vector);
        assert!(input_vector.is_some());
        input_vector.unwrap()(t, sp, (*entry).nio);
    } else {
        ethsw_debug!(t, c"source and dest ports identical, dropping.\n".as_ptr());
    }
}

// Receive a packet and prepare its forwarding
#[inline]
unsafe fn ethsw_receive(t: *mut ethsw_table_t, nio: *mut netio_desc_t, pkt: *mut u_char, pkt_len: ssize_t) -> c_int {
    let dot1q_hdr: *mut n_eth_dot1q_hdr_t;
    let ethertype: m_uint16_t;
    let isl_hdr: *mut n_eth_isl_hdr_t;
    let eth_hdr: *mut n_eth_hdr_t;
    let llc_hdr: *mut n_eth_llc_hdr_t;
    let mut sp: ethsw_packet_t = zeroed();
    let ptr: *mut u_char;

    sp.input_port = nio;
    sp.input_vlan = 0;
    sp.pkt = pkt;
    sp.pkt_len = pkt_len;

    // Skip runt packets
    if sp.pkt_len < N_ETH_HLEN as ssize_t {
        return -1;
    }

    // Determine the input VLAN
    match (*nio).vlan_port_type {
        ETHSW_PORT_TYPE_ACCESS => {
            sp.input_vlan = (*nio).vlan_id as u_int;
        }

        ETHSW_PORT_TYPE_DOT1Q => {
            dot1q_hdr = sp.pkt.cast::<n_eth_dot1q_hdr_t>();
            ethertype = libc::ntohs((*dot1q_hdr).r#type);

            // use the native VLAN if no tag is found
            if ethertype != N_ETH_PROTO_DOT1Q && ethertype != N_ETH_PROTO_DOT1Q_2 && ethertype != N_ETH_PROTO_DOT1Q_3 && ethertype != N_ETH_PROTO_DOT1Q_4 {
                sp.input_vlan = (*nio).vlan_id as u_int;
                sp.input_tag = FALSE;
            } else {
                sp.input_vlan = (libc::ntohs((*dot1q_hdr).vlan_id) & 0xFFF) as u_int;
                sp.input_tag = TRUE;
            }
        }

        ETHSW_PORT_TYPE_QINQ => {
            dot1q_hdr = sp.pkt.cast::<n_eth_dot1q_hdr_t>();
            ethertype = libc::ntohs((*dot1q_hdr).r#type);

            // Drop untagged traffic
            if ethertype != N_ETH_PROTO_DOT1Q && ethertype != N_ETH_PROTO_DOT1Q_2 && ethertype != N_ETH_PROTO_DOT1Q_3 && ethertype != N_ETH_PROTO_DOT1Q_4 {
                return -1;
            }

            // The MAC address lookup is done on the outer VLAN
            sp.input_vlan = (*nio).vlan_id as u_int;
        }

        ETHSW_PORT_TYPE_ISL => 'block: {
            // Check that we have an ISL packet
            eth_hdr = pkt.cast::<n_eth_hdr_t>();

            if 0 == eth_addr_is_cisco_isl(addr_of_mut!((*eth_hdr).daddr)) {
                break 'block;
            }

            // Verify LLC header
            llc_hdr = PTR_ADJUST!(*mut n_eth_llc_hdr_t, eth_hdr, size_of::<n_eth_hdr_t>());
            if 0 == eth_llc_check_snap(llc_hdr) {
                break 'block;
            }

            // Get the VLAN id
            isl_hdr = PTR_ADJUST!(*mut n_eth_isl_hdr_t, llc_hdr, size_of::<n_eth_llc_hdr_t>());
            ptr = addr_of_mut!((*isl_hdr).vlan).cast::<u_char>();
            sp.input_vlan = (((*ptr.add(0) as u_int) << 8) | (*ptr.add(1) as u_int)) >> 1;
        }

        _ => {
            libc::fprintf(c_stderr(), c"ethsw_receive: unknown port type %u\n".as_ptr(), (*nio).vlan_port_type);
            return -1;
        }
    }

    if sp.input_vlan != 0 {
        ethsw_forward(t, addr_of_mut!(sp));
    }
    0
}

// Receive a packet (handle the locking part)
unsafe extern "C" fn ethsw_recv_pkt(nio: *mut netio_desc_t, pkt: *mut u_char, pkt_len: ssize_t, t: *mut c_void, _: *mut c_void) -> c_int {
    let t: *mut ethsw_table_t = t.cast::<_>();
    if ETHSW_TRYLOCK!(t) == 0 {
        ethsw_receive(t, nio, pkt, pkt_len);
        ETHSW_UNLOCK!(t);
    }
    0
}

// Set a port as an access port with the specified VLAN
unsafe fn set_access_port(nio: *mut netio_desc_t, vlan_id: u_int) {
    (*nio).vlan_port_type = ETHSW_PORT_TYPE_ACCESS;
    (*nio).vlan_id = vlan_id as m_uint16_t;
    (*nio).vlan_input_vector = transmute::<ethsw_input_vector_t, *mut c_void>(Some(ethsw_iv_access));
    (*nio).ethertype = N_ETH_PROTO_DOT1Q;
}

// Set a port as a 802.1Q trunk port
unsafe fn set_dot1q_port(nio: *mut netio_desc_t, native_vlan: u_int) {
    (*nio).vlan_port_type = ETHSW_PORT_TYPE_DOT1Q;
    (*nio).vlan_id = native_vlan as m_uint16_t;
    (*nio).vlan_input_vector = transmute::<ethsw_input_vector_t, *mut c_void>(Some(ethsw_iv_dot1q));
    (*nio).ethertype = N_ETH_PROTO_DOT1Q;
}

// Set a port as a Q-in-Q trunk port
unsafe fn set_qinq_port(nio: *mut netio_desc_t, outer_vlan: u_int, ethertype: m_uint16_t) {
    (*nio).vlan_port_type = ETHSW_PORT_TYPE_QINQ;
    (*nio).vlan_id = outer_vlan as m_uint16_t;
    (*nio).vlan_input_vector = transmute::<ethsw_input_vector_t, *mut c_void>(Some(ethsw_iv_qinq));
    (*nio).ethertype = ethertype;
}

// Acquire a reference to an Ethernet switch (increment reference count)
#[no_mangle]
pub unsafe extern "C" fn ethsw_acquire(name: *mut c_char) -> *mut ethsw_table_t {
    registry_find(name, OBJ_TYPE_ETHSW).cast::<_>()
}

// Release an Ethernet switch (decrement reference count)
#[no_mangle]
pub unsafe extern "C" fn ethsw_release(name: *mut c_char) -> c_int {
    registry_unref(name, OBJ_TYPE_ETHSW)
}

// Create a virtual ethernet switch
#[no_mangle]
pub unsafe extern "C" fn ethsw_create(name: *mut c_char) -> *mut ethsw_table_t {
    // Allocate a new switch structure
    let t: *mut ethsw_table_t = libc::malloc(size_of::<ethsw_table_t>()).cast::<_>();
    if t.is_null() {
        return null_mut();
    }

    libc::memset(t.cast::<_>(), 0, size_of::<ethsw_table_t>());
    libc::pthread_mutex_init(addr_of_mut!((*t).lock), null_mut());

    (*t).name = libc::strdup(name);
    if (*t).name.is_null() {
        libc::free(t.cast::<_>());
        return null_mut();
    }

    // Record this object in registry
    if registry_add((*t).name, OBJ_TYPE_ETHSW, t.cast::<_>()) == -1 {
        libc::fprintf(c_stderr(), c"ethsw_create: unable to register switch '%s'\n".as_ptr(), name);
        libc::free((*t).name.cast::<_>());
        libc::free(t.cast::<_>());
        return null_mut();
    }

    t
}

// Add a NetIO descriptor to a virtual ethernet switch
#[no_mangle]
pub unsafe extern "C" fn ethsw_add_netio(t: *mut ethsw_table_t, nio_name: *mut c_char) -> c_int {
    let mut i: c_int;

    ETHSW_LOCK!(t);

    // Try to find a free slot in the NIO array
    i = 0;
    while i < ETHSW_MAX_NIO as c_int {
        if (*t).nio[i as usize].is_null() {
            break;
        }
        i += 1;
    }

    // No free slot found ...
    if i == ETHSW_MAX_NIO as c_int {
        ETHSW_UNLOCK!(t);
        return -1;
    }

    // Acquire the NIO descriptor and increment its reference count
    let nio: *mut netio_desc_t = netio_acquire(nio_name);
    if nio.is_null() {
        ETHSW_UNLOCK!(t);
        return -1;
    }

    // By default, the port is an access port in VLAN 1
    set_access_port(nio, 1);

    (*t).nio[i as usize] = nio;
    netio_rxl_add(nio, Some(ethsw_recv_pkt), t.cast::<_>(), null_mut());
    ETHSW_UNLOCK!(t);
    0
}

// Free resources used by a NIO
unsafe fn ethsw_free_nio(nio: *mut netio_desc_t) {
    netio_rxl_remove(nio);
    netio_release((*nio).name);
}

// Remove a NetIO descriptor from a virtual ethernet switch
#[no_mangle]
pub unsafe extern "C" fn ethsw_remove_netio(t: *mut ethsw_table_t, nio_name: *mut c_char) -> c_int {
    let mut i: c_int;

    ETHSW_LOCK!(t);

    let nio: *mut netio_desc_t = registry_exists(nio_name, OBJ_TYPE_NIO).cast::<_>();
    if nio.is_null() {
        ETHSW_UNLOCK!(t);
        return -1;
    }

    // Try to find the NIO in the NIO array
    i = 0;
    while i < ETHSW_MAX_NIO as c_int {
        if (*t).nio[i as usize] == nio {
            break;
        }
        i += 1;
    }

    if i == ETHSW_MAX_NIO as c_int {
        ETHSW_UNLOCK!(t);
        return -1;
    }

    // Invalidate this port in the MAC address table
    ethsw_invalidate_port(t, nio);
    (*t).nio[i as usize] = null_mut();

    ETHSW_UNLOCK!(t);

    // Remove the NIO from the RX multiplexer
    ethsw_free_nio(nio);
    0
}

// Clear the MAC address table
#[no_mangle]
pub unsafe extern "C" fn ethsw_clear_mac_addr_table(t: *mut ethsw_table_t) -> c_int {
    ETHSW_LOCK!(t);
    ethsw_invalidate(t);
    ETHSW_UNLOCK!(t);
    0
}

// Iterate over all entries of the MAC address table
#[no_mangle]
pub unsafe extern "C" fn ethsw_iterate_mac_addr_table(t: *mut ethsw_table_t, cb: ethsw_foreach_entry_t, opt_arg: *mut c_void) -> c_int {
    let mut entry: *mut ethsw_mac_entry_t;

    ETHSW_LOCK!(t);

    for i in 0..ETHSW_HASH_SIZE as c_int {
        entry = addr_of_mut!((*t).mac_addr_table[i as usize]);

        if (*entry).nio.is_null() {
            continue;
        }

        cb.unwrap()(t, entry, opt_arg);
    }

    ETHSW_UNLOCK!(t);
    0
}

// Set port as an access port
#[no_mangle]
pub unsafe extern "C" fn ethsw_set_access_port(t: *mut ethsw_table_t, nio_name: *mut c_char, vlan_id: u_int) -> c_int {
    let mut res: c_int = -1;

    ETHSW_LOCK!(t);

    for i in 0..ETHSW_MAX_NIO as c_int {
        if !(*t).nio[i as usize].is_null() && 0 == libc::strcmp((*(*t).nio[i as usize]).name, nio_name) {
            set_access_port((*t).nio[i as usize], vlan_id);
            res = 0;
            break;
        }
    }

    ETHSW_UNLOCK!(t);
    res
}

// Set port as a 802.1q trunk port
#[no_mangle]
pub unsafe extern "C" fn ethsw_set_dot1q_port(t: *mut ethsw_table_t, nio_name: *mut c_char, native_vlan: u_int) -> c_int {
    let mut res: c_int = -1;

    ETHSW_LOCK!(t);

    for i in 0..ETHSW_MAX_NIO as c_int {
        if !(*t).nio[i as usize].is_null() && 0 == libc::strcmp((*(*t).nio[i as usize]).name, nio_name) {
            set_dot1q_port((*t).nio[i as usize], native_vlan);
            res = 0;
            break;
        }
    }

    ETHSW_UNLOCK!(t);
    res
}

// Set port as a Q-in-Q port
#[no_mangle]
pub unsafe extern "C" fn ethsw_set_qinq_port(t: *mut ethsw_table_t, nio_name: *mut c_char, outer_vlan: u_int, ethertype: m_uint16_t) -> c_int {
    let mut res: c_int = -1;

    if ethertype != N_ETH_PROTO_DOT1Q && ethertype != N_ETH_PROTO_DOT1Q_2 && ethertype != N_ETH_PROTO_DOT1Q_3 && ethertype != N_ETH_PROTO_DOT1Q_4 {
        return -1;
    }

    ETHSW_LOCK!(t);

    for i in 0..ETHSW_MAX_NIO as c_int {
        if !(*t).nio[i as usize].is_null() && 0 == libc::strcmp((*(*t).nio[i as usize]).name, nio_name) {
            set_qinq_port((*t).nio[i as usize], outer_vlan, ethertype);
            res = 0;
            break;
        }
    }

    ETHSW_UNLOCK!(t);
    res
}

// Save the configuration of a switch
#[no_mangle]
pub unsafe extern "C" fn ethsw_save_config(t: *mut ethsw_table_t, fd: *mut libc::FILE) {
    let mut nio: *mut netio_desc_t;

    libc::fprintf(fd, c"ethsw create %s\n".as_ptr(), (*t).name);

    ETHSW_LOCK!(t);

    for i in 0..ETHSW_MAX_NIO as c_int {
        nio = (*t).nio[i as usize];

        libc::fprintf(fd, c"ethsw add_nio %s %s\n".as_ptr(), (*t).name, (*nio).name);

        match (*nio).vlan_port_type {
            ETHSW_PORT_TYPE_ACCESS => {
                libc::fprintf(fd, c"ethsw set_access_port %s %s %u\n".as_ptr(), (*t).name, (*nio).name, (*nio).vlan_id as c_uint);
            }

            ETHSW_PORT_TYPE_DOT1Q => {
                libc::fprintf(fd, c"ethsw set_dot1q_port %s %s %u\n".as_ptr(), (*t).name, (*nio).name, (*nio).vlan_id as c_uint);
            }

            ETHSW_PORT_TYPE_QINQ => {
                libc::fprintf(fd, c"ethsw set_qinq_port %s %s %u 0x%x\n".as_ptr(), (*t).name, (*nio).name, (*nio).vlan_id as c_uint, (*nio).ethertype as c_uint);
            }

            _ => {
                libc::fprintf(c_stderr(), c"ethsw_save_config: unknown port type %u\n".as_ptr(), (*nio).vlan_port_type);
            }
        }
    }

    ETHSW_UNLOCK!(t);

    libc::fprintf(fd, c"\n".as_ptr());
}

// Save configurations of all Ethernet switches
unsafe extern "C" fn ethsw_reg_save_config(entry: *mut registry_entry_t, opt: *mut c_void, _err: *mut c_int) {
    ethsw_save_config((*entry).data.cast::<ethsw_table_t>(), opt.cast::<libc::FILE>());
}

#[no_mangle]
pub unsafe extern "C" fn ethsw_save_config_all(fd: *mut libc::FILE) {
    registry_foreach_type(OBJ_TYPE_ETHSW, Some(ethsw_reg_save_config), fd.cast::<_>(), null_mut());
}

// Free resources used by a virtual ethernet switch
unsafe extern "C" fn ethsw_free(data: *mut c_void, _arg: *mut c_void) -> c_int {
    let t: *mut ethsw_table_t = data.cast::<_>();

    for i in 0..ETHSW_MAX_NIO as c_int {
        if (*t).nio[i as usize].is_null() {
            continue;
        }

        ethsw_free_nio((*t).nio[i as usize]);
    }

    libc::free((*t).name.cast::<_>());
    libc::free(t.cast::<_>());
    TRUE
}

// Delete a virtual ethernet switch
#[no_mangle]
pub unsafe extern "C" fn ethsw_delete(name: *mut c_char) -> c_int {
    registry_delete_if_unused(name, OBJ_TYPE_ETHSW, Some(ethsw_free), null_mut())
}

// Delete all virtual ethernet switches
#[no_mangle]
pub unsafe extern "C" fn ethsw_delete_all() -> c_int {
    registry_delete_type(OBJ_TYPE_ETHSW, Some(ethsw_free), null_mut())
}

// Create a new interface
unsafe fn ethsw_cfg_create_if(t: *mut ethsw_table_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    let mut nio: *mut netio_desc_t = null_mut();

    let nio_type: c_int = netio_get_type(*tokens.add(2));
    match nio_type as u_int {
        NETIO_TYPE_UNIX => 'block: {
            if count != 5 {
                libc::fprintf(c_stderr(), c"ETHSW: invalid number of arguments for UNIX NIO\n".as_ptr());
                break 'block;
            }

            nio = netio_desc_create_unix(*tokens.add(1), *tokens.add(3), *tokens.add(4));
        }

        NETIO_TYPE_TAP => 'block: {
            if count != 4 {
                libc::fprintf(c_stderr(), c"ETHSW: invalid number of arguments for TAP NIO\n".as_ptr());
                break 'block;
            }

            nio = netio_desc_create_tap(*tokens.add(1), *tokens.add(3));
        }

        NETIO_TYPE_UDP => 'block: {
            if count != 6 {
                libc::fprintf(c_stderr(), c"ETHSW: invalid number of arguments for UDP NIO\n".as_ptr());
                break 'block;
            }

            nio = netio_desc_create_udp(*tokens.add(1), libc::atoi(*tokens.add(3)), *tokens.add(4), libc::atoi(*tokens.add(5)));
        }

        NETIO_TYPE_TCP_CLI => 'block: {
            if count != 5 {
                libc::fprintf(c_stderr(), c"ETHSW: invalid number of arguments for TCP CLI NIO\n".as_ptr());
                break 'block;
            }

            nio = netio_desc_create_tcp_cli(*tokens.add(1), *tokens.add(3), *tokens.add(4));
        }

        NETIO_TYPE_TCP_SER => 'block: {
            if count != 4 {
                libc::fprintf(c_stderr(), c"ETHSW: invalid number of arguments for TCP SER NIO\n".as_ptr());
                break 'block;
            }

            nio = netio_desc_create_tcp_ser(*tokens.add(1), *tokens.add(3));
        }

        #[cfg(feature = "ENABLE_GEN_ETH")]
        NETIO_TYPE_GEN_ETH => 'block: {
            if count != 4 {
                libc::fprintf(c_stderr(), c"ETHSW: invalid number of arguments for Generic Ethernet NIO\n".as_ptr());
                break 'block;
            }

            nio = netio_desc_create_geneth(*tokens.add(1), *tokens.add(3));
        }

        #[cfg(feature = "ENABLE_LINUX_ETH")]
        NETIO_TYPE_LINUX_ETH => 'block: {
            if count != 4 {
                libc::fprintf(c_stderr(), c"ETHSW: invalid number of arguments for Linux Ethernet NIO\n".as_ptr());
                break 'block;
            }

            nio = netio_desc_create_lnxeth(*tokens.add(1), *tokens.add(3));
        }

        _ => {
            libc::fprintf(c_stderr(), c"ETHSW: unknown/invalid NETIO type '%s'\n".as_ptr(), *tokens.add(2));
        }
    }

    if nio.is_null() {
        libc::fprintf(c_stderr(), c"ETHSW: unable to create NETIO descriptor\n".as_ptr());
        return -1;
    }

    if ethsw_add_netio(t, *tokens.add(1)) == -1 {
        libc::fprintf(c_stderr(), c"ETHSW: unable to add NETIO descriptor.\n".as_ptr());
        netio_release((*nio).name);
        return -1;
    }

    netio_release((*nio).name);
    0
}

// Set a port as an access port
unsafe fn ethsw_cfg_set_access_port(t: *mut ethsw_table_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    // 3 parameters: "ACCESS", IF, VLAN
    if count != 3 {
        libc::fprintf(c_stderr(), c"ETHSW: invalid access port description.\n".as_ptr());
        return -1;
    }

    ethsw_set_access_port(t, *tokens.add(1), libc::atoi(*tokens.add(2)) as u_int)
}

// Set a port as a 802.1q trunk port
unsafe fn ethsw_cfg_set_dot1q_port(t: *mut ethsw_table_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    // 3 parameters: "DOT1Q", IF, Native VLAN
    if count != 3 {
        libc::fprintf(c_stderr(), c"ETHSW: invalid trunk port description.\n".as_ptr());
        return -1;
    }

    ethsw_set_dot1q_port(t, *tokens.add(1), libc::atoi(*tokens.add(2)) as u_int)
}

// Set a port as a Q-in-Q port
unsafe fn ethsw_cfg_set_qinq_port(t: *mut ethsw_table_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    let mut ethertype: m_uint16_t = N_ETH_PROTO_DOT1Q;
    // 3 + 1 parameters: "QINQ", IF, Outer VLAN, Ethertype (optional)
    if count == 4 {
        libc::sscanf(*tokens.add(3), c"0x%hx".as_ptr(), addr_of_mut!(ethertype));
    } else if count != 3 {
        libc::fprintf(c_stderr(), c"ETHSW: invalid QinQ port description.\n".as_ptr());
        return -1;
    }

    ethsw_set_qinq_port(t, *tokens.add(1), libc::atoi(*tokens.add(2)) as u_int, ethertype)
}

const ETHSW_MAX_TOKENS: usize = 16;

// Handle a ETHSW configuration line
unsafe fn ethsw_handle_cfg_line(t: *mut ethsw_table_t, str_: *mut c_char) -> c_int {
    let mut tokens: [*mut c_char; ETHSW_MAX_TOKENS] = [null_mut(); ETHSW_MAX_TOKENS];

    let count: c_int = m_strsplit(str_, b':' as c_char, tokens.as_mut_ptr(), ETHSW_MAX_TOKENS as c_int);
    if count <= 1 {
        return -1;
    }

    if 0 == libc::strcmp(tokens[0], c"IF".as_ptr()) {
        return ethsw_cfg_create_if(t, tokens.as_mut_ptr(), count);
    } else if 0 == libc::strcmp(tokens[0], c"ACCESS".as_ptr()) {
        return ethsw_cfg_set_access_port(t, tokens.as_mut_ptr(), count);
    } else if 0 == libc::strcmp(tokens[0], c"DOT1Q".as_ptr()) {
        return ethsw_cfg_set_dot1q_port(t, tokens.as_mut_ptr(), count);
    } else if 0 == libc::strcmp(tokens[0], c"QINQ".as_ptr()) {
        return ethsw_cfg_set_qinq_port(t, tokens.as_mut_ptr(), count);
    }

    libc::fprintf(c_stderr(), c"ETHSW: Unknown statement \"%s\" (allowed: IF,ACCESS,TRUNK)\n".as_ptr(), tokens[0]);
    -1
}

// Read a ETHSW configuration file
unsafe fn ethsw_read_cfg_file(t: *mut ethsw_table_t, filename: *mut c_char) -> c_int {
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
            ethsw_handle_cfg_line(t, buffer.as_mut_ptr());
        }
    }

    libc::fclose(fd);
    0
}

// Start a virtual Ethernet switch
#[no_mangle]
pub unsafe extern "C" fn ethsw_start(filename: *mut c_char) -> c_int {
    let t: *mut ethsw_table_t = ethsw_create(c"default".as_ptr().cast_mut());
    if t.is_null() {
        libc::fprintf(c_stderr(), c"ETHSW: unable to create virtual fabric table.\n".as_ptr());
        return -1;
    }

    if ethsw_read_cfg_file(t, filename) == -1 {
        libc::fprintf(c_stderr(), c"ETHSW: unable to parse configuration file.\n".as_ptr());
        return -1;
    }

    ethsw_release(c"default".as_ptr().cast_mut());
    0
}
