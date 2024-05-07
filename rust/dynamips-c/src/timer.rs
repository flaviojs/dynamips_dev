//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Management of timers.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::hash::*;
use crate::utils::*;

pub type timer_entry_t = timer_entry;
pub type timer_queue_t = timer_queue;

/// Default number of Timer Queues
pub const TIMERQ_NUMBER: c_int = 10;

/// Timer definitions
pub type timer_id = m_uint64_t;

/// Defines callback function format
pub type timer_proc = Option<unsafe extern "C" fn(_: *mut c_void, _: *mut timer_entry_t) -> c_int>;

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
    pub interval: c_long,  // Interval in msecs
    pub expire: m_tmcnt_t, // Next execution date
    pub offset: m_tmcnt_t,
    pub callback: timer_proc,  // User callback function
    pub user_arg: *mut c_void, // Optional user data
    pub flags: c_int,          // Flags
    pub id: timer_id,          // Unique identifier
    pub level: c_int,          // Criticity level

    pub queue: *mut timer_queue_t, // Associated Timer Queue
    pub prev: *mut timer_entry_t,  // Double linked-list
    pub next: *mut timer_entry_t,
}

/// Timer Queue
#[repr(C)]
#[derive(Copy, Clone)]
pub struct timer_queue {
    pub list: Volatile<*mut timer_entry_t>, // List of timers
    pub lock: libc::pthread_mutex_t,        // Mutex for concurrent accesses
    pub schedule: libc::pthread_cond_t,     // Scheduling condition
    pub thread: libc::pthread_t,            // Thread running timer loop
    pub running: Volatile<c_int>,           // Running flag
    pub timer_count: c_int,                 // Number of timers
    pub level: c_int,                       // Sum of criticity levels
    pub next: *mut timer_queue_t,           // Next Timer Queue (for pools)
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

/// Pool of Timer Queues
static mut timer_queue_pool: *mut timer_queue_t = null_mut();

/// Hash table to map Timer ID to timer entries
static mut timer_id_hash: *mut hash_table_t = null_mut();

/// Last ID used.
static mut timer_next_id: timer_id = 1;

/// Mutex to access to global structures (Hash Tables, Pool of queues, ...)
static mut timer_mutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;

/// Find a timer by its ID
#[inline]
unsafe fn timer_find_by_id(mut id: timer_id) -> *mut timer_entry_t {
    hash_table_lookup(timer_id_hash, addr_of_mut!(id).cast::<_>()).cast::<_>()
}

/// Allocate a new ID. Disgusting method but it should work.
#[inline]
unsafe fn timer_alloc_id() -> timer_id {
    while !hash_table_lookup(timer_id_hash, addr_of_mut!(timer_next_id).cast::<_>()).is_null() {
        timer_next_id += 1;
    }

    timer_next_id
}

/// Free an ID
#[inline]
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
#[inline]
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

/// Add a timer in a queue atomically
#[inline]
unsafe fn timer_add_to_queue_atomic(queue: *mut timer_queue_t, timer: *mut timer_entry_t) {
    TIMERQ_LOCK(queue);
    timer_add_to_queue(queue, timer);
    TIMERQ_UNLOCK(queue);
}

/// Remove a timer from queue
#[inline]
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
#[inline]
unsafe fn timer_remove_from_queue_atomic(queue: *mut timer_queue_t, timer: *mut timer_entry_t) {
    TIMERQ_LOCK(queue);
    timer_remove_from_queue(queue, timer);
    TIMERQ_UNLOCK(queue);
}

/// Free ressources used by a timer
#[inline]
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

/// Run timer action
#[inline]
unsafe fn timer_exec(timer: *mut timer_entry_t) -> c_int {
    (*timer).callback.unwrap()((*timer).user_arg, timer)
}

/// Schedule a timer in a queue
#[inline]
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

/// Timer loop
extern "C" fn timer_loop(queue: *mut c_void) -> *mut c_void {
    unsafe {
        let queue: *mut timer_queue_t = queue.cast::<_>();
        // Set signal properties
        m_signal_block(libc::SIGINT);
        m_signal_block(libc::SIGQUIT);
        m_signal_block(libc::SIGTERM);

        loop {
            // Prevent asynchronous access problems
            TIMERQ_LOCK(queue);

            // We need to check "running" flags to know if we must stop
            if (*queue).running.get() == 0 {
                TIMERQ_UNLOCK(queue);
                break;
            }

            // Get first event
            let mut timer: *mut timer_entry_t = (*queue).list.get();

            // If we have timers in queue, we setup a timer to wait for first one.
            // In all cases, thread is woken up when a reschedule occurs.
            if !timer.is_null() {
                let mut t_spc: libc::timespec = zeroed::<_>();
                t_spc.tv_sec = ((*timer).expire / 1000) as _;
                t_spc.tv_nsec = (((*timer).expire % 1000) * 1000000) as _;
                libc::pthread_cond_timedwait(addr_of_mut!((*queue).schedule), addr_of_mut!((*queue).lock), addr_of_mut!(t_spc));
            } else {
                // We just wait for reschedule since we don't have any timer
                libc::pthread_cond_wait(addr_of_mut!((*queue).schedule), addr_of_mut!((*queue).lock));
            }

            // We need to check "running" flags to know if we must stop
            if (*queue).running.get() == 0 {
                TIMERQ_UNLOCK(queue);
                break;
            }

            // Now, we need to find why we were woken up. So, we compare current
            // time with first timer to see if we must execute action associated
            // with it.
            let c_time: m_tmcnt_t = m_gettime();

            // Get first event
            timer = (*queue).list.get();

            // If there is nothing to do for now, wait again
            if timer.is_null() || (*timer).expire > c_time {
                TIMERQ_UNLOCK(queue);
                continue;
            }

            // We have a timer to manage. Remove it from queue and mark it as
            // running.
            timer_remove_from_queue(queue, timer);
            (*timer).flags |= TIMER_RUNNING;

            // Execute user function and reschedule timer if required
            if timer_exec(timer) != 0 {
                timer_schedule_in_queue(queue, timer);
            }

            TIMERQ_UNLOCK(queue);
        }

        null_mut()
    }
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

/// Set a new interval for a timer
#[no_mangle]
pub unsafe extern "C" fn timer_set_interval(id: timer_id, interval: c_long) -> c_int {
    TIMER_LOCK();

    // Locate timer
    let timer: *mut timer_entry_t = timer_find_by_id(id);
    if timer.is_null() {
        TIMER_UNLOCK();
        return -1;
    }

    let queue: *mut timer_queue_t = (*timer).queue;

    TIMERQ_LOCK(queue);

    // Compute new expiration date
    (*timer).interval = interval;
    (*timer).expire = m_gettime() + interval as m_tmcnt_t;

    timer_remove_from_queue(queue, timer);
    timer_schedule_in_queue(queue, timer);

    TIMERQ_UNLOCK(queue);
    TIMER_UNLOCK();

    // Reschedule
    libc::pthread_cond_signal(addr_of_mut!((*queue).schedule));
    0
}

/// Create a new timer queue
#[no_mangle]
pub unsafe extern "C" fn timer_create_queue() -> *mut timer_queue_t {
    // Create new queue structure
    let queue: *mut timer_queue_t = libc::malloc(size_of::<timer_queue_t>()).cast::<_>();
    if queue.is_null() {
        return null_mut();
    }

    (*queue).running.set(1);
    (*queue).list.set(null_mut());
    (*queue).level = 0;

    // Create mutex
    if libc::pthread_mutex_init(addr_of_mut!((*queue).lock), null_mut()) != 0 {
        libc::free(queue.cast::<_>());
        return null_mut();
    }

    // Create condition
    if libc::pthread_cond_init(addr_of_mut!((*queue).schedule), null_mut()) != 0 {
        libc::pthread_mutex_destroy(addr_of_mut!((*queue).lock));
        libc::free(queue.cast::<_>());
        return null_mut();
    }

    // Create thread
    if libc::pthread_create(addr_of_mut!((*queue).thread), null_mut(), timer_loop, queue.cast::<_>()) != 0 {
        // (void *(*)(void *))
        libc::pthread_cond_destroy(addr_of_mut!((*queue).schedule));
        libc::pthread_mutex_destroy(addr_of_mut!((*queue).lock));
        libc::free(queue.cast::<_>());
        return null_mut();
    }

    queue
}

/// Flush queues
#[no_mangle]
pub unsafe extern "C" fn timer_flush_queues() {
    TIMER_LOCK();

    let mut queue: *mut timer_queue_t = timer_queue_pool;
    while !queue.is_null() {
        TIMERQ_LOCK(queue);
        let next_queue: *mut timer_queue_t = (*queue).next;
        let thread: libc::pthread_t = (*queue).thread;

        // mark queue as not running
        (*queue).running.set(0);

        // suppress all timers
        let mut timer: *mut timer_entry_t = (*queue).list.get();
        while !timer.is_null() {
            let next_timer: *mut timer_entry_t = (*timer).next;
            timer_free_id((*timer).id);
            libc::free(timer.cast::<_>());
            timer = next_timer;
        }

        // signal changes to the queue thread
        libc::pthread_cond_signal(addr_of_mut!((*queue).schedule));

        TIMERQ_UNLOCK(queue);

        // wait for thread to terminate
        libc::pthread_join(thread, null_mut());

        libc::pthread_cond_destroy(addr_of_mut!((*queue).schedule));
        libc::pthread_mutex_destroy(addr_of_mut!((*queue).lock));
        libc::free(queue.cast::<_>());
        queue = next_queue;
    }
    timer_queue_pool = null_mut();

    TIMER_UNLOCK();
}

/// Add a specified number of queues to the pool
#[no_mangle]
pub unsafe extern "C" fn timer_pool_add_queues(nr_queues: c_int) -> c_int {
    for _ in 0..nr_queues {
        let queue: *mut timer_queue_t = timer_create_queue();
        if queue.is_null() {
            return -1;
        }

        TIMER_LOCK();
        (*queue).next = timer_queue_pool;
        timer_queue_pool = queue;
        TIMER_UNLOCK();
    }

    0
}

/// Terminate timer sub-system
extern "C" fn timer_terminate() {
    unsafe {
        timer_flush_queues();

        assert!(!timer_id_hash.is_null());
        hash_table_delete(timer_id_hash);
        timer_id_hash = null_mut();
    }
}

/// Initialize timer sub-system
#[no_mangle]
pub unsafe extern "C" fn timer_init() -> c_int {
    // Initialize hash table which maps ID to timer entries
    assert!(timer_id_hash.is_null());
    timer_id_hash = hash_u64_create(TIMER_HASH_SIZE);
    if timer_id_hash.is_null() {
        libc::fprintf(c_stderr(), cstr!("timer_init: unable to create hash table."));
        return -1;
    }

    // Initialize default queues. If this fails, try to continue.
    if timer_pool_add_queues(TIMERQ_NUMBER) == -1 {
        libc::fprintf(c_stderr(), cstr!("timer_init: unable to initialize at least one timer queue."));
    }

    libc::atexit(timer_terminate);

    0
}
