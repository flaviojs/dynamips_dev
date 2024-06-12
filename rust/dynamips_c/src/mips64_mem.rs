//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::_private::*;
use crate::mips64::*;

extern "C" {
    pub fn mips64_mts32_init(cpu: *mut cpu_mips_t) -> c_int;
    pub fn mips64_mts32_init_memop_vectors(cpu: *mut cpu_mips_t);
    pub fn mips64_mts64_init(cpu: *mut cpu_mips_t) -> c_int;
    pub fn mips64_mts64_init_memop_vectors(cpu: *mut cpu_mips_t);
}

/// Shutdown MTS subsystem
#[no_mangle]
pub unsafe extern "C" fn mips64_mem_shutdown(cpu: *mut cpu_mips_t) {
    if (*cpu).mts_shutdown.is_some() {
        (*cpu).mts_shutdown.unwrap()(cpu);
    }
}

/// Set the address mode
#[no_mangle]
pub unsafe extern "C" fn mips64_set_addr_mode(cpu: *mut cpu_mips_t, addr_mode: u_int) -> c_int {
    if (*cpu).addr_mode != addr_mode {
        mips64_mem_shutdown(cpu);

        match addr_mode {
            32 => {
                mips64_mts32_init(cpu);
                mips64_mts32_init_memop_vectors(cpu);
            }
            64 => {
                mips64_mts64_init(cpu);
                mips64_mts64_init_memop_vectors(cpu);
            }
            _ => {
                libc::fprintf(c_stderr(), cstr!("mts_set_addr_mode: internal error (addr_mode=%u)\n"), addr_mode);
                libc::exit(libc::EXIT_FAILURE);
            }
        }
    }

    0
}
