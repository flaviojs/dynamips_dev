use crate::_private::*;

use crate::dynamips_common::*;
use crate::mips64::*;
use crate::mips64_jit::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::utils::*;

pub const JIT_SUPPORT: c_int = 0;

#[cfg(not(feature = "USE_UNSTABLE"))]
#[macro_export]
macro_rules! mips64_jit_tcb_set_patch {
    ($a:expr, $b:expr) => {};
}
#[cfg(not(feature = "USE_UNSTABLE"))]
pub use mips64_jit_tcb_set_patch;
#[cfg(not(feature = "USE_UNSTABLE"))]
#[macro_export]
macro_rules! mips64_jit_tcb_set_jump {
    ($a:expr, $b:expr) => {};
}
#[cfg(not(feature = "USE_UNSTABLE"))]
pub use mips64_jit_tcb_set_jump;

#[cfg(feature = "USE_UNSTABLE")]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_set_patch(_code: *mut u_char, _target: *mut u_char) {}
#[cfg(feature = "USE_UNSTABLE")]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_set_jump(_instp: *mut *mut u_char, _target: *mut u_char) {}

/// Set an IRQ
#[no_mangle]
pub unsafe extern "C" fn mips64_set_irq(cpu: *mut cpu_mips_t, irq: m_uint8_t) {
    let m: m_uint32_t = (1 << (irq as c_int + MIPS_CP0_CAUSE_ISHIFT)) & MIPS_CP0_CAUSE_IMASK;
    MIPS64_IRQ_LOCK(cpu);
    (*cpu).irq_cause |= m;
    MIPS64_IRQ_UNLOCK(cpu);
}

/// Clear an IRQ
#[no_mangle]
pub unsafe extern "C" fn mips64_clear_irq(cpu: *mut cpu_mips_t, irq: m_uint8_t) {
    let m: m_uint32_t = (1 << (irq as c_int + MIPS_CP0_CAUSE_ISHIFT)) & MIPS_CP0_CAUSE_IMASK;
    MIPS64_IRQ_LOCK(cpu);
    (*cpu).irq_cause &= !m;
    MIPS64_IRQ_UNLOCK(cpu);

    if (*cpu).irq_cause == 0 {
        (*cpu).irq_pending = 0;
    }
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_push_epilog(_block: *mut mips64_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_jit_tcb_push_epilog\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_exec(_cpu: *mut cpu_mips_t, _block: *mut mips64_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_jit_tcb_exec\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_set_pc(_b: *mut mips64_jit_tcb_t, _new_pc: m_uint64_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_set_pc\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_set_ra(_b: *mut mips64_jit_tcb_t, _ret_pc: m_uint64_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_set_ra\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_emit_breakpoint(_b: *mut mips64_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_emit_breakpoint\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_emit_single_step(_b: *mut mips64_jit_tcb_t, _insn: mips_insn_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_emit_single_step\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_emit_invalid_delay_slot(_b: *mut mips64_jit_tcb_t) -> c_int {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_emit_invalid_delay_slot\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_inc_cp0_count_reg(_b: *mut mips64_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_inc_cp0_count_reg\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_check_pending_irq(_b: *mut mips64_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_check_pending_irq\n"));
    panic!();
}

#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_inc_perf_counter(_b: *mut mips64_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_inc_perf_counter\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_push_epilog(_tc: *mut cpu_tc_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_jit_tcb_push_epilog\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_exec(_cpu: *mut cpu_mips_t, _tb: *mut cpu_tb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_jit_tcb_exec\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_set_pc(_tc: *mut cpu_tc_t, _new_pc: m_uint64_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_set_pc\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_set_ra(_tc: *mut cpu_tc_t, _ret_pc: m_uint64_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_set_ra\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_emit_breakpoint(_tc: *mut cpu_tc_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_emit_breakpoint\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_emit_single_step(_tc: *mut cpu_tc_t, _insn: mips_insn_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_emit_single_step\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_emit_invalid_delay_slot(_tc: *mut cpu_tc_t) -> c_int {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_emit_invalid_delay_slot\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_inc_cp0_count_reg(_tc: *mut cpu_tc_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_inc_cp0_count_reg\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_check_pending_irq(_tc: *mut cpu_tc_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_check_pending_irq\n"));
    panic!();
}

#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_inc_perf_counter(_tc: *mut cpu_tc_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: mips64_inc_perf_counter\n"));
    panic!();
}

/// MIPS instruction array
#[rustfmt::skip]
#[no_mangle]
pub static mut mips64_insn_tags: [mips64_insn_tag; 1] = [
    mips64_insn_tag::null()
];
