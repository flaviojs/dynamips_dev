//! MIPS Coprocessor 0 (System Coprocessor) implementation.
//! We don't use the JIT here, since there is no high performance needed.

use crate::mips64::*;
use crate::prelude::*;

/// MIPS cp0 registers names
#[no_mangle]
#[rustfmt::skip]
pub static mut mips64_cp0_reg_names: [*mut c_char; MIPS64_CP0_REG_NR] = [
    cstr!("index"), 
    cstr!("random"), 
    cstr!("entry_lo0"), 
    cstr!("entry_lo1"), 
    cstr!("context"), 
    cstr!("pagemask"),
    cstr!("wired"),
    cstr!("info"),
    cstr!("badvaddr"), 
    cstr!("count"), 
    cstr!("entry_hi"), 
    cstr!("compare"), 
    cstr!("status"), 
    cstr!("cause"),
    cstr!("epc"), 
    cstr!("prid"), 
    cstr!("config"), 
    cstr!("ll_addr"), 
    cstr!("watch_lo"), 
    cstr!("watch_hi"), 
    cstr!("xcontext"),
    cstr!("cp0_r21"),
    cstr!("cp0_r22"),
    cstr!("cp0_r23"),
    cstr!("cp0_r24"),
    cstr!("cp0_r25"),
    cstr!("ecc"), 
    cstr!("cache_err"), 
    cstr!("tag_lo"), 
    cstr!("tag_hi"), 
    cstr!("err_epc"),
    cstr!("cp0_r31"),
];

/// Get cp0 register index given its name
#[allow(clippy::needless_range_loop)]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_reg_index(name: *mut c_char) -> c_int {
    for i in 0..MIPS64_CP0_REG_NR {
        if libc::strcmp(mips64_cp0_reg_names[i], name) == 0 {
            return i as c_int;
        }
    }

    -1
}

/// Get the CPU operating mode (User,Supervisor or Kernel) - inline version
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
unsafe fn mips64_cp0_get_mode_inline(cpu: *mut cpu_mips_t) -> u_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);

    let mut cpu_mode: u_int = ((*cp0).reg[MIPS_CP0_STATUS] >> MIPS_CP0_STATUS_KSU_SHIFT) as u_int;
    cpu_mode &= MIPS_CP0_STATUS_KSU_MASK;
    cpu_mode
}

/// Get the CPU operating mode (User,Supervisor or Kernel)
#[cfg(not(feature = "USE_UNSTABLE"))]
unsafe fn mips64_cp0_get_mode(cpu: *mut cpu_mips_t) -> u_int {
    mips64_cp0_get_mode_inline(cpu)
}

/// Check that we are running in kernel mode
#[cfg(not(feature = "USE_UNSTABLE"))]
pub unsafe fn mips64_cp0_check_kernel_mode(cpu: *mut cpu_mips_t) -> c_int {
    let cpu_mode: u_int = mips64_cp0_get_mode(cpu);

    if cpu_mode != MIPS_CP0_STATUS_KM {
        // XXX Branch delay slot
        mips64_trigger_exception(cpu, MIPS_CP0_CAUSE_ILLOP, 0);
        return 1;
    }

    0
}

/// Get the CPU operating mode (User,Supervisor or Kernel)
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
unsafe fn mips64_cp0_get_mode(cpu: *mut cpu_mips_t) -> u_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);

    let mut cpu_mode: u_int = ((*cp0).reg[MIPS_CP0_STATUS] >> MIPS_CP0_STATUS_KSU_SHIFT) as u_int;
    cpu_mode &= MIPS_CP0_STATUS_KSU_MASK;
    cpu_mode
}

/// Check that we are running in kernel mode
#[cfg(feature = "USE_UNSTABLE")]
pub unsafe fn mips64_cp0_check_kernel_mode(cpu: *mut cpu_mips_t) -> c_int {
    let cpu_mode: u_int = mips64_cp0_get_mode(cpu);

    if cpu_mode != MIPS_CP0_STATUS_KM {
        mips64_general_exception(cpu, MIPS_CP0_CAUSE_ILLOP);
        return 1;
    }

    0
}
