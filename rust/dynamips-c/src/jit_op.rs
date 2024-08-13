//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! JIT operations.

use crate::_private::*;

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
