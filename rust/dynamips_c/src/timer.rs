//! Management of timers.

use crate::dynamips_common::*;
use crate::hash::*;
use crate::prelude::*;
use crate::utils::*;

/// Default number of Timer Queues
pub const TIMERQ_NUMBER: c_int = 10;

/// Timer definitions
pub type timer_id = m_uint64_t;

pub type timer_entry_t = timer_entry;
pub type timer_queue_t = timer_queue;

/// Defines callback function format
pub type timer_proc = Option<unsafe extern "C" fn(arg1: *mut c_void, arg2: *mut timer_entry_t) -> c_int>;

/// Timer flags
pub const TIMER_DELETED: c_int = 1;
pub const TIMER_RUNNING: c_int = 2;
pub const TIMER_BOUNDARY: c_int = 4;

/// Number of entries in hash table
pub const TIMER_HASH_SIZE: c_int = 512;

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

/// Lock and unlock access to a timer queue
unsafe fn TIMERQ_LOCK(queue: *mut timer_queue_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*queue).lock));
}
unsafe fn TIMERQ_UNLOCK(queue: *mut timer_queue_t) {
    libc::pthread_mutex_unlock(addr_of_mut!((*queue).lock));
}

/// Lock and unlock access to global structures
unsafe fn TIMER_LOCK() {
    libc::pthread_mutex_lock(addr_of_mut!(timer_mutex));
}
unsafe fn TIMER_UNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!(timer_mutex));
}

/// Hash table to map Timer ID to timer entries
#[no_mangle]
pub static mut timer_id_hash: *mut hash_table_t = null_mut(); // TODO private

/// Pool of Timer Queues
#[no_mangle]
pub static mut timer_queue_pool: *mut timer_queue_t = null_mut(); // TODO private

/// Last ID used.
#[no_mangle]
pub static mut timer_next_id: timer_id = 1; // TODO private

/// Mutex to access to global structures (Hash Tables, Pool of queues, ...)
#[no_mangle]
pub static mut timer_mutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER; // TODO private

/// Find a timer by its ID
unsafe fn timer_find_by_id(mut id: timer_id) -> *mut timer_entry_t {
    hash_table_lookup(timer_id_hash, addr_of_mut!(id).cast::<_>()).cast::<_>()
}

/// Allocate a new ID. Disgusting method but it should work.
unsafe fn timer_alloc_id() -> timer_id {
    while !hash_table_lookup(timer_id_hash, addr_of_mut!(timer_next_id).cast::<_>()).is_null() {
        timer_next_id += 1;
    }

    timer_next_id
}

/// Free an ID
unsafe fn timer_free_id(mut id: timer_id) {
    hash_table_remove(timer_id_hash, addr_of_mut!(id).cast::<_>());
}

/// Select the queue of the pool that has the lowest criticity level. This
// is a stupid method.
unsafe fn timer_select_queue_from_pool() -> *mut timer_queue_t {
    // to begin, select the first queue of the pool
    let mut s_queue: *mut timer_queue_t = timer_queue_pool;
    let mut level: c_int = (*s_queue).level;

    // walk through timer queues
    let mut queue: *mut timer_queue_t = (*timer_queue_pool).next;
    while !queue.is_null() {
        if (*queue).level < level {
            level = (*queue).level;
            s_queue = queue;
        }
        queue = (*queue).next;
    }

    // returns selected queue
    s_queue
}

/// Add a timer in a queue
unsafe fn timer_add_to_queue(queue: *mut timer_queue_t, timer: *mut timer_entry_t) {
    // Insert after the last timer with the same or earlier time
    let mut t: *mut timer_entry_t = (*queue).list.get();
    let mut prev: *mut timer_entry_t = null_mut();
    while !t.is_null() {
        if (*t).expire > (*timer).expire {
            break;
        }
        prev = t;
        t = (*t).next;
    }

    // Add it in linked list
    (*timer).next = t;
    (*timer).prev = prev;
    (*timer).queue = queue;

    if !(*timer).next.is_null() {
        (*(*timer).next).prev = timer;
    }

    if !(*timer).prev.is_null() {
        (*(*timer).prev).next = timer;
    } else {
        (*queue).list.set(timer);
    }

    // Increment number of timers in queue
    (*queue).timer_count += 1;

    // Increment criticity level
    (*queue).level += (*timer).level;
}

/// Remove a timer from queue
unsafe fn timer_remove_from_queue(queue: *mut timer_queue_t, timer: *mut timer_entry_t) {
    if !(*timer).prev.is_null() {
        (*(*timer).prev).next = (*timer).next;
    } else {
        (*queue).list.set((*timer).next);
    }

    if !(*timer).next.is_null() {
        (*(*timer).next).prev = (*timer).prev;
    }

    (*timer).next = null_mut();
    (*timer).prev = null_mut();

    // Decrement number of timers in queue
    (*queue).timer_count -= 1;

    // Decrement criticity level
    (*queue).level -= (*timer).level;
}

/// Remove a timer from a queue atomically
unsafe fn timer_remove_from_queue_atomic(queue: *mut timer_queue_t, timer: *mut timer_entry_t) {
    TIMERQ_LOCK(queue);
    timer_remove_from_queue(queue, timer);
    TIMERQ_UNLOCK(queue);
}

/// Free ressources used by a timer
unsafe fn timer_free(timer: *mut timer_entry_t, take_lock: c_int) {
    if take_lock != 0 {
        TIMER_LOCK();
    }

    // Remove ID from hash table
    hash_table_remove(timer_id_hash, addr_of_mut!((*timer).id).cast::<_>());

    if take_lock != 0 {
        TIMER_UNLOCK();
    }

    // Free memory used by timer
    libc::free(timer.cast::<_>());
}

/// Remove a timer
#[no_mangle]
pub unsafe extern "C" fn timer_remove(id: timer_id) -> c_int {
    TIMER_LOCK();

    // Find timer
    let timer: *mut timer_entry_t = timer_find_by_id(id);
    if timer.is_null() {
        TIMER_UNLOCK();
        return -1;
    }

    // If we have a queue, remove timer from it atomically
    let mut queue: *mut timer_queue_t = null_mut();
    if !(*timer).queue.is_null() {
        queue = (*timer).queue;
        timer_remove_from_queue_atomic(queue, timer);
    }

    // Release timer ID
    timer_free_id(id);

    // Free memory used by timer
    libc::free(timer.cast::<_>());
    TIMER_UNLOCK();

    // Signal to this queue that it has been modified
    if !queue.is_null() {
        libc::pthread_cond_signal(addr_of_mut!((*queue).schedule));
    }
    0
}

/// Schedule a timer in a queue
unsafe fn timer_schedule_in_queue(queue: *mut timer_queue_t, timer: *mut timer_entry_t) {
    // Set new expiration date and clear "run" flag
    if ((*timer).flags & TIMER_BOUNDARY) != 0 {
        let current_adj: m_tmcnt_t = m_gettime_adj();
        let current = m_gettime();

        (*timer).expire = current + (*timer).offset + ((*timer).interval as m_tmcnt_t - (current_adj % (*timer).interval as m_tmcnt_t));
    } else {
        (*timer).expire += (*timer).interval as m_tmcnt_t;
    }

    (*timer).flags &= !TIMER_RUNNING;
    timer_add_to_queue(queue, timer);
}

/// Schedule a timer
unsafe fn timer_schedule(timer: *mut timer_entry_t) -> c_int {
    // Select the least used queue of the pool
    let queue: *mut timer_queue_t = timer_select_queue_from_pool();
    if queue.is_null() {
        libc::fprintf(c_stderr(), cstr!("timer_schedule: no pool available for timer with ID %llu"), (*timer).id as c_ulonglong);
        return -1;
    }

    // Reschedule it in queue
    TIMERQ_LOCK(queue);
    timer_schedule_in_queue(queue, timer);
    TIMERQ_UNLOCK(queue);
    0
}

/// Enable a timer
unsafe fn timer_enable(timer: *mut timer_entry_t) -> timer_id {
    // Allocate a new ID
    TIMER_LOCK();
    (*timer).id = timer_alloc_id();

    // Insert ID in hash table
    if hash_table_insert(timer_id_hash, addr_of_mut!((*timer).id).cast::<_>(), timer.cast::<_>()) == -1 {
        TIMER_UNLOCK();
        libc::free(timer.cast::<_>());
        return 0;
    }

    // Schedule event
    if timer_schedule(timer) == -1 {
        timer_free(timer, 0);
        TIMER_UNLOCK();
        return 0;
    }

    // Returns timer ID
    TIMER_UNLOCK();
    libc::pthread_cond_signal(addr_of_mut!((*(*timer).queue).schedule));
    (*timer).id
}

/// Create a new timer
#[no_mangle]
pub unsafe extern "C" fn timer_create_entry(interval: m_tmcnt_t, boundary: c_int, level: c_int, callback: timer_proc, user_arg: *mut c_void) -> timer_id {
    // Allocate memory for new timer entry
    let timer: *mut timer_entry_t = libc::malloc(size_of::<timer_entry_t>()).cast::<_>();
    if timer.is_null() {
        return 0;
    }

    (*timer).interval = interval as c_long;
    (*timer).offset = 0;
    (*timer).callback = callback;
    (*timer).user_arg = user_arg;
    (*timer).flags = 0;
    (*timer).level = level;

    // Set expiration delay
    if boundary != 0 {
        (*timer).flags |= TIMER_BOUNDARY;
    } else {
        (*timer).expire = m_gettime();
    }

    timer_enable(timer)
}

/// Create a timer on boundary, with an offset
#[no_mangle]
pub unsafe extern "C" fn timer_create_with_offset(interval: m_tmcnt_t, _offset: m_tmcnt_t, level: c_int, callback: timer_proc, user_arg: *mut c_void) -> timer_id {
    // Allocate memory for new timer entry
    let timer: *mut timer_entry_t = libc::malloc(size_of::<timer_entry_t>()).cast::<_>();
    if timer.is_null() {
        return 0;
    }

    (*timer).interval = interval as c_long;
    (*timer).offset = 0; // FIXME offset argument is not used
    (*timer).callback = callback;
    (*timer).user_arg = user_arg;
    (*timer).flags = 0;
    (*timer).level = level;
    (*timer).flags |= TIMER_BOUNDARY;

    timer_enable(timer)
}

#[no_mangle]
pub extern "C" fn _export(_: timer_id, _: *mut timer_entry_t, _: *mut timer_queue_t, _: timer_proc) {}
