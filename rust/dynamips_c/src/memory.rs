//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Memory.

use crate::_private::*;
use crate::cpu::*;

extern "C" {
    pub fn memlog_dump(cpu: *mut cpu_gen_t);
}

/// MTS operation
pub const MTS_READ: u_int = 0;
pub const MTS_WRITE: u_int = 1;
