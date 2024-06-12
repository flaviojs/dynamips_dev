//! MIPS64 MTS subsystem.

use crate::mips64::*;

/// Shutdown MTS subsystem
#[no_mangle]
pub unsafe extern "C" fn mips64_mem_shutdown(cpu: *mut cpu_mips_t) {
    if (*cpu).mts_shutdown.is_some() {
        (*cpu).mts_shutdown.unwrap()(cpu);
    }
}
