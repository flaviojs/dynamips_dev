//! PowerPC MMU.

use crate::ppc32::*;
use crate::prelude::*;

extern "C" {
    pub fn ppc32_mem_restart(cpu: *mut cpu_ppc_t) -> c_int;
}
