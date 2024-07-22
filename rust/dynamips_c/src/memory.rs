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

/// Memory access flags
pub const MTS_ACC_AE: u_int = 0x00000002; // Address Error
pub const MTS_ACC_T: u_int = 0x00000004; // TLB Exception
pub const MTS_ACC_U: u_int = 0x00000006; // Unexistent
