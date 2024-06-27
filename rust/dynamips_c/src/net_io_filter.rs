//! NetIO Filtering.

use crate::net_io::*;
use crate::prelude::*;

extern "C" {
    pub fn netio_filter_unbind(nio: *mut netio_desc_t, direction: c_int) -> c_int;
}

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
