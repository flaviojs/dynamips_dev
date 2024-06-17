//! Network Utility functions.

use crate::dynamips_common::*;
use crate::prelude::*;
use crate::utils::*;

pub type n_eth_addr_t = n_eth_addr;

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
