//! Network Input/Output Abstraction Layer.

use crate::dynamips_common::*;
use crate::prelude::*;
use crate::registry::*;

extern "C" {
    pub fn netio_free(data: *mut c_void, arg: *mut c_void) -> c_int;
}

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
    pub type_: vde_request_type,
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
    pub pcap_dev: *mut pcap_t,
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
    pub type_: u_int,
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

#[no_mangle]
pub extern "C" fn _export_net_io(
    _: *mut vde_request_v3,
    _: *mut netio_unix_desc_t,
    _: *mut netio_vde_desc_t,
    _: *mut netio_tap_desc_t,
    _: *mut netio_inet_desc_t,
    _: *mut netio_fifo_pkt_t,
    _: *mut netio_fifo_desc_t,
    _: *mut netio_fifo_desc_t,
    _: *mut netio_pktfilter_t,
    _: *mut netio_stat_t,
    _: *mut netio_desc_t,
    _: *mut netio_rx_listener,
) {
}

#[cfg(feature = "ENABLE_LINUX_ETH")]
#[no_mangle]
pub extern "C" fn _export_net_io_linux_eth(_: *mut netio_lnxeth_desc_t) {}

#[cfg(feature = "ENABLE_GEN_ETH")]
#[no_mangle]
pub extern "C" fn _export_net_io_gen_eth(_: *mut netio_geneth_desc_t) {}

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
#[no_mangle] // TODO private
pub unsafe extern "C" fn netio_record(nio: *mut netio_desc_t) -> c_int {
    registry_add((*nio).name, OBJ_TYPE_NIO, nio.cast::<_>())
}

/// Create a new NetIO descriptor
#[no_mangle] // TODO private
pub unsafe extern "C" fn netio_create(name: *mut c_char) -> *mut netio_desc_t {
    let nio: *mut netio_desc_t = libc::malloc(size_of::<netio_desc_t>()).cast::<_>();
    if nio.is_null() {
        return null_mut();
    }

    // setup as a NULL descriptor
    libc::memset(nio.cast::<_>(), 0, size_of::<netio_desc_t>());
    (*nio).type_ = NETIO_TYPE_NULL;

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
