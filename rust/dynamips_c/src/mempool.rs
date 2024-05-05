//! Simple Memory Pools.

use crate::dynamips_common::*;
use crate::prelude::*;

/// Memory Pool "Fixed" Flag
pub const MEMPOOL_FIXED: c_int = 1;

/// Dummy value used to check if a memory block is invalid
pub const MEMBLOCK_TAG: c_int = 0xdeadbeef_u32 as c_int;

pub type memblock_t = memblock;
pub type mempool_t = mempool;

/// Memory block
#[repr(C)]
#[derive(Debug)]
pub struct memblock {
    /// MEMBLOCK_TAG if block is valid
    pub tag: c_int,
    /// Block size (without header)
    pub block_size: size_t,
    /// Double linked list pointers
    pub next: *mut memblock_t,
    pub prev: *mut memblock_t,
    /// Pool which contains this block
    pub pool: *mut mempool_t,
    /// Memory block itself
    pub data: [m_uint64_t; 0],
}

/// Memory Pool
#[repr(C)]
#[derive(Copy, Clone)]
pub struct mempool {
    /// Double-linked block list
    pub block_list: *mut memblock_t,
    /// Mutex for managing pool
    pub lock: libc::pthread_mutex_t,
    /// Name of this pool
    pub name: *mut c_char,
    /// Flags
    pub flags: c_int,
    /// Number of blocks in this pool
    pub nr_blocks: c_int,
    /// Total bytes allocated
    pub total_size: size_t,
    /// Maximum memory
    pub max_size: size_t,
}

/// Lock and unlock access to a memory pool
unsafe fn MEMPOOL_LOCK(pool: *mut mempool_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*pool).lock));
}
unsafe fn MEMPOOL_UNLOCK(pool: *mut mempool_t) {
    libc::pthread_mutex_unlock(addr_of_mut!((*pool).lock));
}

// Internal function used to allocate a memory block, and do basic operations
// on it. It does not manipulate pools, so no mutex is needed.
unsafe fn memblock_alloc(size: size_t, zeroed: c_int) -> *mut memblock_t {
    let total_size: size_t = size + size_of::<memblock_t>();
    let block: *mut memblock_t = libc::malloc(total_size).cast::<_>();
    if block.is_null() {
        return null_mut();
    }

    if zeroed != 0 {
        libc::memset(block.cast::<_>(), 0, total_size);
    }

    (*block).tag = MEMBLOCK_TAG;
    (*block).block_size = size;
    (*block).prev = null_mut();
    (*block).next = null_mut();
    block
}

/// Insert block in linked list
unsafe fn memblock_insert(pool: *mut mempool_t, block: *mut memblock_t) {
    MEMPOOL_LOCK(pool);

    (*pool).nr_blocks += 1;
    (*pool).total_size += (*block).block_size;

    (*block).prev = null_mut();
    (*block).next = (*pool).block_list;

    if !(*block).next.is_null() {
        (*(*block).next).prev = block;
    }

    (*pool).block_list = block;

    MEMPOOL_UNLOCK(pool);
}

/// Allocate a new block in specified pool (internal function)
unsafe fn mp_alloc_inline(pool: *mut mempool_t, size: size_t, zeroed: c_int) -> *mut c_void {
    let block: *mut memblock_t = memblock_alloc(size, zeroed);
    if block.is_null() {
        return null_mut();
    }

    (*block).pool = pool;
    memblock_insert(pool, block);
    (*block).data.as_c_void_mut()
}

/// Allocate a new block in specified pool
#[no_mangle]
pub unsafe extern "C" fn mp_alloc(pool: *mut mempool_t, size: size_t) -> *mut c_void {
    mp_alloc_inline(pool, size, TRUE)
}

/// Allocate a new block which will not be zeroed
#[no_mangle]
pub unsafe extern "C" fn mp_alloc_n0(pool: *mut mempool_t, size: size_t) -> *mut c_void {
    mp_alloc_inline(pool, size, FALSE)
}

#[no_mangle]
pub extern "C" fn _export(_: *mut memblock_t, _: *mut mempool_t) {}
