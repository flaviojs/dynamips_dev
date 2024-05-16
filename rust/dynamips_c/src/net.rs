//! Network Utility functions.

use crate::prelude::*;

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
