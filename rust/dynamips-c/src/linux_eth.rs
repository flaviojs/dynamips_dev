//! Copyright (c) 2006 Christophe Fillot.
//! E-mail: cf@utc.fr
//!
//! module used to send/receive Ethernet packets.
//!
//! Specific to the Linux operating system.

use crate::_private::*;

/// Get interface index of specified device
#[no_mangle]
pub unsafe extern "C" fn lnx_eth_get_dev_index(name: *mut c_char) -> c_int {
    let mut if_req: libc::ifreq = zeroed::<_>();

    // Create dummy file descriptor
    let fd: c_int = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
    if fd < 0 {
        libc::fprintf(c_stderr(), cstr!("eth_get_dev_index: socket: %s\n"), libc::strerror(c_errno()));
        return -1;
    }

    libc::memset(addr_of_mut!(if_req).cast::<c_void>(), 0, size_of::<libc::ifreq>());
    libc::strncpy(if_req.ifr_name.as_c_mut(), name, libc::IFNAMSIZ - 1);
    if_req.ifr_name[libc::IFNAMSIZ - 1] = 0;

    if libc::ioctl(fd, libc::SIOCGIFINDEX, addr_of_mut!(if_req)) < 0 {
        libc::fprintf(c_stderr(), cstr!("eth_get_dev_index: SIOCGIFINDEX: %s\n"), libc::strerror(c_errno()));
        libc::close(fd);
        return -1;
    }

    libc::close(fd);
    if_req.ifr_ifru.ifru_ifindex
}

/// Initialize a new ethernet raw socket
#[no_mangle]
pub unsafe extern "C" fn lnx_eth_init_socket(device: *mut c_char) -> c_int {
    let mut sa: libc::sockaddr_ll = zeroed::<_>();
    let mut mreq: libc::packet_mreq = zeroed::<_>();

    let sck: c_int = libc::socket(libc::PF_PACKET, libc::SOCK_RAW, htons(libc::ETH_P_ALL as u16) as c_int);
    if sck == -1 {
        libc::fprintf(c_stderr(), cstr!("eth_init_socket: socket: %s\n"), libc::strerror(c_errno()));
        return -1;
    }

    libc::memset(addr_of_mut!(sa).cast::<_>(), 0, size_of::<libc::sockaddr_ll>());
    sa.sll_family = libc::AF_PACKET as _;
    sa.sll_protocol = htons(libc::ETH_P_ALL as u16);
    sa.sll_hatype = libc::ARPHRD_ETHER;
    sa.sll_halen = libc::ETH_ALEN as _;
    sa.sll_ifindex = lnx_eth_get_dev_index(device);

    libc::memset(addr_of_mut!(mreq).cast::<_>(), 0, size_of::<libc::packet_mreq>());
    mreq.mr_ifindex = sa.sll_ifindex;
    mreq.mr_type = libc::PACKET_MR_PROMISC as _;

    if libc::bind(sck, addr_of_mut!(sa).cast::<_>(), size_of::<libc::sockaddr_ll>() as libc::socklen_t) == -1 {
        libc::fprintf(c_stderr(), cstr!("eth_init_socket: bind: %s\n"), libc::strerror(c_errno()));
        libc::close(sck);
        return -1;
    }

    if libc::setsockopt(sck, libc::SOL_PACKET, libc::PACKET_ADD_MEMBERSHIP, addr_of_mut!(mreq).cast::<_>(), size_of::<libc::packet_mreq>() as libc::socklen_t) == -1 {
        libc::fprintf(c_stderr(), cstr!("eth_init_socket: setsockopt: %s\n"), libc::strerror(c_errno()));
        libc::close(sck);
        return -1;
    }

    sck
}

/// Send an ethernet frame
#[no_mangle]
pub unsafe extern "C" fn lnx_eth_send(sck: c_int, dev_id: c_int, buffer: *mut c_char, len: size_t) -> ssize_t {
    let mut sa: libc::sockaddr_ll = zeroed::<_>();

    libc::memset(addr_of_mut!(sa).cast::<_>(), 0, size_of::<libc::sockaddr_ll>());
    sa.sll_family = libc::AF_PACKET as _;
    sa.sll_protocol = htons(libc::ETH_P_ALL as u16);
    sa.sll_hatype = libc::ARPHRD_ETHER;
    sa.sll_halen = libc::ETH_ALEN as _;
    sa.sll_ifindex = dev_id;

    libc::sendto(sck, buffer.cast::<_>(), len, 0, addr_of_mut!(sa).cast::<_>(), size_of::<libc::sockaddr_ll>() as libc::socklen_t)
}

/// Receive an ethernet frame
#[no_mangle]
pub unsafe extern "C" fn lnx_eth_recv(sck: c_int, buffer: *mut c_char, len: size_t) -> ssize_t {
    libc::recv(sck, buffer.cast::<_>(), len, 0)
}
