//! NetIO Filtering.

use crate::_private::*;
use crate::net_io::*;

extern "C" {
    pub fn netio_filter_unbind(nio: *mut netio_desc_t, direction: c_int) -> c_int;
}

pub const NETIO_FILTER_DIR_RX: c_int = 0;
pub const NETIO_FILTER_DIR_TX: c_int = 1;
pub const NETIO_FILTER_DIR_BOTH: c_int = 2;
