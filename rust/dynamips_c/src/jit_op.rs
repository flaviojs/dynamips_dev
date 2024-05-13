//! JIT operations.

use crate::prelude::*;

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

/// Release a JIT op
#[no_mangle]
pub unsafe extern "C" fn jit_op_free(data: *mut jit_op_data_t, op: *mut jit_op_t) {
    assert!(((*op).ob_size_index as usize) < JIT_OP_POOL_NR);
    (*op).next = (*data).pool[(*op).ob_size_index as usize];
    (*data).pool[(*op).ob_size_index as usize] = op;
}

/// Free a list of JIT ops
#[no_mangle]
pub unsafe extern "C" fn jit_op_free_list(data: *mut jit_op_data_t, op_list: *mut jit_op_t) {
    let mut op: *mut jit_op_t = op_list;
    while !op.is_null() {
        let opn: *mut jit_op_t = (*op).next;
        jit_op_free(data, op);
        op = opn;
    }
}

/// Initialize JIT op pools for the specified CPU
#[no_mangle]
pub unsafe extern "C" fn jit_op_init_cpu(data: *mut jit_op_data_t) -> c_int {
    (*data).array = libc::calloc((*data).array_size as size_t, size_of::<*mut jit_op_t>()).cast::<_>();

    if (*data).array.is_null() {
        return -1;
    }

    libc::memset(addr_of_mut!((*data).pool).cast::<_>(), 0, size_of::<[*mut jit_op_t; JIT_OP_POOL_NR]>());
    0
}

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
