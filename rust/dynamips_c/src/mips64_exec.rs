//! MIPS64 Step-by-step execution.

use crate::dynamips_common::*;
use crate::mips64::*;
use crate::prelude::*;
use crate::utils::*;

extern "C" {
    fn mips64_exec_bdslot(cpu: *mut cpu_mips_t);
}

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

/// AND
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_AND(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let rd: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rs as usize] & (*cpu).gpr[rt as usize];
    0
}

/// ANDI
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_ANDI(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let rt: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    (*cpu).gpr[rt as usize] = (*cpu).gpr[rs as usize] & imm as m_uint64_t;
    0
}

/// B (Branch, virtual instruction)
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_exec_B(cpu: *mut cpu_mips_t, insn: mips_insn_t) -> c_int {
    let offset: c_int = bits(insn, 0, 15);

    // compute the new pc
    let new_pc: m_uint64_t = ((*cpu).pc + 4).wrapping_add_signed(sign_extend((offset << 2) as m_int64_t, 18));

    // exec the instruction in the delay slot
    mips64_exec_bdslot(cpu);

    // set the new pc in cpu structure
    (*cpu).pc = new_pc;
    1
}
