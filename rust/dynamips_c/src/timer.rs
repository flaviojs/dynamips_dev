//! Management of timers.

use crate::dynamips_common::*;
use crate::prelude::*;

/// Default number of Timer Queues
pub const TIMERQ_NUMBER: c_int = 10;

/// Timer definitions
pub type timer_id = m_uint64_t;

pub type timer_entry_t = timer_entry;
pub type timer_queue_t = timer_queue;

/// Defines callback function format
pub type timer_proc = Option<unsafe extern "C" fn(arg1: *mut c_void, arg2: *mut timer_entry_t) -> c_int>;

/// Timer properties
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timer_entry {
    /// Interval in msecs
    pub interval: c_long,
    /// Next execution date
    pub expire: m_tmcnt_t,
    pub offset: m_tmcnt_t,
    /// User callback function
    pub callback: timer_proc,
    /// Optional user data
    pub user_arg: *mut c_void,
    /// Flags
    pub flags: c_int,
    /// Unique identifier
    pub id: timer_id,
    /// Criticity level
    pub level: c_int,
    /// Associated Timer Queue
    pub queue: *mut timer_queue_t,
    /// Double linked-list
    pub prev: *mut timer_entry_t,
    pub next: *mut timer_entry_t,
}

/// Timer Queue
#[repr(C)]
#[derive(Copy, Clone)]
pub struct timer_queue {
    /// List of timers
    pub list: Volatile<*mut timer_entry_t>,
    /// Mutex for concurrent accesses
    pub lock: libc::pthread_mutex_t,
    /// Scheduling condition
    pub schedule: libc::pthread_cond_t,
    /// Thread running timer loop
    pub thread: libc::pthread_t,
    /// Running flag
    pub running: Volatile<c_int>,
    /// Number of timers
    pub timer_count: c_int,
    /// Sum of criticity levels
    pub level: c_int,
    /// Next Timer Queue (for pools)
    pub next: *mut timer_queue_t,
}

#[no_mangle]
pub extern "C" fn _export(_: timer_id, _: *mut timer_entry_t, _: *mut timer_queue_t, _: timer_proc) {}
