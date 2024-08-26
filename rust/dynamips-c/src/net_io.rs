//! Cisco router) simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Network Input/Output Abstraction Layer.

use crate::_private::*;
use crate::dynamips_common::*;
#[cfg(feature = "ENABLE_GEN_ETH")]
use crate::gen_eth::*;
#[cfg(feature = "ENABLE_LINUX_ETH")]
use crate::linux_eth::*;
use crate::net::*;
use crate::net_io_filter::*;
use crate::ptask::*;
use crate::registry::*;
use crate::utils::*;
use std::cmp::min;

pub type netio_unix_desc_t = netio_unix_desc;
pub type netio_vde_desc_t = netio_vde_desc;
pub type netio_tap_desc_t = netio_tap_desc;
pub type netio_inet_desc_t = netio_inet_desc;
#[cfg(feature = "ENABLE_LINUX_ETH")]
pub type netio_lnxeth_desc_t = netio_lnxeth_desc;
#[cfg(feature = "ENABLE_GEN_ETH")]
pub type netio_geneth_desc_t = netio_geneth_desc;
pub type netio_fifo_pkt_t = netio_fifo_pkt;
pub type netio_fifo_desc_t = netio_fifo_desc;
pub type netio_pktfilter_t = netio_pktfilter;
pub type netio_stat_t = netio_stat;
pub type netio_desc_t = netio_desc;

/// Maximum packet size
pub const NETIO_MAX_PKT_SIZE: usize = 32768;

/// Maximum device length
pub const NETIO_DEV_MAXLEN: usize = 64;

// TODO enum
pub const NETIO_TYPE_UNIX: u_int = 0;
pub const NETIO_TYPE_VDE: u_int = 1;
pub const NETIO_TYPE_TAP: u_int = 2;
pub const NETIO_TYPE_UDP: u_int = 3;
pub const NETIO_TYPE_UDP_AUTO: u_int = 4;
pub const NETIO_TYPE_TCP_CLI: u_int = 5;
pub const NETIO_TYPE_TCP_SER: u_int = 6;

#[cfg(all(feature = "ENABLE_LINUX_ETH", feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_LINUX_ETH: u_int = 7;
#[cfg(all(feature = "ENABLE_LINUX_ETH", feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_GEN_ETH: u_int = 8;
#[cfg(all(feature = "ENABLE_LINUX_ETH", feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_FIFO: u_int = 9;
#[cfg(all(feature = "ENABLE_LINUX_ETH", feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_NULL: u_int = 10;
#[cfg(all(feature = "ENABLE_LINUX_ETH", feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_MAX: usize = 11;

#[cfg(all(not(feature = "ENABLE_LINUX_ETH"), feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_GEN_ETH: u_int = 7;
#[cfg(all(not(feature = "ENABLE_LINUX_ETH"), feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_FIFO: u_int = 8;
#[cfg(all(not(feature = "ENABLE_LINUX_ETH"), feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_NULL: u_int = 9;
#[cfg(all(not(feature = "ENABLE_LINUX_ETH"), feature = "ENABLE_GEN_ETH"))]
pub const NETIO_TYPE_MAX: usize = 10;

#[cfg(all(feature = "ENABLE_LINUX_ETH", not(feature = "ENABLE_GEN_ETH")))]
pub const NETIO_TYPE_LINUX_ETH: u_int = 7;
#[cfg(all(feature = "ENABLE_LINUX_ETH", not(feature = "ENABLE_GEN_ETH")))]
pub const NETIO_TYPE_FIFO: u_int = 8;
#[cfg(all(feature = "ENABLE_LINUX_ETH", not(feature = "ENABLE_GEN_ETH")))]
pub const NETIO_TYPE_NULL: u_int = 9;
#[cfg(all(feature = "ENABLE_LINUX_ETH", not(feature = "ENABLE_GEN_ETH")))]
pub const NETIO_TYPE_MAX: usize = 10;

#[cfg(all(not(feature = "ENABLE_LINUX_ETH"), not(feature = "ENABLE_GEN_ETH")))]
pub const NETIO_TYPE_FIFO: u_int = 7;
#[cfg(all(not(feature = "ENABLE_LINUX_ETH"), not(feature = "ENABLE_GEN_ETH")))]
pub const NETIO_TYPE_NULL: u_int = 8;
#[cfg(all(not(feature = "ENABLE_LINUX_ETH"), not(feature = "ENABLE_GEN_ETH")))]
pub const NETIO_TYPE_MAX: usize = 9;

// TODO enum
pub const NETIO_FILTER_ACTION_DROP: c_int = 0;
pub const NETIO_FILTER_ACTION_PASS: c_int = 1;
pub const NETIO_FILTER_ACTION_ALTER: c_int = 2;
pub const NETIO_FILTER_ACTION_DUPLICATE: c_int = 3;

/// VDE switch definitions
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum vde_request_type {
    VDE_REQ_NEW_CONTROL,
}

pub const VDE_SWITCH_MAGIC: m_uint32_t = 0xfeedface;
pub const VDE_SWITCH_VERSION: m_uint32_t = 3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct vde_request_v3 {
    pub magic: m_uint32_t,
    pub version: m_uint32_t,
    pub r#type: vde_request_type,
    pub sock: libc::sockaddr_un,
}

/// netio unix descriptor
#[repr(C)]
#[derive(Copy, Clone)]
pub struct netio_unix_desc {
    pub local_filename: *mut c_char,
    pub remote_sock: libc::sockaddr_un,
    pub fd: c_int,
}

/// netio vde descriptor
#[repr(C)]
#[derive(Copy, Clone)]
pub struct netio_vde_desc {
    pub local_filename: *mut c_char,
    pub remote_sock: libc::sockaddr_un,
    pub ctrl_fd: c_int,
    pub data_fd: c_int,
}

/// netio tap descriptor
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct netio_tap_desc {
    pub filename: [c_char; NETIO_DEV_MAXLEN],
    pub fd: c_int,
}

/// netio udp/tcp descriptor
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct netio_inet_desc {
    pub local_port: c_int,
    pub remote_port: c_int,
    pub remote_host: *mut c_char,
    pub fd: c_int,
}

/// netio linux raw ethernet descriptor
#[cfg(feature = "ENABLE_LINUX_ETH")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct netio_lnxeth_desc {
    pub dev_name: [c_char; NETIO_DEV_MAXLEN],
    pub dev_id: c_int,
    pub fd: c_int,
}

/// netio generic raw ethernet descriptor
#[cfg(feature = "ENABLE_GEN_ETH")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct netio_geneth_desc {
    pub dev_name: [c_char; NETIO_DEV_MAXLEN],
    pub pcap_dev: *mut pcap_sys::pcap_t,
}

/// FIFO packet
#[repr(C)]
#[derive(Debug)]
pub struct netio_fifo_pkt {
    pub next: *mut netio_fifo_pkt_t,
    pub pkt_len: size_t,
    pub pkt: [c_char; 0], // incomplete array
}

/// Netio FIFO
#[repr(C)]
#[derive(Copy, Clone)]
pub struct netio_fifo_desc {
    pub cond: libc::pthread_cond_t,
    pub lock: libc::pthread_mutex_t,
    pub endpoint_lock: libc::pthread_mutex_t,
    pub endpoint: *mut netio_fifo_desc_t,
    pub head: *mut netio_fifo_pkt_t,
    pub last: *mut netio_fifo_pkt_t,
    pub pkt_count: u_int,
}

/// Packet filter
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct netio_pktfilter {
    pub name: *mut c_char,
    pub setup: Option<unsafe extern "C" fn(nio: *mut netio_desc_t, opt: *mut *mut c_void, argc: c_int, argv: *mut *mut c_char) -> c_int>,
    pub free: Option<unsafe extern "C" fn(nio: *mut netio_desc_t, opt: *mut *mut c_void)>,
    pub pkt_handler: Option<unsafe extern "C" fn(nio: *mut netio_desc_t, pkt: *mut c_void, len: size_t, opt: *mut c_void) -> c_int>,
    pub next: *mut netio_pktfilter_t,
}
impl netio_pktfilter {
    pub const fn new(
        name: *mut c_char,
        setup: Option<unsafe extern "C" fn(nio: *mut netio_desc_t, opt: *mut *mut c_void, argc: c_int, argv: *mut *mut c_char) -> c_int>,
        free: Option<unsafe extern "C" fn(nio: *mut netio_desc_t, opt: *mut *mut c_void)>,
        pkt_handler: Option<unsafe extern "C" fn(nio: *mut netio_desc_t, pkt: *mut c_void, len: size_t, opt: *mut c_void) -> c_int>,
        next: *mut netio_pktfilter_t,
    ) -> Self {
        Self { name, setup, free, pkt_handler, next }
    }
}

/// Statistics
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct netio_stat {
    pub pkts: m_uint64_t,
    pub bytes: m_uint64_t,
}

pub const NETIO_BW_SAMPLES: usize = 10;
pub const NETIO_BW_SAMPLE_ITV: usize = 30;

/// Generic netio descriptor
#[repr(C)]
#[derive(Copy, Clone)]
pub struct netio_desc {
    pub r#type: u_int,
    pub dptr: *mut c_void,
    pub name: *mut c_char,
    pub debug: c_int,

    /// Frame Relay specific information
    pub fr_lmi_seq: m_uint8_t,
    pub fr_conn_list: *mut c_void,

    /// Ethernet specific information
    pub vlan_port_type: u_int,
    pub vlan_id: m_uint16_t,
    pub vlan_input_vector: *mut c_void,
    pub ethertype: m_uint16_t,

    pub u: netio_desc_u,

    /// Send and receive prototypes
    pub send: Option<unsafe extern "C" fn(desc: *mut c_void, pkt: *mut c_void, len: size_t) -> ssize_t>,
    pub recv: Option<unsafe extern "C" fn(desc: *mut c_void, pkt: *mut c_void, len: size_t) -> ssize_t>,

    /// Configuration saving
    pub save_cfg: Option<unsafe extern "C" fn(nio: *mut netio_desc_t, fd: *mut libc::FILE)>,

    /// Free ressources
    pub free: Option<unsafe extern "C" fn(desc: *mut c_void)>,

    /// Bandwidth constraint (in Kb/s)
    pub bandwidth: u_int,
    pub bw_cnt: [m_uint64_t; NETIO_BW_SAMPLES],
    pub bw_cnt_total: m_uint64_t,
    pub bw_pos: u_int,
    pub bw_ptask_cnt: u_int,

    /// Packet filters
    pub rx_filter: *mut netio_pktfilter_t,
    pub tx_filter: *mut netio_pktfilter_t,
    pub both_filter: *mut netio_pktfilter_t,
    pub rx_filter_data: *mut c_void,
    pub tx_filter_data: *mut c_void,
    pub both_filter_data: *mut c_void,

    /// Statistics
    pub stats_pkts_in: m_uint64_t,
    pub stats_pkts_out: m_uint64_t,
    pub stats_bytes_in: m_uint64_t,
    pub stats_bytes_out: m_uint64_t,

    /// Next pointer (for RX listener)
    pub rxl_next: *mut netio_desc_t,

    /// Packet data
    pub rx_pkt: [u_char; NETIO_MAX_PKT_SIZE],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union netio_desc_u {
    pub nud: netio_unix_desc_t,
    pub nvd: netio_vde_desc_t,
    pub ntd: netio_tap_desc_t,
    pub nid: netio_inet_desc_t,
    #[cfg(feature = "ENABLE_LINUX_ETH")]
    pub nled: netio_lnxeth_desc_t,
    #[cfg(feature = "ENABLE_GEN_ETH")]
    pub nged: netio_geneth_desc_t,
    pub nfd: netio_fifo_desc_t,
}

/// RX listener
pub type netio_rx_handler_t = Option<unsafe extern "C" fn(nio: *mut netio_desc_t, pkt: *mut u_char, pkt_len: ssize_t, arg1: *mut c_void, arg2: *mut c_void) -> c_int>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct netio_rx_listener {
    pub nio: *mut netio_desc_t,
    pub ref_count: u_int,
    pub running: Volatile<c_int>,
    pub rx_handler: netio_rx_handler_t,
    pub arg1: *mut c_void,
    pub arg2: *mut c_void,
    pub spec_thread: libc::pthread_t,
    pub prev: *mut netio_rx_listener,
    pub next: *mut netio_rx_listener,
}

/// NIO RX listener
static mut netio_rxl_mutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
static mut netio_rxq_mutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
static mut netio_rxl_list: *mut netio_rx_listener = null_mut();
static mut netio_rxl_add_list: *mut netio_rx_listener = null_mut();
static mut netio_rxl_remove_list: *mut netio_desc_t = null_mut();
static mut netio_rxl_thread: libc::pthread_t = 0;
static mut netio_rxl_cond: libc::pthread_cond_t = unsafe { zeroed::<_>() };

unsafe fn NETIO_RXL_LOCK() {
    libc::pthread_mutex_lock(addr_of_mut!(netio_rxl_mutex));
}
unsafe fn NETIO_RXL_UNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!(netio_rxl_mutex));
}

unsafe fn NETIO_RXQ_LOCK() {
    libc::pthread_mutex_lock(addr_of_mut!(netio_rxq_mutex));
}
unsafe fn NETIO_RXQ_UNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!(netio_rxq_mutex));
}

/// NetIO type
struct netio_type_t {
    name: *mut c_char,
    desc: *mut c_char,
}
impl netio_type_t {
    pub const fn new(name: *mut c_char, desc: *mut c_char) -> Self {
        Self { name, desc }
    }
}

/// NETIO types (must follow the enum definition)
static mut netio_types: [netio_type_t; NETIO_TYPE_MAX] = [
    netio_type_t::new(cstr!("unix"), cstr!("UNIX local sockets")),
    netio_type_t::new(cstr!("vde"), cstr!("Virtual Distributed Ethernet / UML switch")),
    netio_type_t::new(cstr!("tap"), cstr!("Linux/FreeBSD TAP device")),
    netio_type_t::new(cstr!("udp"), cstr!("UDP sockets")),
    netio_type_t::new(cstr!("udp_auto"), cstr!("Auto UDP sockets")),
    netio_type_t::new(cstr!("tcp_cli"), cstr!("TCP client")),
    netio_type_t::new(cstr!("tcp_ser"), cstr!("TCP server")),
    #[cfg(feature = "ENABLE_LINUX_ETH")]
    netio_type_t::new(cstr!("linux_eth"), cstr!("Linux Ethernet device")),
    #[cfg(feature = "ENABLE_GEN_ETH")]
    netio_type_t::new(cstr!("gen_eth"), cstr!("Generic Ethernet device (PCAP)")),
    netio_type_t::new(cstr!("fifo"), cstr!("FIFO (intra-hypervisor)")),
    netio_type_t::new(cstr!("null"), cstr!("Null device")),
];

/// Get NETIO type given a description
#[no_mangle]
pub unsafe extern "C" fn netio_get_type(type_: *mut c_char) -> c_int {
    #[allow(clippy::needless_range_loop)]
    for i in 0..NETIO_TYPE_MAX {
        if libc::strcmp(type_, netio_types[i].name) == 0 {
            return i as c_int;
        }
    }

    -1
}

/// Show the NETIO types
#[no_mangle]
pub unsafe extern "C" fn netio_show_types() {
    libc::printf(cstr!("Available NETIO types:\n"));

    #[allow(clippy::needless_range_loop)]
    for i in 0..NETIO_TYPE_MAX {
        libc::printf(cstr!("  * %-10s : %s\n"), netio_types[i].name, netio_types[i].desc);
    }

    libc::printf(cstr!("\n"));
}

// =========================================================================
// Generic functions (abstraction layer)
// =========================================================================

/// Acquire a reference to NIO from registry (increment reference count)
#[no_mangle]
pub unsafe extern "C" fn netio_acquire(name: *mut c_char) -> *mut netio_desc_t {
    registry_find(name, OBJ_TYPE_NIO).cast::<_>()
}

/// Release an NIO (decrement reference count)
#[no_mangle]
pub unsafe extern "C" fn netio_release(name: *mut c_char) -> c_int {
    registry_unref(name, OBJ_TYPE_NIO)
}

/// Record an NIO in registry
unsafe fn netio_record(nio: *mut netio_desc_t) -> c_int {
    registry_add((*nio).name, OBJ_TYPE_NIO, nio.cast::<_>())
}

/// Create a new NetIO descriptor
unsafe fn netio_create(name: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = libc::malloc(size_of::<netio_desc_t>()).cast::<_>();
    if nio.is_null() {
        return null_mut();
    }

    // setup as a NULL descriptor
    libc::memset(nio.cast::<_>(), 0, size_of::<netio_desc_t>());
    (*nio).r#type = NETIO_TYPE_NULL;

    // save name for registry
    (*nio).name = libc::strdup(name);
    if (*nio).name.is_null() {
        libc::free(nio.cast::<_>());
        return null_mut();
    }

    nio
}

/// Delete a NetIO descriptor
#[no_mangle]
pub unsafe extern "C" fn netio_delete(name: *mut c_char) -> c_int {
    registry_delete_if_unused(name, OBJ_TYPE_NIO, Some(netio_free), null_mut())
}

/// Delete all NetIO descriptors
#[no_mangle]
pub unsafe extern "C" fn netio_delete_all() -> c_int {
    registry_delete_type(OBJ_TYPE_NIO, Some(netio_free), null_mut())
}

/// Save the configuration of a NetIO descriptor
#[no_mangle]
pub unsafe extern "C" fn netio_save_config(nio: *mut netio_desc_t, fd: *mut libc::FILE) {
    if (*nio).save_cfg.is_some() {
        (*nio).save_cfg.unwrap()(nio, fd);
    }
}

unsafe extern "C" fn netio_reg_save_config(entry: *mut registry_entry_t, opt: *mut c_void, _err: *mut c_int) {
    netio_save_config((*entry).data.cast::<netio_desc_t>(), opt.cast::<libc::FILE>());
}

/// Save configurations of all NetIO descriptors
#[no_mangle]
pub unsafe extern "C" fn netio_save_config_all(fd: *mut libc::FILE) {
    registry_foreach_type(OBJ_TYPE_NIO, Some(netio_reg_save_config), fd.cast::<_>(), null_mut());
    libc::fprintf(fd, cstr!("\n"));
}

/// Send a packet through a NetIO descriptor
#[no_mangle]
pub unsafe extern "C" fn netio_send(nio: *mut netio_desc_t, pkt: *mut c_void, len: size_t) -> ssize_t {
    if nio.is_null() {
        return -1;
    }

    if (*nio).debug != 0 {
        libc::printf(cstr!("NIO %s: sending a packet of %lu bytes:\n"), (*nio).name, len as u_long);
        mem_dump(c_stdout(), pkt.cast::<_>(), len as u_int);
    }

    // Apply the TX filter
    if !(*nio).tx_filter.is_null() {
        let res: c_int = (*(*nio).tx_filter).pkt_handler.unwrap()(nio, pkt, len, (*nio).tx_filter_data);

        if res <= 0 {
            return -1;
        }
    }

    // Apply the bidirectional filter
    if !(*nio).both_filter.is_null() {
        let res: c_int = (*(*nio).both_filter).pkt_handler.unwrap()(nio, pkt, len, (*nio).both_filter_data);

        if res == NETIO_FILTER_ACTION_DROP {
            return -1;
        }
    }

    // Update output statistics
    (*nio).stats_pkts_out += 1;
    (*nio).stats_bytes_out += len as m_uint64_t;

    netio_update_bw_stat(nio, len as m_uint64_t);

    (*nio).send.unwrap()((*nio).dptr, pkt, len)
}

/// Receive a packet through a NetIO descriptor
#[no_mangle]
pub unsafe extern "C" fn netio_recv(nio: *mut netio_desc_t, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    if nio.is_null() {
        return -1;
    }

    // Receive the packet
    libc::memset(pkt, 0, max_len);
    let len: ssize_t = (*nio).recv.unwrap()((*nio).dptr, pkt, max_len);
    if len <= 0 {
        return -1;
    }

    if (*nio).debug != 0 {
        libc::printf(cstr!("NIO %s: receiving a packet of %ld bytes:\n"), (*nio).name, len as c_long);
        mem_dump(c_stdout(), pkt.cast::<_>(), len as u_int);
    }

    // Apply the RX filter
    if !(*nio).rx_filter.is_null() {
        let res: c_int = (*(*nio).rx_filter).pkt_handler.unwrap()(nio, pkt, len as size_t, (*nio).rx_filter_data);

        if res == NETIO_FILTER_ACTION_DROP {
            return -1;
        }
    }

    // Apply the bidirectional filter
    if !(*nio).both_filter.is_null() {
        let res: c_int = (*(*nio).both_filter).pkt_handler.unwrap()(nio, pkt, len as size_t, (*nio).both_filter_data);

        if res == NETIO_FILTER_ACTION_DROP {
            return -1;
        }
    }

    // Update input statistics
    (*nio).stats_pkts_in += 1;
    (*nio).stats_bytes_in += len as m_uint64_t;
    len
}

/// Get a NetIO FD
#[no_mangle]
pub unsafe extern "C" fn netio_get_fd(nio: *mut netio_desc_t) -> c_int {
    let mut fd: c_int = -1;

    match (*nio).r#type {
        NETIO_TYPE_UNIX => {
            fd = (*nio).u.nud.fd;
        }
        NETIO_TYPE_VDE => {
            fd = (*nio).u.nvd.data_fd;
        }
        NETIO_TYPE_TAP => {
            fd = (*nio).u.ntd.fd;
        }
        NETIO_TYPE_TCP_CLI | NETIO_TYPE_TCP_SER | NETIO_TYPE_UDP | NETIO_TYPE_UDP_AUTO => {
            fd = (*nio).u.nid.fd;
        }
        #[cfg(feature = "ENABLE_LINUX_ETH")]
        NETIO_TYPE_LINUX_ETH => {
            fd = (*nio).u.nled.fd;
        }
        _ => {}
    }

    fd
}

// =========================================================================
// UNIX sockets
// =========================================================================

// Create an UNIX socket
unsafe fn netio_unix_create_socket(nud: *mut netio_unix_desc_t) -> c_int {
    let mut local_sock: libc::sockaddr_un = zeroed::<_>();

    (*nud).fd = libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM, 0);
    if (*nud).fd == -1 {
        libc::perror(cstr!("netio_unix: socket"));
        return -1;
    }

    libc::memset(addr_of_mut!(local_sock).cast::<_>(), 0, size_of::<libc::sockaddr_un>());
    local_sock.sun_family = libc::AF_UNIX as libc::sa_family_t;
    libc::strncpy(local_sock.sun_path.as_c_mut(), (*nud).local_filename, local_sock.sun_path.len() - 1);
    local_sock.sun_path[local_sock.sun_path.len() - 1] = 0;

    if libc::bind((*nud).fd, addr_of!(local_sock).cast::<_>(), size_of::<libc::sockaddr_un>() as libc::socklen_t) == -1 {
        libc::perror(cstr!("netio_unix: bind"));
        return -1;
    }

    (*nud).fd
}

/// Free a NetIO unix descriptor
unsafe extern "C" fn netio_unix_free(nud: *mut c_void) {
    let nud: *mut netio_unix_desc_t = nud.cast::<_>();
    if (*nud).fd != -1 {
        libc::close((*nud).fd);
    }

    if !(*nud).local_filename.is_null() {
        libc::unlink((*nud).local_filename);
        libc::free((*nud).local_filename.cast::<_>());
    }
}

/// Allocate a new NetIO UNIX descriptor
unsafe fn netio_unix_create(nud: *mut netio_unix_desc_t, local: *mut c_char, remote: *mut c_char) -> c_int {
    libc::memset(nud.cast::<_>(), 0, size_of::<netio_unix_desc_t>());
    (*nud).fd = -1;

    // check lengths
    if (libc::strlen(local) >= (*nud).remote_sock.sun_path.len()) || (libc::strlen(remote) >= (*nud).remote_sock.sun_path.len()) {
        libc::fprintf(c_stderr(), cstr!("netio_unix_create: invalid file size or insufficient memory\n"));
        return -1;
    }

    (*nud).local_filename = libc::strdup(local);
    if (*nud).local_filename.is_null() {
        libc::fprintf(c_stderr(), cstr!("netio_unix_create: invalid file size or insufficient memory\n"));
        return -1;
    }

    if netio_unix_create_socket(nud) == -1 {
        return -1;
    }

    // prepare the remote info
    (*nud).remote_sock.sun_family = libc::AF_UNIX as libc::sa_family_t;
    libc::strcpy((*nud).remote_sock.sun_path.as_c_mut(), remote);
    0
}

/// Send a packet to an UNIX socket
unsafe extern "C" fn netio_unix_send(nud: *mut c_void, pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    let nud: *mut netio_unix_desc_t = nud.cast::<_>();
    libc::sendto((*nud).fd, pkt, pkt_len, 0, addr_of!((*nud).remote_sock).cast::<_>(), size_of::<libc::sockaddr_un>() as libc::socklen_t)
}

/// Receive a packet from an UNIX socket
unsafe extern "C" fn netio_unix_recv(nud: *mut c_void, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    let nud: *mut netio_unix_desc_t = nud.cast::<_>();
    libc::recvfrom((*nud).fd, pkt, max_len, 0, null_mut(), null_mut())
}

/// Save the NIO configuration
unsafe extern "C" fn netio_unix_save_cfg(nio: *mut netio_desc_t, fd: *mut libc::FILE) {
    let nud: *mut netio_unix_desc_t = (*nio).dptr.cast::<_>();
    libc::fprintf(fd, cstr!("nio create_unix %s %s %s\n"), (*nio).name, (*nud).local_filename, (*nud).remote_sock.sun_path.as_c());
}

/// Create a new NetIO descriptor with UNIX method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_unix(nio_name: *mut c_char, local: *mut c_char, remote: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    if netio_unix_create(addr_of_mut!((*nio).u.nud), local, remote) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_UNIX;
    (*nio).send = Some(netio_unix_send);
    (*nio).recv = Some(netio_unix_recv);
    (*nio).free = Some(netio_unix_free);
    (*nio).save_cfg = Some(netio_unix_save_cfg);
    (*nio).dptr = addr_of_mut!((*nio).u.nud).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// VDE (Virtual Distributed Ethernet) interface
// =========================================================================

/// Free a NetIO VDE descriptor
unsafe extern "C" fn netio_vde_free(nvd: *mut c_void) {
    let nvd: *mut netio_vde_desc_t = nvd.cast::<_>();
    if (*nvd).data_fd != -1 {
        libc::close((*nvd).data_fd);
    }

    if (*nvd).ctrl_fd != -1 {
        libc::close((*nvd).ctrl_fd);
    }

    if !(*nvd).local_filename.is_null() {
        libc::unlink((*nvd).local_filename);
        libc::free((*nvd).local_filename.cast::<_>());
    }
}

/// Create a new NetIO VDE descriptor
unsafe fn netio_vde_create(nvd: *mut netio_vde_desc_t, control: *mut c_char, local: *mut c_char) -> c_int {
    let mut ctrl_sock: libc::sockaddr_un = zeroed::<_>();
    let mut tst: libc::sockaddr_un = zeroed::<_>();
    let mut req: vde_request_v3 = zeroed::<_>();
    let mut len: ssize_t;

    libc::memset(nvd.cast::<_>(), 0, size_of::<netio_vde_desc_t>());
    (*nvd).ctrl_fd = -1;
    (*nvd).data_fd = -1;

    if (libc::strlen(control) >= ctrl_sock.sun_path.len()) || (libc::strlen(local) >= (*nvd).remote_sock.sun_path.len()) {
        libc::fprintf(c_stderr(), cstr!("netio_vde_create: bad filenames specified\n"));
        return -1;
    }

    // Copy the local filename
    (*nvd).local_filename = libc::strdup(local);
    if (*nvd).local_filename.is_null() {
        libc::fprintf(c_stderr(), cstr!("netio_vde_create: insufficient memory\n"));
        return -1;
    }

    // Connect to the VDE switch controller
    (*nvd).ctrl_fd = libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0);
    if (*nvd).ctrl_fd < 0 {
        libc::perror(cstr!("netio_vde_create: socket(control)"));
        return -1;
    }

    libc::memset(addr_of_mut!(ctrl_sock).cast::<_>(), 0, size_of::<libc::sockaddr_un>());
    ctrl_sock.sun_family = libc::AF_UNIX as libc::sa_family_t;
    libc::strcpy(ctrl_sock.sun_path.as_c_mut(), control);

    let res: c_int = libc::connect((*nvd).ctrl_fd, addr_of!(ctrl_sock).cast::<_>(), size_of::<libc::sockaddr_un>() as libc::socklen_t);

    if res < 0 {
        libc::perror(cstr!("netio_vde_create: connect(control)"));
        return -1;
    }

    tst.sun_family = libc::AF_UNIX as libc::sa_family_t;
    libc::strcpy(tst.sun_path.as_c_mut(), local);

    // Create the data connection
    (*nvd).data_fd = libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM, 0);
    if (*nvd).data_fd < 0 {
        libc::perror(cstr!("netio_vde_create: socket(data)"));
        return -1;
    }

    if libc::bind((*nvd).data_fd, addr_of!(tst).cast::<_>(), size_of::<libc::sockaddr_un>() as libc::socklen_t) < 0 {
        libc::perror(cstr!("netio_vde_create: bind(data)"));
        return -1;
    }

    // Now, process to registration
    libc::memset(addr_of_mut!(req).cast::<_>(), 0, size_of::<vde_request_v3>());
    req.sock.sun_family = libc::AF_UNIX as libc::sa_family_t;
    libc::strcpy(req.sock.sun_path.as_c_mut(), local);
    req.magic = VDE_SWITCH_MAGIC;
    req.version = VDE_SWITCH_VERSION;
    req.r#type = vde_request_type::VDE_REQ_NEW_CONTROL;

    len = libc::write((*nvd).ctrl_fd, addr_of!(req).cast::<_>(), size_of::<vde_request_v3>());
    if len != size_of::<vde_request_v3>() as ssize_t {
        libc::perror(cstr!("netio_vde_create: write(req)"));
        return -1;
    }

    // Read the remote socket descriptor
    len = libc::read((*nvd).ctrl_fd, addr_of_mut!((*nvd).remote_sock).cast::<_>(), size_of::<libc::sockaddr_un>());
    if len != size_of::<libc::sockaddr_un>() as ssize_t {
        libc::perror(cstr!("netio_vde_create: read(req)"));
        return -1;
    }

    0
}

/// Send a packet to a VDE data socket
unsafe extern "C" fn netio_vde_send(nvd: *mut c_void, pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    let nvd: *mut netio_vde_desc_t = nvd.cast::<_>();
    libc::sendto((*nvd).data_fd, pkt, pkt_len, 0, addr_of!((*nvd).remote_sock).cast::<_>(), size_of::<libc::sockaddr_un>() as libc::socklen_t)
}

/// Receive a packet from a VDE socket
unsafe extern "C" fn netio_vde_recv(nvd: *mut c_void, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    let nvd: *mut netio_vde_desc_t = nvd.cast::<_>();
    libc::recvfrom((*nvd).data_fd, pkt, max_len, 0, null_mut(), null_mut())
}

/// Save the NIO configuration
unsafe extern "C" fn netio_vde_save_cfg(nio: *mut netio_desc_t, fd: *mut libc::FILE) {
    let nvd: *mut netio_vde_desc_t = (*nio).dptr.cast::<_>();
    libc::fprintf(fd, cstr!("nio create_vde %s %s %s\n"), (*nio).name, (*nvd).remote_sock.sun_path.as_c(), (*nvd).local_filename);
}

/// Create a new NetIO descriptor with VDE method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_vde(nio_name: *mut c_char, control: *mut c_char, local: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    let nvd: *mut netio_vde_desc_t = addr_of_mut!((*nio).u.nvd);

    if netio_vde_create(nvd, control, local) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_VDE;
    (*nio).send = Some(netio_vde_send);
    (*nio).recv = Some(netio_vde_recv);
    (*nio).free = Some(netio_vde_free);
    (*nio).save_cfg = Some(netio_vde_save_cfg);
    (*nio).dptr = addr_of_mut!((*nio).u.nvd).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// TAP devices
// =========================================================================

/// Free a NetIO TAP descriptor
unsafe extern "C" fn netio_tap_free(ntd: *mut c_void) {
    let ntd: *mut netio_tap_desc_t = ntd.cast::<_>();
    if (*ntd).fd != -1 {
        libc::close((*ntd).fd);
    }
}

/// Open a TAP device
unsafe fn netio_tap_open(tap_devname: *mut c_char) -> c_int {
    #[cfg(target_os = "linux")]
    unsafe fn netio_tap_open_linux(tap_devname: *mut c_char) -> c_int {
        let mut ifr: libc::ifreq = zeroed::<_>();

        let fd: c_int = libc::open(cstr!("/dev/net/tun"), libc::O_RDWR);
        if fd < 0 {
            return -1;
        }

        libc::memset(addr_of_mut!(ifr).cast::<_>(), 0, size_of::<libc::ifreq>());

        // Flags: IFF_TUN   - TUN device (no Ethernet headers)
        //        IFF_TAP   - TAP device
        //
        //        IFF_NO_PI - Do not provide packet information
        ifr.ifr_ifru.ifru_flags = (libc::IFF_TAP | libc::IFF_NO_PI) as c_short;
        if *tap_devname != 0 {
            libc::strncpy(ifr.ifr_name.as_c_mut(), tap_devname, libc::IFNAMSIZ - 1);
            ifr.ifr_name[libc::IFNAMSIZ - 1] = 0;
        }

        let err: c_int = libc::ioctl(fd, linux_raw_sys::ioctl::TUNSETIFF as _, addr_of!(ifr));
        if err < 0 {
            libc::close(fd);
            return err;
        }

        libc::strcpy(tap_devname, ifr.ifr_name.as_c());
        fd
    }
    unsafe fn netio_tap_open_not_linux(tap_devname: *mut c_char) -> c_int {
        let mut fd: c_int = -1;
        let mut tap_fullname: [c_char; NETIO_DEV_MAXLEN] = [0; NETIO_DEV_MAXLEN];

        if *tap_devname != 0 {
            libc::snprintf(tap_fullname.as_c_mut(), NETIO_DEV_MAXLEN, cstr!("/dev/%s"), tap_devname);
            fd = libc::open(tap_fullname.as_c(), libc::O_RDWR);
        } else {
            for i in 0..16 {
                libc::snprintf(tap_devname, NETIO_DEV_MAXLEN, cstr!("/dev/tap%d"), i);

                fd = libc::open(tap_devname, libc::O_RDWR);
                if fd >= 0 {
                    break;
                }
            }
        }

        fd
    }
    #[cfg(target_os = "linux")]
    {
        netio_tap_open_linux(tap_devname)
    }
    #[cfg(not(target_os = "linux"))]
    {
        netio_tap_open_not_linux(tap_devname)
    }
}

/// Allocate a new NetIO TAP descriptor
unsafe fn netio_tap_create(ntd: *mut netio_tap_desc_t, tap_name: *mut c_char) -> c_int {
    if libc::strlen(tap_name) >= NETIO_DEV_MAXLEN {
        libc::fprintf(c_stderr(), cstr!("netio_tap_create: bad TAP device string specified.\n"));
        return -1;
    }

    libc::memset(ntd.cast::<_>(), 0, size_of::<netio_tap_desc_t>());
    libc::strcpy((*ntd).filename.as_c_mut(), tap_name);
    (*ntd).fd = netio_tap_open((*ntd).filename.as_c_mut());

    if (*ntd).fd == -1 {
        libc::fprintf(c_stderr(), cstr!("netio_tap_create: unable to open TAP device %s (%s)\n"), tap_name, libc::strerror(c_errno()));
        return -1;
    }

    0
}

/// Send a packet to a TAP device
unsafe extern "C" fn netio_tap_send(ntd: *mut c_void, pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    let ntd: *mut netio_tap_desc_t = ntd.cast::<_>();
    libc::write((*ntd).fd, pkt, pkt_len)
}

/// Receive a packet through a TAP device
unsafe extern "C" fn netio_tap_recv(ntd: *mut c_void, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    let ntd: *mut netio_tap_desc_t = ntd.cast::<_>();
    libc::read((*ntd).fd, pkt, max_len)
}

/// Save the NIO configuration
unsafe extern "C" fn netio_tap_save_cfg(nio: *mut netio_desc_t, fd: *mut libc::FILE) {
    let ntd: *mut netio_tap_desc_t = (*nio).dptr.cast::<_>();
    libc::fprintf(fd, cstr!("nio create_tap %s %s\n"), (*nio).name, (*ntd).filename);
}

/// Create a new NetIO descriptor with TAP method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_tap(nio_name: *mut c_char, tap_name: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    let ntd: *mut netio_tap_desc_t = addr_of_mut!((*nio).u.ntd);

    if netio_tap_create(ntd, tap_name) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_TAP;
    (*nio).send = Some(netio_tap_send);
    (*nio).recv = Some(netio_tap_recv);
    (*nio).free = Some(netio_tap_free);
    (*nio).save_cfg = Some(netio_tap_save_cfg);
    (*nio).dptr = addr_of_mut!((*nio).u.ntd).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// TCP sockets
// =========================================================================

/// Free a NetIO TCP descriptor
unsafe extern "C" fn netio_tcp_free(nid: *mut c_void) {
    let nid: *mut netio_inet_desc_t = nid.cast::<_>();
    if (*nid).fd != -1 {
        libc::close((*nid).fd);
    }
}

// very simple protocol to send packets over tcp
// 32 bits in network format - size of packet, then packet itself and so on.
unsafe extern "C" fn netio_tcp_send(nid: *mut c_void, pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    let nid: *mut netio_inet_desc_t = nid.cast::<_>();
    let l: m_uint32_t = htonl(pkt_len as m_uint32_t);

    if libc::write((*nid).fd, addr_of!(l).cast::<_>(), size_of::<m_uint32_t>()) == -1 {
        return -1;
    }

    libc::write((*nid).fd, pkt, pkt_len)
}

unsafe extern "C" fn netio_tcp_recv(nid: *mut c_void, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    let nid: *mut netio_inet_desc_t = nid.cast::<_>();
    let mut l: m_uint32_t = 0;

    if libc::read((*nid).fd, addr_of_mut!(l).cast::<_>(), size_of::<m_uint32_t>()) != size_of::<m_uint32_t>() as ssize_t {
        return -1;
    }

    if ntohl(l) as size_t > max_len {
        return -1;
    }

    libc::read((*nid).fd, pkt, ntohl(l) as size_t)
}

unsafe fn netio_tcp_cli_create(nid: *mut netio_inet_desc_t, host: *mut c_char, port: *mut c_char) -> c_int {
    let mut serv: libc::sockaddr_in = zeroed::<_>();
    let sp: *mut libc::servent;
    let hp: *mut libc::hostent;

    (*nid).fd = libc::socket(libc::PF_INET, libc::SOCK_STREAM, 0);
    if (*nid).fd < 0 {
        libc::perror(cstr!("netio_tcp_cli_create: socket"));
        return -1;
    }

    libc::memset(addr_of_mut!(serv).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
    serv.sin_family = libc::AF_INET as libc::sa_family_t;

    if libc::atoi(port) == 0 {
        sp = libc::getservbyname(port, cstr!("tcp"));
        if sp.is_null() {
            libc::fprintf(c_stderr(), cstr!("netio_tcp_cli_create: port %s is neither number not service %s\n"), port, libc::strerror(c_errno()));
            libc::close((*nid).fd);
            return -1;
        }
        serv.sin_port = (*sp).s_port as u16;
    } else {
        serv.sin_port = htons(libc::atoi(port) as u16);
    }

    if inet_addr(host) == libc::INADDR_NONE {
        hp = gethostbyname(host);
        if hp.is_null() {
            libc::fprintf(c_stderr(), cstr!("netio_tcp_cli_create: no host %s\n"), host);
            libc::close((*nid).fd);
            return -1;
        }
        serv.sin_addr.s_addr = *(*(*hp).h_addr_list).cast::<u32>();
    } else {
        serv.sin_addr.s_addr = inet_addr(host);
    }

    if libc::connect((*nid).fd, addr_of!(serv).cast::<_>(), size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0 {
        libc::fprintf(c_stderr(), cstr!("netio_tcp_cli_create: connect to %s:%s failed %s\n"), host, port, libc::strerror(c_errno()));
        libc::close((*nid).fd);
        return -1;
    }
    0
}

/// Create a new NetIO descriptor with TCP_CLI method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_tcp_cli(nio_name: *mut c_char, host: *mut c_char, port: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    if netio_tcp_cli_create(addr_of_mut!((*nio).u.nid), host, port) < 0 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_TCP_CLI;
    (*nio).send = Some(netio_tcp_send);
    (*nio).recv = Some(netio_tcp_recv);
    (*nio).free = Some(netio_tcp_free);
    (*nio).dptr = addr_of_mut!((*nio).u.nid).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

unsafe fn netio_tcp_ser_create(nid: *mut netio_inet_desc_t, port: *mut c_char) -> c_int {
    let mut serv: libc::sockaddr_in = zeroed::<_>();
    let sp: *mut libc::servent;

    let sock_fd: c_int = libc::socket(libc::PF_INET, libc::SOCK_STREAM, 0);
    if sock_fd < 0 {
        libc::perror(cstr!("netio_tcp_cli_create: socket\n"));
        return -1;
    }

    libc::memset(addr_of_mut!(serv).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
    serv.sin_family = libc::AF_INET as libc::sa_family_t;
    serv.sin_addr.s_addr = htonl(libc::INADDR_ANY);

    if libc::atoi(port) == 0 {
        sp = libc::getservbyname(port, cstr!("tcp"));
        if sp.is_null() {
            libc::fprintf(c_stderr(), cstr!("netio_tcp_ser_create: port %s is neither number not service %s\n"), port, libc::strerror(c_errno()));
            libc::close(sock_fd);
            return -1;
        }
        serv.sin_port = (*sp).s_port as u16;
    } else {
        serv.sin_port = htons(libc::atoi(port) as u16);
    }

    if libc::bind(sock_fd, addr_of!(serv).cast::<_>(), size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0 {
        libc::fprintf(c_stderr(), cstr!("netio_tcp_ser_create: bind %s failed %s\n"), port, libc::strerror(c_errno()));
        libc::close(sock_fd);
        return -1;
    }

    if libc::listen(sock_fd, 1) < 0 {
        libc::fprintf(c_stderr(), cstr!("netio_tcp_ser_create: listen %s failed %s\n"), port, libc::strerror(c_errno()));
        libc::close(sock_fd);
        return -1;
    }

    libc::fprintf(c_stderr(), cstr!("Waiting connection on port %s...\n"), port);

    (*nid).fd = libc::accept(sock_fd, null_mut(), null_mut());
    if (*nid).fd < 0 {
        libc::fprintf(c_stderr(), cstr!("netio_tcp_ser_create: accept %s failed %s\n"), port, libc::strerror(c_errno()));
        libc::close(sock_fd);
        return -1;
    }

    libc::fprintf(c_stderr(), cstr!("Connected\n"));

    libc::close(sock_fd);
    0
}

/// Create a new NetIO descriptor with TCP_SER method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_tcp_ser(nio_name: *mut c_char, port: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    if netio_tcp_ser_create(addr_of_mut!((*nio).u.nid), port) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_TCP_SER;
    (*nio).send = Some(netio_tcp_send);
    (*nio).recv = Some(netio_tcp_recv);
    (*nio).free = Some(netio_tcp_free);
    (*nio).dptr = addr_of_mut!((*nio).u.nid).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// UDP sockets
// =========================================================================

/// Free a NetIO UDP descriptor
unsafe extern "C" fn netio_udp_free(nid: *mut c_void) {
    let nid: *mut netio_inet_desc_t = nid.cast::<_>();
    if !(*nid).remote_host.is_null() {
        libc::free((*nid).remote_host.cast::<_>());
        (*nid).remote_host = null_mut();
    }

    if (*nid).fd != -1 {
        libc::close((*nid).fd);
    }
}

/// Send a packet to an UDP socket
unsafe extern "C" fn netio_udp_send(nid: *mut c_void, pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    let nid: *mut netio_inet_desc_t = nid.cast::<_>();
    libc::send((*nid).fd, pkt, pkt_len, 0)
}

/// Receive a packet from an UDP socket
unsafe extern "C" fn netio_udp_recv(nid: *mut c_void, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    let nid: *mut netio_inet_desc_t = nid.cast::<_>();
    libc::recvfrom((*nid).fd, pkt, max_len, 0, null_mut(), null_mut())
}

/// Save the NIO configuration
unsafe extern "C" fn netio_udp_save_cfg(nio: *mut netio_desc_t, fd: *mut libc::FILE) {
    let nid: *mut netio_inet_desc_t = (*nio).dptr.cast::<_>();
    libc::fprintf(fd, cstr!("nio create_udp %s %d %s %d\n"), (*nio).name, (*nid).local_port, (*nid).remote_host, (*nid).remote_port);
}

/// Create a new NetIO descriptor with UDP method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_udp(nio_name: *mut c_char, local_port: c_int, remote_host: *mut c_char, remote_port: c_int) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    let nid: *mut netio_inet_desc_t = addr_of_mut!((*nio).u.nid);
    (*nid).local_port = local_port;
    (*nid).remote_port = remote_port;

    (*nid).remote_host = libc::strdup(remote_host);
    if nid.is_null() {
        libc::fprintf(c_stderr(), cstr!("netio_desc_create_udp: insufficient memory\n"));
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nid).fd = udp_connect(local_port, remote_host, remote_port);
    if (*nid).fd < 0 {
        libc::fprintf(c_stderr(), cstr!("netio_desc_create_udp: unable to connect to %s:%d\n"), remote_host, remote_port);
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_UDP;
    (*nio).send = Some(netio_udp_send);
    (*nio).recv = Some(netio_udp_recv);
    (*nio).free = Some(netio_udp_free);
    (*nio).save_cfg = Some(netio_udp_save_cfg);
    (*nio).dptr = addr_of_mut!((*nio).u.nid).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// UDP sockets with auto allocation
// =========================================================================

/// Get local port
#[no_mangle]
pub unsafe extern "C" fn netio_udp_auto_get_local_port(nio: *mut netio_desc_t) -> c_int {
    if (*nio).r#type != NETIO_TYPE_UDP_AUTO {
        return -1;
    }

    (*nio).u.nid.local_port
}

/// Connect to a remote host/port
#[no_mangle]
pub unsafe extern "C" fn netio_udp_auto_connect(nio: *mut netio_desc_t, host: *mut c_char, port: c_int) -> c_int {
    let nid: *mut netio_inet_desc_t = (*nio).dptr.cast::<_>();

    // NIO already connected
    if (*nid).remote_host.is_null() {
        return -1;
    }

    (*nid).remote_host = libc::strdup(host);
    if (*nid).remote_host.is_null() {
        libc::fprintf(c_stderr(), cstr!("netio_desc_create_udp_auto: insufficient memory\n"));
        return -1;
    }

    (*nid).remote_port = port;

    if ip_connect_fd((*nid).fd, (*nid).remote_host, (*nid).remote_port) < 0 {
        libc::free((*nid).remote_host.cast::<_>());
        (*nid).remote_host = null_mut();
        return -1;
    }

    0
}

/// Create a new NetIO descriptor with auto UDP method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_udp_auto(nio_name: *mut c_char, local_addr: *mut c_char, port_start: c_int, port_end: c_int) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    let nid: *mut netio_inet_desc_t = addr_of_mut!((*nio).u.nid);
    (*nid).local_port = -1;
    (*nid).remote_host = null_mut();
    (*nid).remote_port = -1;

    (*nid).fd = udp_listen_range(local_addr, port_start, port_end, addr_of_mut!((*nid).local_port));
    if (*nid).fd < 0 {
        libc::fprintf(c_stderr(), cstr!("netio_desc_create_udp_auto: unable to create socket (addr=%s,port_start=%d,port_end=%d)\n"), local_addr, port_start, port_end);
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_UDP_AUTO;
    (*nio).send = Some(netio_udp_send);
    (*nio).recv = Some(netio_udp_recv);
    (*nio).free = Some(netio_udp_free);
    (*nio).save_cfg = Some(netio_udp_save_cfg);
    (*nio).dptr = addr_of_mut!((*nio).u.nid).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// Linux RAW Ethernet driver
// =========================================================================

/// Free a NetIO raw ethernet descriptor
#[cfg(feature = "ENABLE_LINUX_ETH")]
unsafe extern "C" fn netio_lnxeth_free(nled: *mut c_void) {
    let nled: *mut netio_lnxeth_desc_t = nled.cast::<_>();
    if (*nled).fd != -1 {
        libc::close((*nled).fd);
    }
}

/// Send a packet to a raw Ethernet socket
#[cfg(feature = "ENABLE_LINUX_ETH")]
unsafe extern "C" fn netio_lnxeth_send(nled: *mut c_void, pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    let nled: *mut netio_lnxeth_desc_t = nled.cast::<_>();
    lnx_eth_send((*nled).fd, (*nled).dev_id, pkt.cast::<_>(), pkt_len)
}

/// Receive a packet from an raw Ethernet socket
#[cfg(feature = "ENABLE_LINUX_ETH")]
unsafe extern "C" fn netio_lnxeth_recv(nled: *mut c_void, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    let nled: *mut netio_lnxeth_desc_t = nled.cast::<_>();
    lnx_eth_recv((*nled).fd, pkt.cast::<_>(), max_len)
}

/// Save the NIO configuration
#[cfg(feature = "ENABLE_LINUX_ETH")]
unsafe extern "C" fn netio_lnxeth_save_cfg(nio: *mut netio_desc_t, fd: *mut libc::FILE) {
    let nled: *mut netio_lnxeth_desc_t = (*nio).dptr.cast::<_>();
    libc::fprintf(fd, cstr!("nio create_linux_eth %s %s\n"), (*nio).name, (*nled).dev_name);
}

/// Create a new NetIO descriptor with raw Ethernet method
#[cfg(feature = "ENABLE_LINUX_ETH")]
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_lnxeth(nio_name: *mut c_char, dev_name: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    let nled: *mut netio_lnxeth_desc_t = addr_of_mut!((*nio).u.nled);

    if libc::strlen(dev_name) >= NETIO_DEV_MAXLEN {
        libc::fprintf(c_stderr(), cstr!("netio_desc_create_lnxeth: bad Ethernet device string specified.\n"));
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    libc::strcpy((*nled).dev_name.as_c_mut(), dev_name);

    (*nled).fd = lnx_eth_init_socket(dev_name);
    (*nled).dev_id = lnx_eth_get_dev_index(dev_name);

    if (*nled).fd < 0 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_LINUX_ETH;
    (*nio).send = Some(netio_lnxeth_send);
    (*nio).recv = Some(netio_lnxeth_recv);
    (*nio).free = Some(netio_lnxeth_free);
    (*nio).save_cfg = Some(netio_lnxeth_save_cfg);
    (*nio).dptr = addr_of_mut!((*nio).u.nled).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// Generic RAW Ethernet driver
// =========================================================================

/// Free a NetIO raw ethernet descriptor
#[cfg(feature = "ENABLE_GEN_ETH")]
unsafe extern "C" fn netio_geneth_free(nged: *mut c_void) {
    let nged: *mut netio_geneth_desc_t = nged.cast::<_>();
    gen_eth_close((*nged).pcap_dev);
}

/// Send a packet to an Ethernet device
#[cfg(feature = "ENABLE_GEN_ETH")]
unsafe extern "C" fn netio_geneth_send(nged: *mut c_void, pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    let nged: *mut netio_geneth_desc_t = nged.cast::<_>();
    gen_eth_send((*nged).pcap_dev, pkt.cast::<_>(), pkt_len)
}

/// Receive a packet from an Ethernet device
#[cfg(feature = "ENABLE_GEN_ETH")]
unsafe extern "C" fn netio_geneth_recv(nged: *mut c_void, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    let nged: *mut netio_geneth_desc_t = nged.cast::<_>();
    gen_eth_recv((*nged).pcap_dev, pkt.cast::<_>(), max_len)
}

/// Save the NIO configuration
#[cfg(feature = "ENABLE_GEN_ETH")]
unsafe extern "C" fn netio_geneth_save_cfg(nio: *mut netio_desc_t, fd: *mut libc::FILE) {
    let nged: *mut netio_geneth_desc_t = (*nio).dptr.cast::<_>();
    libc::fprintf(fd, cstr!("nio create_gen_eth %s %s\n"), (*nio).name, (*nged).dev_name);
}

/// Create a new NetIO descriptor with generic raw Ethernet method
#[cfg(feature = "ENABLE_GEN_ETH")]
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_geneth(nio_name: *mut c_char, dev_name: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    let nged: *mut netio_geneth_desc_t = addr_of_mut!((*nio).u.nged);

    if libc::strlen(dev_name) >= NETIO_DEV_MAXLEN {
        libc::fprintf(c_stderr(), cstr!("netio_desc_create_geneth: bad Ethernet device string specified.\n"));
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    libc::strcpy((*nged).dev_name.as_c_mut(), dev_name);

    (*nged).pcap_dev = gen_eth_init(dev_name);
    if (*nged).pcap_dev.is_null() {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_GEN_ETH;
    (*nio).send = Some(netio_geneth_send);
    (*nio).recv = Some(netio_geneth_recv);
    (*nio).free = Some(netio_geneth_free);
    (*nio).save_cfg = Some(netio_geneth_save_cfg);
    (*nio).dptr = addr_of_mut!((*nio).u.nged).cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// FIFO Driver (intra-hypervisor communications)
// =========================================================================

/// Extract the first packet of the FIFO
unsafe fn netio_fifo_extract_pkt(nfd: *mut netio_fifo_desc_t) -> *mut netio_fifo_pkt_t {
    let p: *mut netio_fifo_pkt_t = (*nfd).head;
    if p.is_null() {
        return null_mut();
    }

    (*nfd).pkt_count -= 1;
    (*nfd).head = (*p).next;

    if (*nfd).head.is_null() {
        (*nfd).last = null_mut();
    }

    p
}

/// Insert a packet into the FIFO (in tail)
unsafe fn netio_fifo_insert_pkt(nfd: *mut netio_fifo_desc_t, p: *mut netio_fifo_pkt_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*nfd).lock));

    (*nfd).pkt_count += 1;
    (*p).next = null_mut();

    if !(*nfd).last.is_null() {
        (*(*nfd).last).next = p;
    } else {
        (*nfd).head = p;
    }

    (*nfd).last = p;
    libc::pthread_mutex_unlock(addr_of_mut!((*nfd).lock));
}

/// Free the packet list
unsafe fn netio_fifo_free_pkt_list(nfd: *mut netio_fifo_desc_t) {
    let mut p: *mut netio_fifo_pkt_t;
    let mut next: *mut netio_fifo_pkt_t;

    p = (*nfd).head;
    while !p.is_null() {
        next = (*p).next;
        libc::free(p.cast::<_>());
        p = next;
    }

    (*nfd).head = null_mut();
    (*nfd).last = null_mut();
    (*nfd).pkt_count = 0;
}

/// Establish a cross-connect between two FIFO NetIO
#[no_mangle]
pub unsafe extern "C" fn netio_fifo_crossconnect(a: *mut netio_desc_t, b: *mut netio_desc_t) -> c_int {
    if ((*a).r#type != NETIO_TYPE_FIFO) || ((*b).r#type != NETIO_TYPE_FIFO) {
        return -1;
    }

    let pa: *mut netio_fifo_desc_t = addr_of_mut!((*a).u.nfd);
    let pb: *mut netio_fifo_desc_t = addr_of_mut!((*b).u.nfd);

    // A => B
    libc::pthread_mutex_lock(addr_of_mut!((*pa).endpoint_lock));
    libc::pthread_mutex_lock(addr_of_mut!((*pa).lock));
    (*pa).endpoint = pb;
    netio_fifo_free_pkt_list(pa);
    libc::pthread_mutex_unlock(addr_of_mut!((*pa).lock));
    libc::pthread_mutex_unlock(addr_of_mut!((*pa).endpoint_lock));

    // B => A
    libc::pthread_mutex_lock(addr_of_mut!((*pb).endpoint_lock));
    libc::pthread_mutex_lock(addr_of_mut!((*pb).lock));
    (*pb).endpoint = pa;
    netio_fifo_free_pkt_list(pb);
    libc::pthread_mutex_unlock(addr_of_mut!((*pb).lock));
    libc::pthread_mutex_unlock(addr_of_mut!((*pb).endpoint_lock));
    0
}

/// Unbind an endpoint
unsafe fn netio_fifo_unbind_endpoint(nfd: *mut netio_fifo_desc_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*nfd).endpoint_lock));
    (*nfd).endpoint = null_mut();
    libc::pthread_mutex_unlock(addr_of_mut!((*nfd).endpoint_lock));
}

/// Free a NetIO FIFO descriptor
unsafe extern "C" fn netio_fifo_free(nfd: *mut c_void) {
    let nfd: *mut netio_fifo_desc_t = nfd.cast::<_>();
    if !(*nfd).endpoint.is_null() {
        netio_fifo_unbind_endpoint((*nfd).endpoint);
    }

    netio_fifo_free_pkt_list(nfd);
    libc::pthread_mutex_destroy(addr_of_mut!((*nfd).lock));
    libc::pthread_cond_destroy(addr_of_mut!((*nfd).cond));
}

/// Send a packet (to the endpoint FIFO)
unsafe extern "C" fn netio_fifo_send(nfd: *mut c_void, pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    let nfd: *mut netio_fifo_desc_t = nfd.cast::<_>();

    libc::pthread_mutex_lock(addr_of_mut!((*nfd).endpoint_lock));

    // The cross-connect must have been established before
    if (*nfd).endpoint.is_null() {
        libc::pthread_mutex_unlock(addr_of_mut!((*nfd).endpoint_lock));
        return -1;
    }

    // Allocate a a new packet and insert it into the endpoint FIFO
    let len: size_t = size_of::<netio_fifo_pkt_t>() + pkt_len;
    let p: *mut netio_fifo_pkt_t = libc::malloc(len).cast::<_>();
    if p.is_null() {
        libc::pthread_mutex_unlock(addr_of_mut!((*nfd).endpoint_lock));
        return -1;
    }

    libc::memcpy((*p).pkt.as_c_void_mut(), pkt, pkt_len);
    (*p).pkt_len = pkt_len;
    netio_fifo_insert_pkt((*nfd).endpoint, p);
    libc::pthread_cond_signal(addr_of_mut!((*(*nfd).endpoint).cond));
    libc::pthread_mutex_unlock(addr_of_mut!((*nfd).endpoint_lock));
    pkt_len as ssize_t
}

/// Read a packet from the local FIFO queue
unsafe extern "C" fn netio_fifo_recv(nfd: *mut c_void, pkt: *mut c_void, max_len: size_t) -> ssize_t {
    let nfd: *mut netio_fifo_desc_t = nfd.cast::<_>();
    let mut ts: libc::timespec = zeroed::<_>();
    let mut len: size_t = -1 as ssize_t as size_t;

    // Wait for the endpoint to signal a new arriving packet
    let expire: m_tmcnt_t = m_gettime_usec() + 50000;
    ts.tv_sec = (expire / 1000000) as _;
    ts.tv_nsec = ((expire % 1000000) * 1000) as _;

    libc::pthread_mutex_lock(addr_of_mut!((*nfd).lock));
    libc::pthread_cond_timedwait(addr_of_mut!((*nfd).cond), addr_of_mut!((*nfd).lock), addr_of!(ts));

    // Extract a packet from the list
    let p: *mut netio_fifo_pkt_t = netio_fifo_extract_pkt(nfd);
    libc::pthread_mutex_unlock(addr_of_mut!((*nfd).lock));

    if !p.is_null() {
        len = min((*p).pkt_len, max_len);
        libc::memcpy(pkt, (*p).pkt.as_c_void(), len);
        libc::free(p.cast::<_>());
    }

    len as ssize_t
}

/// Create a new NetIO descriptor with FIFO method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_fifo(nio_name: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    let nfd: *mut netio_fifo_desc_t = addr_of_mut!((*nio).u.nfd);
    libc::pthread_mutex_init(addr_of_mut!((*nfd).lock), null_mut());
    libc::pthread_mutex_init(addr_of_mut!((*nfd).endpoint_lock), null_mut());
    libc::pthread_cond_init(addr_of_mut!((*nfd).cond), null_mut());

    (*nio).r#type = NETIO_TYPE_FIFO;
    (*nio).send = Some(netio_fifo_send);
    (*nio).recv = Some(netio_fifo_recv);
    (*nio).free = Some(netio_fifo_free);
    (*nio).dptr = nfd.cast::<_>();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

// =========================================================================
// NULL Driver (does nothing, used for debugging)
// =========================================================================

unsafe extern "C" fn netio_null_send(_null_ptr: *mut c_void, _pkt: *mut c_void, pkt_len: size_t) -> ssize_t {
    pkt_len as ssize_t
}

unsafe extern "C" fn netio_null_recv(_null_ptr: *mut c_void, _pkt: *mut c_void, _max_len: size_t) -> ssize_t {
    libc::usleep(200000);
    -1
}

unsafe extern "C" fn netio_null_save_cfg(nio: *mut netio_desc_t, fd: *mut libc::FILE) {
    libc::fprintf(fd, cstr!("nio create_null %s\n"), (*nio).name);
}

/// Create a new NetIO descriptor with NULL method
#[no_mangle]
pub unsafe extern "C" fn netio_desc_create_null(nio_name: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = netio_create(nio_name);
    if nio.is_null() {
        return null_mut();
    }

    (*nio).r#type = NETIO_TYPE_NULL;
    (*nio).send = Some(netio_null_send);
    (*nio).recv = Some(netio_null_recv);
    (*nio).save_cfg = Some(netio_null_save_cfg);
    (*nio).dptr = null_mut();

    if netio_record(nio) == -1 {
        netio_free(nio.cast::<_>(), null_mut());
        return null_mut();
    }

    nio
}

/// Free a NetIO descriptor
unsafe extern "C" fn netio_free(data: *mut c_void, _arg: *mut c_void) -> c_int {
    let nio: *mut netio_desc_t = data.cast::<_>();

    if !nio.is_null() {
        netio_filter_unbind(nio, NETIO_FILTER_DIR_RX);
        netio_filter_unbind(nio, NETIO_FILTER_DIR_TX);
        netio_filter_unbind(nio, NETIO_FILTER_DIR_BOTH);

        if (*nio).free.is_some() {
            (*nio).free.unwrap()((*nio).dptr);
        }

        libc::free((*nio).name.cast::<_>());
        libc::free(nio.cast::<_>());
    }

    TRUE
}

/// Reset NIO statistics
#[no_mangle]
pub unsafe extern "C" fn netio_reset_stats(nio: *mut netio_desc_t) {
    (*nio).stats_pkts_in = 0;
    (*nio).stats_pkts_out = 0;
    (*nio).stats_bytes_in = 0;
    (*nio).stats_bytes_out = 0;
}

/// Indicate if a NetIO can transmit a packet
#[no_mangle]
pub unsafe extern "C" fn netio_can_transmit(nio: *mut netio_desc_t) -> c_int {
    let mut bw_current: u_int;

    // No bandwidth constraint applied, can always transmit
    if (*nio).bandwidth == 0 {
        return TRUE;
    }

    // Check that we verify the bandwidth constraint
    bw_current = ((*nio).bw_cnt_total * 8 * 1000) as u_int;
    bw_current /= (1024 * NETIO_BW_SAMPLE_ITV * NETIO_BW_SAMPLES) as u_int;

    (bw_current < (*nio).bandwidth).into()
}

/// Update bandwidth counter
#[no_mangle]
pub unsafe extern "C" fn netio_update_bw_stat(nio: *mut netio_desc_t, bytes: m_uint64_t) {
    (*nio).bw_cnt[(*nio).bw_pos as usize] += bytes;
    (*nio).bw_cnt_total += bytes;
}

/// Reset NIO bandwidth counter
#[no_mangle]
pub unsafe extern "C" fn netio_clear_bw_stat(nio: *mut netio_desc_t) {
    (*nio).bw_ptask_cnt += 1;
    if (*nio).bw_ptask_cnt == NETIO_BW_SAMPLE_ITV as u_int / ptask_sleep_time {
        (*nio).bw_ptask_cnt = 0;

        (*nio).bw_pos += 1;
        if (*nio).bw_pos == NETIO_BW_SAMPLES as u_int {
            (*nio).bw_pos = 0;
        }

        (*nio).bw_cnt_total -= (*nio).bw_cnt[(*nio).bw_pos as usize];
        (*nio).bw_cnt[(*nio).bw_pos as usize] = 0;
    }
}

/// Set the bandwidth constraint
#[no_mangle]
pub unsafe extern "C" fn netio_set_bandwidth(nio: *mut netio_desc_t, bandwidth: u_int) {
    (*nio).bandwidth = bandwidth;
}

// =========================================================================
// RX Listeners
// =========================================================================

/// Find a RX listener
#[inline]
unsafe fn netio_rxl_find(nio: *mut netio_desc_t) -> *mut netio_rx_listener {
    let mut rxl: *mut netio_rx_listener = netio_rxl_list;

    while !rxl.is_null() {
        if (*rxl).nio == nio {
            return rxl;
        }
        rxl = (*rxl).next;
    }

    null_mut()
}

/// Remove a NIO from the listener list
unsafe fn netio_rxl_remove_internal(nio: *mut netio_desc_t) -> c_int {
    let mut res: c_int = -1;

    let rxl: *mut netio_rx_listener = netio_rxl_find(nio);
    if !rxl.is_null() {
        // we suppress this NIO only when the ref count hits 0
        (*rxl).ref_count -= 1;

        if (*rxl).ref_count == 0 {
            // remove this listener from the double linked list
            if !(*rxl).next.is_null() {
                (*(*rxl).next).prev = (*rxl).prev;
            }

            if !(*rxl).prev.is_null() {
                (*(*rxl).prev).next = (*rxl).next;
            } else {
                netio_rxl_list = (*rxl).next;
            }

            // if this is non-FD NIO, wait for thread to terminate
            if netio_get_fd((*rxl).nio) == -1 {
                (*rxl).running.set(FALSE);
                libc::pthread_join((*rxl).spec_thread, null_mut());
            }

            libc::free(rxl.cast::<_>());
        }

        res = 0;
    }

    res
}

/// Add a RXL listener to the listener list
unsafe fn netio_rxl_add_internal(rxl: *mut netio_rx_listener) {
    let tmp: *mut netio_rx_listener = netio_rxl_find((*rxl).nio);
    if !tmp.is_null() {
        (*tmp).ref_count += 1;
        libc::free(rxl.cast::<_>());
    } else {
        (*rxl).prev = null_mut();
        (*rxl).next = netio_rxl_list;
        if !(*rxl).next.is_null() {
            (*(*rxl).next).prev = rxl;
        }
        netio_rxl_list = rxl;
    }
}

/// RX Listener dedicated thread (for non-FD NIO)
extern "C" fn netio_rxl_spec_thread(arg: *mut c_void) -> *mut c_void {
    unsafe {
        let rxl: *mut netio_rx_listener = arg.cast::<_>();
        let nio: *mut netio_desc_t = (*rxl).nio;

        while (*rxl).running.get() != 0 {
            let pkt_len: ssize_t = netio_recv(nio, (*nio).rx_pkt.as_c_void_mut(), (*nio).rx_pkt.len());

            if pkt_len > 0 {
                (*rxl).rx_handler.unwrap()(nio, (*nio).rx_pkt.as_c_mut(), pkt_len, (*rxl).arg1, (*rxl).arg2);
            }
        }

        null_mut()
    }
}

/// RX Listener General Thread
extern "C" fn netio_rxl_gen_thread(_arg: *mut c_void) -> *mut c_void {
    unsafe {
        let mut rxl: *mut netio_rx_listener;
        let mut pkt_len: ssize_t;
        let mut nio: *mut netio_desc_t;
        let mut tv: libc::timeval = zeroed::<_>();
        let mut fd: c_int;
        let mut fd_max: c_int;
        let mut res: c_int;
        let mut rfds: libc::fd_set = zeroed::<_>();

        loop {
            NETIO_RXL_LOCK();

            NETIO_RXQ_LOCK();
            // Add the new waiting NIO to the active list
            while !netio_rxl_add_list.is_null() {
                rxl = netio_rxl_add_list;
                netio_rxl_add_list = (*netio_rxl_add_list).next;
                netio_rxl_add_internal(rxl);
            }

            // Delete the NIO present in the remove list
            while !netio_rxl_remove_list.is_null() {
                nio = netio_rxl_remove_list;
                netio_rxl_remove_list = (*netio_rxl_remove_list).rxl_next;
                netio_rxl_remove_internal(nio);
            }

            libc::pthread_cond_broadcast(addr_of_mut!(netio_rxl_cond));
            NETIO_RXQ_UNLOCK();

            // Build the FD set
            libc::FD_ZERO(addr_of_mut!(rfds));
            fd_max = -1;
            rxl = netio_rxl_list;
            while !rxl.is_null() {
                fd = netio_get_fd((*rxl).nio);
                if fd == -1 {
                    rxl = (*rxl).next;
                    continue;
                }

                if fd > fd_max {
                    fd_max = fd;
                }
                libc::FD_SET(fd, addr_of_mut!(rfds));
                rxl = (*rxl).next;
            }
            NETIO_RXL_UNLOCK();

            // Wait for incoming packets
            tv.tv_sec = 0;
            tv.tv_usec = 20 * 1000; // 200 ms
            res = libc::select(fd_max + 1, addr_of_mut!(rfds), null_mut(), null_mut(), addr_of_mut!(tv));

            if res == -1 {
                if c_errno() != libc::EINTR {
                    libc::perror(cstr!("netio_rxl_thread: select"));
                }
                continue;
            }

            // Examine active FDs and call user handlers
            NETIO_RXL_LOCK();

            rxl = netio_rxl_list;
            while !rxl.is_null() {
                nio = (*rxl).nio;

                fd = netio_get_fd(nio);
                if fd == -1 {
                    rxl = (*rxl).next;
                    continue;
                }

                if libc::FD_ISSET(fd, addr_of!(rfds)) {
                    pkt_len = netio_recv(nio, (*nio).rx_pkt.as_c_void_mut(), (*nio).rx_pkt.len());

                    if pkt_len > 0 {
                        (*rxl).rx_handler.unwrap()(nio, (*nio).rx_pkt.as_c_mut(), pkt_len, (*rxl).arg1, (*rxl).arg2);
                    }
                }
                rxl = (*rxl).next;
            }

            NETIO_RXL_UNLOCK();
        }
    }
}

/// Add a RX listener in the listener list
#[no_mangle]
pub unsafe extern "C" fn netio_rxl_add(nio: *mut netio_desc_t, rx_handler: netio_rx_handler_t, arg1: *mut c_void, arg2: *mut c_void) -> c_int {
    NETIO_RXQ_LOCK();

    let rxl: *mut netio_rx_listener = libc::malloc(size_of::<netio_rx_listener>()).cast::<_>();
    if rxl.is_null() {
        NETIO_RXQ_UNLOCK();
        libc::fprintf(c_stderr(), cstr!("netio_rxl_add: unable to create structure.\n"));
        return -1;
    }

    libc::memset(rxl.cast::<_>(), 0, size_of::<netio_rx_listener>());
    (*rxl).nio = nio;
    (*rxl).ref_count = 1;
    (*rxl).rx_handler = rx_handler;
    (*rxl).arg1 = arg1;
    (*rxl).arg2 = arg2;
    (*rxl).running.set(TRUE);

    if (netio_get_fd((*rxl).nio) == -1) && libc::pthread_create(addr_of_mut!((*rxl).spec_thread), null_mut(), netio_rxl_spec_thread, rxl.cast::<_>()) != 0 {
        NETIO_RXQ_UNLOCK();
        libc::fprintf(c_stderr(), cstr!("netio_rxl_add: unable to create specific thread.\n"));
        libc::free(rxl.cast::<_>());
        return -1;
    }

    (*rxl).next = netio_rxl_add_list;
    netio_rxl_add_list = rxl;
    while !netio_rxl_add_list.is_null() {
        libc::pthread_cond_wait(addr_of_mut!(netio_rxl_cond), addr_of_mut!(netio_rxq_mutex));
    }
    NETIO_RXQ_UNLOCK();
    0
}

/// Remove a NIO from the listener list
#[no_mangle]
pub unsafe extern "C" fn netio_rxl_remove(nio: *mut netio_desc_t) -> c_int {
    NETIO_RXQ_LOCK();
    (*nio).rxl_next = netio_rxl_remove_list;
    netio_rxl_remove_list = nio;
    while !netio_rxl_remove_list.is_null() {
        libc::pthread_cond_wait(addr_of_mut!(netio_rxl_cond), addr_of_mut!(netio_rxq_mutex));
    }
    NETIO_RXQ_UNLOCK();
    0
}

/// Initialize the RXL thread
#[no_mangle]
pub unsafe extern "C" fn netio_rxl_init() -> c_int {
    libc::pthread_cond_init(addr_of_mut!(netio_rxl_cond), null_mut());

    if libc::pthread_create(addr_of_mut!(netio_rxl_thread), null_mut(), netio_rxl_gen_thread, null_mut()) != 0 {
        libc::perror(cstr!("netio_rxl_init: pthread_create"));
        return -1;
    }

    0
}
