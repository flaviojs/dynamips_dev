//! Memory.

use crate::cpu::*;

extern "C" {
    pub fn memlog_dump(cpu: *mut cpu_gen_t);
}
