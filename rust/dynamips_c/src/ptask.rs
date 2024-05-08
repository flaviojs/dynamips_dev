//! Periodic tasks centralization. Used for TX part of network devices.

use crate::dynamips_common::*;
use crate::prelude::*;

/// ptask identifier
pub type ptask_id_t = m_int64_t;

/// periodic task callback prototype
pub type ptask_callback = Option<unsafe extern "C" fn(object: *mut c_void, arg: *mut c_void) -> c_int>;

#[no_mangle]
pub extern "C" fn _export(_: ptask_id_t, _: ptask_callback) {}
