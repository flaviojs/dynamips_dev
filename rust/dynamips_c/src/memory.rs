//! Memory.

use crate::cpu::*;
use crate::prelude::*;

extern "C" {
    pub fn memlog_dump(cpu: *mut cpu_gen_t);
}

/// MTS operation
pub const MTS_READ: u_int = 0;
pub const MTS_WRITE: u_int = 1;
