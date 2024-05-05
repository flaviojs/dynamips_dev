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

#[no_mangle]
pub extern "C" fn _export(_: *mut memblock_t, _: *mut mempool_t) {}
