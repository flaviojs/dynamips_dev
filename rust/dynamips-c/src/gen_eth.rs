//! Copyright (c) 2006 Christophe Fillot.
//! E-mail: cf@utc.fr
//!
//! gen_eth.c: module used to send/receive Ethernet packets.
//!
//! Use libpcap (0.9+) or WinPcap (0.4alpha1+) to receive and send packets.

use crate::_private::*;
use crate::dynamips_common::*;

/// Initialize a generic ethernet driver
#[no_mangle]
pub unsafe extern "C" fn gen_eth_init(device: *mut c_char) -> *mut pcap_sys::pcap_t {
    let mut pcap_errbuf: [c_char; pcap_sys::PCAP_ERRBUF_SIZE as usize] = [0; pcap_sys::PCAP_ERRBUF_SIZE as usize];
    let p: *mut pcap_sys::pcap_t;

    #[cfg(not(if_0))]
    {
        p = pcap_sys::pcap_open_live(device, 65535, TRUE, 10, pcap_errbuf.as_c_mut());
        if p.is_null() {
            libc::fprintf(c_stderr(), cstr!("gen_eth_init: unable to open device '%s' with PCAP (%s)\n"), device, pcap_errbuf.as_c());
            return null_mut();
        }

        if cfg!(target_os = "macos") {
            pcap_sys::pcap_setdirection(p, _pcap::PCAP_D_IN);
        } else {
            pcap_sys::pcap_setdirection(p, _pcap::PCAP_D_INOUT);
        }
        #[cfg(has_libc_BIOCFEEDBACK)]
        {
            let mut on: c_int = 1;
            libc::ioctl(pcap_sys::pcap_fileno(p), libc::BIOCFEEDBACK, addr_of_mut!(on));
        }
    }
    #[cfg(if_0)]
    {
        // XXX cygwin requires pcap_open?
        p = pcap_sys::pcap_open(device, 65535, pcap_sys::PCAP_OPENFLAG_PROMISCUOUS | pcap_sys::PCAP_OPENFLAG_NOCAPTURE_LOCAL | pcap_sys::PCAP_OPENFLAG_MAX_RESPONSIVENESS | pcap_sys::PCAP_OPENFLAG_NOCAPTURE_RPCAP, 10, null_mut(), pcap_errbuf.as_c_mut());

        if p.is_null() {
            libc::fprintf(c_stderr(), cstr!("gen_eth_init: unable to open device '%s' with PCAP (%s)\n"), device, pcap_errbuf.as_c());
            return null_mut();
        }
    }

    p
}

/// Free resources of a generic ethernet driver
#[no_mangle]
pub unsafe extern "C" fn gen_eth_close(p: *mut pcap_sys::pcap_t) {
    pcap_sys::pcap_close(p);
}

/// Send an ethernet frame
#[no_mangle]
pub unsafe extern "C" fn gen_eth_send(p: *mut pcap_sys::pcap_t, buffer: *mut c_char, len: size_t) -> ssize_t {
    pcap_sys::pcap_sendpacket(p, buffer.cast::<_>(), len as c_int) as ssize_t
}

/// Receive an ethernet frame
#[no_mangle]
pub unsafe extern "C" fn gen_eth_recv(p: *mut pcap_sys::pcap_t, buffer: *mut c_char, len: size_t) -> ssize_t {
    let mut pkt_info: pcap_sys::pcap_pkthdr = zeroed::<_>();

    let pkt_ptr: *const u_char = pcap_sys::pcap_next(p, addr_of_mut!(pkt_info)).cast::<_>();
    if pkt_ptr.is_null() {
        return -1;
    }

    let rlen: ssize_t = m_min(len as ssize_t, pkt_info.caplen as ssize_t);

    libc::memcpy(buffer.cast::<_>(), pkt_ptr.cast::<_>(), rlen as size_t);
    rlen
}

/// Display Ethernet interfaces of the system
#[no_mangle]
pub unsafe extern "C" fn gen_eth_show_dev_list() -> c_int {
    let mut pcap_errbuf: [c_char; pcap_sys::PCAP_ERRBUF_SIZE as usize] = [0; pcap_sys::PCAP_ERRBUF_SIZE as usize];
    let mut dev_list: *mut pcap_sys::pcap_if_t = null_mut();
    let mut dev: *mut pcap_sys::pcap_if_t;
    let res: c_int;

    libc::printf(cstr!("Network device list:\n\n"));

    #[cfg(not(if_0))]
    {
        res = pcap_sys::pcap_findalldevs(addr_of_mut!(dev_list), pcap_errbuf.as_c_mut());
    }
    #[cfg(if_0)]
    {
        // XXX cygwin requires pcap_findalldevs_ex?
        res = pcap_sys::pcap_findalldevs_ex(pcap_sys::PCAP_SRC_IF_STRING, null_mut(), addr_of_mut!(dev_list), pcap_errbuf.as_c_mut());
    }

    if res < 0 {
        libc::fprintf(c_stderr(), cstr!("PCAP: unable to find device list (%s)\n"), pcap_errbuf.as_c());
        return -1;
    }

    dev = dev_list;
    while !dev.is_null() {
        libc::printf(cstr!("   %s : %s\n"), (*dev).name, if !(*dev).description.is_null() { (*dev).description } else { cstr!("no info provided") });
        dev = (*dev).next;
    }

    libc::printf(cstr!("\n"));

    pcap_sys::pcap_freealldevs(dev_list);
    0
}
