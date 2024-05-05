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

/// Remove block from linked list
unsafe fn memblock_delete(pool: *mut mempool_t, block: *mut memblock_t) {
    MEMPOOL_LOCK(pool);

    (*pool).nr_blocks -= 1;
    (*pool).total_size -= (*block).block_size;

    if (*block).prev.is_null() {
        (*pool).block_list = (*block).next;
    } else {
        (*(*block).prev).next = (*block).next;
    }

    if !(*block).next.is_null() {
        (*(*block).next).prev = (*block).prev;
    }

    (*block).next = null_mut();
    (*block).prev = null_mut();
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

/// Reallocate a block
#[no_mangle]
pub unsafe extern "C" fn mp_realloc(addr: *mut c_void, new_size: size_t) -> *mut c_void {
    let block: *mut memblock_t = addr.cast::<memblock_t>().sub(1);

    assert!((*block).tag == MEMBLOCK_TAG);
    let pool: *mut mempool_t = (*block).pool;

    // remove this block from list
    memblock_delete(pool, block);

    // reallocate block with specified size
    let total_size = new_size + size_of::<memblock_t>();

    let ptr: *mut memblock_t = libc::realloc(block.cast::<_>(), total_size).cast::<_>();
    if ptr.is_null() {
        memblock_insert(pool, block);
        return null_mut();
    }

    (*ptr).block_size = new_size;
    memblock_insert(pool, ptr);
    (*ptr).data.as_c_void_mut()
}

/// Allocate a new memory block and copy data into it
#[no_mangle]
pub unsafe extern "C" fn mp_dup(pool: *mut mempool_t, data: *mut c_void, size: size_t) -> *mut c_void {
    let p = mp_alloc_n0(pool, size);
    if !p.is_null() {
        libc::memcpy(p, data, size);
    }

    p
}

/// Duplicate specified string and insert it in a memory pool
#[no_mangle]
pub unsafe extern "C" fn mp_strdup(pool: *mut mempool_t, str_: *mut c_char) -> *mut c_char {
    let new_str: *mut c_char = mp_alloc(pool, libc::strlen(str_) + 1).cast::<_>();

    if new_str.is_null() {
        return null_mut();
    }

    libc::strcpy(new_str, str_);
    new_str
}

/// Free block at specified address
#[no_mangle]
pub unsafe extern "C" fn mp_free(addr: *mut c_void) -> c_int {
    if !addr.is_null() {
        let block: *mut memblock_t = addr.cast::<memblock_t>().sub(1);
        assert!((*block).tag == MEMBLOCK_TAG);
        let pool: *mut mempool_t = (*block).pool;

        memblock_delete(pool, block);
        libc::memset(block.cast::<_>(), 0, size_of::<memblock_t>());
        libc::free(block.cast::<_>());
    }

    0
}

/// Free block at specified address and clean pointer
#[no_mangle]
pub unsafe extern "C" fn mp_free_ptr(addr: *mut c_void) -> c_int {
    assert!(!addr.is_null());
    let p: *mut c_void = *addr.cast::<*mut c_void>();
    *addr.cast::<*mut c_void>() = null_mut();
    mp_free(p);
    0
}

/// Free all blocks of specified pool
#[no_mangle]
pub unsafe extern "C" fn mp_free_all_blocks(pool: *mut mempool_t) {
    MEMPOOL_LOCK(pool);

    let mut block: *mut memblock_t = (*pool).block_list;
    while !block.is_null() {
        let next = (*block).next;
        libc::free(block.cast::<_>());
        block = next;
    }

    (*pool).block_list = null_mut();
    (*pool).nr_blocks = 0;
    (*pool).total_size = 0;

    MEMPOOL_UNLOCK(pool);
}

/// Free specified memory pool
#[no_mangle]
pub unsafe extern "C" fn mp_free_pool(pool: *mut mempool_t) {
    mp_free_all_blocks(pool);

    if ((*pool).flags & MEMPOOL_FIXED) == 0 {
        libc::free(pool.cast::<_>());
    }
}

#[no_mangle]
pub extern "C" fn _export(_: *mut memblock_t, _: *mut mempool_t) {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memblock_roundtrip() {
        unsafe {
            let mut memory = Box::new(zeroed::<memblock_t>());
            let block: *mut memblock_t = addr_of_mut!(*memory);
            // memblock to addr (mp_alloc_inline)
            let addr: *mut c_void = (*block).data.as_c_void_mut();
            // addr to memblock (mp_realloc)
            let roundtrip_block: *mut memblock_t = addr.cast::<memblock_t>().sub(1);
            assert!(block == roundtrip_block);
        }
    }
}
