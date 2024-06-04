//! PPC32 JIT compiler.

use crate::_private::*;
use crate::ppc32::*;

extern "C" {
    pub fn ppc32_jit_flush(cpu: *mut cpu_ppc_t, threshold: u_int) -> u_int;
}

pub type ppc32_jit_tcb_t = ppc32_jit_tcb;

/// cbindgen:no-export
#[repr(C)]
pub struct ppc32_jit_tcb {
    _todo: u8,
}
