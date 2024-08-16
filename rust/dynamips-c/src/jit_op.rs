//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! JIT operations.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips_common::*;

pub type jit_op_t = jit_op;

/// Number of JIT pools
pub const JIT_OP_POOL_NR: usize = 8;

/// Invalid register in op
pub const JIT_OP_INV_REG: c_int = -1;

/// All flags
pub const JIT_OP_PPC_ALL_FLAGS: c_int = -1;

/// All registers
pub const JIT_OP_ALL_REGS: c_int = -1;

/// JIT opcodes // TODO enum
pub const JIT_OP_INVALID: u_int = 0;
pub const JIT_OP_INSN_OUTPUT: u_int = 1;
pub const JIT_OP_BRANCH_TARGET: u_int = 2;
pub const JIT_OP_BRANCH_JUMP: u_int = 3;
pub const JIT_OP_EOB: u_int = 4;
pub const JIT_OP_LOAD_GPR: u_int = 5;
pub const JIT_OP_STORE_GPR: u_int = 6;
pub const JIT_OP_UPDATE_FLAGS: u_int = 7;
pub const JIT_OP_REQUIRE_FLAGS: u_int = 8;
pub const JIT_OP_TRASH_FLAGS: u_int = 9;
pub const JIT_OP_ALTER_HOST_REG: u_int = 10;
pub const JIT_OP_MOVE_HOST_REG: u_int = 11;
pub const JIT_OP_SET_HOST_REG_IMM32: u_int = 12;

/// JIT operation
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
    pub ob_data: [u_char; 0], // XXX size is determined by ob_size_index
}

/// Find a specific opcode in a JIT op list
#[inline]
#[no_mangle]
pub unsafe extern "C" fn jit_op_find_opcode(op_list: *mut jit_op_t, opcode: u_int) -> *mut jit_op_t {
    let mut op: *mut jit_op_t = op_list;
    while !op.is_null() {
        if (*op).opcode == opcode {
            return op;
        }
        op = (*op).next;
    }

    null_mut()
}

#[no_mangle]
pub static mut jit_op_blk_sizes: [u_int; JIT_OP_POOL_NR] = [0, 32, 64, 128, 256, 384, 512, 1024];

/// Get a JIT op (allocate one if necessary)
#[no_mangle]
pub unsafe extern "C" fn jit_op_get(cpu: *mut cpu_gen_t, size_index: c_int, opcode: u_int) -> *mut jit_op_t {
    assert!((size_index as usize) < JIT_OP_POOL_NR);
    let mut op: *mut jit_op_t = (*cpu).jit_op_pool[size_index as usize];

    if !op.is_null() {
        assert!((*op).ob_size_index == size_index as m_uint32_t);
        (*cpu).jit_op_pool[size_index as usize] = (*op).next;
    } else {
        // no block found, allocate one
        let len: size_t = size_of::<jit_op_t>() + jit_op_blk_sizes[size_index as usize] as size_t;

        op = libc::malloc(len).cast::<_>();
        assert!(!op.is_null());
        (*op).ob_size_index = size_index as m_uint32_t;
    }

    (*op).opcode = opcode;
    (*op).param[0] = -1;
    (*op).param[1] = -1;
    (*op).param[2] = -1;
    (*op).next = null_mut();
    (*op).ob_ptr = (*op).ob_data.as_c_mut();
    (*op).arg_ptr = null_mut();
    (*op).insn_name = null_mut();
    op
}

/// Release a JIT op
#[no_mangle]
pub unsafe extern "C" fn jit_op_free(cpu: *mut cpu_gen_t, op: *mut jit_op_t) {
    assert!(((*op).ob_size_index as usize) < JIT_OP_POOL_NR);
    (*op).next = (*cpu).jit_op_pool[(*op).ob_size_index as usize];
    (*cpu).jit_op_pool[(*op).ob_size_index as usize] = op;
}

/// Free a list of JIT ops
#[no_mangle]
pub unsafe extern "C" fn jit_op_free_list(cpu: *mut cpu_gen_t, op_list: *mut jit_op_t) {
    let mut op: *mut jit_op_t;
    let mut opn: *mut jit_op_t;

    op = op_list;
    while !op.is_null() {
        opn = (*op).next;
        jit_op_free(cpu, op);
        op = opn;
    }
}

/// Initialize JIT op pools for the specified CPU
#[no_mangle]
pub unsafe extern "C" fn jit_op_init_cpu(cpu: *mut cpu_gen_t) -> c_int {
    (*cpu).jit_op_array = libc::calloc((*cpu).jit_op_array_size as size_t, size_of::<*mut jit_op_t>()).cast::<_>();

    if !(*cpu).jit_op_array.is_null() {
        return -1;
    }

    libc::memset((*cpu).jit_op_pool.as_c_void_mut(), 0, size_of::<[*mut jit_op_t; JIT_OP_POOL_NR]>());
    0
}

/// Free memory used by pools
#[no_mangle]
pub unsafe extern "C" fn jit_op_free_pools(cpu: *mut cpu_gen_t) {
    let mut op: *mut jit_op_t;
    let mut opn: *mut jit_op_t;

    for i in 0..JIT_OP_POOL_NR {
        op = (*cpu).jit_op_pool[i];
        while !op.is_null() {
            opn = (*op).next;
            libc::free(op.cast::<_>());
            op = opn;
        }

        (*cpu).jit_op_pool[i] = null_mut();
    }
}
