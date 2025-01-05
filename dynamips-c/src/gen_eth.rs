//!
//! Copyright (c) 2006 Christophe Fillot.
//! E-mail: cf@utc.fr
//!
//! gen_eth.c: module used to send/receive Ethernet packets.
//!
//! Use libpcap (0.9+) or WinPcap (0.4alpha1+) to receive and send packets.

use crate::_extra::*;
use crate::dynamips_common::*;
use libc::size_t;
use libc::ssize_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::mem::zeroed;
use std::ptr::addr_of;
use std::ptr::addr_of_mut;
use std::ptr::null_mut;

// Initialize a generic ethernet driver
#[no_mangle]
pub unsafe extern "C" fn gen_eth_init(device: *mut c_char) -> *mut _sys::pcap_t {
    let mut pcap_errbuf: [c_char; _sys::PCAP_ERRBUF_SIZE as usize] = [0; _sys::PCAP_ERRBUF_SIZE as usize];
    let p: *mut _sys::pcap_t;

    if true {
        p = _sys::pcap_open_live(device, 65535, TRUE, 10, pcap_errbuf.as_mut_ptr());
        if p.is_null() {
            libc::fprintf(c_stderr(), c"gen_eth_init: unable to open device '%s' with PCAP (%s)\n".as_ptr(), device, pcap_errbuf);
            return null_mut();
        }

        if cfg!(target_os = "macos") {
            _sys::pcap_setdirection(p, _sys::pcap_direction_t_PCAP_D_IN);
        } else {
            _sys::pcap_setdirection(p, _sys::pcap_direction_t_PCAP_D_INOUT);
        }
        #[cfg(has_net_bpf_biocfeedback)]
        {
            let on: c_int = 1;
            libc::ioctl(_sys::pcap_fileno(p), libc::BIOCFEEDBACK, addr_of!(on));
        }
    } else {
        // FIXME rust does not have a cygwin target
        p = _sys::pcap_open(
            device,
            65535,
            (_sys::PCAP_OPENFLAG_PROMISCUOUS | _sys::PCAP_OPENFLAG_NOCAPTURE_LOCAL | _sys::PCAP_OPENFLAG_MAX_RESPONSIVENESS | _sys::PCAP_OPENFLAG_NOCAPTURE_RPCAP) as _,
            10,
            null_mut(),
            pcap_errbuf.as_mut_ptr(),
        );

        if p.is_null() {
            libc::fprintf(c_stderr(), c"gen_eth_init: unable to open device '%s' with PCAP (%s)\n".as_ptr(), device, pcap_errbuf);
            return null_mut();
        }
    }

    p
}

// Free resources of a generic ethernet driver
#[no_mangle]
pub unsafe extern "C" fn gen_eth_close(p: *mut _sys::pcap_t) {
    _sys::pcap_close(p);
}

// Send an ethernet frame
#[no_mangle]
pub unsafe extern "C" fn gen_eth_send(p: *mut _sys::pcap_t, buffer: *mut c_char, len: size_t) -> ssize_t {
    _sys::pcap_sendpacket(p, buffer.cast::<_>(), len as _) as _
}

// Receive an ethernet frame
#[no_mangle]
pub unsafe extern "C" fn gen_eth_recv(p: *mut _sys::pcap_t, buffer: *mut c_char, len: size_t) -> ssize_t {
    let mut pkt_info: _sys::pcap_pkthdr = zeroed();

    let pkt_ptr: *mut u_char = _sys::pcap_next(p, addr_of_mut!(pkt_info)).cast_mut().cast::<_>();
    if pkt_ptr.is_null() {
        return -1;
    }

    let rlen: ssize_t = m_min!(len, pkt_info.caplen as size_t) as ssize_t;

    libc::memcpy(buffer.cast::<_>(), pkt_ptr.cast::<_>(), rlen as size_t);
    rlen
}

// Display Ethernet interfaces of the system
#[no_mangle]
pub unsafe extern "C" fn gen_eth_show_dev_list() -> c_int {
    let mut pcap_errbuf: [c_char; _sys::PCAP_ERRBUF_SIZE as usize] = [0; _sys::PCAP_ERRBUF_SIZE as usize];
    let mut dev_list: *mut _sys::pcap_if_t = null_mut();
    let mut dev: *mut _sys::pcap_if_t;

    libc::printf(c"Network device list:\n\n".as_ptr());

    let res: c_int = if true {
        _sys::pcap_findalldevs(addr_of_mut!(dev_list), pcap_errbuf.as_mut_ptr())
    } else {
        // FIXME rust does not have a cygwin target
        _sys::pcap_findalldevs_ex(_sys::PCAP_SRC_IF_STRING.as_ptr().cast::<_>(), null_mut(), addr_of_mut!(dev_list), pcap_errbuf.as_mut_ptr())
    };

    if res < 0 {
        libc::fprintf(c_stderr(), c"PCAP: unable to find device list (%s)\n".as_ptr(), pcap_errbuf.as_mut_ptr());
        return -1;
    }

    dev = dev_list;
    while !dev.is_null() {
        libc::printf(c"   %s : %s\n".as_ptr(), (*dev).name, if !(*dev).description.is_null() { (*dev).description } else { c"no info provided".as_ptr() });
        dev = (*dev).next;
    }

    libc::printf(c"\n".as_ptr());

    _sys::pcap_freealldevs(dev_list);
    0
}
