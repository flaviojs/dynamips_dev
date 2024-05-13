//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! JIT operations.

use crate::_private::*;

pub type jit_op_t = jit_op;
pub type jit_op_data_t = jit_op_data;

/// Number of JIT pools
pub const JIT_OP_POOL_NR: size_t = 8;

/* JIT operation */
#[repr(C)]
#[derive(Debug)]
pub struct jit_op {
    pub opcode: u_int,
    pub param: [c_int; 3],
    pub arg_ptr: *mut c_void,
    pub insn_name: *mut c_char,
    pub next: *mut jit_op,

    /// JIT output buffer
    pub ob_size_index: u_int,
    pub ob_final: *mut u_char,
    pub ob_ptr: *mut u_char,
    pub ob_data: [u8; 0],
}

/// JIT operation data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct jit_op_data {
    /// JIT op array for current compiled page
    pub array_size: u_int,
    pub array: *mut *mut jit_op_t,
    pub current: *mut *mut jit_op_t,

    /// JIT op pool
    pub pool: [*mut jit_op_t; JIT_OP_POOL_NR],
}

#[no_mangle]
pub static mut jit_op_blk_sizes: [u_int; JIT_OP_POOL_NR] = [0, 32, 64, 128, 256, 384, 512, 1024];

/// Free memory used by pools
#[no_mangle]
pub unsafe extern "C" fn jit_op_free_pools(data: *mut jit_op_data_t) {
    for i in 0..JIT_OP_POOL_NR {
        let mut op: *mut jit_op_t = (*data).pool[i];
        while !op.is_null() {
            let opn: *mut jit_op_t = (*op).next;
            libc::free(op.cast::<_>());
            op = opn;
        }

        (*data).pool[i] = null_mut();
    }
}
