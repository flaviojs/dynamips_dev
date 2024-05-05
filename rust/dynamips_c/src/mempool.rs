//! Simple Memory Pools.

use crate::prelude::*;

/// Memory Pool "Fixed" Flag
pub const MEMPOOL_FIXED: c_int = 1;

/// Dummy value used to check if a memory block is invalid
pub const MEMBLOCK_TAG: c_int = 0xdeadbeef_u32 as c_int;
