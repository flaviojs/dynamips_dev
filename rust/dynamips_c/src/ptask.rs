//! Periodic tasks centralization. Used for TX part of network devices.

use crate::dynamips_common::*;

/// ptask identifier
pub type ptask_id_t = m_int64_t;

#[no_mangle]
pub extern "C" fn _export(_: ptask_id_t) {}
