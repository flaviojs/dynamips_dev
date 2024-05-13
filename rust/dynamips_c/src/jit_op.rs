//! JIT operations.

use crate::prelude::*;

/// Number of JIT pools
pub const JIT_OP_POOL_NR: size_t = 8;

#[no_mangle]
pub static mut jit_op_blk_sizes: [u_int; JIT_OP_POOL_NR] = [0, 32, 64, 128, 256, 384, 512, 1024];
