//! MIPS Coprocessor 0 (System Coprocessor) implementation.
//! We don't use the JIT here, since there is no high performance needed.

use crate::dynamips_common::*;
use crate::mips64::*;
use crate::prelude::*;
use crate::utils::*;

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

/// Get value of random register
#[no_mangle] // TODO private
#[inline]
pub unsafe extern "C" fn mips64_cp0_get_random_reg(cpu: *mut cpu_mips_t) -> u_int {
    // We use the virtual count register as a basic "random" value
    let wired: u_int = (*cpu).cp0.reg[MIPS_CP0_WIRED] as u_int;
    wired + ((*cpu).cp0_virt_cnt_reg % ((*cpu).cp0.tlb_entries - wired))
}

/// Get a cp0 register (fast version)
#[no_mangle] // TODO private
#[inline]
pub unsafe extern "C" fn mips64_cp0_get_reg_fast(cpu: *mut cpu_mips_t, cp0_reg: u_int) -> m_uint64_t {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let delta: m_uint32_t;
    let mut res: m_uint32_t;

    match cp0_reg as usize {
        MIPS_CP0_COUNT => {
            delta = (*cpu).cp0_virt_cmp_reg - (*cpu).cp0_virt_cnt_reg;
            res = (*cp0).reg[MIPS_CP0_COMPARE] as m_uint32_t;
            res -= (*(*cpu).vm).clock_divisor * delta;
            sign_extend(res as m_int64_t, 32) as m_uint64_t
        }

        MIPS_CP0_COMPARE => {
            if true {
                sign_extend((*cp0).reg[MIPS_CP0_COMPARE] as m_int64_t, 32) as m_uint64_t
            } else {
                // really useful and logical ?
                delta = (*cpu).cp0_virt_cmp_reg - (*cpu).cp0_virt_cnt_reg;
                res = (*cp0).reg[MIPS_CP0_COUNT] as m_uint32_t;
                res += (*(*cpu).vm).clock_divisor * delta;
                res as m_uint64_t
            }
        }

        MIPS_CP0_INFO => MIPS64_R7000_TLB64_ENABLE as m_uint64_t,

        MIPS_CP0_RANDOM => mips64_cp0_get_random_reg(cpu) as m_uint64_t,

        _ => (*cp0).reg[cp0_reg as usize],
    }
}

/// Get a cp0 register
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_reg(cpu: *mut cpu_mips_t, cp0_reg: u_int) -> m_uint64_t {
    mips64_cp0_get_reg_fast(cpu, cp0_reg)
}
