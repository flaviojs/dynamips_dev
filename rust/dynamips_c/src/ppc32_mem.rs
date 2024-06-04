//! PowerPC MMU.

use crate::_private::*;
use crate::ppc32::*;

extern "C" {
    pub fn ppc32_mem_restart(cpu: *mut cpu_ppc_t) -> c_int;
}
