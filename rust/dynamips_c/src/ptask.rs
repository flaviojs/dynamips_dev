//! Periodic tasks centralization. Used for TX part of network devices.

use crate::dynamips_common::*;
use crate::prelude::*;

pub type ptask_t = ptask;

/// ptask identifier
pub type ptask_id_t = m_int64_t;

/// periodic task callback prototype
pub type ptask_callback = Option<unsafe extern "C" fn(object: *mut c_void, arg: *mut c_void) -> c_int>;

/// periodic task definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ptask {
    pub id: ptask_id_t,
    pub next: *mut ptask_t,
    pub cbk: ptask_callback,
    pub object: *mut c_void,
    pub arg: *mut c_void,
}

#[no_mangle]
pub static mut ptask_sleep_time: c_uint = 10;

#[no_mangle]
pub extern "C" fn _export(_: ptask_id_t, _: ptask_callback, _: *mut ptask_t) {}
