//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Memory.

use crate::cpu::*;

extern "C" {
    pub fn memlog_dump(cpu: *mut cpu_gen_t);
}
