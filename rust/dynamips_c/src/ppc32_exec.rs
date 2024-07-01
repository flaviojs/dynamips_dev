//! PowerPC (32-bit) step-by-step execution.

use crate::ppc32::*;
use crate::prelude::*;
use crate::utils::*;

extern "C" {
    pub fn ppc32_dump_stats(cpu: *mut cpu_ppc_t);
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
