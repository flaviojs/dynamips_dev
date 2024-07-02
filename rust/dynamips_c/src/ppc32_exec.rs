//! PowerPC (32-bit) step-by-step execution.

use crate::dynamips_common::*;
use crate::ppc32::*;
use crate::prelude::*;
use crate::utils::*;

extern "C" {
    pub fn ppc32_dump_stats(cpu: *mut cpu_ppc_t);
}

/// =========================================================================

/// Update CR0
#[no_mangle] // TODO private
#[inline(always)]
pub unsafe extern "C" fn ppc32_exec_update_cr0(cpu: *mut cpu_ppc_t, val: m_uint32_t) {
    let mut res: m_uint32_t;

    if (val & 0x80000000) != 0 {
        res = 1 << PPC32_CR_LT_BIT;
    } else if val > 0 {
        res = 1 << PPC32_CR_GT_BIT;
    } else {
        res = 1 << PPC32_CR_EQ_BIT;
    }

    if ((*cpu).xer & PPC32_XER_SO) != 0 {
        res |= 1 << PPC32_CR_SO_BIT;
    }

    (*cpu).cr_fields[0] = res;
}

/// MFLR - Move From Link Register
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn ppc32_exec_MFLR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    (*cpu).gpr[rd as usize] = (*cpu).lr;
    0
}

/// MTLR - Move To Link Register
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn ppc32_exec_MTLR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    (*cpu).lr = (*cpu).gpr[rs as usize];
    0
}

/// MFCTR - Move From Counter Register
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn ppc32_exec_MFCTR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    (*cpu).gpr[rd as usize] = (*cpu).ctr;
    0
}

/// MTCTR - Move To Counter Register
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn ppc32_exec_MTCTR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    (*cpu).ctr = (*cpu).gpr[rs as usize];
    0
}

/// ADD
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn ppc32_exec_ADD(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    0
}

/// ADD.
#[no_mangle] // TODO private
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn ppc32_exec_ADD_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[rd as usize] = tmp;
    0
}
