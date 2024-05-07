//! Management of timers.

use crate::dynamips_common::*;
use crate::prelude::*;

/// Default number of Timer Queues
pub const TIMERQ_NUMBER: c_int = 10;

/// Timer definitions
pub type timer_id = m_uint64_t;

#[no_mangle]
pub extern "C" fn _export(_: timer_id) {}
