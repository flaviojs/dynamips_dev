//! PowerPC (32-bit) step-by-step execution.

use crate::ppc32::*;

extern "C" {
    pub fn ppc32_dump_stats(cpu: *mut cpu_ppc_t);
}
