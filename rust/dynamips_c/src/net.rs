//! Network Utility functions.

use crate::_private::*;
use crate::utils::*;

/// Create a new socket to connect to specified host
#[no_mangle]
pub unsafe extern "C" fn udp_connect(local_port: c_int, remote_host: *mut c_char, remote_port: c_int) -> c_int {
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

        res = res0;
        while !res.is_null() {
            // We want only IPv4 or IPv6
            if ((*res).ai_family != libc::PF_INET) && ((*res).ai_family != libc::PF_INET6) {
                res = (*res).ai_next;
                continue;
            }

            // create new socket
            sck = libc::socket((*res).ai_family, libc::SOCK_DGRAM, (*res).ai_protocol);
            if sck < 0 {
                libc::perror(cstr!("udp_connect: socket"));
                res = (*res).ai_next;
                continue;
            }

            // bind to the local port
            libc::memset(addr_of_mut!(st).cast::<_>(), 0, size_of::<libc::sockaddr_storage>());

            match (*res).ai_family {
                libc::PF_INET => {
                    let sin: *mut libc::sockaddr_in = addr_of_mut!(st).cast::<_>();
                    (*sin).sin_family = libc::PF_INET as libc::sa_family_t;
                    (*sin).sin_port = htons(local_port as u16);
                }

                libc::PF_INET6 => {
                    let sin6: *mut libc::sockaddr_in6 = addr_of_mut!(st).cast::<_>();
                    #[cfg(has_libc_sockaddr_in6_sin6_len)]
                    {
                        (*sin6).sin6_len = (*res).ai_addrlen as _;
                    }
                    (*sin6).sin6_family = libc::PF_INET6 as libc::sa_family_t;
                    (*sin6).sin6_port = htons(local_port as u16);
                }

                _ => {
                    // shouldn't happen
                    libc::close(sck);
                    sck = -1;
                    res = (*res).ai_next;
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
            res = (*res).ai_next;
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
        sin.sin_port = htons(local_port as u16);

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
        sin.sin_port = htons(remote_port as u16);

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
    if cfg!(feature = "ENABLE_IPV6") {
        udp_connect_ipv4_ipv6(local_port, remote_host, remote_port)
    } else {
        udp_connect_ipv4(local_port, remote_host, remote_port)
    }
}

/// Listen on the specified port
#[no_mangle]
pub unsafe extern "C" fn ip_listen(ip_addr: *mut c_char, port: c_int, sock_type: c_int, max_fd: c_int, fd_array: *mut c_int) -> c_int {
    /// Listen on the specified port
    unsafe fn ip_listen_ipv4_ipv6(ip_addr: *mut c_char, port: c_int, sock_type: c_int, max_fd: c_int, fd_array: *mut c_int) -> c_int {
        let mut hints: libc::addrinfo = zeroed::<_>();
        let mut res: *mut libc::addrinfo;
        let mut res0: *mut libc::addrinfo = null_mut();
        let mut port_str: [c_char; 20] = [0; 20];
        let mut nsock: c_int;
        let reuse: c_int = 1;

        for i in 0..max_fd {
            *fd_array.offset(i as isize) = -1;
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
        res = res0;
        while !res.is_null() && (nsock < max_fd) {
            if ((*res).ai_family != libc::PF_INET) && ((*res).ai_family != libc::PF_INET6) {
                res = (*res).ai_next;
                continue;
            }

            *fd_array.offset(nsock as isize) = libc::socket((*res).ai_family, (*res).ai_socktype, (*res).ai_protocol);

            if *fd_array.offset(nsock as isize) < 0 {
                res = (*res).ai_next;
                continue;
            }

            if libc::setsockopt(*fd_array.offset(nsock as isize), libc::SOL_SOCKET, libc::SO_REUSEADDR, addr_of!(reuse).cast::<_>(), size_of::<c_int>() as libc::socklen_t) != 0 {
                libc::perror(cstr!("Warning: ip_listen: setsockopt(SO_REUSEADDR): The same address-port combination can be retried after the TCP TIME_WAIT state expires."));
            }

            if (libc::bind(*fd_array.offset(nsock as isize), (*res).ai_addr, (*res).ai_addrlen) < 0) || ((sock_type == libc::SOCK_STREAM) && (libc::listen(*fd_array.offset(nsock as isize), 5) < 0)) {
                libc::perror(cstr!("ip_listen: bind/listen"));
                libc::close(*fd_array.offset(nsock as isize));
                *fd_array.offset(nsock as isize) = -1;
                res = (*res).ai_next;
                continue;
            }

            nsock += 1;
            res = (*res).ai_next;
        }

        libc::freeaddrinfo(res0);
        nsock
    }
    /// Listen on the specified port
    unsafe fn ip_listen_ipv4(ip_addr: *mut c_char, port: c_int, sock_type: c_int, max_fd: c_int, fd_array: *mut c_int) -> c_int {
        let mut sin: libc::sockaddr_in = zeroed::<_>();
        let reuse: c_int = 1;

        for i in 0..max_fd {
            *fd_array.offset(i as isize) = -1;
        }

        let sck: c_int = libc::socket(libc::AF_INET, sock_type, 0);
        if sck < 0 {
            libc::perror(cstr!("ip_listen: socket"));
            return -1;
        }

        // bind local port
        libc::memset(addr_of_mut!(sin).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
        sin.sin_family = libc::PF_INET as libc::sa_family_t;
        sin.sin_port = htons(port as u16);

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

        *fd_array.add(0) = sck;
        1
    }
    if cfg!(feature = "ENABLE_IPV6") {
        ip_listen_ipv4_ipv6(ip_addr, port, sock_type, max_fd, fd_array)
    } else {
        ip_listen_ipv4(ip_addr, port, sock_type, max_fd, fd_array)
    }
}
