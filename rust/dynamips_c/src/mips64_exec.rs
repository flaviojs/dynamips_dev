//! MIPS64 Step-by-step execution.

use crate::dynamips_common::*;
use crate::mips64::*;
use crate::prelude::*;
use crate::utils::*;

/// ADD
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ADD(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    // TODO: Exception handling
    let res: m_uint64_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_add((*cpu).gpr[rt as usize] as m_uint32_t) as m_uint64_t;
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// ADDI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ADDI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let val: m_uint32_t = sign_extend(imm as m_int64_t, 16) as m_uint32_t;

    // TODO: Exception handling
    let res: m_uint32_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_add(val);
    (*cpu).gpr[rt as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// ADDIU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ADDIU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let val: m_uint32_t = sign_extend(imm as m_int64_t, 16) as m_uint32_t;

    let res: m_uint32_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_add(val);
    (*cpu).gpr[rt as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}

/// ADDU
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ADDU(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    let res: m_uint32_t = ((*cpu).gpr[rs as usize] as m_uint32_t).wrapping_add((*cpu).gpr[rt as usize] as m_uint32_t);
    (*cpu).gpr[rd as usize] = sign_extend(res as m_int64_t, 32) as m_uint64_t;
    0
}
