//! Copyright (c) 2006 Christophe Fillot.
//! E-mail: cf@utc.fr
//!
//! linux_eth.c: module used to send/receive Ethernet packets.
//!
//! Specific to the Linux operating system.

use crate::_extra::*;
use libc::size_t;
use libc::ssize_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;
use std::mem::zeroed;
use std::ptr::addr_of;
use std::ptr::addr_of_mut;

// Get interface index of specified device
#[no_mangle]
pub unsafe extern "C" fn lnx_eth_get_dev_index(name: *mut c_char) -> c_int {
    let mut if_req: libc::ifreq = zeroed();

    // Create dummy file descriptor
    let fd: c_int = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
    if fd < 0 {
        libc::fprintf(c_stderr(), c"eth_get_dev_index: socket: %s\n".as_ptr(), libc::strerror(c_errno()));
        return -1;
    }

    libc::memset(addr_of_mut!(if_req).cast::<_>(), 0, size_of::<libc::ifreq>());
    libc::strncpy(if_req.ifr_name.as_mut_ptr(), name, libc::IFNAMSIZ - 1);
    if_req.ifr_name[libc::IFNAMSIZ - 1] = b'\0' as c_char;

    if libc::ioctl(fd, libc::SIOCGIFINDEX as _, addr_of!(if_req)) < 0 {
        libc::fprintf(c_stderr(), c"eth_get_dev_index: SIOCGIFINDEX: %s\n".as_ptr(), libc::strerror(c_errno()));
        libc::close(fd);
        return -1;
    }

    libc::close(fd);
    if_req.ifr_ifru.ifru_ifindex
}

// Initialize a new ethernet raw socket
#[no_mangle]
pub unsafe extern "C" fn lnx_eth_init_socket(device: *mut c_char) -> c_int {
    let mut sa: libc::sockaddr_ll = zeroed();
    let mut mreq: libc::packet_mreq = zeroed();

    let sck: c_int = libc::socket(libc::PF_PACKET, libc::SOCK_RAW, libc::htons(libc::ETH_P_ALL as _) as _);
    if sck == -1 {
        libc::fprintf(c_stderr(), c"eth_init_socket: socket: %s\n".as_ptr(), libc::strerror(c_errno()));
        return -1;
    }

    libc::memset(addr_of_mut!(sa).cast::<_>(), 0, size_of::<libc::sockaddr_ll>());
    sa.sll_family = libc::AF_PACKET as _;
    sa.sll_protocol = libc::htons(libc::ETH_P_ALL as _);
    sa.sll_hatype = libc::ARPHRD_ETHER;
    sa.sll_halen = libc::ETH_ALEN as _;
    sa.sll_ifindex = lnx_eth_get_dev_index(device);

    libc::memset(addr_of_mut!(mreq).cast::<_>(), 0, size_of::<libc::packet_mreq>());
    mreq.mr_ifindex = sa.sll_ifindex;
    mreq.mr_type = libc::PACKET_MR_PROMISC as _;

    if libc::bind(sck, addr_of_mut!(sa).cast::<_>(), size_of::<libc::sockaddr_ll>() as _) == -1 {
        libc::fprintf(c_stderr(), c"eth_init_socket: bind: %s\n".as_ptr(), libc::strerror(c_errno()));
        libc::close(sck);
        return -1;
    }

    if libc::setsockopt(sck, libc::SOL_PACKET, libc::PACKET_ADD_MEMBERSHIP, addr_of!(mreq).cast::<_>(), size_of::<libc::packet_mreq>() as _) == -1 {
        libc::fprintf(c_stderr(), c"eth_init_socket: setsockopt: %s\n".as_ptr(), libc::strerror(c_errno()));
        libc::close(sck);
        return -1;
    }

    sck
}

// Send an ethernet frame
#[no_mangle]
pub unsafe extern "C" fn lnx_eth_send(sck: c_int, dev_id: c_int, buffer: *mut c_char, len: size_t) -> ssize_t {
    let mut sa: libc::sockaddr_ll = zeroed();

    libc::memset(addr_of_mut!(sa).cast::<_>(), 0, size_of::<libc::sockaddr_ll>());
    sa.sll_family = libc::AF_PACKET as _;
    sa.sll_protocol = libc::htons(libc::ETH_P_ALL as _);
    sa.sll_hatype = libc::ARPHRD_ETHER;
    sa.sll_halen = libc::ETH_ALEN as _;
    sa.sll_ifindex = dev_id;

    libc::sendto(sck, buffer.cast::<_>(), len, 0, addr_of!(sa).cast::<_>(), size_of::<libc::sockaddr_ll>() as _)
}

// Receive an ethernet frame
#[no_mangle]
pub unsafe extern "C" fn lnx_eth_recv(sck: c_int, buffer: *mut c_char, len: size_t) -> ssize_t {
    libc::recv(sck, buffer.cast::<_>(), len, 0)
}
