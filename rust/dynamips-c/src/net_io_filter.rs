//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! NetIO Packet Filters.
//! NetIO Filtering.

use crate::_private::*;
use crate::net_io::*;
#[cfg(feature = "ENABLE_GEN_ETH")]
use std::cmp::min;

pub const NETIO_FILTER_DIR_RX: c_int = 0;
pub const NETIO_FILTER_DIR_TX: c_int = 1;
pub const NETIO_FILTER_DIR_BOTH: c_int = 2;

/// Filter list
static mut pf_list: *mut netio_pktfilter_t = null_mut();

/// Find a filter
#[no_mangle]
pub unsafe extern "C" fn netio_filter_find(name: *mut c_char) -> *mut netio_pktfilter_t {
    let mut pf: *mut netio_pktfilter_t = pf_list;

    while !pf.is_null() {
        if libc::strcmp((*pf).name, name) == 0 {
            return pf;
        }
        pf = (*pf).next;
    }

    null_mut()
}

/// Add a new filter
#[no_mangle]
pub unsafe extern "C" fn netio_filter_add(pf: *mut netio_pktfilter_t) -> c_int {
    if !netio_filter_find((*pf).name).is_null() {
        return -1;
    }

    (*pf).next = pf_list;
    pf_list = pf;
    0
}

/// Bind a filter to a NIO
#[no_mangle]
pub unsafe extern "C" fn netio_filter_bind(nio: *mut netio_desc_t, direction: c_int, pf_name: *mut c_char) -> c_int {
    let pf: *mut netio_pktfilter_t = netio_filter_find(pf_name);

    if pf.is_null() {
        return -1;
    }

    if direction == NETIO_FILTER_DIR_RX {
        (*nio).rx_filter_data = null_mut();
        (*nio).rx_filter = pf;
    } else if direction == NETIO_FILTER_DIR_TX {
        (*nio).tx_filter_data = null_mut();
        (*nio).tx_filter = pf;
    } else {
        (*nio).both_filter_data = null_mut();
        (*nio).both_filter = pf;
    }
    0
}

/// Unbind a filter from a NIO
#[no_mangle]
pub unsafe extern "C" fn netio_filter_unbind(nio: *mut netio_desc_t, direction: c_int) -> c_int {
    let pf: *mut netio_pktfilter_t;
    let opt: *mut *mut c_void;

    if direction == NETIO_FILTER_DIR_RX {
        opt = addr_of_mut!((*nio).rx_filter_data);
        pf = (*nio).rx_filter;
    } else if direction == NETIO_FILTER_DIR_TX {
        opt = addr_of_mut!((*nio).tx_filter_data);
        pf = (*nio).tx_filter;
    } else {
        opt = addr_of_mut!((*nio).both_filter_data);
        pf = (*nio).both_filter;
    }

    if pf.is_null() {
        return -1;
    }

    (*pf).free.unwrap()(nio, opt);
    0
}

/// Setup a filter
#[no_mangle]
pub unsafe extern "C" fn netio_filter_setup(nio: *mut netio_desc_t, direction: c_int, argc: c_int, argv: *mut *mut c_char) -> c_int {
    let pf: *mut netio_pktfilter_t;
    let opt: *mut *mut c_void;

    if direction == NETIO_FILTER_DIR_RX {
        opt = addr_of_mut!((*nio).rx_filter_data);
        pf = (*nio).rx_filter;
    } else if direction == NETIO_FILTER_DIR_TX {
        opt = addr_of_mut!((*nio).tx_filter_data);
        pf = (*nio).tx_filter;
    } else {
        opt = addr_of_mut!((*nio).both_filter_data);
        pf = (*nio).both_filter;
    }

    if pf.is_null() {
        return -1;
    }

    (*pf).setup.unwrap()(nio, opt, argc, argv)
}

// ========================================================================
// Packet Capture ("capture")
// GFA
// ========================================================================

#[cfg(feature = "ENABLE_GEN_ETH")]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct netio_filter_capture {
    pub desc: *mut pcap_sys::pcap_t,
    pub dumper: *mut pcap_sys::pcap_dumper_t,
    pub lock: libc::pthread_mutex_t,
}

/// Free resources used by filter
#[cfg(feature = "ENABLE_GEN_ETH")]
unsafe extern "C" fn pf_capture_free(nio: *mut netio_desc_t, opt: *mut *mut c_void) {
    let c: *mut netio_filter_capture = (*opt).cast::<_>();

    if !c.is_null() {
        libc::printf(cstr!("NIO %s: ending packet capture.\n"), (*nio).name);

        // Close dumper
        if !(*c).dumper.is_null() {
            pcap_sys::pcap_dump_close((*c).dumper);
        }

        // Close PCAP descriptor
        if !(*c).desc.is_null() {
            pcap_sys::pcap_close((*c).desc);
        }

        libc::pthread_mutex_destroy(addr_of_mut!((*c).lock));

        libc::free(c.cast::<_>());
        *opt = null_mut();
    }
}

/// Setup filter resources
#[cfg(feature = "ENABLE_GEN_ETH")]
unsafe extern "C" fn pf_capture_setup(nio: *mut netio_desc_t, opt: *mut *mut c_void, argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut link_type: c_int;

    // We must have a link type and a filename
    if argc != 2 {
        return -1;
    }

    // Free resources if something has already been done
    pf_capture_free(nio, opt);

    // Allocate structure to hold PCAP info
    let c: *mut netio_filter_capture = libc::malloc(size_of::<netio_filter_capture>()).cast::<_>();
    if c.is_null() {
        return -1;
    }

    if libc::pthread_mutex_init(addr_of_mut!((*c).lock), null_mut()) != 0 {
        libc::fprintf(c_stderr(), cstr!("NIO %s: pthread_mutex_init failure (file %s)\n"), (*nio).name, *argv.add(0));
        libc::free(c.cast::<_>());
        return -1;
    }

    link_type = pcap_sys::pcap_datalink_name_to_val(*argv.add(0));
    if link_type == -1 {
        libc::fprintf(c_stderr(), cstr!("NIO %s: unknown link type %s, assuming Ethernet.\n"), (*nio).name, *argv.add(0));
        link_type = _pcap::DLT_EN10MB as c_int;
    }

    // Open a dead pcap descriptor
    (*c).desc = pcap_sys::pcap_open_dead(link_type, 65535);
    if (*c).desc.is_null() {
        libc::fprintf(c_stderr(), cstr!("NIO %s: pcap_open_dead failure\n"), (*nio).name);
        libc::pthread_mutex_destroy(addr_of_mut!((*c).lock));
        libc::free(c.cast::<_>());
        return -1;
    }

    // Open the output file
    (*c).dumper = pcap_sys::pcap_dump_open((*c).desc, *argv.add(1));
    if (*c).dumper.is_null() {
        libc::fprintf(c_stderr(), cstr!("NIO %s: pcap_dump_open failure (file %s)\n"), (*nio).name, *argv.add(0));
        pcap_sys::pcap_close((*c).desc);
        libc::pthread_mutex_destroy(addr_of_mut!((*c).lock));
        libc::free(c.cast::<_>());
        return -1;
    }

    libc::printf(cstr!("NIO %s: capturing to file '%s'\n"), (*nio).name, *argv.add(1));
    *opt = c.cast::<_>();
    0
}

/// Packet handler: write packets to a file in CAP format
#[cfg(feature = "ENABLE_GEN_ETH")]
unsafe extern "C" fn pf_capture_pkt_handler(_nio: *mut netio_desc_t, pkt: *mut c_void, len: size_t, opt: *mut c_void) -> c_int {
    let c: *mut netio_filter_capture = opt.cast::<_>();
    let mut pkt_hdr: pcap_sys::pcap_pkthdr = zeroed::<_>();

    if !c.is_null() {
        libc::gettimeofday(addr_of_mut!(pkt_hdr.ts), null_mut());
        pkt_hdr.caplen = min(len as u_int, pcap_sys::pcap_snapshot((*c).desc) as u_int);
        pkt_hdr.len = len as u_int;

        // thread safe dump
        libc::pthread_mutex_lock(addr_of_mut!((*c).lock));
        pcap_sys::pcap_dump((*c).dumper as *mut u_char, addr_of_mut!(pkt_hdr), pkt.cast::<_>());
        pcap_sys::pcap_dump_flush((*c).dumper);
        libc::pthread_mutex_unlock(addr_of_mut!((*c).lock));
    }

    NETIO_FILTER_ACTION_PASS
}

/// Packet capture
#[cfg(feature = "ENABLE_GEN_ETH")]
#[rustfmt::skip]
static mut pf_capture_def: netio_pktfilter_t = netio_pktfilter_t {
    name: cstr!("capture"),
    setup: Some(pf_capture_setup),
    free: Some(pf_capture_free),
    pkt_handler: Some(pf_capture_pkt_handler),
    next: null_mut()
};

// ========================================================================
// Frequency Dropping ("freq_drop").
// ========================================================================

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pf_freqdrop_data {
    pub frequency: c_int,
    pub current: c_int,
}

/// Setup filter ressources
unsafe extern "C" fn pf_freqdrop_setup(_nio: *mut netio_desc_t, opt: *mut *mut c_void, argc: c_int, argv: *mut *mut c_char) -> c_int {
    let mut data: *mut pf_freqdrop_data = (*opt).cast::<_>();

    if argc != 1 {
        return -1;
    }

    if data.is_null() {
        data = libc::malloc(size_of::<pf_freqdrop_data>()).cast::<_>();
        if data.is_null() {
            return -1;
        }

        *opt = data.cast::<_>();
    }

    (*data).current = 0;
    (*data).frequency = libc::atoi(*argv.add(0));
    0
}

/// Free ressources used by filter
unsafe extern "C" fn pf_freqdrop_free(_nio: *mut netio_desc_t, opt: *mut *mut c_void) {
    if !(*opt).is_null() {
        libc::free(*opt);
    }

    *opt = null_mut();
}

/// Packet handler: drop 1 out of n packets
unsafe extern "C" fn pf_freqdrop_pkt_handler(_nio: *mut netio_desc_t, _pkt: *mut c_void, _len: size_t, opt: *mut c_void) -> c_int {
    let data: *mut pf_freqdrop_data = opt.cast::<_>();

    if !data.is_null() {
        match (*data).frequency {
            -1 => {
                return NETIO_FILTER_ACTION_DROP;
            }
            0 => {
                return NETIO_FILTER_ACTION_PASS;
            }
            _ => {
                (*data).current += 1;

                if (*data).current == (*data).frequency {
                    (*data).current = 0;
                    return NETIO_FILTER_ACTION_DROP;
                }
            }
        }
    }

    NETIO_FILTER_ACTION_PASS
}

/// Packet dropping at 1/n frequency
#[rustfmt::skip]
static mut pf_freqdrop_def: netio_pktfilter_t = netio_pktfilter_t {
    name: cstr!("freq_drop"),
    setup: Some(pf_freqdrop_setup),
    free: Some(pf_freqdrop_free),
    pkt_handler: Some(pf_freqdrop_pkt_handler),
    next: null_mut(),
};

// ========================================================================
// Initialization of packet filters.
// ========================================================================

#[no_mangle]
pub unsafe extern "C" fn netio_filter_load_all() {
    netio_filter_add(addr_of_mut!(pf_freqdrop_def));
    #[cfg(feature = "ENABLE_GEN_ETH")]
    netio_filter_add(addr_of_mut!(pf_capture_def));
}
