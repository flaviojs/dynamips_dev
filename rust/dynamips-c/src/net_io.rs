//! Cisco router) simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Network Input/Output Abstraction Layer.

use crate::_private::*;
use crate::dynamips_common::*;

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
