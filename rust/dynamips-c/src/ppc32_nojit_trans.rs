//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Just an empty JIT template file for architectures not supported by the JIT
//! code.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::jit_op::*;
use crate::ppc32::*;
use crate::ppc32_jit::*;

/// cbindgen:no-export
pub const JIT_SUPPORT: c_int = 0;

/// Wrappers to x86-codegen functions
#[macro_export]
macro_rules! ppc32_jit_tcb_set_patch {
    ($a:expr, $b:expr) => {
        let _ = $a;
        let _ = $b;
    };
}
pub use ppc32_jit_tcb_set_patch;
#[macro_export]
macro_rules! ppc32_jit_tcb_set_jump {
    ($a:expr, $b:expr) => {
        let _ = $a;
        let _ = $b;
    };
}
pub use ppc32_jit_tcb_set_jump;

#[no_mangle]
pub unsafe extern "C" fn ppc32_emit_breakpoint(_cpu: *mut cpu_ppc_t, _block: *mut ppc32_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_emit_breakpoint\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_tcb_push_epilog(_ptr: *mut *mut u_char) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_jit_tcb_push_epilog\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_tcb_exec(_cpu: *mut cpu_ppc_t, _block: *mut ppc32_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_jit_tcb_exec\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_set_ia(_ptr: *mut *mut u_char, _new_ia: m_uint32_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_set_ia\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_update_cr_set_altered_hreg(_cpu: *mut cpu_ppc_t) {
    // XXX this function was missing, assuming it should be EMPTY too
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_update_cr_set_altered_hreg\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_inc_perf_counter(_cpu: *mut cpu_ppc_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_inc_perf_counter\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_init_hreg_mapping(_cpu: *mut cpu_ppc_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_jit_init_hreg_mapping\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_insn_output(_b: *mut ppc32_jit_tcb_t, _op: *mut jit_op_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_op_insn_output\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_load_gpr(_b: *mut ppc32_jit_tcb_t, _op: *mut jit_op_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_op_load_gpr\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_store_gpr(_b: *mut ppc32_jit_tcb_t, _op: *mut jit_op_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_op_store_gpr\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_update_flags(_b: *mut ppc32_jit_tcb_t, _op: *mut jit_op_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_op_update_flags\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_move_host_reg(_b: *mut ppc32_jit_tcb_t, _op: *mut jit_op_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_op_move_host_reg\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_set_host_reg_imm32(_b: *mut ppc32_jit_tcb_t, _op: *mut jit_op_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_op_set_host_reg_imm32\n"));
    panic!();
}
#[no_mangle]
pub unsafe extern "C" fn ppc32_set_page_jump(_cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t) {
    libc::fprintf(c_stderr(), cstr!("This function should not be called: ppc32_set_page_jump\n"));
    panic!();
}

/// PowerPC instruction array
#[rustfmt::skip]
#[no_mangle]
pub static mut ppc32_insn_tags: [ppc32_insn_tag; 1] = [
    ppc32_insn_tag::null(),
];
