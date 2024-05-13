//! JIT operations.

use crate::prelude::*;

pub type jit_op_t = jit_op;
pub type jit_op_data_t = jit_op_data;

/// Number of JIT pools
pub const JIT_OP_POOL_NR: size_t = 8;

/// Invalid register in op
pub const JIT_OP_INV_REG: c_int = -1;

/// All flags
pub const JIT_OP_PPC_ALL_FLAGS: c_int = -1;

/// All registers
pub const JIT_OP_ALL_REGS: c_int = -1;

/// JIT opcodes // TODO enum
pub const JIT_OP_INVALID: c_uint = 0;
pub const JIT_OP_INSN_OUTPUT: c_uint = 1;
pub const JIT_OP_BRANCH_TARGET: c_uint = 2;
pub const JIT_OP_BRANCH_JUMP: c_uint = 3;
pub const JIT_OP_EOB: c_uint = 4;
pub const JIT_OP_LOAD_GPR: c_uint = 5;
pub const JIT_OP_STORE_GPR: c_uint = 6;
pub const JIT_OP_UPDATE_FLAGS: c_uint = 7;
pub const JIT_OP_REQUIRE_FLAGS: c_uint = 8;
pub const JIT_OP_TRASH_FLAGS: c_uint = 9;
pub const JIT_OP_ALTER_HOST_REG: c_uint = 10;
pub const JIT_OP_MOVE_HOST_REG: c_uint = 11;
pub const JIT_OP_SET_HOST_REG_IMM32: c_uint = 12;

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

/// Find a specific opcode in a JIT op list
#[no_mangle]
pub unsafe extern "C" fn jit_op_find_opcode(op_list: *mut jit_op_t, opcode: u_int) -> *mut jit_op_t {
    let mut op: *mut jit_op_t = op_list;
    while !op.is_null() {
        if (*op).opcode == opcode {
            return op;
        }
        op = (*op).next
    }

    null_mut()
}

/// Get a JIT op (allocate one if necessary)
#[no_mangle]
pub unsafe extern "C" fn jit_op_get(data: *mut jit_op_data_t, size_index: c_int, opcode: u_int) -> *mut jit_op_t {
    assert!((size_index as size_t) < JIT_OP_POOL_NR);
    let mut op: *mut jit_op_t = (*data).pool[size_index as size_t];

    if !op.is_null() {
        assert!((*op).ob_size_index == size_index as c_uint);
        (*data).pool[size_index as size_t] = (*op).next;
    } else {
        // no block found, allocate one
        let len: size_t = size_of::<jit_op_t>() + jit_op_blk_sizes[size_index as size_t] as size_t;

        op = libc::malloc(len).cast::<_>();
        assert!(!op.is_null());
        (*op).ob_size_index = size_index as c_uint;
    }

    (*op).opcode = opcode;
    (*op).param[0] = -1;
    (*op).param[1] = -1;
    (*op).param[2] = -1;
    (*op).next = null_mut();
    (*op).ob_ptr = (*op).ob_data.as_ptr().cast_mut();
    (*op).arg_ptr = null_mut();
    (*op).insn_name = null_mut();
    op
}

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
