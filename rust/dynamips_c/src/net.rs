//! Network Utility functions.

use crate::dynamips_common::*;
use crate::prelude::*;
use crate::utils::*;

pub type n_eth_addr_t = n_eth_addr;
pub type n_ip_network_t = n_ip_network;
pub type n_ipv6_network_t = n_ipv6_network;
pub type n_eth_hdr_t = n_eth_hdr;
pub type n_eth_dot1q_hdr_t = n_eth_dot1q_hdr;
pub type n_eth_llc_hdr_t = n_eth_llc_hdr;
pub type n_eth_snap_hdr_t = n_eth_snap_hdr;
pub type n_eth_isl_hdr_t = n_eth_isl_hdr;
pub type n_scp_hdr_t = n_scp_hdr;
pub type n_arp_hdr_t = n_arp_hdr;
pub type n_ip_hdr_t = n_ip_hdr;
pub type n_udp_hdr_t = n_udp_hdr;
pub type n_tcp_hdr_t = n_tcp_hdr;
pub type n_pkt_ctx_t = n_pkt_ctx;

#[no_mangle]
pub extern "C" fn _export_net(
    _: *mut n_ip_network_t,
    _: *mut n_ipv6_network,
    _: *mut n_eth_hdr_t,
    _: *mut n_eth_dot1q_hdr_t,
    _: *mut n_eth_llc_hdr_t,
    _: *mut n_eth_snap_hdr_t,
    _: *mut n_eth_isl_hdr_t,
    _: *mut n_scp_hdr_t,
    _: *mut n_arp_hdr_t,
    _: *mut n_ip_hdr_t,
    _: *mut n_udp_hdr_t,
    _: *mut n_tcp_hdr_t,
    _: *mut n_pkt_ctx_t,
) {
}

pub const N_IP_ADDR_LEN: usize = 4;
pub const N_IP_ADDR_BITS: usize = 32;

pub const N_IPV6_ADDR_LEN: usize = 16;
pub const N_IPV6_ADDR_BITS: usize = 128;

/// IPv4 Address definition
pub type n_ip_addr_t = m_uint32_t;

/// IPv6 Address definition
pub type n_ipv6_addr_t = n_ipv6_addr;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct n_ipv6_addr {
    pub ip6: n_ipv6_addr_ip6,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union n_ipv6_addr_ip6 {
    pub u6_addr32: [m_uint32_t; 4],
    pub u6_addr16: [m_uint16_t; 8],
    pub u6_addr8: [m_uint8_t; 16],
}

/// Ethernet Constants
pub const N_ETH_ALEN: usize = 6;

/// Ethernet Address
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct n_eth_addr {
    pub eth_addr_byte: [m_uint8_t; N_ETH_ALEN],
}

/// IP Network definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct n_ip_network {
    pub net_addr: n_ip_addr_t,
    pub net_mask: n_ip_addr_t,
}

/// IPv6 Network definition
#[repr(C)]
#[derive(Copy, Clone)]
pub struct n_ipv6_network {
    pub net_addr: n_ipv6_addr_t,
    pub net_mask: u_int,
}

/// IP header minimum length
pub const N_IP_MIN_HLEN: u_int = 5;

/// IP: Common Protocols
pub const N_IP_PROTO_ICMP: u_int = 1;
pub const N_IP_PROTO_IGMP: u_int = 2;
pub const N_IP_PROTO_TCP: u_int = 6;
pub const N_IP_PROTO_UDP: u_int = 17;
pub const N_IP_PROTO_IPV6: u_int = 41;
pub const N_IP_PROTO_GRE: u_int = 47;
pub const N_IP_PROTO_ESP: u_int = 50;
pub const N_IP_PROTO_AH: u_int = 51;
pub const N_IP_PROTO_ICMPV6: u_int = 58;
pub const N_IP_PROTO_EIGRP: u_int = 88;
pub const N_IP_PROTO_OSPF: u_int = 89;
pub const N_IP_PROTO_PIM: u_int = 103;
pub const N_IP_PROTO_SCTP: u_int = 132;
pub const N_IP_PROTO_MAX: u_int = 256;

pub const N_IP_FLAG_DF: u_int = 0x4000;
pub const N_IP_FLAG_MF: u_int = 0x2000;
pub const N_IP_OFFMASK: u_int = 0x1fff;

/// Maximum number of ports
pub const N_IP_PORT_MAX: usize = 65536;

/// TCP: Header Flags
pub const N_TCP_FIN: u8 = 0x01;
pub const N_TCP_SYN: u8 = 0x02;
pub const N_TCP_RST: u8 = 0x04;
pub const N_TCP_PUSH: u8 = 0x08;
pub const N_TCP_ACK: u8 = 0x10;
pub const N_TCP_URG: u8 = 0x20;

pub const N_TCP_FLAGMASK: u8 = 0x3F;

/// IPv6 Header Codes
pub const N_IPV6_PROTO_ICMP: u8 = 58;
pub const N_IPV6_OPT_HOP_BY_HOP: u8 = 0; // Hop-by-Hop header
pub const N_IPV6_OPT_DST: u8 = 60; // Destination Options Header
pub const N_IPV6_OPT_ROUTE: u8 = 43; // Routing header
pub const N_IPV6_OPT_FRAG: u8 = 44; // Fragment Header
pub const N_IPV6_OPT_AH: u8 = 51; // Authentication Header
pub const N_IPV6_OPT_ESP: u8 = 50; // Encryption Security Payload
pub const N_IPV6_OPT_COMP: u8 = 108; // Payload Compression Protocol
pub const N_IPV6_OPT_END: u8 = 59; // No more headers

/// Standard Ethernet MTU
pub const N_ETH_MTU: u16 = 1500;

/// Ethernet Constants
pub const N_ETH_HLEN: usize = 14; // XXX cbindgen does not support size_of
#[cfg(test)]
#[test]
fn test_N_ETH_HLEN() {
    assert_eq!(N_ETH_HLEN, size_of::<n_eth_hdr_t>());
}

/// CRC Length
pub const N_ETH_CRC_LEN: usize = 4;

/// Minimum size for ethernet payload
pub const N_ETH_MIN_DATA_LEN: usize = 46;
pub const N_ETH_MIN_FRAME_LEN: usize = N_ETH_MIN_DATA_LEN + N_ETH_HLEN;

pub const N_ETH_PROTO_IP: u16 = 0x0800;
pub const N_ETH_PROTO_IPV6: u16 = 0x86DD;
pub const N_ETH_PROTO_ARP: u16 = 0x0806;
pub const N_ETH_PROTO_DOT1Q: u16 = 0x8100;
pub const N_ETH_PROTO_DOT1Q_2: u16 = 0x9100;
pub const N_ETH_PROTO_DOT1Q_3: u16 = 0x9200;
pub const N_ETH_PROTO_DOT1Q_4: u16 = 0x88A8;
pub const N_ETH_PROTO_MPLS: u16 = 0x8847;
pub const N_ETH_PROTO_MPLS_MC: u16 = 0x8848;
pub const N_ETH_PROTO_LOOP: u16 = 0x9000;

/// size needed for a string buffer
pub const N_ETH_SLEN: usize = N_ETH_ALEN * 3;

/// ARP opcodes
pub const N_ARP_REQUEST: u16 = 0x1;
pub const N_ARP_REPLY: u16 = 0x2;

/// Ethernet Header
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct n_eth_hdr {
    pub daddr: n_eth_addr_t, // destination eth addr
    pub saddr: n_eth_addr_t, // source ether addr
    pub type_: m_uint16_t,   // packet type ID field
}

/* 802.1Q Ethernet Header */
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct n_eth_dot1q_hdr {
    pub daddr: n_eth_addr_t, // destination eth addr
    pub saddr: n_eth_addr_t, // source ether addr
    pub type_: m_uint16_t,   // packet type ID field (0x8100)
    pub vlan_id: m_uint16_t, // VLAN id + CoS
}

/// LLC header
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct n_eth_llc_hdr {
    pub dsap: m_uint8_t,
    pub ssap: m_uint8_t,
    pub ctrl: m_uint8_t,
}

/// SNAP header
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct n_eth_snap_hdr {
    pub oui: [m_uint8_t; 3],
    pub type_: m_uint16_t,
}

/* Cisco ISL header */
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct n_eth_isl_hdr {
    pub hsa1: m_uint16_t,  // High bits of source MAC address
    pub hsa2: m_uint8_t,   // (in theory: 0x00-00-0c)
    pub vlan: m_uint16_t,  // VLAN + BPDU
    pub index: m_uint16_t, // Index port of source
    pub res: m_uint16_t,   // Reserved for TokenRing and FDDI
}

pub const N_ISL_HDR_SIZE: usize = 12; // XXX cbindgen does not support size_of
#[cfg(test)]
#[test]
fn test_N_ISL_HDR_SIZE() {
    assert_eq!(N_ISL_HDR_SIZE, size_of::<n_eth_llc_hdr_t>() + size_of::<n_eth_isl_hdr_t>());
}

/// Cisco SCP/RBCP header
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct n_scp_hdr {
    pub sa: m_uint8_t,      // Source Address
    pub da: m_uint8_t,      // Destination Address
    pub len: m_uint16_t,    // Data Length
    pub dsap: m_uint8_t,    // Destination Service Access Point
    pub ssap: m_uint8_t,    // Source Service Access Point
    pub opcode: m_uint16_t, // Opcode
    pub seqno: m_uint16_t,  // Sequence Number
    pub flags: m_uint8_t,   // Flags: command/response
    pub unk1: m_uint8_t,    // Unknown
    pub unk2: m_uint16_t,   // Unknown
    pub unk3: m_uint16_t,   // Unknown
}

/// ----- ARP Header for the IPv4 protocol over Ethernet ------------------
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct n_arp_hdr {
    pub hw_type: m_uint16_t,     // Hardware type
    pub proto_type: m_uint16_t,  // L3 protocol
    pub hw_len: m_uint8_t,       // Length of hardware address
    pub proto_len: m_uint8_t,    // Length of L3 address
    pub opcode: m_uint16_t,      // ARP Opcode
    pub eth_saddr: n_eth_addr_t, // Source hardware address
    pub ip_saddr: m_uint32_t,    // Source IP address
    pub eth_daddr: n_eth_addr_t, // Dest. hardware address
    pub ip_daddr: m_uint32_t,    // Dest. IP address
}

/// ----- IP Header -------------------------------------------------------
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct n_ip_hdr {
    pub ihl: m_uint8_t,
    pub tos: m_uint8_t,
    pub tot_len: m_uint16_t,
    pub id: m_uint16_t,
    pub frag_off: m_uint16_t,
    pub ttl: m_uint8_t,
    pub proto: m_uint8_t,
    pub cksum: m_uint16_t,
    pub saddr: m_uint32_t,
    pub daddr: m_uint32_t,
}

/// ----- UDP Header ------------------------------------------------------
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct n_udp_hdr {
    pub sport: m_uint16_t,
    pub dport: m_uint16_t,
    pub len: m_uint16_t,
    pub cksum: m_uint16_t,
}

/// ----- TCP Header ------------------------------------------------------
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct n_tcp_hdr {
    pub sport: m_uint16_t,
    pub dport: m_uint16_t,
    pub seq: m_uint32_t,
    pub ack_seq: m_uint32_t,
    pub offset: m_uint8_t,
    pub flags: m_uint8_t,
    pub window: m_uint16_t,
    pub cksum: m_uint16_t,
    pub urg_ptr: m_uint16_t,
}

/// ----- Packet Context --------------------------------------------------
pub const N_PKT_CTX_FLAG_ETHV2: m_uint32_t = 0x0001;
pub const N_PKT_CTX_FLAG_VLAN: m_uint32_t = 0x0002;
pub const N_PKT_CTX_FLAG_L3_ARP: m_uint32_t = 0x0008;
pub const N_PKT_CTX_FLAG_L3_IP: m_uint32_t = 0x0010;
pub const N_PKT_CTX_FLAG_L4_UDP: m_uint32_t = 0x0020;
pub const N_PKT_CTX_FLAG_L4_TCP: m_uint32_t = 0x0040;
pub const N_PKT_CTX_FLAG_L4_ICMP: m_uint32_t = 0x0080;
pub const N_PKT_CTX_FLAG_IPH_OK: m_uint32_t = 0x0100;
pub const N_PKT_CTX_FLAG_IP_FRAG: m_uint32_t = 0x0200;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct n_pkt_ctx {
    /// full packet
    pub pkt: *mut m_uint8_t,
    pub pkt_len: size_t,

    /// Packet flags
    pub flags: m_uint32_t,

    /// VLAN information
    pub vlan_id: m_uint16_t,

    /// L4 protocol for IP
    pub ip_l4_proto: u_int,

    /// L3 header
    pub l3: n_pkt_ctx_l3,

    /// L4 header
    pub l4: n_pkt_ctx_l4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union n_pkt_ctx_l3 {
    pub arp: *mut n_arp_hdr_t,
    pub ip: *mut n_ip_hdr_t,
    pub ptr: *mut c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union n_pkt_ctx_l4 {
    pub udp: *mut n_udp_hdr_t,
    pub tcp: *mut n_tcp_hdr_t,
    pub ptr: *mut c_void,
}

/// IP mask table, which allows to find quickly a network mask 
/// with a prefix length.
#[rustfmt::skip]
static mut ip_masks: [n_ip_addr_t; N_IP_ADDR_BITS+1] = [
    0x0,
    0x80000000, 0xC0000000, 0xE0000000, 0xF0000000,
    0xF8000000, 0xFC000000, 0xFE000000, 0xFF000000,
    0xFF800000, 0xFFC00000, 0xFFE00000, 0xFFF00000,
    0xFFF80000, 0xFFFC0000, 0xFFFE0000, 0xFFFF0000,
    0xFFFF8000, 0xFFFFC000, 0xFFFFE000, 0xFFFFF000,
    0xFFFFF800, 0xFFFFFC00, 0xFFFFFE00, 0xFFFFFF00,
    0xFFFFFF80, 0xFFFFFFC0, 0xFFFFFFE0, 0xFFFFFFF0,
    0xFFFFFFF8, 0xFFFFFFFC, 0xFFFFFFFE, 0xFFFFFFFF
];

/// IPv6 mask table, which allows to find quickly a network mask
/// with a prefix length. Note this is a particularly ugly way
/// to do this, since we use statically 2 Kb.
static mut ipv6_masks: [n_ipv6_addr_t; N_IPV6_ADDR_BITS + 1] = unsafe { zeroed::<_>() };

/// Initialize IPv6 masks
#[no_mangle]
pub unsafe extern "C" fn ipv6_init_masks() {
    // Set all bits to 1
    libc::memset(ipv6_masks.as_c_void_mut(), 0xff, size_of::<[n_ipv6_addr_t; N_IPV6_ADDR_BITS + 1]>());

    #[allow(clippy::needless_range_loop)]
    for i in 0..N_IPV6_ADDR_BITS {
        let mut index = i >> 3; /* Compute byte index (divide by 8) */

        // rotate byte
        ipv6_masks[i].ip6.u6_addr8[index] <<= 8 - (i & 7);
        index += 1;

        // clear following bytes
        while index < N_IPV6_ADDR_LEN {
            ipv6_masks[i].ip6.u6_addr8[index] = 0;
            index += 1;
        }
    }
}

/// Create a new socket to connect to specified host
unsafe fn udp_connect_ipv4_ipv6(local_port: c_int, remote_host: *mut c_char, remote_port: c_int) -> c_int {
    let mut hints: libc::addrinfo = zeroed::<_>();
    let mut res: *mut libc::addrinfo;
    let mut res0: *mut libc::addrinfo = null_mut();
    let mut st: libc::sockaddr_storage = zeroed::<_>();
    let mut sck: c_int = -1;
    let yes: c_int = 1;
    let mut port_str: [c_char; 20] = [0; 20];

    libc::memset(addr_of_mut!(hints).cast::<_>(), 0, size_of::<libc::addrinfo>());
    hints.ai_family = libc::PF_UNSPEC;
    hints.ai_socktype = libc::SOCK_DGRAM;

    libc::snprintf(port_str.as_c_mut(), port_str.len(), cstr!("%d"), remote_port);

    let error: c_int = libc::getaddrinfo(remote_host, port_str.as_c(), addr_of!(hints), addr_of_mut!(res0));
    if error != 0 {
        libc::fprintf(c_stderr(), cstr!("%s\n"), libc::gai_strerror(error));
        return -1;
    }

    let mut res_next: *mut libc::addrinfo = res0;
    while !res_next.is_null() {
        res = res_next;
        res_next = (*res).ai_next;

        // We want only IPv4 or IPv6
        if ((*res).ai_family != libc::PF_INET) && ((*res).ai_family != libc::PF_INET6) {
            continue;
        }

        // create new socket
        sck = libc::socket((*res).ai_family, libc::SOCK_DGRAM, (*res).ai_protocol);
        if sck < 0 {
            libc::perror(cstr!("udp_connect: socket"));
            continue;
        }

        // bind to the local port
        libc::memset(addr_of_mut!(st).cast::<_>(), 0, size_of::<libc::sockaddr_storage>());

        match (*res).ai_family {
            libc::PF_INET => {
                let sin: *mut libc::sockaddr_in = addr_of_mut!(st).cast::<_>();
                (*sin).sin_family = libc::PF_INET as libc::sa_family_t;
                (*sin).sin_port = htons(local_port.try_into().expect("c_int->u16"));
            }

            libc::PF_INET6 => {
                let sin6: *mut libc::sockaddr_in6 = addr_of_mut!(st).cast::<_>();
                #[cfg(has_libc_sockaddr_in6_sin6_len)]
                {
                    (*sin6).sin6_len = (*res).ai_addrlen.try_into().expect("socklen_t->u8");
                }
                (*sin6).sin6_family = libc::PF_INET6 as libc::sa_family_t;
                (*sin6).sin6_port = htons(local_port.try_into().expect("c_int->u16"));
            }

            _ => {
                // shouldn't happen
                libc::close(sck);
                sck = -1;
                continue;
            }
        }

        // try to connect to remote host
        if libc::setsockopt(sck, libc::SOL_SOCKET, libc::SO_REUSEADDR, addr_of!(yes).cast::<_>(), size_of::<c_int>() as libc::socklen_t) != 0 {
            libc::perror(cstr!("Warning: upd_connect: setsockopt(SO_REUSEADDR)"));
        }

        if libc::bind(sck, addr_of!(st).cast::<_>(), (*res).ai_addrlen) == 0 && libc::connect(sck, (*res).ai_addr, (*res).ai_addrlen) == 0 {
            libc::perror(cstr!("udp_connect: bind/connect"));
            break;
        }

        libc::close(sck);
        sck = -1;
    }

    libc::freeaddrinfo(res0);

    if sck >= 0 && m_fd_set_non_block(sck) < 0 {
        libc::perror(cstr!("Warning: udp_connect: m_fd_set_non_block"));
    }

    sck
}

/// Create a new socket to connect to specified host.
/// Version for old systems that do not support RFC 2553 (getaddrinfo())
///
/// See http://www.faqs.org/rfcs/rfc2553.html for more info.
unsafe fn udp_connect_ipv4(local_port: c_int, remote_host: *mut c_char, remote_port: c_int) -> c_int {
    let mut sin: libc::sockaddr_in = zeroed::<_>();
    let yes: c_int = 1;

    let hp: *mut libc::hostent = gethostbyname(remote_host);
    if hp.is_null() {
        libc::fprintf(c_stderr(), cstr!("udp_connect: unable to resolve '%s'\n"), remote_host);
        return -1;
    }

    let sck: c_int = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
    if sck < 0 {
        libc::perror(cstr!("udp_connect: socket"));
        return -1;
    }

    // bind local port
    libc::memset(addr_of_mut!(sin).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
    sin.sin_family = libc::PF_INET as libc::sa_family_t;
    sin.sin_port = htons(local_port.try_into().expect("c_int->u16"));

    if libc::setsockopt(sck, libc::SOL_SOCKET, libc::SO_REUSEADDR, addr_of!(yes).cast::<_>(), size_of::<c_int>() as libc::socklen_t) != 0 {
        libc::perror(cstr!("Warning: upd_connect: setsockopt(SO_REUSEADDR)"));
    }

    if libc::bind(sck, addr_of!(sin).cast::<_>(), size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0 {
        libc::perror(cstr!("udp_connect: bind"));
        libc::close(sck);
        return -1;
    }

    // try to connect to remote host
    libc::memset(addr_of_mut!(sin).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
    libc::memcpy(addr_of_mut!(sin.sin_addr).cast::<_>(), (*(*hp).h_addr_list).cast::<_>(), size_of::<libc::in_addr>());
    sin.sin_family = libc::PF_INET as libc::sa_family_t;
    sin.sin_port = htons(remote_port.try_into().expect("c_int->u16"));

    if libc::connect(sck, addr_of!(sin).cast::<_>(), size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0 {
        libc::perror(cstr!("udp_connect: connect"));
        libc::close(sck);
        return -1;
    }

    if m_fd_set_non_block(sck) < 0 {
        libc::perror(cstr!("Warning: udp_connect: m_fd_set_non_block"));
    }

    sck
}

/// Listen on the specified port
#[no_mangle]
pub unsafe extern "C" fn udp_connect(local_port: c_int, remote_host: *mut c_char, remote_port: c_int) -> c_int {
    if cfg!(feature = "ENABLE_IPV6") {
        udp_connect_ipv4_ipv6(local_port, remote_host, remote_port)
    } else {
        udp_connect_ipv4(local_port, remote_host, remote_port)
    }
}

/// Listen on the specified port
unsafe fn ip_listen_ipv4_ipv6(ip_addr: *mut c_char, port: c_int, sock_type: c_int, max_fd: c_int, mut fd_array: CArray<c_int>) -> c_int {
    let mut hints: libc::addrinfo = zeroed::<_>();
    let mut res_next: *mut libc::addrinfo;
    let mut res: *mut libc::addrinfo;
    let mut res0: *mut libc::addrinfo = null_mut();
    let mut port_str: [c_char; 20] = [0; 20];
    let mut nsock: c_int;
    let reuse: c_int = 1;

    for i in 0..max_fd {
        fd_array[i] = -1;
    }

    libc::memset(addr_of_mut!(hints).cast::<_>(), 0, size_of::<libc::addrinfo>());
    hints.ai_family = libc::PF_UNSPEC;
    hints.ai_socktype = sock_type;
    hints.ai_flags = libc::AI_PASSIVE;

    libc::snprintf(port_str.as_c_mut(), port_str.len(), cstr!("%d"), port);
    let addr: *mut c_char = if !ip_addr.is_null() && libc::strlen(ip_addr) != 0 { ip_addr } else { null_mut() };

    let error: c_int = libc::getaddrinfo(addr, port_str.as_c(), addr_of!(hints), addr_of_mut!(res0));
    if error != 0 {
        libc::fprintf(c_stderr(), cstr!("ip_listen: %s"), libc::gai_strerror(error));
        return -1;
    }

    nsock = 0;
    res_next = res0;
    while !res_next.is_null() && nsock < max_fd {
        res = res_next;
        res_next = (*res).ai_next;

        if (*res).ai_family != libc::PF_INET && (*res).ai_family != libc::PF_INET6 {
            continue;
        }

        fd_array[nsock] = libc::socket((*res).ai_family, (*res).ai_socktype, (*res).ai_protocol);

        if fd_array[nsock] < 0 {
            continue;
        }

        if libc::setsockopt(fd_array[nsock], libc::SOL_SOCKET, libc::SO_REUSEADDR, addr_of!(reuse).cast::<_>(), size_of::<c_int>() as libc::socklen_t) != 0 {
            libc::perror(cstr!("Warning: ip_listen: setsockopt(SO_REUSEADDR): The same address-port combination can be retried after the TCP TIME_WAIT state expires."));
        }

        if (libc::bind(fd_array[nsock], (*res).ai_addr, (*res).ai_addrlen) < 0) || ((sock_type == libc::SOCK_STREAM) && (libc::listen(fd_array[nsock], 5) < 0)) {
            libc::perror(cstr!("ip_listen: bind/listen"));
            libc::close(fd_array[nsock]);
            fd_array[nsock] = -1;
            continue;
        }

        nsock += 1;
    }

    libc::freeaddrinfo(res0);
    nsock
}

/// Listen on the specified port
unsafe fn ip_listen_ipv4(ip_addr: *mut c_char, port: c_int, sock_type: c_int, max_fd: c_int, mut fd_array: CArray<c_int>) -> c_int {
    let mut sin: libc::sockaddr_in = zeroed::<_>();
    let reuse: c_int = 1;

    for i in 0..max_fd {
        fd_array[i] = -1;
    }

    let sck: c_int = libc::socket(libc::AF_INET, sock_type, 0);
    if sck < 0 {
        libc::perror(cstr!("ip_listen: socket"));
        return -1;
    }

    // bind local port
    libc::memset(addr_of_mut!(sin).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
    sin.sin_family = libc::PF_INET as libc::sa_family_t;
    sin.sin_port = htons(port.try_into().expect("c_int->u16"));

    if !ip_addr.is_null() && libc::strlen(ip_addr) != 0 {
        sin.sin_addr.s_addr = inet_addr(ip_addr);
    }

    if libc::setsockopt(sck, libc::SOL_SOCKET, libc::SO_REUSEADDR, addr_of!(reuse).cast::<_>(), size_of::<c_int>() as libc::socklen_t) != 0 {
        libc::perror(cstr!("Warning: ip_listen: setsockopt(SO_REUSEADDR): The same address-port combination can be retried after the TCP TIME_WAIT state expires."));
    }

    if libc::bind(sck, addr_of!(sin).cast::<_>(), size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0 {
        libc::perror(cstr!("ip_listen: bind"));
        libc::close(sck);
        return -1;
    }

    if (sock_type == libc::SOCK_STREAM) && (libc::listen(sck, 5) < 0) {
        libc::perror(cstr!("ip_listen: listen"));
        libc::close(sck);
        return -1;
    }

    fd_array[0] = sck;
    1
}

/// Listen on the specified port
#[no_mangle]
pub unsafe extern "C" fn ip_listen(ip_addr: *mut c_char, port: c_int, sock_type: c_int, max_fd: c_int, fd_array: *mut c_int) -> c_int {
    if cfg!(feature = "ENABLE_IPV6") {
        ip_listen_ipv4_ipv6(ip_addr, port, sock_type, max_fd, fd_array.into())
    } else {
        ip_listen_ipv4(ip_addr, port, sock_type, max_fd, fd_array.into())
    }
}

/// Convert a string containing an IP address in binary
#[no_mangle]
pub unsafe extern "C" fn n_ip_aton(ip_addr: *mut n_ip_addr_t, ip_str: *mut c_char) -> c_int {
    let mut addr: libc::in_addr = zeroed::<_>();

    if inet_aton(ip_str, addr_of_mut!(addr)) == 0 {
        return -1;
    }

    *ip_addr = ntohl(addr.s_addr);
    0
}

/* Parse an IPv4 CIDR prefix */
#[no_mangle]
pub unsafe extern "C" fn ip_parse_cidr(token: *mut c_char, net_addr: *mut n_ip_addr_t, net_mask: *mut n_ip_addr_t) -> c_int {
    // Find separator
    let sl: *mut c_char = libc::strchr(token, b'/' as c_int);
    if sl.is_null() {
        return -1;
    }

    // Get mask
    let mut err: *mut c_char = null_mut();
    let mask: u_long = libc::strtoul(sl.add(1), addr_of_mut!(err), 0);
    if *err != 0 {
        return -1;
    }

    // Ensure that mask has a correct value
    if mask as usize > N_IP_ADDR_BITS {
        return -1;
    }

    let tmp: *mut c_char = libc::strdup(token);
    if tmp.is_null() {
        return -1;
    }

    *tmp.offset(sl.offset_from(token)) = 0;

    // Parse IP Address
    if n_ip_aton(net_addr, tmp) == -1 {
        libc::free(tmp.cast::<_>());
        return -1;
    }

    // Set netmask
    *net_mask = ip_masks[mask as usize];

    libc::free(tmp.cast::<_>());
    0
}

/// Convert an IPv6 address from string into binary
#[no_mangle]
pub unsafe extern "C" fn n_ipv6_aton(ipv6_addr: *mut n_ipv6_addr_t, ip_str: *mut c_char) -> c_int {
    inet_pton(libc::AF_INET6, ip_str, ipv6_addr.cast::<_>())
}

/// Parse an IPv6 CIDR prefix
#[no_mangle]
pub unsafe extern "C" fn ipv6_parse_cidr(token: *mut c_char, net_addr: *mut n_ipv6_addr_t, net_mask: *mut u_int) -> c_int {
    // Find separator
    let sl: *mut c_char = libc::strchr(token, b'/' as c_int);
    if sl.is_null() {
        return -1;
    }

    // Get mask
    let mut err: *mut c_char = null_mut();
    let mask: u_long = libc::strtoul(sl.add(1), addr_of_mut!(err), 0);
    if *err != 0 {
        return -1;
    }

    // Ensure that mask has a correct value
    if mask as usize > N_IPV6_ADDR_BITS {
        return -1;
    }

    let tmp: *mut c_char = libc::strdup(token);
    if tmp.is_null() {
        return -1;
    }

    *tmp.offset(sl.offset_from(token)) = 0;

    // Parse IP Address
    if n_ipv6_aton(net_addr, tmp) <= 0 {
        libc::free(tmp.cast::<_>());
        return -1;
    }

    // Set netmask
    *net_mask = mask as u_int;

    libc::free(tmp.cast::<_>());
    0
}

/// Check for a broadcast ethernet address
#[no_mangle]
pub unsafe extern "C" fn eth_addr_is_bcast(addr: *mut n_eth_addr_t) -> c_int {
    let bcast_addr: [u8; 6] = *b"\xff\xff\xff\xff\xff\xff";
    (libc::memcmp(addr.cast::<_>(), bcast_addr.as_c_void(), 6) != 0).into()
}

/// Convert an IPv4 address into a string
#[no_mangle]
pub unsafe extern "C" fn n_ip_ntoa(buffer: *mut c_char, mut ip_addr: n_ip_addr_t) -> *mut c_char {
    let p: *mut u_char = addr_of_mut!(ip_addr).cast::<_>();
    libc::sprintf(buffer, cstr!("%u.%u.%u.%u"), *p.add(0) as c_uint, *p.add(1) as c_uint, *p.add(2) as c_uint, *p.add(3) as c_uint);
    buffer
}

/// Convert in IPv6 address into a string
#[no_mangle]
pub unsafe extern "C" fn n_ipv6_ntoa(buffer: *mut c_char, ipv6_addr: *mut n_ipv6_addr_t) -> *mut c_char {
    inet_ntop(libc::AF_INET6, ipv6_addr.cast::<_>(), buffer, c_INET6_ADDRSTRLEN()).cast_mut()
}

/// Parse a processor board id and return the eeprom settings in a buffer
#[no_mangle]
pub unsafe extern "C" fn parse_board_id(buf: *mut m_uint8_t, id: *const c_char, encode: c_int) -> c_int {
    // Encode the serial board id
    //   encode 4 maps this into 4 bytes
    //   encode 9 maps into 9 bytes
    //   encode 11 maps into 11 bytes

    libc::memset(buf.cast::<_>(), 0, 11);
    if encode == 4 {
        let mut v: c_int = 0;
        let res: c_int = libc::sscanf(id, cstr!("%d"), addr_of_mut!(v));
        if res != 1 {
            return -1;
        }
        *buf.add(3) = (v & 0xFF) as m_uint8_t;
        v >>= 8;
        *buf.add(2) = (v & 0xFF) as m_uint8_t;
        v >>= 8;
        *buf.add(1) = (v & 0xFF) as m_uint8_t;
        v >>= 8;
        *buf.add(0) = (v & 0xFF) as m_uint8_t;
        v >>= 8;
        if false {
            let _ = v;
            libc::printf(cstr!("%x %x %x %x \n"), *buf.add(0) as c_uint, *buf.add(1) as c_uint, *buf.add(2) as c_uint, *buf.add(3) as c_uint);
        }
        return 0;
    } else if encode == 9 {
        let res: c_int = libc::sscanf(id, cstr!("%c%c%c%2hx%2hx%c%c%c%c"), buf.add(0), buf.add(1), buf.add(2), buf.add(3).cast::<c_ushort>(), buf.add(4).cast::<c_ushort>(), buf.add(5), buf.add(6), buf.add(7), buf.add(8));
        if res != 9 {
            return -1;
        }
        if false {
            libc::printf(cstr!("%x %x %x %x %x %x %x %x .. %x\n"), *buf.add(0) as c_uint, *buf.add(1) as c_uint, *buf.add(2) as c_uint, *buf.add(3) as c_uint, *buf.add(4) as c_uint, *buf.add(5) as c_uint, *buf.add(6) as c_uint, *buf.add(7) as c_uint, *buf.add(8) as c_uint);
        }
        return 0;
    } else if encode == 11 {
        let res: c_int = libc::sscanf(id, cstr!("%c%c%c%c%c%c%c%c%c%c%c"), buf.add(0), buf.add(1), buf.add(2), buf.add(3), buf.add(4), buf.add(5), buf.add(6), buf.add(7), buf.add(8), buf.add(9), buf.add(10));
        if res != 11 {
            return -1;
        }
        if false {
            libc::printf(
                cstr!("%x %x %x %x %x %x %x %x %x %x .. %x\n"),
                *buf.add(0) as c_uint,
                *buf.add(1) as c_uint,
                *buf.add(2) as c_uint,
                *buf.add(3) as c_uint,
                *buf.add(4) as c_uint,
                *buf.add(5) as c_uint,
                *buf.add(6) as c_uint,
                *buf.add(7) as c_uint,
                *buf.add(8) as c_uint,
                *buf.add(9) as c_uint,
                *buf.add(10) as c_uint,
            );
        }
        return 0;
    }
    -1
}

/// Parse a MAC address
#[no_mangle]
pub unsafe extern "C" fn parse_mac_addr(addr: *mut n_eth_addr_t, str_: *mut c_char) -> c_int {
    let mut v: [u_int; N_ETH_ALEN] = [0; N_ETH_ALEN];
    let mut res: c_int;

    // First try, standard format (00:01:02:03:04:05)
    res = libc::sscanf(str_, cstr!("%x:%x:%x:%x:%x:%x"), addr_of_mut!(v[0]), addr_of_mut!(v[1]), addr_of_mut!(v[2]), addr_of_mut!(v[3]), addr_of_mut!(v[4]), addr_of_mut!(v[5]));

    if res == 6 {
        #[allow(clippy::needless_range_loop)]
        for i in 0..N_ETH_ALEN {
            (*addr).eth_addr_byte[i] = v[i] as m_uint8_t;
        }
        return 0;
    }

    // Second try, Cisco format (0001.0002.0003)
    res = libc::sscanf(str_, cstr!("%x.%x.%x"), addr_of_mut!(v[0]), addr_of_mut!(v[1]), addr_of_mut!(v[2]));

    if res == 3 {
        (*addr).eth_addr_byte[0] = ((v[0] >> 8) & 0xFF) as m_uint8_t;
        (*addr).eth_addr_byte[1] = (v[0] & 0xFF) as m_uint8_t;
        (*addr).eth_addr_byte[2] = ((v[1] >> 8) & 0xFF) as m_uint8_t;
        (*addr).eth_addr_byte[3] = (v[1] & 0xFF) as m_uint8_t;
        (*addr).eth_addr_byte[4] = ((v[2] >> 8) & 0xFF) as m_uint8_t;
        (*addr).eth_addr_byte[5] = (v[2] & 0xFF) as m_uint8_t;
        return 0;
    }

    -1
}

/// Convert an Ethernet address into a string
#[no_mangle]
pub unsafe extern "C" fn n_eth_ntoa(buffer: *mut c_char, addr: *mut n_eth_addr_t, format: c_int) -> *mut c_char {
    let str_format: *mut c_char = if format == 0 { cstr!("%2.2x:%2.2x:%2.2x:%2.2x:%2.2x:%2.2x") } else { cstr!("%2.2x%2.2x.%2.2x%2.2x.%2.2x%2.2x") };

    libc::sprintf(buffer, str_format, (*addr).eth_addr_byte[0] as c_uint, (*addr).eth_addr_byte[1] as c_uint, (*addr).eth_addr_byte[2] as c_uint, (*addr).eth_addr_byte[3] as c_uint, (*addr).eth_addr_byte[4] as c_uint, (*addr).eth_addr_byte[5] as c_uint);
    buffer
}

/// Get port in an address info structure
#[no_mangle]
pub unsafe extern "C" fn ip_socket_get_port(addr: *mut libc::sockaddr) -> c_int {
    match (*addr).sa_family as c_int {
        libc::AF_INET => ntohs((*addr.cast::<libc::sockaddr_in>()).sin_port) as c_int,
        libc::AF_INET6 => ntohs((*addr.cast::<libc::sockaddr_in6>()).sin6_port) as c_int,
        _ => {
            libc::fprintf(c_stderr(), cstr!("ip_socket_get_port: unknown address family %d\n"), (*addr).sa_family as c_int);
            -1
        }
    }
}

/// Set port in an address info structure
#[no_mangle]
pub unsafe extern "C" fn ip_socket_set_port(addr: *mut libc::sockaddr, port: c_int) -> c_int {
    if addr.is_null() {
        return -1;
    }

    match (*addr).sa_family as c_int {
        libc::AF_INET => {
            (*addr.cast::<libc::sockaddr_in>()).sin_port = htons(port as u16);
            0
        }
        libc::AF_INET6 => {
            (*addr.cast::<libc::sockaddr_in6>()).sin6_port = htons(port as u16);
            0
        }
        _ => {
            libc::fprintf(c_stderr(), cstr!("ip_socket_set_port: unknown address family %d\n"), (*addr).sa_family as c_int);
            -1
        }
    }
}

/// Try to create a socket and bind to the specified address info
unsafe fn ip_socket_bind_ipv4_ipv6(addr: *mut libc::addrinfo) -> c_int {
    let off: c_int = 0;

    let fd: c_int = libc::socket((*addr).ai_family, (*addr).ai_socktype, (*addr).ai_protocol);
    if fd < 0 {
        return -1;
    }

    #[cfg(has_libc_IPV6_V6ONLY)]
    {
        if (*addr).ai_family == libc::AF_INET6 {
            // if supported, allow packets to/from IPv4-mapped IPv6 addresses
            libc::setsockopt(fd, libc::IPPROTO_IPV6, libc::IPV6_V6ONLY, addr_of!(off).cast::<_>(), size_of::<c_int>() as libc::socklen_t);
        }
    }

    if (libc::bind(fd, (*addr).ai_addr, (*addr).ai_addrlen) < 0) || (((*addr).ai_socktype == libc::SOCK_STREAM) && (libc::listen(fd, 5) < 0)) {
        libc::close(fd);
        return -1;
    }

    fd
}

/// Listen on a TCP/UDP port - port is choosen in the specified range
unsafe fn ip_listen_range_ipv4_ipv6(ip_addr: *mut c_char, port_start: c_int, port_end: c_int, port: *mut c_int, sock_type: c_int) -> c_int {
    let mut hints: libc::addrinfo = zeroed::<_>();
    let mut res: *mut libc::addrinfo;
    let mut res0: *mut libc::addrinfo = null_mut();
    let mut st: libc::sockaddr_storage = zeroed::<_>();
    let mut st_len: libc::socklen_t;
    let mut port_str: [c_char; 20] = [0; 20];
    let mut fd: c_int = -1;

    libc::memset(addr_of_mut!(hints).cast::<_>(), 0, size_of::<libc::addrinfo>());
    hints.ai_family = libc::PF_UNSPEC;
    hints.ai_socktype = sock_type;
    hints.ai_flags = libc::AI_PASSIVE;

    libc::snprintf(port_str.as_c_mut(), port_str.len(), cstr!("%d"), port_start);
    let addr: *mut c_char = if !ip_addr.is_null() && libc::strlen(ip_addr) != 0 { ip_addr } else { null_mut() };

    let error: c_int = libc::getaddrinfo(addr, port_str.as_c(), addr_of!(hints), addr_of_mut!(res0));
    if error != 0 {
        libc::fprintf(c_stderr(), cstr!("ip_listen_range: %s"), libc::gai_strerror(error));
        return -1;
    }

    'done: for i in port_start..=port_end {
        res = res0;
        while !res.is_null() {
            ip_socket_set_port((*res).ai_addr, i);

            fd = ip_socket_bind_ipv4_ipv6(res);
            if fd >= 0 {
                st_len = size_of::<libc::sockaddr_storage>() as libc::socklen_t;
                if libc::getsockname(fd, addr_of_mut!(st).cast::<libc::sockaddr>(), addr_of_mut!(st_len)) != 0 {
                    libc::close(fd);
                    res = (*res).ai_next;
                    continue;
                }
                *port = ip_socket_get_port(addr_of_mut!(st).cast::<libc::sockaddr>());
                break 'done;
            }
            res = (*res).ai_next;
        }
    }

    libc::freeaddrinfo(res0);
    fd
}

/// Try to create a socket and bind to the specified address info
unsafe fn ip_socket_bind_ipv4(sin: *mut libc::sockaddr_in, sock_type: c_int) -> c_int {
    let fd: c_int = libc::socket((*sin).sin_family as c_int, sock_type, 0);
    if fd < 0 {
        return -1;
    }

    if (libc::bind(fd, sin.cast::<libc::sockaddr>(), size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0) || ((sock_type == libc::SOCK_STREAM) && (libc::listen(fd, 5) < 0)) {
        libc::close(fd);
        return -1;
    }

    fd
}

/// Listen on a TCP/UDP port - port is choosen in the specified range
unsafe fn ip_listen_range_ipv4(ip_addr: *mut c_char, port_start: c_int, port_end: c_int, port: *mut c_int, sock_type: c_int) -> c_int {
    let hp: *mut libc::hostent;
    let mut sin: libc::sockaddr_in = zeroed();
    let mut len: libc::socklen_t;
    let mut fd: c_int;

    libc::memset(addr_of_mut!(sin).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
    sin.sin_family = libc::PF_INET as libc::sa_family_t;

    if !ip_addr.is_null() && libc::strlen(ip_addr) != 0 {
        hp = gethostbyname(ip_addr);
        if hp.is_null() {
            libc::fprintf(c_stderr(), cstr!("ip_listen_range: unable to resolve '%s'\n"), ip_addr);
            return -1;
        }

        libc::memcpy(addr_of_mut!(sin.sin_addr).cast::<_>(), (*(*hp).h_addr_list.add(0)).cast::<_>(), size_of::<libc::in_addr>());
    }

    for i in port_start..=port_end {
        sin.sin_port = htons(i as u16);

        fd = ip_socket_bind_ipv4(addr_of_mut!(sin), sock_type);
        if fd >= 0 {
            len = size_of::<libc::sockaddr_in>() as libc::socklen_t;
            if libc::getsockname(fd, addr_of_mut!(sin).cast::<libc::sockaddr>(), addr_of_mut!(len)) != 0 {
                libc::close(fd);
                continue;
            }
            *port = ntohs(sin.sin_port) as c_int;
            return fd;
        }
    }

    -1
}

/// Listen on a TCP/UDP port - port is choosen in the specified range
#[no_mangle]
pub unsafe extern "C" fn ip_listen_range(ip_addr: *mut c_char, port_start: c_int, port_end: c_int, port: *mut c_int, sock_type: c_int) -> c_int {
    #[cfg(feature = "ENABLE_IPV6")]
    {
        ip_listen_range_ipv4_ipv6(ip_addr, port_start, port_end, port, sock_type)
    }
    #[cfg(not(feature = "ENABLE_IPV6"))]
    {
        ip_listen_range_ipv4(ip_addr, port_start, port_end, port, sock_type)
    }
}

/// Connect an existing socket to connect to specified host
unsafe fn ip_connect_fd_ipv4_ipv6(fd: c_int, remote_host: *mut c_char, remote_port: c_int) -> c_int {
    let mut hints: libc::addrinfo = zeroed::<_>();
    let mut res: *mut libc::addrinfo;
    let mut res0: *mut libc::addrinfo = null_mut();
    let mut port_str: [c_char; 20] = [0; 20];

    libc::memset(addr_of_mut!(hints).cast::<_>(), 0, size_of::<libc::addrinfo>());
    hints.ai_family = libc::PF_UNSPEC;

    libc::snprintf(port_str.as_c_mut(), port_str.len(), cstr!("%d"), remote_port);

    let error: c_int = libc::getaddrinfo(remote_host, port_str.as_c(), addr_of!(hints), addr_of_mut!(res0));
    if error != 0 {
        libc::fprintf(c_stderr(), cstr!("%s\n"), libc::gai_strerror(error));
        return -1;
    }

    res = res0;
    while !res.is_null() {
        if ((*res).ai_family != libc::PF_INET) && ((*res).ai_family != libc::PF_INET6) {
            res = (*res).ai_next;
            continue;
        }

        if libc::connect(fd, (*res).ai_addr, (*res).ai_addrlen) == 0 {
            break;
        }
        res = (*res).ai_next;
    }

    libc::freeaddrinfo(res0);
    0
}

/// Connect an existing socket to connect to specified host
unsafe fn ip_connect_fd_ipv4(fd: c_int, remote_host: *mut c_char, remote_port: c_int) -> c_int {
    let mut sin: libc::sockaddr_in = zeroed::<_>();

    let hp: *mut libc::hostent = gethostbyname(remote_host);
    if hp.is_null() {
        libc::fprintf(c_stderr(), cstr!("ip_connect_fd: unable to resolve '%s'\n"), remote_host);
        return -1;
    }

    // try to connect to remote host
    libc::memset(addr_of_mut!(sin).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
    libc::memcpy(addr_of_mut!(sin.sin_addr).cast::<_>(), (*(*hp).h_addr_list.add(0)).cast::<_>(), size_of::<libc::in_addr>());
    sin.sin_family = libc::PF_INET as libc::sa_family_t;
    sin.sin_port = htons(remote_port as u16);

    libc::connect(fd, addr_of!(sin).cast::<libc::sockaddr>(), size_of::<libc::sockaddr_in>() as libc::socklen_t)
}

/// Connect an existing socket to connect to specified host
#[no_mangle]
pub unsafe extern "C" fn ip_connect_fd(fd: c_int, remote_host: *mut c_char, remote_port: c_int) -> c_int {
    #[cfg(feature = "ENABLE_IPV6")]
    {
        ip_connect_fd_ipv4_ipv6(fd, remote_host, remote_port)
    }
    #[cfg(not(feature = "ENABLE_IPV6"))]
    {
        ip_connect_fd_ipv4(fd, remote_host, remote_port)
    }
}

/// Create a socket UDP listening in a port of specified range
#[no_mangle]
pub unsafe extern "C" fn udp_listen_range(ip_addr: *mut c_char, port_start: c_int, port_end: c_int, port: *mut c_int) -> c_int {
    ip_listen_range(ip_addr, port_start, port_end, port, libc::SOCK_DGRAM)
}

/// Check for a broadcast/multicast ethernet address
#[no_mangle]
pub unsafe extern "C" fn eth_addr_is_mcast(addr: *mut n_eth_addr_t) -> c_int {
    ((*addr).eth_addr_byte[0] & 1).into()
}

/// Check for Cisco ISL destination address
#[no_mangle]
pub unsafe extern "C" fn eth_addr_is_cisco_isl(addr: *mut n_eth_addr_t) -> c_int {
    static mut isl_addr: *const c_char = cstr!("\x01\x00\x0c\x00\x00");
    (libc::memcmp(addr.cast::<_>(), isl_addr.cast::<_>(), 5) == 0).into() // only 40 bits to compare
}

/// Check for a SNAP header
#[no_mangle]
pub unsafe extern "C" fn eth_llc_check_snap(llc_hdr: *mut n_eth_llc_hdr_t) -> c_int {
    ((*llc_hdr).dsap == 0xAA && (*llc_hdr).ssap == 0xAA && (*llc_hdr).ctrl == 0x03).into()
}

/// Number of bits in a contiguous netmask
#[no_mangle]
pub unsafe extern "C" fn ip_bits_mask(mut mask: n_ip_addr_t) -> c_int {
    let mut prefix: c_int = 0;

    while mask != 0 {
        prefix += 1;
        mask = mask & (mask - 1);
    }

    prefix
}
