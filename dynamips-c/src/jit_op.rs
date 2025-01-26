//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)

use crate::_extra::*;
use crate::cpu::*;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;
use std::ptr::null_mut;

pub type jit_op_t = jit_op;

// Number of JIT pools
pub const JIT_OP_POOL_NR: usize = 8;

// Invalid register in op
pub const JIT_OP_INV_REG: c_int = -1;

// All flags
pub const JIT_OP_PPC_ALL_FLAGS: c_int = -1;

// All registers
pub const JIT_OP_ALL_REGS: c_int = -1;

// JIT opcodes
pub type _JIT_OP_ENUM = u_int; // TODO enum
pub const JIT_OP_INVALID: _JIT_OP_ENUM = 0;
pub const JIT_OP_INSN_OUTPUT: _JIT_OP_ENUM = 1;
pub const JIT_OP_BRANCH_TARGET: _JIT_OP_ENUM = 2;
pub const JIT_OP_BRANCH_JUMP: _JIT_OP_ENUM = 3;
pub const JIT_OP_EOB: _JIT_OP_ENUM = 4;
pub const JIT_OP_LOAD_GPR: _JIT_OP_ENUM = 5;
pub const JIT_OP_STORE_GPR: _JIT_OP_ENUM = 6;
pub const JIT_OP_UPDATE_FLAGS: _JIT_OP_ENUM = 7;
pub const JIT_OP_REQUIRE_FLAGS: _JIT_OP_ENUM = 8;
pub const JIT_OP_TRASH_FLAGS: _JIT_OP_ENUM = 9;
pub const JIT_OP_ALTER_HOST_REG: _JIT_OP_ENUM = 10;
pub const JIT_OP_MOVE_HOST_REG: _JIT_OP_ENUM = 11;
pub const JIT_OP_SET_HOST_REG_IMM32: _JIT_OP_ENUM = 12;

/* JIT operation */
#[repr(C)]
#[derive(Debug)]
pub struct jit_op {
    pub opcode: u_int,
    pub param: [c_int; 3],
    pub arg_ptr: *mut c_void,
    pub insn_name: *mut c_char,
    pub next: *mut jit_op,

    // JIT output buffer
    pub ob_size_index: u_int,
    pub ob_final: *mut u_char,
    pub ob_ptr: *mut u_char,
    pub ob_data: [u_char; 0],
}

// Find a specific opcode in a JIT op list
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

extern "C" {
    pub fn jit_op_get(cpu: *mut cpu_gen_t, size_index: c_int, opcode: u_int) -> *mut jit_op_t; // TODO replace
}
