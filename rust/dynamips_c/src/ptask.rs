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
pub static mut ptask_mutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER; // TODO private
#[no_mangle]
pub static mut ptask_list: *mut ptask_t = null_mut(); // TODO private
static mut ptask_current_id: ptask_id_t = 0;

#[no_mangle]
pub static mut ptask_sleep_time: c_uint = 10;

unsafe fn PTASK_LOCK() {
    libc::pthread_mutex_lock(addr_of_mut!(ptask_mutex));
}
unsafe fn PTASK_UNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!(ptask_mutex));
}

/// Add a new task
#[no_mangle]
pub unsafe extern "C" fn ptask_add(cbk: ptask_callback, object: *mut c_void, arg: *mut c_void) -> ptask_id_t {
    let task: *mut ptask_t = libc::malloc(size_of::<ptask_t>()).cast::<_>();
    if task.is_null() {
        libc::fprintf(c_stderr(), cstr!("ptask_add: unable to add new task.\n"));
        return -1;
    }

    libc::memset(task.cast::<_>(), 0, size_of::<ptask_t>());
    (*task).cbk = cbk;
    (*task).object = object;
    (*task).arg = arg;

    PTASK_LOCK();
    ptask_current_id += 1;
    let id: ptask_id_t = ptask_current_id;
    assert!(id != 0);
    (*task).id = id;
    (*task).next = ptask_list;
    ptask_list = task;
    PTASK_UNLOCK();
    id
}

/// Remove a task
#[no_mangle]
pub unsafe extern "C" fn ptask_remove(id: ptask_id_t) -> c_int {
    let mut res: c_int = -1;

    PTASK_LOCK();

    let mut task: *mut *mut ptask_t = addr_of_mut!(ptask_list);
    while !(*task).is_null() {
        if (**task).id == id {
            let p: *mut ptask_t = *task;
            *task = (**task).next;
            libc::free(p.cast::<_>());
            res = 0;
            break;
        }
        task = addr_of_mut!((**task).next);
    }

    PTASK_UNLOCK();
    res
}

#[no_mangle]
pub extern "C" fn _export(_: ptask_id_t, _: ptask_callback, _: *mut ptask_t) {}
