//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Periodic tasks centralization. Used for TX part of network devices.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::utils::*;

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

static mut ptask_thread: libc::pthread_t = 0;
static mut ptask_mutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
static mut ptask_list: *mut ptask_t = null_mut();
static mut ptask_current_id: ptask_id_t = 0;

#[no_mangle]
pub static mut ptask_sleep_time: u_int = 10;

unsafe fn PTASK_LOCK() {
    libc::pthread_mutex_lock(addr_of_mut!(ptask_mutex));
}
unsafe fn PTASK_UNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!(ptask_mutex));
}

/// Periodic task thread
extern "C" fn ptask_run(_arg: *mut c_void) -> *mut c_void {
    unsafe {
        let mut umutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
        let mut ucond: libc::pthread_cond_t = libc::PTHREAD_COND_INITIALIZER;

        loop {
            PTASK_LOCK();
            let mut task: *mut ptask_t = ptask_list;
            while !task.is_null() {
                (*task).cbk.unwrap()((*task).object, (*task).arg);
                task = (*task).next;
            }
            PTASK_UNLOCK();

            // For testing!
            {
                let mut t_spc: libc::timespec = zeroed::<_>();
                let expire: m_tmcnt_t = m_gettime_usec() + (ptask_sleep_time * 1000) as m_tmcnt_t;

                libc::pthread_mutex_lock(addr_of_mut!(umutex));
                t_spc.tv_sec = (expire / 1000000) as _;
                t_spc.tv_nsec = ((expire % 1000000) * 1000) as _;
                libc::pthread_cond_timedwait(addr_of_mut!(ucond), addr_of_mut!(umutex), addr_of_mut!(t_spc));
                libc::pthread_mutex_unlock(addr_of_mut!(umutex));
            }

            // Old method...
            if false {
                libc::usleep(ptask_sleep_time * 1000);
            }
        }
    }
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

/// Initialize ptask module
#[no_mangle]
pub unsafe extern "C" fn ptask_init(sleep_time: c_uint) -> c_int {
    if sleep_time != 0 {
        ptask_sleep_time = sleep_time;
    }

    if libc::pthread_create(addr_of_mut!(ptask_thread), null_mut(), ptask_run, null_mut()) != 0 {
        libc::fprintf(c_stderr(), cstr!("ptask_init: unable to create thread.\n"));
        return -1;
    }

    0
}
