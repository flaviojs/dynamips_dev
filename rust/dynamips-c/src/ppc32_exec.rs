//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! PowerPC (32-bit) step-by-step execution.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips::*;
use crate::dynamips_common::*;
use crate::insn_lookup::*;
use crate::ppc32::*;
use crate::utils::*;
use crate::vm::*;

/// PowerPC instruction recognition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_insn_exec_tag {
    pub name: *mut c_char,
    pub exec: ppc32_insn_exec_tag_exec,
    pub mask: m_uint32_t,
    pub value: m_uint32_t,
    pub instr_type: c_int,
    pub count: m_uint64_t,
}
impl ppc32_insn_exec_tag {
    pub const fn new(name: *mut c_char, exec: ppc32_insn_exec_tag_exec, mask: m_uint32_t, value: m_uint32_t, instr_type: c_int) -> Self {
        Self { name, exec, mask, value, instr_type, count: 0 }
    }
    pub const fn null() -> Self {
        Self { name: null_mut(), exec: None, mask: 0x00000000, value: 0x00000000, instr_type: 0, count: 0 }
    }
}

pub type ppc32_insn_exec_tag_exec = Option<unsafe extern "C" fn(_: *mut cpu_ppc_t, _: ppc_insn_t) -> c_int>;

/// Get a rotation mask
#[inline(always)]
unsafe fn ppc32_rotate_mask(mb: m_uint32_t, me: m_uint32_t) -> m_uint32_t {
    let mut mask: m_uint32_t = (0xFFFFFFFF >> mb) ^ ((0xFFFFFFFF >> me) >> 1);

    if me < mb {
        mask = !mask;
    }

    mask
}

static mut ilt: *mut insn_lookup_t = null_mut();

/// ILT
#[inline(always)]
unsafe extern "C" fn ppc32_exec_get_insn(index: c_int) -> *mut c_void {
    addr_of_mut!(ppc32_exec_tags[index as usize]).cast::<_>()
}

unsafe extern "C" fn ppc32_exec_chk_lo(tag: *mut c_void, value: c_int) -> c_int {
    let tag: *mut ppc32_insn_exec_tag = tag.cast::<_>();
    ((value as m_uint32_t & (*tag).mask) == ((*tag).value & 0xFFFF)) as c_int
}

unsafe extern "C" fn ppc32_exec_chk_hi(tag: *mut c_void, value: c_int) -> c_int {
    let tag: *mut ppc32_insn_exec_tag = tag.cast::<_>();
    ((value as m_uint32_t & ((*tag).mask >> 16)) == ((*tag).value >> 16)) as c_int
}

/// Destroy instruction lookup table
extern "C" fn destroy_ilt() {
    unsafe {
        assert!(!ilt.is_null());
        ilt_destroy(ilt);
        ilt = null_mut();
    }
}

/// Initialize instruction lookup table
#[no_mangle]
pub unsafe extern "C" fn ppc32_exec_create_ilt() {
    let mut i: c_int = 0;
    let mut count: c_int = 0;
    while ppc32_exec_tags[i as usize].exec.is_some() {
        count += 1;
        i += 1;
    }

    ilt = ilt_create(cstr!("ppc32e"), count, Some(ppc32_exec_get_insn), Some(ppc32_exec_chk_lo), Some(ppc32_exec_chk_hi));

    libc::atexit(destroy_ilt);
}

/// Dump statistics
#[no_mangle]
pub unsafe extern "C" fn ppc32_dump_stats(cpu: *mut cpu_ppc_t) {
    if NJM_STATS_ENABLE != 0 {
        libc::printf(cstr!("\n"));

        let mut i: c_int = 0;
        while ppc32_exec_tags[i as usize].exec.is_some() {
            libc::printf(cstr!("  * %-10s : %10llu\n"), ppc32_exec_tags[i as usize].name, ppc32_exec_tags[i as usize].count);
            i += 1;
        }

        libc::printf(cstr!("%llu instructions executed since startup.\n"), (*cpu).insn_exec_count);
    } else {
        libc::printf(cstr!("Statistics support is not compiled in.\n"));
    }
}

/// Execute a memory operation
#[inline(always)]
unsafe fn ppc32_exec_memop(cpu: *mut cpu_ppc_t, memop: c_int, vaddr: m_uint32_t, dst_reg: u_int) {
    let fn_: ppc_memop_fn = (*cpu).mem_op_fn[memop as usize];
    fn_.unwrap_unchecked()(cpu, vaddr, dst_reg);
}

/// Fetch an instruction
#[inline(always)]
unsafe fn ppc32_exec_fetch(cpu: *mut cpu_ppc_t, ia: m_uint32_t, insn: *mut ppc_insn_t) -> c_int {
    let exec_page: m_uint32_t = ia & !PPC32_MIN_PAGE_IMASK;

    if unlikely(exec_page as m_uint64_t != (*cpu).njm_exec_page) {
        #[cfg(feature = "USE_UNSTABLE")]
        {
            (*cpu).njm_exec_ptr = (*cpu).mem_op_ifetch.unwrap_unchecked()(cpu, exec_page).cast::<_>();
        }
        (*cpu).njm_exec_page = exec_page as m_uint64_t;
        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            (*cpu).njm_exec_ptr = (*cpu).mem_op_lookup.unwrap_unchecked()(cpu, exec_page, PPC32_MTS_ICACHE as u_int).cast::<_>();
        }
    }

    let offset: m_uint32_t = (ia & PPC32_MIN_PAGE_IMASK) >> 2;
    *insn = vmtoh32(*(*cpu).njm_exec_ptr.add(offset as usize));
    0
}

/// Unknown opcode
unsafe extern "C" fn ppc32_exec_unknown(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    libc::printf(cstr!("PPC32: unknown opcode 0x%8.8x at ia = 0x%x\n"), insn, (*cpu).ia);
    ppc32_dump_regs((*cpu).gen);
    0
}

/// Execute a single instruction
#[inline(always)]
unsafe fn ppc32_exec_single_instruction(cpu: *mut cpu_ppc_t, instruction: ppc_insn_t) -> c_int {
    if DEBUG_INSN_PERF_CNT != 0 {
        (*cpu).perf_counter += 1;
    }

    // Lookup for instruction
    let index: c_int = ilt_lookup(ilt, instruction);
    let tag: *mut ppc32_insn_exec_tag = ppc32_exec_get_insn(index).cast::<_>();
    let exec: ppc32_insn_exec_tag_exec = (*tag).exec;

    if NJM_STATS_ENABLE != 0 {
        (*cpu).insn_exec_count += 1;
        ppc32_exec_tags[index as usize].count += 1;
    }
    exec.unwrap_unchecked()(cpu, instruction)
}

/// Execute a single instruction (external)
#[no_mangle]
pub unsafe extern "C" fn ppc32_exec_single_insn_ext(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let res: c_int = ppc32_exec_single_instruction(cpu, insn);
    if likely(res == 0) {
        (*cpu).ia += size_of::<ppc_insn_t>() as m_uint32_t;
    }
    res
}

/// Execute a page
#[no_mangle]
pub unsafe extern "C" fn ppc32_exec_page(cpu: *mut cpu_ppc_t) -> c_int {
    let exec_page: m_uint32_t;
    let mut offset: m_uint32_t;
    let mut insn: ppc_insn_t;
    let mut res: c_int;

    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        exec_page = (*cpu).ia & !PPC32_MIN_PAGE_IMASK;
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        exec_page = (*cpu).ia & PPC32_MIN_PAGE_MASK;
    }
    (*cpu).njm_exec_page = exec_page as m_uint64_t;
    (*cpu).njm_exec_ptr = (*cpu).mem_op_lookup.unwrap_unchecked()(cpu, exec_page, PPC32_MTS_ICACHE as u_int).cast::<_>();

    loop {
        offset = ((*cpu).ia & PPC32_MIN_PAGE_IMASK) >> 2;
        insn = vmtoh32(*(*cpu).njm_exec_ptr.add(offset as usize));

        res = ppc32_exec_single_instruction(cpu, insn);
        if likely(res == 0) {
            (*cpu).ia = (*cpu).ia.wrapping_add(size_of::<ppc_insn_t>() as m_uint32_t);
        }
        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            if ((*cpu).ia & !PPC32_MIN_PAGE_IMASK) == exec_page {
                continue;
            }
        }
        #[cfg(feature = "USE_UNSTABLE")]
        {
            if ((*cpu).ia & PPC32_MIN_PAGE_MASK) == exec_page {
                continue;
            }
        }
        break;
    }

    0
}

/// Run PowerPC code in step-by-step mode
#[no_mangle]
pub unsafe extern "C" fn ppc32_exec_run_cpu(gen: *mut cpu_gen_t) -> *mut c_void {
    let cpu: *mut cpu_ppc_t = CPU_PPC32(gen);
    let mut timer_irq_thread: libc::pthread_t = 0;
    let mut timer_irq_check: c_int = 0;
    let mut insn: ppc_insn_t = 0;
    let mut res: c_int;

    if libc::pthread_create(addr_of_mut!(timer_irq_thread), null_mut(), std::mem::transmute::<unsafe extern "C" fn(*mut cpu_ppc_t) -> *mut c_void, extern "C" fn(_: *mut c_void) -> *mut c_void>(ppc32_timer_irq_run), cpu.cast::<_>()) != 0 {
        libc::fprintf(c_stderr(), cstr!("VM '%s': unable to create Timer IRQ thread for CPU%u.\n"), (*(*cpu).vm).name, (*gen).id);
        cpu_stop(gen);
        return null_mut();
    }

    (*gen).cpu_thread_running.set(TRUE);
    cpu_exec_loop_set(gen);

    'start_cpu: loop {
        'run: loop {
            if unlikely((*gen).state.get() != CPU_STATE_RUNNING) {
                break 'run;
            }

            // Check IRQ
            if unlikely((*cpu).irq_check.get() != 0) {
                ppc32_trigger_irq(cpu);
            }

            // Handle virtual idle loop
            if unlikely((*cpu).ia == (*cpu).idle_pc.get()) {
                (*gen).idle_count += 1;
                if (*gen).idle_count == (*gen).idle_max {
                    cpu_idle_loop(gen);
                    (*gen).idle_count = 0;
                }
            }

            // Handle the virtual CPU clock
            timer_irq_check += 1;
            if timer_irq_check as u_int == (*cpu).timer_irq_check_itv {
                timer_irq_check = 0;

                if (*cpu).timer_irq_pending.get() != 0 && (*cpu).irq_disable.get() == 0 && ((*cpu).msr & PPC32_MSR_EE) != 0 {
                    (*cpu).timer_irq_armed.set(0);
                    (*cpu).timer_irq_pending.set((*cpu).timer_irq_pending.get() - 1);

                    vm_set_irq((*cpu).vm, 0);
                    if false {
                        ppc32_trigger_timer_irq(cpu);
                    }
                }
            }

            // Increment the time base
            (*cpu).tb += 100;

            // Fetch and execute the instruction
            ppc32_exec_fetch(cpu, (*cpu).ia, addr_of_mut!(insn));
            res = ppc32_exec_single_instruction(cpu, insn);

            // Normal flow ?
            if likely(res == 0) {
                (*cpu).ia = (*cpu).ia.wrapping_add(size_of::<ppc_insn_t>() as m_uint32_t);
            }
        }

        // Check regularly if the CPU has been restarted
        while (*gen).cpu_thread_running.get() != 0 {
            (*gen).seq_state.set((*gen).seq_state.get() + 1);

            match (*gen).state.get() {
                CPU_STATE_RUNNING => {
                    (*gen).state.set(CPU_STATE_RUNNING);
                    continue 'start_cpu;
                }

                CPU_STATE_HALTED => {
                    (*gen).cpu_thread_running.set(FALSE);
                    libc::pthread_join(timer_irq_thread, null_mut());
                }

                _ => {}
            }

            // CPU is paused
            libc::usleep(200000);
        }

        return null_mut();
    }
}

/// =========================================================================

/// Update CR0
#[inline(always)]
unsafe fn ppc32_exec_update_cr0(cpu: *mut cpu_ppc_t, val: m_uint32_t) {
    let mut res: m_uint32_t;

    if (val & 0x80000000) != 0 {
        res = 1 << PPC32_CR_LT_BIT;
    } else if val > 0 {
        res = 1 << PPC32_CR_GT_BIT;
    } else {
        res = 1 << PPC32_CR_EQ_BIT;
    }

    if ((*cpu).xer & PPC32_XER_SO) != 0 {
        res |= 1 << PPC32_CR_SO_BIT;
    }

    (*cpu).cr_fields[0] = res;
}

// Update Overflow bit from a sum result (r = a + b)
//
// (a > 0) && (b > 0) => r > 0, otherwise overflow
// (a < 0) && (a < 0) => r < 0, otherwise overflow.
#[inline(always)]
unsafe fn ppc32_exec_ov_sum(cpu: *mut cpu_ppc_t, r: m_uint32_t, a: m_uint32_t, b: m_uint32_t) {
    let sc: m_uint32_t = !(a ^ b) & (a ^ r) & 0x80000000;
    if unlikely(sc != 0) {
        (*cpu).xer |= PPC32_XER_SO | PPC32_XER_OV;
    } else {
        (*cpu).xer &= !PPC32_XER_OV;
    }
}

/// Update Overflow bit from a substraction result (r = a - b)
///
/// (a > 0) && (b < 0) => r > 0, otherwise overflow
/// (a < 0) && (a > 0) => r < 0, otherwise overflow.
#[inline(always)]
unsafe fn ppc32_exec_ov_sub(cpu: *mut cpu_ppc_t, r: m_uint32_t, a: m_uint32_t, b: m_uint32_t) {
    let sc: m_uint32_t = (a ^ b) & (a ^ r) & 0x80000000;
    if unlikely(sc != 0) {
        (*cpu).xer |= PPC32_XER_SO | PPC32_XER_OV;
    } else {
        (*cpu).xer &= !PPC32_XER_OV;
    }
}

/// Update CA bit from a sum result (r = a + b)
#[inline(always)]
unsafe fn ppc32_exec_ca_sum(cpu: *mut cpu_ppc_t, r: m_uint32_t, a: m_uint32_t, _b: m_uint32_t) {
    (*cpu).xer_ca = if r < a { 1 } else { 0 };
}

/// Update CA bit from a substraction result (r = a - b)
#[inline(always)]
unsafe fn ppc32_exec_ca_sub(cpu: *mut cpu_ppc_t, _r: m_uint32_t, a: m_uint32_t, b: m_uint32_t) {
    (*cpu).xer_ca = if b > a { 1 } else { 0 };
}

/// Check condition code
#[inline(always)]
unsafe fn ppc32_check_cond(cpu: *mut cpu_ppc_t, bo: m_uint32_t, bi: m_uint32_t) -> c_int {
    let mut ctr_ok: u_int = TRUE as u_int;

    if (bo & 0x04) == 0 {
        (*cpu).ctr -= 1;
        ctr_ok = (((*cpu).ctr != 0) as u_int) ^ ((bo >> 1) & 0x1);
    }

    let cr_bit: u_int = ppc32_read_cr_bit(cpu, bi);
    let cond_ok: u_int = (bo >> 4) | ((cr_bit ^ (!bo >> 3)) & 0x1);

    (ctr_ok & cond_ok) as c_int
}

/// MFLR - Move From Link Register
unsafe extern "C" fn ppc32_exec_MFLR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    (*cpu).gpr[rd as usize] = (*cpu).lr;
    0
}

/// MTLR - Move To Link Register
unsafe extern "C" fn ppc32_exec_MTLR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    (*cpu).lr = (*cpu).gpr[rs as usize];
    0
}

/// MFCTR - Move From Counter Register
unsafe extern "C" fn ppc32_exec_MFCTR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    (*cpu).gpr[rd as usize] = (*cpu).ctr;
    0
}

/// MTCTR - Move To Counter Register
unsafe extern "C" fn ppc32_exec_MTCTR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    (*cpu).ctr = (*cpu).gpr[rs as usize];
    0
}

/// ADD
unsafe extern "C" fn ppc32_exec_ADD(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    0
}

/// ADD.
unsafe extern "C" fn ppc32_exec_ADD_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// ADDO - Add with Overflow
unsafe extern "C" fn ppc32_exec_ADDO(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b);

    ppc32_exec_ov_sum(cpu, d, a, b);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDO.
unsafe extern "C" fn ppc32_exec_ADDO_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b);

    ppc32_exec_ov_sum(cpu, d, a, b);
    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDC - Add Carrying
unsafe extern "C" fn ppc32_exec_ADDC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b);

    ppc32_exec_ca_sum(cpu, d, a, b);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDC.
unsafe extern "C" fn ppc32_exec_ADDC_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b);

    ppc32_exec_ca_sum(cpu, d, a, b);
    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDCO - Add Carrying with Overflow
unsafe extern "C" fn ppc32_exec_ADDCO(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b);

    ppc32_exec_ca_sum(cpu, d, a, b);
    ppc32_exec_ov_sum(cpu, d, a, b);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDCO.
unsafe extern "C" fn ppc32_exec_ADDCO_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b);

    ppc32_exec_ca_sum(cpu, d, a, b);
    ppc32_exec_ov_sum(cpu, d, a, b);
    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDE - Add Extended
unsafe extern "C" fn ppc32_exec_ADDE(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let carry: m_uint32_t = (*cpu).xer_ca;
    (*cpu).xer_ca = 0;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b).wrapping_add(carry);

    if ((b.wrapping_add(carry)) < b) || (d < a) {
        (*cpu).xer_ca = 1;
    }

    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDE.
unsafe extern "C" fn ppc32_exec_ADDE_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let carry: m_uint32_t = (*cpu).xer_ca;
    (*cpu).xer_ca = 0;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b).wrapping_add(carry);

    if ((b.wrapping_add(carry)) < b) || (d < a) {
        (*cpu).xer_ca = 1;
    }

    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDEO - Add Extended with Overflow
unsafe extern "C" fn ppc32_exec_ADDEO(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let carry: m_uint32_t = (*cpu).xer_ca;
    (*cpu).xer_ca = 0;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b).wrapping_add(carry);

    if ((b.wrapping_add(carry)) < b) || (d < a) {
        (*cpu).xer_ca = 1;
    }

    ppc32_exec_ov_sum(cpu, d, a, b);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDEO.
unsafe extern "C" fn ppc32_exec_ADDEO_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let carry: m_uint32_t = (*cpu).xer_ca;
    (*cpu).xer_ca = 0;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let d: m_uint32_t = a.wrapping_add(b).wrapping_add(carry);

    if ((b.wrapping_add(carry)) < b) || (d < a) {
        (*cpu).xer_ca = 1;
    }

    ppc32_exec_ov_sum(cpu, d, a, b);
    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDI - ADD Immediate
unsafe extern "C" fn ppc32_exec_ADDI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    let mut tmp: m_uint32_t = sign_extend_32(imm, 16) as m_uint32_t;

    if ra != 0 {
        tmp = tmp.wrapping_add((*cpu).gpr[ra as usize]);
    }

    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// ADDIC - ADD Immediate with Carry
unsafe extern "C" fn ppc32_exec_ADDIC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let d: m_uint32_t = a.wrapping_add(sign_extend_32(imm, 16) as m_uint32_t);
    ppc32_exec_ca_sum(cpu, d, a, 0);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDIC.
unsafe extern "C" fn ppc32_exec_ADDIC_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let d: m_uint32_t = a.wrapping_add(sign_extend_32(imm, 16) as m_uint32_t);
    ppc32_exec_ca_sum(cpu, d, a, 0);
    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDIS - ADD Immediate Shifted
unsafe extern "C" fn ppc32_exec_ADDIS(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;

    let mut tmp: m_uint32_t = imm << 16;

    if ra != 0 {
        tmp = tmp.wrapping_add((*cpu).gpr[ra as usize]);
    }

    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// ADDME - Add to Minus One Extended
unsafe extern "C" fn ppc32_exec_ADDME(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let carry: m_uint32_t = (*cpu).xer_ca;
    (*cpu).xer_ca = 0;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = 0xFFFFFFFF;
    let d = a.wrapping_add(b).wrapping_add(carry);

    if ((b.wrapping_add(carry)) < b) || (d < a) {
        (*cpu).xer_ca = 1;
    }

    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDME.
unsafe extern "C" fn ppc32_exec_ADDME_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let carry: m_uint32_t = (*cpu).xer_ca;
    (*cpu).xer_ca = 0;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = 0xFFFFFFFF;
    let d = a.wrapping_add(b).wrapping_add(carry);

    if ((b.wrapping_add(carry)) < b) || (d < a) {
        (*cpu).xer_ca = 1;
    }

    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDZE - Add to Zero Extended
unsafe extern "C" fn ppc32_exec_ADDZE(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let carry: m_uint32_t = (*cpu).xer_ca;
    (*cpu).xer_ca = 0;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let d: m_uint32_t = a.wrapping_add(carry);

    if d < a {
        (*cpu).xer_ca = 1;
    }

    (*cpu).gpr[rd as usize] = d;
    0
}

/// ADDZE.
unsafe extern "C" fn ppc32_exec_ADDZE_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let carry: m_uint32_t = (*cpu).xer_ca;
    (*cpu).xer_ca = 0;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let d: m_uint32_t = a.wrapping_add(carry);

    if d < a {
        (*cpu).xer_ca = 1;
    }

    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// AND
unsafe extern "C" fn ppc32_exec_AND(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] & (*cpu).gpr[rb as usize];
    0
}

/// AND.
unsafe extern "C" fn ppc32_exec_AND_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = (*cpu).gpr[rs as usize] & (*cpu).gpr[rb as usize];
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// ANDC - AND with Complement
unsafe extern "C" fn ppc32_exec_ANDC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] & (!(*cpu).gpr[rb as usize]);
    0
}

/// ANDC. - AND with Complement
unsafe extern "C" fn ppc32_exec_ANDC_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = (*cpu).gpr[rs as usize] & (!(*cpu).gpr[rb as usize]);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// ANDI. - AND Immediate
unsafe extern "C" fn ppc32_exec_ANDI_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let tmp: m_uint32_t = (*cpu).gpr[rs as usize] & (imm as m_uint32_t);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// ANDIS. - AND Immediate Shifted
unsafe extern "C" fn ppc32_exec_ANDIS_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let tmp: m_uint32_t = (*cpu).gpr[rs as usize] & ((imm as m_uint32_t) << 16);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// B - Branch
unsafe extern "C" fn ppc32_exec_B(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let offset: m_uint32_t = bits(insn, 2, 25) as m_uint32_t;

    (*cpu).ia = (*cpu).ia.wrapping_add(sign_extend_32((offset << 2) as m_int32_t, 26) as m_uint32_t);
    1
}

/// BA - Branch Absolute
unsafe extern "C" fn ppc32_exec_BA(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let offset: m_uint32_t = bits(insn, 2, 25) as m_uint32_t;

    (*cpu).ia = sign_extend_32((offset << 2) as m_int32_t, 26) as m_uint32_t;
    1
}

/// BL - Branch and Link
unsafe extern "C" fn ppc32_exec_BL(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let offset: m_uint32_t = bits(insn, 2, 25) as m_uint32_t;

    (*cpu).lr = (*cpu).ia.wrapping_add(4);
    (*cpu).ia = (*cpu).ia.wrapping_add(sign_extend_32((offset << 2) as m_int32_t, 26) as m_uint32_t);
    1
}

/// BLA - Branch and Link Absolute
unsafe extern "C" fn ppc32_exec_BLA(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let offset: m_uint32_t = bits(insn, 2, 25) as m_uint32_t;

    (*cpu).lr = (*cpu).ia.wrapping_add(4);
    (*cpu).ia = sign_extend_32((offset << 2) as m_int32_t, 26) as m_uint32_t;
    1
}

/// BC - Branch Conditional
unsafe extern "C" fn ppc32_exec_BC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);
    let bd: c_int = bits(insn, 2, 15);

    if ppc32_check_cond(cpu, bo as m_uint32_t, bi as m_uint32_t) != 0 {
        (*cpu).ia = (*cpu).ia.wrapping_add(sign_extend_32(bd << 2, 16) as m_uint32_t);
        return 1;
    }

    0
}

/// BCA - Branch Conditional (absolute)
unsafe extern "C" fn ppc32_exec_BCA(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);
    let bd: c_int = bits(insn, 2, 15);

    if ppc32_check_cond(cpu, bo as m_uint32_t, bi as m_uint32_t) != 0 {
        (*cpu).ia = sign_extend_32(bd << 2, 16) as m_uint32_t;
        return 1;
    }

    0
}

/// BCL - Branch Conditional and Link
unsafe extern "C" fn ppc32_exec_BCL(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);
    let bd: c_int = bits(insn, 2, 15);

    (*cpu).lr = (*cpu).ia.wrapping_add(4);

    if ppc32_check_cond(cpu, bo as m_uint32_t, bi as m_uint32_t) != 0 {
        (*cpu).ia = (*cpu).ia.wrapping_add(sign_extend_32(bd << 2, 16) as m_uint32_t);
        return 1;
    }

    0
}

/// BCLA - Branch Conditional and Link (absolute)
unsafe extern "C" fn ppc32_exec_BCLA(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);
    let bd: c_int = bits(insn, 2, 15);

    (*cpu).lr = (*cpu).ia.wrapping_add(4);

    if ppc32_check_cond(cpu, bo as m_uint32_t, bi as m_uint32_t) != 0 {
        (*cpu).ia = sign_extend_32(bd << 2, 16) as m_uint32_t;
        return 1;
    }

    0
}

/// BCLR - Branch Conditional to Link register
unsafe extern "C" fn ppc32_exec_BCLR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);

    if ppc32_check_cond(cpu, bo as m_uint32_t, bi as m_uint32_t) != 0 {
        (*cpu).ia = (*cpu).lr & !0x3;
        return 1;
    }

    0
}

/// BCLRL - Branch Conditional to Link register
unsafe extern "C" fn ppc32_exec_BCLRL(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);

    let new_ia: m_uint32_t = (*cpu).lr & !0x03;
    (*cpu).lr = (*cpu).ia.wrapping_add(4);

    if ppc32_check_cond(cpu, bo as m_uint32_t, bi as m_uint32_t) != 0 {
        (*cpu).ia = new_ia;
        return 1;
    }

    0
}

/// BCCTR - Branch Conditional to Count register
unsafe extern "C" fn ppc32_exec_BCCTR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);

    if ppc32_check_cond(cpu, bo as m_uint32_t, bi as m_uint32_t) != 0 {
        (*cpu).ia = (*cpu).ctr & !0x3;
        return 1;
    }

    0
}

/// BCCTRL - Branch Conditional to Count register and Link
unsafe extern "C" fn ppc32_exec_BCCTRL(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);

    (*cpu).lr = (*cpu).ia.wrapping_add(4);

    if ppc32_check_cond(cpu, bo as m_uint32_t, bi as m_uint32_t) != 0 {
        (*cpu).ia = (*cpu).ctr & !0x3;
        return 1;
    }

    0
}

/// CMP - Compare
unsafe extern "C" fn ppc32_exec_CMP(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut res: m_uint32_t;

    let a: m_int32_t = (*cpu).gpr[ra as usize] as m_int32_t;
    let b: m_int32_t = (*cpu).gpr[rb as usize] as m_int32_t;

    #[allow(clippy::comparison_chain)]
    if a < b {
        res = 0x08;
    } else if a > b {
        res = 0x04;
    } else {
        res = 0x02;
    }

    if ((*cpu).xer & PPC32_XER_SO) != 0 {
        res |= 0x01;
    }

    (*cpu).cr_fields[rd as usize] = res;
    0
}

/// CMPI - Compare Immediate
unsafe extern "C" fn ppc32_exec_CMPI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;
    let mut res: m_uint32_t;

    let a: m_int32_t = (*cpu).gpr[ra as usize] as m_int32_t;
    let b: m_int32_t = sign_extend_32(imm as m_int32_t, 16);

    #[allow(clippy::comparison_chain)]
    if a < b {
        res = 0x08;
    } else if a > b {
        res = 0x04;
    } else {
        res = 0x02;
    }

    if ((*cpu).xer & PPC32_XER_SO) != 0 {
        res |= 0x01;
    }

    (*cpu).cr_fields[rd as usize] = res;
    0
}

/// CMPL - Compare Logical
unsafe extern "C" fn ppc32_exec_CMPL(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut res: m_uint32_t;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];

    #[allow(clippy::comparison_chain)]
    if a < b {
        res = 0x08;
    } else if a > b {
        res = 0x04;
    } else {
        res = 0x02;
    }

    if ((*cpu).xer & PPC32_XER_SO) != 0 {
        res |= 0x01;
    }

    (*cpu).cr_fields[rd as usize] = res;
    0
}

/// CMPLI - Compare Logical Immediate
unsafe extern "C" fn ppc32_exec_CMPLI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;
    let mut res: m_uint32_t;

    let a: m_uint32_t = (*cpu).gpr[ra as usize];

    #[allow(clippy::comparison_chain)]
    if a < imm {
        res = 0x08;
    } else if a > imm {
        res = 0x04;
    } else {
        res = 0x02;
    }

    if ((*cpu).xer & PPC32_XER_SO) != 0 {
        res |= 0x01;
    }

    (*cpu).cr_fields[rd as usize] = res;
    0
}

/// CNTLZW - Count Leading Zeros Word
unsafe extern "C" fn ppc32_exec_CNTLZW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let mut i: c_int;

    let val: m_uint32_t = (*cpu).gpr[rs as usize];
    let mut mask: m_uint32_t = 0x80000000;

    i = 0;
    while i < 32 {
        if (val & mask) != 0 {
            break;
        }

        mask >>= 1;
        i += 1;
    }

    (*cpu).gpr[ra as usize] = i as m_uint32_t;
    0
}

/// CRAND - Condition Register AND
unsafe extern "C" fn ppc32_exec_CRAND(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint32_t;

    tmp = ppc32_read_cr_bit(cpu, ba as u_int);
    tmp &= ppc32_read_cr_bit(cpu, bb as u_int);

    if (tmp & 0x1) != 0 {
        ppc32_set_cr_bit(cpu, bd as u_int);
    } else {
        ppc32_clear_cr_bit(cpu, bd as u_int);
    }

    0
}

/// CREQV - Condition Register Equivalent
unsafe extern "C" fn ppc32_exec_CREQV(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint32_t;

    tmp = ppc32_read_cr_bit(cpu, ba as u_int);
    tmp ^= ppc32_read_cr_bit(cpu, bb as u_int);

    if (tmp & 0x1) == 0 {
        ppc32_set_cr_bit(cpu, bd as u_int);
    } else {
        ppc32_clear_cr_bit(cpu, bd as u_int);
    }

    0
}

/// CRANDC - Condition Register AND with Complement
unsafe extern "C" fn ppc32_exec_CRANDC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint32_t;

    tmp = ppc32_read_cr_bit(cpu, ba as u_int);
    tmp &= !ppc32_read_cr_bit(cpu, bb as u_int);

    if (tmp & 0x1) != 0 {
        ppc32_set_cr_bit(cpu, bd as u_int);
    } else {
        ppc32_clear_cr_bit(cpu, bd as u_int);
    }

    0
}

/// CRNAND - Condition Register NAND
unsafe extern "C" fn ppc32_exec_CRNAND(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint32_t;

    tmp = ppc32_read_cr_bit(cpu, ba as u_int);
    tmp &= ppc32_read_cr_bit(cpu, bb as u_int);

    if (tmp & 0x1) == 0 {
        ppc32_set_cr_bit(cpu, bd as u_int);
    } else {
        ppc32_clear_cr_bit(cpu, bd as u_int);
    }

    0
}

/// CRNOR - Condition Register NOR
unsafe extern "C" fn ppc32_exec_CRNOR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint32_t;

    tmp = ppc32_read_cr_bit(cpu, ba as u_int);
    tmp |= ppc32_read_cr_bit(cpu, bb as u_int);

    if (tmp & 0x1) == 0 {
        ppc32_set_cr_bit(cpu, bd as u_int);
    } else {
        ppc32_clear_cr_bit(cpu, bd as u_int);
    }

    0
}

/// CROR - Condition Register OR
unsafe extern "C" fn ppc32_exec_CROR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint32_t;

    tmp = ppc32_read_cr_bit(cpu, ba as u_int);
    tmp |= ppc32_read_cr_bit(cpu, bb as u_int);

    if (tmp & 0x1) != 0 {
        ppc32_set_cr_bit(cpu, bd as u_int);
    } else {
        ppc32_clear_cr_bit(cpu, bd as u_int);
    }

    0
}

/// CRORC - Condition Register OR with complement
unsafe extern "C" fn ppc32_exec_CRORC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint32_t;

    tmp = ppc32_read_cr_bit(cpu, ba as u_int);
    tmp |= !ppc32_read_cr_bit(cpu, bb as u_int);

    if (tmp & 0x1) != 0 {
        ppc32_set_cr_bit(cpu, bd as u_int);
    } else {
        ppc32_clear_cr_bit(cpu, bd as u_int);
    }

    0
}

/// CRXOR - Condition Register XOR
unsafe extern "C" fn ppc32_exec_CRXOR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint32_t;

    tmp = ppc32_read_cr_bit(cpu, ba as u_int);
    tmp ^= ppc32_read_cr_bit(cpu, bb as u_int);

    if (tmp & 0x1) != 0 {
        ppc32_set_cr_bit(cpu, bd as u_int);
    } else {
        ppc32_clear_cr_bit(cpu, bd as u_int);
    }

    0
}

/// DCBF - Data Cache Block Flush
unsafe extern "C" fn ppc32_exec_DCBF(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut vaddr: m_uint32_t;

    vaddr = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    if false {
        libc::printf(cstr!("PPC32: DBCF: vaddr=0x%8.8x\n"), vaddr);
    }
    0
}

/// DCBI - Data Cache Block Invalidate
unsafe extern "C" fn ppc32_exec_DCBI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut vaddr: m_uint32_t;

    vaddr = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    if false {
        libc::printf(cstr!("PPC32: DBCI: vaddr=0x%8.8x\n"), vaddr);
    }
    0
}

/// DCBT - Data Cache Block Touch
unsafe extern "C" fn ppc32_exec_DCBT(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut vaddr: m_uint32_t;

    vaddr = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    if false {
        libc::printf(cstr!("PPC32: DBCT: vaddr=0x%8.8x\n"), vaddr);
    }
    0
}

/// DCBST - Data Cache Block Store
unsafe extern "C" fn ppc32_exec_DCBST(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut vaddr: m_uint32_t;

    vaddr = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    if false {
        libc::printf(cstr!("PPC32: DBCST: vaddr=0x%8.8x\n"), vaddr);
    }
    0
}

/// DIVW - Divide Word
unsafe extern "C" fn ppc32_exec_DIVW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    #[cfg(not(feature = "USE_UNSTABLE"))]
    type T = m_uint32_t; // FIXME can produce different results
    #[cfg(feature = "USE_UNSTABLE")]
    type T = m_int32_t;
    let a: T = (*cpu).gpr[ra as usize] as T;
    let b: T = (*cpu).gpr[rb as usize] as T;

    if !((b == 0) || (((*cpu).gpr[ra as usize] == 0x80000000) && (b == -1_i32 as T))) {
        (*cpu).gpr[rd as usize] = (a / b) as m_uint32_t;
    }
    0
}

/// DIVW. - Divide Word
unsafe extern "C" fn ppc32_exec_DIVW_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_int32_t = (*cpu).gpr[ra as usize] as m_int32_t;
    let b: m_int32_t = (*cpu).gpr[rb as usize] as m_int32_t;
    let mut d: m_int32_t = 0;

    if !((b == 0) || (((*cpu).gpr[ra as usize] == 0x80000000) && (b == -1))) {
        d = a / b;
    }

    ppc32_exec_update_cr0(cpu, d as m_uint32_t);
    (*cpu).gpr[rd as usize] = d as m_uint32_t;
    0
}

/// DIVWU - Divide Word Unsigned
unsafe extern "C" fn ppc32_exec_DIVWU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];

    if b != 0 {
        (*cpu).gpr[rd as usize] = a / b;
    }
    0
}

/// DIVWU. - Divide Word Unsigned
unsafe extern "C" fn ppc32_exec_DIVWU_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let mut d: m_uint32_t = 0;

    if b != 0 {
        d = a / b;
    }

    ppc32_exec_update_cr0(cpu, d);
    (*cpu).gpr[rd as usize] = d;
    0
}

/// EIEIO - Enforce In-order Execution of I/O
unsafe extern "C" fn ppc32_exec_EIEIO(_cpu: *mut cpu_ppc_t, _insn: ppc_insn_t) -> c_int {
    0
}

/// EQV
unsafe extern "C" fn ppc32_exec_EQV(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[ra as usize] = !((*cpu).gpr[rs as usize] ^ (*cpu).gpr[rb as usize]);
    0
}

/// EXTSB - Extend Sign Byte
unsafe extern "C" fn ppc32_exec_EXTSB(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    (*cpu).gpr[ra as usize] = sign_extend_32((*cpu).gpr[rs as usize] as m_int32_t, 8) as m_uint32_t;
    0
}

/// EXTSB.
unsafe extern "C" fn ppc32_exec_EXTSB_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let tmp: m_uint32_t = sign_extend_32((*cpu).gpr[rs as usize] as m_int32_t, 8) as m_uint32_t;
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// EXTSH - Extend Sign Word
unsafe extern "C" fn ppc32_exec_EXTSH(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    (*cpu).gpr[ra as usize] = sign_extend_32((*cpu).gpr[rs as usize] as m_int32_t, 16) as m_uint32_t;
    0
}

/// EXTSH.
unsafe extern "C" fn ppc32_exec_EXTSH_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let tmp: m_uint32_t = sign_extend_32((*cpu).gpr[rs as usize] as m_int32_t, 16) as m_uint32_t;
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// ICBI - Instruction Cache Block Invalidate
unsafe extern "C" fn ppc32_exec_ICBI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_ICBI as c_int, vaddr, 0);
    0
}

/// ISYNC - Instruction Synchronize
unsafe extern "C" fn ppc32_exec_ISYNC(_cpu: *mut cpu_ppc_t, _insn: ppc_insn_t) -> c_int {
    0
}

/// LBZ - Load Byte and Zero
unsafe extern "C" fn ppc32_exec_LBZ(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LBZ as c_int, vaddr, rd as u_int);
    0
}

/// LBZU - Load Byte and Zero with Update
unsafe extern "C" fn ppc32_exec_LBZU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_LBZ as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LBZUX - Load Byte and Zero with Update Indexed
unsafe extern "C" fn ppc32_exec_LBZUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_LBZ as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LBZX - Load Byte and Zero Indexed
unsafe extern "C" fn ppc32_exec_LBZX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LBZ as c_int, vaddr, rd as u_int);
    0
}

/// LHA - Load Half-Word Algebraic
unsafe extern "C" fn ppc32_exec_LHA(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LHA as c_int, vaddr, rd as u_int);
    0
}

/// LHAU - Load Half-Word Algebraic with Update
unsafe extern "C" fn ppc32_exec_LHAU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_LHA as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LHAUX - Load Half-Word Algebraic with Update Indexed
unsafe extern "C" fn ppc32_exec_LHAUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_LHA as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LHAX - Load Half-Word Algebraic ndexed
unsafe extern "C" fn ppc32_exec_LHAX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LHA as c_int, vaddr, rd as u_int);
    0
}

/// LHZ - Load Half-Word and Zero
unsafe extern "C" fn ppc32_exec_LHZ(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LHZ as c_int, vaddr, rd as u_int);
    0
}

/// LHZU - Load Half-Word and Zero with Update
unsafe extern "C" fn ppc32_exec_LHZU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_LHZ as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LHZUX - Load Half-Word and Zero with Update Indexed
unsafe extern "C" fn ppc32_exec_LHZUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_LHZ as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LHZX - Load Half-Word and Zero Indexed
unsafe extern "C" fn ppc32_exec_LHZX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LHZ as c_int, vaddr, rd as u_int);
    0
}

/// LMW - Load Multiple Word
unsafe extern "C" fn ppc32_exec_LMW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    for r in rd..=31 {
        ppc32_exec_memop(cpu, PPC_MEMOP_LWZ as c_int, vaddr, r as u_int);
        vaddr = vaddr.wrapping_add(size_of::<m_uint32_t>() as m_uint32_t);
    }

    0
}

/// LWBRX - Load Word Byte-Reverse Indexed
unsafe extern "C" fn ppc32_exec_LWBRX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LWBR as c_int, vaddr, rd as u_int);
    0
}

/// LWZ - Load Word and Zero
unsafe extern "C" fn ppc32_exec_LWZ(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LWZ as c_int, vaddr, rd as u_int);
    0
}

/// LWZU - Load Word and Zero with Update
unsafe extern "C" fn ppc32_exec_LWZU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_LWZ as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LWZUX - Load Word and Zero with Update Indexed
unsafe extern "C" fn ppc32_exec_LWZUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_LWZ as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LWZX - Load Word and Zero Indexed
unsafe extern "C" fn ppc32_exec_LWZX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LWZ as c_int, vaddr, rd as u_int);
    0
}

/// LWARX - Load Word and Reserve Indexed
unsafe extern "C" fn ppc32_exec_LWARX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    (*cpu).reserve = 1;
    ppc32_exec_memop(cpu, PPC_MEMOP_LWZ as c_int, vaddr, rd as u_int);
    0
}

/// LFD - Load Floating-Point Double
unsafe extern "C" fn ppc32_exec_LFD(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LFD as c_int, vaddr, rd as u_int);
    0
}

/// LFDU - Load Floating-Point Double with Update
unsafe extern "C" fn ppc32_exec_LFDU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_LFD as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LFDUX - Load Floating-Point Double with Update Indexed
unsafe extern "C" fn ppc32_exec_LFDUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_LFD as c_int, vaddr, rd as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// LFDX - Load Floating-Point Double Indexed
unsafe extern "C" fn ppc32_exec_LFDX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_LFD as c_int, vaddr, rd as u_int);
    0
}

/// LSWI - Load String Word Immediate
unsafe extern "C" fn ppc32_exec_LSWI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let mut nb: c_int = bits(insn, 11, 15);
    let mut vaddr: m_uint32_t = 0;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    if nb == 0 {
        nb = 32;
    }

    let mut r: c_int = rd - 1;
    (*cpu).sw_pos = 0;

    while nb > 0 {
        if (*cpu).sw_pos == 0 {
            r = (r + 1) & 0x1F;
            (*cpu).gpr[r as usize] = 0;
        }

        ppc32_exec_memop(cpu, PPC_MEMOP_LSW as c_int, vaddr, r as u_int);
        (*cpu).sw_pos += 8;

        if (*cpu).sw_pos == 32 {
            (*cpu).sw_pos = 0;
        }

        vaddr = vaddr.wrapping_add(1);
        nb -= 1;
    }

    0
}

/// LSWX - Load String Word Indexed
unsafe extern "C" fn ppc32_exec_LSWX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut vaddr: m_uint32_t;

    vaddr = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    let mut nb: c_int = ((*cpu).xer & PPC32_XER_BC_MASK) as c_int;
    let mut r = rd - 1;
    (*cpu).sw_pos = 0;

    while nb > 0 {
        if (*cpu).sw_pos == 0 {
            r = (r + 1) & 0x1F;
            (*cpu).gpr[r as usize] = 0;
        }

        ppc32_exec_memop(cpu, PPC_MEMOP_LSW as c_int, vaddr, r as u_int);
        (*cpu).sw_pos += 8;

        if (*cpu).sw_pos == 32 {
            (*cpu).sw_pos = 0;
        }

        vaddr = vaddr.wrapping_add(1);
        nb -= 1;
    }

    0
}

/// MCRF - Move Condition Register Field
unsafe extern "C" fn ppc32_exec_MCRF(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let rs: c_int = bits(insn, 18, 20);

    (*cpu).cr_fields[rd as usize] = (*cpu).cr_fields[rs as usize];
    0
}

/// MFCR - Move from Condition Register
unsafe extern "C" fn ppc32_exec_MFCR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    (*cpu).gpr[rd as usize] = ppc32_get_cr(cpu);
    0
}

/// MFMSR - Move from Machine State Register
unsafe extern "C" fn ppc32_exec_MFMSR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    (*cpu).gpr[rd as usize] = (*cpu).msr;
    0
}

/// MFTBU - Move from Time Base (Up)
unsafe extern "C" fn ppc32_exec_MFTBU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    (*cpu).gpr[rd as usize] = ((*cpu).tb >> 32) as m_uint32_t;
    0
}

/// MFTBL - Move from Time Base (Lo)
unsafe extern "C" fn ppc32_exec_MFTBL(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    (*cpu).tb += 50;

    (*cpu).gpr[rd as usize] = ((*cpu).tb & 0xFFFFFFFF) as m_uint32_t;
    0
}

/// MFSPR - Move from Special-Purpose Register
unsafe extern "C" fn ppc32_exec_MFSPR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let spr0: c_int = bits(insn, 16, 20);
    let spr1: c_int = bits(insn, 11, 15);

    let spr: u_int = ((spr1 << 5) | spr0) as u_int;
    (*cpu).gpr[rd as usize] = 0;

    if false {
        cpu_log!((*cpu).gen, cstr!("SPR"), cstr!("reading SPR=%d at cpu->ia=0x%8.8x\n"), spr, (*cpu).ia);
    }

    if (spr1 == 0x10) || (spr1 == 0x11) {
        (*cpu).gpr[rd as usize] = ppc32_get_bat_spr(cpu, spr);
        return 0;
    }

    match spr {
        PPC32_SPR_XER => {
            (*cpu).gpr[rd as usize] = (*cpu).xer | ((*cpu).xer_ca << PPC32_XER_CA_BIT);
        }
        PPC32_SPR_DSISR => {
            (*cpu).gpr[rd as usize] = (*cpu).dsisr;
        }
        PPC32_SPR_DAR => {
            (*cpu).gpr[rd as usize] = (*cpu).dar;
        }
        PPC32_SPR_DEC => {
            (*cpu).gpr[rd as usize] = (*cpu).dec;
        }
        PPC32_SPR_SDR1 => {
            (*cpu).gpr[rd as usize] = (*cpu).sdr1;
        }
        PPC32_SPR_SRR0 => {
            (*cpu).gpr[rd as usize] = (*cpu).srr0;
        }
        PPC32_SPR_SRR1 => {
            (*cpu).gpr[rd as usize] = (*cpu).srr1;
        }
        PPC32_SPR_TBL_READ => {
            (*cpu).gpr[rd as usize] = ((*cpu).tb & 0xFFFFFFFF) as m_uint32_t;
        }
        PPC32_SPR_TBU_READ => {
            (*cpu).gpr[rd as usize] = ((*cpu).tb >> 32) as m_uint32_t;
        }
        PPC32_SPR_SPRG0 => {
            (*cpu).gpr[rd as usize] = (*cpu).sprg[0];
        }
        PPC32_SPR_SPRG1 => {
            (*cpu).gpr[rd as usize] = (*cpu).sprg[1];
        }
        PPC32_SPR_SPRG2 => {
            (*cpu).gpr[rd as usize] = (*cpu).sprg[2];
        }
        PPC32_SPR_SPRG3 => {
            (*cpu).gpr[rd as usize] = (*cpu).sprg[3];
        }
        PPC32_SPR_PVR => {
            (*cpu).gpr[rd as usize] = (*cpu).pvr;
        }
        PPC32_SPR_HID0 => {
            (*cpu).gpr[rd as usize] = (*cpu).hid0;
        }
        PPC32_SPR_HID1 => {
            (*cpu).gpr[rd as usize] = (*cpu).hid1;
        }
        PPC405_SPR_PID => {
            (*cpu).gpr[rd as usize] = (*cpu).ppc405_pid;
        }

        // MPC860 IMMR
        638 => {
            (*cpu).gpr[rd as usize] = (*cpu).mpc860_immr;
        }

        _ => {
            (*cpu).gpr[rd as usize] = 0x0;
            if false {
                libc::printf(cstr!("READING SPR = %d\n"), spr);
            }
        }
    }

    0
}

/// MFSR - Move From Segment Register
unsafe extern "C" fn ppc32_exec_MFSR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let sr: c_int = bits(insn, 16, 19);

    (*cpu).gpr[rd as usize] = (*cpu).sr[sr as usize];
    0
}

/// MFSRIN - Move From Segment Register Indirect
unsafe extern "C" fn ppc32_exec_MFSRIN(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).sr[((*cpu).gpr[rb as usize] >> 28) as usize];
    0
}

/// MTCRF - Move to Condition Register Fields
unsafe extern "C" fn ppc32_exec_MTCRF(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let crm: c_int = bits(insn, 12, 19);

    for i in 0..8 {
        if (crm & (1 << (7 - i))) != 0 {
            (*cpu).cr_fields[i] = ((*cpu).gpr[rs as usize] >> (28 - (i << 2))) & 0x0F;
        }
    }

    0
}

/// MTMSR - Move to Machine State Register
unsafe extern "C" fn ppc32_exec_MTMSR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    (*cpu).msr = (*cpu).gpr[rs as usize];
    (*cpu).irq_check.set((((*cpu).msr & PPC32_MSR_EE) != 0 && (*cpu).irq_pending.get() != 0).into());

    if false {
        libc::printf(cstr!("New MSR = 0x%8.8x at cpu->ia=0x%8.8x\n"), (*cpu).msr, (*cpu).ia);
    }
    0
}

/// MTSPR - Move to Special-Purpose Register
unsafe extern "C" fn ppc32_exec_MTSPR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let spr0: c_int = bits(insn, 16, 20);
    let spr1: c_int = bits(insn, 11, 15);

    let spr: u_int = ((spr1 << 5) | spr0) as u_int;

    if false {
        cpu_log!((*cpu).gen, cstr!("SPR"), cstr!("writing SPR=%d, val=0x%8.8x at cpu->ia=0x%8.8x\n"), spr, (*cpu).ia, (*cpu).gpr[rd as usize]);
    }

    if (spr1 == 0x10) || (spr1 == 0x11) {
        ppc32_set_bat_spr(cpu, spr, (*cpu).gpr[rd as usize]);
        return 0;
    }

    match spr {
        PPC32_SPR_XER => {
            (*cpu).xer = (*cpu).gpr[rd as usize] & !PPC32_XER_CA;
            (*cpu).xer_ca = ((*cpu).gpr[rd as usize] >> PPC32_XER_CA_BIT) & 0x1;
        }
        PPC32_SPR_DEC => {
            if false {
                libc::printf(cstr!("WRITING DECR 0x%8.8x AT IA=0x%8.8x\n"), (*cpu).gpr[rd as usize], (*cpu).ia);
            }
            (*cpu).dec = (*cpu).gpr[rd as usize];
            (*cpu).timer_irq_armed.set(TRUE as u_int);
        }
        PPC32_SPR_SDR1 => {
            #[cfg(not(feature = "USE_UNSTABLE"))]
            {
                (*cpu).sdr1 = (*cpu).gpr[rd as usize];
                ppc32_mem_invalidate_cache(cpu);
            }
            #[cfg(feature = "USE_UNSTABLE")]
            {
                ppc32_set_sdr1(cpu, (*cpu).gpr[rd as usize]);
            }
        }
        PPC32_SPR_SRR0 => {
            (*cpu).srr0 = (*cpu).gpr[rd as usize];
        }
        PPC32_SPR_SRR1 => {
            (*cpu).srr1 = (*cpu).gpr[rd as usize];
        }
        PPC32_SPR_SPRG0 => {
            (*cpu).sprg[0] = (*cpu).gpr[rd as usize];
        }
        PPC32_SPR_SPRG1 => {
            (*cpu).sprg[1] = (*cpu).gpr[rd as usize];
        }
        PPC32_SPR_SPRG2 => {
            (*cpu).sprg[2] = (*cpu).gpr[rd as usize];
        }
        PPC32_SPR_SPRG3 => {
            (*cpu).sprg[3] = (*cpu).gpr[rd as usize];
        }
        PPC32_SPR_HID0 => {
            (*cpu).hid0 = (*cpu).gpr[rd as usize];
        }
        PPC32_SPR_HID1 => {
            (*cpu).hid1 = (*cpu).gpr[rd as usize];
        }
        PPC405_SPR_PID => {
            (*cpu).ppc405_pid = (*cpu).gpr[rd as usize];
        }
        _ => {
            if false {
                libc::printf(cstr!("WRITING SPR=%d, data=0x%8.8x\n"), spr, (*cpu).gpr[rd as usize]);
            }
        }
    }

    0
}

/// MTSR - Move To Segment Register
unsafe extern "C" fn ppc32_exec_MTSR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let sr: c_int = bits(insn, 16, 19);

    (*cpu).sr[sr as usize] = (*cpu).gpr[rs as usize];
    ppc32_mem_invalidate_cache(cpu);
    0
}

/// MULHW - Multiply High Word
unsafe extern "C" fn ppc32_exec_MULHW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut tmp: m_int64_t;

    tmp = (*cpu).gpr[ra as usize] as m_int32_t as m_int64_t;
    tmp *= (*cpu).gpr[rb as usize] as m_int32_t as m_int64_t;
    let res: m_uint32_t = (tmp >> 32) as m_uint32_t;

    (*cpu).gpr[rd as usize] = res;
    0
}

/// MULHW.
unsafe extern "C" fn ppc32_exec_MULHW_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut tmp: m_int64_t;

    tmp = (*cpu).gpr[ra as usize] as m_int32_t as m_int64_t;
    tmp *= (*cpu).gpr[rb as usize] as m_int32_t as m_int64_t;
    let res: m_uint32_t = (tmp >> 32) as m_uint32_t;
    ppc32_exec_update_cr0(cpu, res);
    (*cpu).gpr[rd as usize] = res;
    0
}

/// MULHWU - Multiply High Word Unsigned
unsafe extern "C" fn ppc32_exec_MULHWU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint64_t;

    tmp = (*cpu).gpr[ra as usize] as m_uint64_t;
    tmp *= (*cpu).gpr[rb as usize] as m_uint64_t;
    let res: m_uint32_t = (tmp >> 32) as m_uint32_t;

    (*cpu).gpr[rd as usize] = res;
    0
}

/// MULHWU.
unsafe extern "C" fn ppc32_exec_MULHWU_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut tmp: m_uint64_t;

    tmp = (*cpu).gpr[ra as usize] as m_uint64_t;
    tmp *= (*cpu).gpr[rb as usize] as m_uint64_t;
    let res: m_uint32_t = (tmp >> 32) as m_uint32_t;
    ppc32_exec_update_cr0(cpu, res);
    (*cpu).gpr[rd as usize] = res;
    0
}

/// MULLI - Multiply Low Immediate
unsafe extern "C" fn ppc32_exec_MULLI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;

    (*cpu).gpr[rd as usize] = ((*cpu).gpr[ra as usize] as m_int32_t * sign_extend_32(imm as m_int32_t, 16)) as m_uint32_t;
    0
}

/// MULLW - Multiply Low Word
unsafe extern "C" fn ppc32_exec_MULLW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut tmp: m_int64_t;

    tmp = (*cpu).gpr[ra as usize] as m_int32_t as m_int64_t;
    tmp *= (*cpu).gpr[rb as usize] as m_int32_t as m_int64_t;
    (*cpu).gpr[rd as usize] = tmp as m_uint32_t;
    0
}

/// MULLW.
unsafe extern "C" fn ppc32_exec_MULLW_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut tmp: m_int64_t;

    tmp = (*cpu).gpr[ra as usize] as m_int32_t as m_int64_t;
    tmp *= (*cpu).gpr[rb as usize] as m_int32_t as m_int64_t;

    let res: m_uint32_t = tmp as m_uint32_t;
    ppc32_exec_update_cr0(cpu, res);
    (*cpu).gpr[rd as usize] = res;
    0
}

/// MULLWO - Multiply Low Word with Overflow
unsafe extern "C" fn ppc32_exec_MULLWO(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut tmp: m_int64_t;

    tmp = (*cpu).gpr[ra as usize] as m_int32_t as m_int64_t;
    tmp *= (*cpu).gpr[rb as usize] as m_int32_t as m_int64_t;

    (*cpu).xer &= !PPC32_XER_OV;

    if unlikely(tmp != tmp as m_int32_t as m_int64_t) {
        (*cpu).xer |= PPC32_XER_OV | PPC32_XER_SO;
    }

    (*cpu).gpr[rd as usize] = tmp as m_uint32_t;
    0
}

/// MULLWO.
unsafe extern "C" fn ppc32_exec_MULLWO_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut tmp: m_int64_t;

    tmp = (*cpu).gpr[ra as usize] as m_int32_t as m_int64_t;
    tmp *= (*cpu).gpr[rb as usize] as m_int32_t as m_int64_t;

    (*cpu).xer &= !PPC32_XER_OV;

    if unlikely(tmp != tmp as m_int32_t as m_int64_t) {
        (*cpu).xer |= PPC32_XER_OV | PPC32_XER_SO;
    }

    let res: m_uint32_t = tmp as m_uint32_t;
    ppc32_exec_update_cr0(cpu, res);
    (*cpu).gpr[rd as usize] = res;
    0
}

/// NAND
unsafe extern "C" fn ppc32_exec_NAND(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[ra as usize] = !((*cpu).gpr[rs as usize] & (*cpu).gpr[rb as usize]);
    0
}

/// NAND.
unsafe extern "C" fn ppc32_exec_NAND_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = !((*cpu).gpr[rs as usize] & (*cpu).gpr[rb as usize]);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// NEG - Negate
unsafe extern "C" fn ppc32_exec_NEG(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    (*cpu).gpr[rd as usize] = !(*cpu).gpr[ra as usize] + 1;
    0
}

/// NEG.
unsafe extern "C" fn ppc32_exec_NEG_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let tmp: m_uint32_t = !(*cpu).gpr[ra as usize] + 1;
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// NEGO
unsafe extern "C" fn ppc32_exec_NEGO(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let tmp: m_uint32_t = !(*cpu).gpr[ra as usize] + 1;
    (*cpu).gpr[rd as usize] = tmp;

    (*cpu).xer &= !PPC32_XER_OV;

    if unlikely(tmp == 0x80000000) {
        (*cpu).xer |= PPC32_XER_OV | PPC32_XER_SO;
    }

    ppc32_exec_update_cr0(cpu, tmp);
    0
}

/// NEGO.
unsafe extern "C" fn ppc32_exec_NEGO_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let tmp: m_uint32_t = !(*cpu).gpr[ra as usize] + 1;
    (*cpu).gpr[rd as usize] = tmp;

    (*cpu).xer &= !PPC32_XER_OV;

    if unlikely(tmp == 0x80000000) {
        (*cpu).xer |= PPC32_XER_OV | PPC32_XER_SO;
    }

    ppc32_exec_update_cr0(cpu, tmp);
    0
}

/// NOR
unsafe extern "C" fn ppc32_exec_NOR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[ra as usize] = !((*cpu).gpr[rs as usize] | (*cpu).gpr[rb as usize]);
    0
}

/// NOR.
unsafe extern "C" fn ppc32_exec_NOR_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = !((*cpu).gpr[rs as usize] | (*cpu).gpr[rb as usize]);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// OR
unsafe extern "C" fn ppc32_exec_OR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] | (*cpu).gpr[rb as usize];
    0
}

/// OR.
unsafe extern "C" fn ppc32_exec_OR_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = (*cpu).gpr[rs as usize] | (*cpu).gpr[rb as usize];
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// ORC - OR with Complement
unsafe extern "C" fn ppc32_exec_ORC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] | !(*cpu).gpr[rb as usize];
    0
}

/// ORC.
unsafe extern "C" fn ppc32_exec_ORC_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = (*cpu).gpr[rs as usize] | !(*cpu).gpr[rb as usize];
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// ORI - OR Immediate
unsafe extern "C" fn ppc32_exec_ORI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] | imm as m_uint32_t;
    0
}

/// ORIS - OR Immediate Shifted
unsafe extern "C" fn ppc32_exec_ORIS(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] | (imm << 16);
    0
}

/// RFI - Return From Interrupt
unsafe extern "C" fn ppc32_exec_RFI(cpu: *mut cpu_ppc_t, _insn: ppc_insn_t) -> c_int {
    if false {
        libc::printf(cstr!("RFI: srr0=0x%8.8x, srr1=0x%8.8x\n"), (*cpu).srr0, (*cpu).srr1);
    }

    (*cpu).msr &= !PPC32_RFI_MSR_MASK;
    (*cpu).msr |= (*cpu).srr1 & PPC32_RFI_MSR_MASK;

    (*cpu).msr &= !(1 << 13);
    (*cpu).ia = (*cpu).srr0 & !0x03;

    (*cpu).irq_check.set((((*cpu).msr & PPC32_MSR_EE) != 0 && (*cpu).irq_pending.get() != 0).into());

    if false {
        libc::printf(cstr!("NEW IA=0x%8.8x, NEW MSR=0x%8.8x\n"), (*cpu).ia, (*cpu).msr);
    }
    1
}

/// RLWIMI - Rotate Left Word Immediate then Mask Insert
unsafe extern "C" fn ppc32_exec_RLWIMI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    let r: m_uint32_t = ((*cpu).gpr[rs as usize] << sh) | ((*cpu).gpr[rs as usize] >> (32 - sh));
    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);
    (*cpu).gpr[ra as usize] = (r & mask) | ((*cpu).gpr[ra as usize] & !mask);
    0
}

/// RLWIMI. - Rotate Left Word Immediate then Mask Insert
unsafe extern "C" fn ppc32_exec_RLWIMI_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    let r: m_uint32_t = ((*cpu).gpr[rs as usize] << sh) | ((*cpu).gpr[rs as usize] >> (32 - sh));
    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);
    let tmp: m_uint32_t = (r & mask) | ((*cpu).gpr[ra as usize] & !mask);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// RLWINM - Rotate Left Word Immediate AND with Mask
unsafe extern "C" fn ppc32_exec_RLWINM(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    let r: m_uint32_t = ((*cpu).gpr[rs as usize] << sh) | ((*cpu).gpr[rs as usize] >> (32 - sh));
    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);
    (*cpu).gpr[ra as usize] = r & mask;
    0
}

/// RLWINM. - Rotate Left Word Immediate AND with Mask
unsafe extern "C" fn ppc32_exec_RLWINM_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    let r: m_uint32_t = ((*cpu).gpr[rs as usize] << sh) | ((*cpu).gpr[rs as usize] >> (32 - sh));
    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);
    let tmp: m_uint32_t = r & mask;
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// RLWNM - Rotate Left Word then Mask Insert
unsafe extern "C" fn ppc32_exec_RLWNM(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    let sh: m_uint32_t = (*cpu).gpr[rb as usize] & 0x1f;
    let r: m_uint32_t = ((*cpu).gpr[rs as usize] << sh) | ((*cpu).gpr[rs as usize] >> (32 - sh));
    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);
    (*cpu).gpr[ra as usize] = r & mask;
    0
}

/// RLWNM. - Rotate Left Word then Mask Insert
unsafe extern "C" fn ppc32_exec_RLWNM_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    let sh: m_uint32_t = (*cpu).gpr[rb as usize] & 0x1f;
    let r: m_uint32_t = ((*cpu).gpr[rs as usize] << sh) | ((*cpu).gpr[rs as usize] >> (32 - sh));
    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);
    let tmp: m_uint32_t = r & mask;
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// SC - System Call
unsafe extern "C" fn ppc32_exec_SC(cpu: *mut cpu_ppc_t, _insn: ppc_insn_t) -> c_int {
    ppc32_trigger_exception(cpu, PPC32_EXC_SYSCALL);
    1
}

/// SLW - Shift Left Word
unsafe extern "C" fn ppc32_exec_SLW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let s: m_uint32_t = (*cpu).gpr[rb as usize] & 0x3f;

    if likely((s & 0x20) == 0) {
        (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] << s;
    } else {
        (*cpu).gpr[ra as usize] = 0;
    }

    0
}

/// SLW.
unsafe extern "C" fn ppc32_exec_SLW_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let s: m_uint32_t = (*cpu).gpr[rb as usize] & 0x3f;

    let tmp: m_uint32_t = if likely((s & 0x20) == 0) { (*cpu).gpr[rs as usize] << s } else { 0 };

    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// SRAW - Shift Right Algebraic Word
unsafe extern "C" fn ppc32_exec_SRAW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).xer_ca = 0;

    let s: m_uint32_t = (*cpu).gpr[rs as usize];
    let sh: c_int = (*cpu).gpr[rb as usize] as c_int;

    if unlikely((sh & 0x20) != 0) {
        (*cpu).gpr[ra as usize] = (s >> 31) as m_int32_t as m_uint32_t;
        (*cpu).xer_ca = (*cpu).gpr[ra as usize] & 0x1;
        return 0;
    }

    (*cpu).gpr[ra as usize] = (s >> sh) as m_int32_t as m_uint32_t;
    let mask: m_uint32_t = !(0xFFFFFFFF << sh);

    if (s & 0x80000000) != 0 && (s & mask) != 0 {
        (*cpu).xer_ca = 1;
    }

    0
}

/// SRAWI - Shift Right Algebraic Word Immediate
unsafe extern "C" fn ppc32_exec_SRAWI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);

    (*cpu).xer_ca = 0;

    let s: m_uint32_t = (*cpu).gpr[rs as usize];
    (*cpu).gpr[ra as usize] = (s >> sh) as m_int32_t as m_uint32_t;
    let mask: m_uint32_t = !(0xFFFFFFFF << sh);

    if (s & 0x80000000) != 0 && (s & mask) != 0 {
        (*cpu).xer_ca = 1;
    }

    0
}

/// SRAWI.
unsafe extern "C" fn ppc32_exec_SRAWI_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);

    (*cpu).xer_ca = 0;

    let s: m_uint32_t = (*cpu).gpr[rs as usize];
    let r: m_uint32_t = (s >> sh) as m_int32_t as m_uint32_t;
    let mask: m_uint32_t = !(0xFFFFFFFF << sh);

    if (s & 0x80000000) != 0 && (s & mask) != 0 {
        (*cpu).xer_ca = 1;
    }

    ppc32_exec_update_cr0(cpu, r);
    (*cpu).gpr[ra as usize] = r;
    0
}

/// SRW - Shift Right Word
unsafe extern "C" fn ppc32_exec_SRW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let s: m_uint32_t = (*cpu).gpr[rb as usize] & 0x3f;

    if likely((s & 0x20) == 0) {
        (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] >> s;
    } else {
        (*cpu).gpr[ra as usize] = 0;
    }

    0
}

/// SRW.
unsafe extern "C" fn ppc32_exec_SRW_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let s: m_uint32_t = (*cpu).gpr[rb as usize] & 0x3f;

    let tmp: m_uint32_t = if likely((s & 0x20) == 0) { (*cpu).gpr[rs as usize] >> s } else { 0 };

    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// STB - Store Byte
unsafe extern "C" fn ppc32_exec_STB(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STB as c_int, vaddr, rs as u_int);
    0
}

/// STBU - Store Byte with Update
unsafe extern "C" fn ppc32_exec_STBU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_STB as c_int, vaddr, rs as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// STBUX - Store Byte with Update Indexed
unsafe extern "C" fn ppc32_exec_STBUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_STB as c_int, vaddr, rs as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// STBX - Store Byte Indexed
unsafe extern "C" fn ppc32_exec_STBX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STB as c_int, vaddr, rs as u_int);
    0
}

/// STH - Store Half-Word
unsafe extern "C" fn ppc32_exec_STH(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STH as c_int, vaddr, rs as u_int);
    0
}

/// STHU - Store Half-Word with Update
unsafe extern "C" fn ppc32_exec_STHU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_STH as c_int, vaddr, rs as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// STHUX - Store Half-Word with Update Indexed
unsafe extern "C" fn ppc32_exec_STHUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_STH as c_int, vaddr, rs as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// STHX - Store Half-Word Indexed
unsafe extern "C" fn ppc32_exec_STHX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STH as c_int, vaddr, rs as u_int);
    0
}

/// STMW - Store Multiple Word
unsafe extern "C" fn ppc32_exec_STMW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    for r in rs..=31 {
        ppc32_exec_memop(cpu, PPC_MEMOP_STW as c_int, vaddr, r as u_int);
        vaddr = vaddr.wrapping_add(size_of::<m_uint32_t>() as m_uint32_t);
    }

    0
}

/// STW - Store Word
unsafe extern "C" fn ppc32_exec_STW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STW as c_int, vaddr, rs as u_int);
    0
}

/// STWU - Store Word with Update
unsafe extern "C" fn ppc32_exec_STWU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_STW as c_int, vaddr, rs as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// STWUX - Store Word with Update Indexed
unsafe extern "C" fn ppc32_exec_STWUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_STW as c_int, vaddr, rs as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// STWX - Store Word Indexed
unsafe extern "C" fn ppc32_exec_STWX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STW as c_int, vaddr, rs as u_int);
    0
}

/// STWBRX - Store Word Byte-Reverse Indexed
unsafe extern "C" fn ppc32_exec_STWBRX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STWBR as c_int, vaddr, rs as u_int);
    0
}

/// STWCX. - Store Word Conditional Indexed
unsafe extern "C" fn ppc32_exec_STWCX_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    if (*cpu).reserve != 0 {
        ppc32_exec_memop(cpu, PPC_MEMOP_STW as c_int, vaddr, rs as u_int);

        (*cpu).cr_fields[0] = 1 << PPC32_CR_EQ_BIT;

        if ((*cpu).xer & PPC32_XER_SO) != 0 {
            (*cpu).cr_fields[0] |= 1 << PPC32_CR_SO_BIT;
        }

        (*cpu).reserve = 0;
    } else {
        (*cpu).cr_fields[0] = 0;

        if ((*cpu).xer & PPC32_XER_SO) != 0 {
            (*cpu).cr_fields[0] |= 1 << PPC32_CR_SO_BIT;
        }
    }

    0
}

/// STFD - Store Floating-Point Double
unsafe extern "C" fn ppc32_exec_STFD(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let mut vaddr: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STFD as c_int, vaddr, rs as u_int);
    0
}

/// STFDU - Store Floating-Point Double with Update
unsafe extern "C" fn ppc32_exec_STFDU(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add(sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    ppc32_exec_memop(cpu, PPC_MEMOP_STFD as c_int, vaddr, rs as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// STFDUX - Store Floating-Point Double with Update Indexed
unsafe extern "C" fn ppc32_exec_STFDUX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let vaddr: m_uint32_t = (*cpu).gpr[ra as usize].wrapping_add((*cpu).gpr[rb as usize]);
    ppc32_exec_memop(cpu, PPC_MEMOP_STFD as c_int, vaddr, rs as u_int);
    (*cpu).gpr[ra as usize] = vaddr;
    0
}

/// STFDX - Store Floating-Point Double Indexed
unsafe extern "C" fn ppc32_exec_STFDX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    ppc32_exec_memop(cpu, PPC_MEMOP_STFD as c_int, vaddr, rs as u_int);
    0
}

/// STSWI - Store String Word Immediate
unsafe extern "C" fn ppc32_exec_STSWI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let mut nb: c_int = bits(insn, 11, 15);
    let mut vaddr: m_uint32_t = 0;
    let mut r: c_int;

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    if nb == 0 {
        nb = 32;
    }

    r = rs - 1;
    (*cpu).sw_pos = 0;

    while nb > 0 {
        if (*cpu).sw_pos == 0 {
            r = (r + 1) & 0x1F;
        }

        ppc32_exec_memop(cpu, PPC_MEMOP_STSW as c_int, vaddr, r as u_int);
        (*cpu).sw_pos += 8;

        if (*cpu).sw_pos == 32 {
            (*cpu).sw_pos = 0;
        }

        vaddr = vaddr.wrapping_add(1);
        nb -= 1;
    }

    0
}

/// STSWX - Store String Word Indexed
unsafe extern "C" fn ppc32_exec_STSWX(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mut r: c_int;
    let mut nb: c_int;

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }

    nb = ((*cpu).xer & PPC32_XER_BC_MASK) as c_int;
    r = rs - 1;
    (*cpu).sw_pos = 0;

    while nb > 0 {
        if (*cpu).sw_pos == 0 {
            r = (r + 1) & 0x1F;
        }

        ppc32_exec_memop(cpu, PPC_MEMOP_STSW as c_int, vaddr, r as u_int);
        (*cpu).sw_pos += 8;

        if (*cpu).sw_pos == 32 {
            (*cpu).sw_pos = 0;
        }

        vaddr = vaddr.wrapping_add(1);
        nb -= 1;
    }

    0
}

/// SUBF - Subtract From
unsafe extern "C" fn ppc32_exec_SUBF(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[rd as usize] = (*cpu).gpr[rb as usize].wrapping_sub((*cpu).gpr[ra as usize]);
    0
}

/// SUBF.
unsafe extern "C" fn ppc32_exec_SUBF_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = (*cpu).gpr[rb as usize].wrapping_sub((*cpu).gpr[ra as usize]);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// SUBFO - Subtract From with Overflow
unsafe extern "C" fn ppc32_exec_SUBFO(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];

    let tmp: m_uint32_t = b.wrapping_sub(a);
    ppc32_exec_ov_sub(cpu, tmp, b, a);
    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// SUBFO.
unsafe extern "C" fn ppc32_exec_SUBFO_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];

    let tmp: m_uint32_t = b.wrapping_sub(a);
    ppc32_exec_ov_sub(cpu, tmp, b, a);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// SUBFC - Subtract From Carrying
unsafe extern "C" fn ppc32_exec_SUBFC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = !(*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];

    let tmp: m_uint32_t = a.wrapping_add(1);
    let r: m_uint32_t = b.wrapping_add(tmp);

    ppc32_exec_ca_sum(cpu, tmp, a, 1);
    if r < tmp {
        (*cpu).xer_ca = 1;
    }

    (*cpu).gpr[rd as usize] = r;
    0
}

/// SUBFC.
unsafe extern "C" fn ppc32_exec_SUBFC_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = !(*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];

    let tmp: m_uint32_t = a.wrapping_add(1);
    let r: m_uint32_t = b.wrapping_add(tmp);

    ppc32_exec_ca_sum(cpu, tmp, a, 1);
    if r < tmp {
        (*cpu).xer_ca = 1;
    }

    ppc32_exec_update_cr0(cpu, r);
    (*cpu).gpr[rd as usize] = r;
    0
}

/// SUBFCO - Subtract From with Overflow
unsafe extern "C" fn ppc32_exec_SUBFCO(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let tmp: m_uint32_t = b.wrapping_sub(a);

    ppc32_exec_ca_sub(cpu, tmp, b, a);
    ppc32_exec_ov_sub(cpu, tmp, b, a);
    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// SUBFCO.
unsafe extern "C" fn ppc32_exec_SUBFCO_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_uint32_t = (*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let tmp: m_uint32_t = b.wrapping_sub(a);

    ppc32_exec_ca_sub(cpu, tmp, b, a);
    ppc32_exec_ov_sub(cpu, tmp, b, a);
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[rd as usize] = tmp;
    0
}

/// SUBFE - Subtract From Carrying
unsafe extern "C" fn ppc32_exec_SUBFE(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let carry: m_uint32_t = (*cpu).xer_ca;

    let a: m_uint32_t = !(*cpu).gpr[ra as usize];
    let b: m_uint32_t = (*cpu).gpr[rb as usize];
    let tmp: m_uint32_t = a.wrapping_add(carry);
    let r: m_uint32_t = b.wrapping_add(tmp);

    ppc32_exec_ca_sum(cpu, tmp, a, carry);
    if r < tmp {
        (*cpu).xer_ca = 1;
    }

    (*cpu).gpr[rd as usize] = r;
    0
}

/// SUBFIC - Subtract From Immediate Carrying
unsafe extern "C" fn ppc32_exec_SUBFIC(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let a: m_uint32_t = !(*cpu).gpr[ra as usize];
    let b: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    let tmp: m_uint32_t = a.wrapping_add(1);
    let r: m_uint32_t = b.wrapping_add(tmp);

    ppc32_exec_ca_sum(cpu, tmp, a, 1);
    if r < tmp {
        (*cpu).xer_ca = 1;
    }

    (*cpu).gpr[rd as usize] = r;
    0
}

/// SUBFZE - Subtract From Zero extended
unsafe extern "C" fn ppc32_exec_SUBFZE(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let carry: m_uint32_t = (*cpu).xer_ca;

    let a: m_uint32_t = !(*cpu).gpr[ra as usize];
    let r: m_uint32_t = a.wrapping_add(carry);

    if r < a {
        (*cpu).xer_ca = 1;
    }

    (*cpu).gpr[rd as usize] = r;
    0
}

/// SUBFZE.
unsafe extern "C" fn ppc32_exec_SUBFZE_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    let carry: m_uint32_t = (*cpu).xer_ca;

    let a: m_uint32_t = !(*cpu).gpr[ra as usize];
    let r: m_uint32_t = a.wrapping_add(carry);

    if r < a {
        (*cpu).xer_ca = 1;
    }

    ppc32_exec_update_cr0(cpu, r);
    (*cpu).gpr[rd as usize] = r;
    0
}

/// SYNC - Synchronize
unsafe extern "C" fn ppc32_exec_SYNC(_cpu: *mut cpu_ppc_t, _insn: ppc_insn_t) -> c_int {
    0
}

/// TLBIA - TLB Invalidate All
unsafe extern "C" fn ppc32_exec_TLBIA(cpu: *mut cpu_ppc_t, _insn: ppc_insn_t) -> c_int {
    ppc32_mem_invalidate_cache(cpu);
    0
}

/// TLBIE - TLB Invalidate Entry
unsafe extern "C" fn ppc32_exec_TLBIE(cpu: *mut cpu_ppc_t, _insn: ppc_insn_t) -> c_int {
    ppc32_mem_invalidate_cache(cpu);
    0
}

/// TLBSYNC - TLB Synchronize
unsafe extern "C" fn ppc32_exec_TLBSYNC(_cpu: *mut cpu_ppc_t, _insn: ppc_insn_t) -> c_int {
    0
}

/// TW - Trap Word
unsafe extern "C" fn ppc32_exec_TW(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let to: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let a: m_int32_t = (*cpu).gpr[ra as usize] as m_int32_t;
    let b: m_int32_t = (*cpu).gpr[rb as usize] as m_int32_t;

    if ((a < b) && (to & 0x10) != 0) || ((a > b) && (to & 0x08) != 0) || ((a == b) && (to & 0x04) != 0) || (((a as m_uint32_t) < (b as m_uint32_t)) && (to & 0x02) != 0) || (((a as m_uint32_t) > (b as m_uint32_t)) && (to & 0x01) != 0) {
        ppc32_trigger_exception(cpu, PPC32_EXC_PROG);
        (*cpu).srr1 |= 1 << 17;
        return 1;
    }

    0
}

/// TWI - Trap Word Immediate
unsafe extern "C" fn ppc32_exec_TWI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let to: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    let a: m_int32_t = (*cpu).gpr[ra as usize] as m_int32_t;
    let b: m_int32_t = sign_extend(imm as m_int64_t, 16) as m_int32_t;

    if ((a < b) && (to & 0x10) != 0) || ((a > b) && (to & 0x08) != 0) || ((a == b) && (to & 0x04) != 0) || (((a as m_uint32_t) < (b as m_uint32_t)) && (to & 0x02) != 0) || (((a as m_uint32_t) > (b as m_uint32_t)) && (to & 0x01) != 0) {
        ppc32_trigger_exception(cpu, PPC32_EXC_PROG);
        (*cpu).srr1 |= 1 << 17;
        return 1;
    }

    0
}

/// XOR
unsafe extern "C" fn ppc32_exec_XOR(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] ^ (*cpu).gpr[rb as usize];
    0
}

/// XOR.
unsafe extern "C" fn ppc32_exec_XOR_dot(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let tmp: m_uint32_t = (*cpu).gpr[rs as usize] ^ (*cpu).gpr[rb as usize];
    ppc32_exec_update_cr0(cpu, tmp);
    (*cpu).gpr[ra as usize] = tmp;
    0
}

/// XORI - XOR Immediate
unsafe extern "C" fn ppc32_exec_XORI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] ^ (imm as m_uint32_t);
    0
}

/// XORIS - XOR Immediate Shifted
unsafe extern "C" fn ppc32_exec_XORIS(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;

    (*cpu).gpr[ra as usize] = (*cpu).gpr[rs as usize] ^ (imm << 16);
    0
}

/// DCCCI - Data Cache Congruence Class Invalidate (PowerPC 405)
unsafe extern "C" fn ppc32_exec_DCCCI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }
    let _ = vaddr;

    0
}

/// ICCCI - Instruction Cache Congruence Class Invalidate (PowerPC 405)
unsafe extern "C" fn ppc32_exec_ICCCI(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    let mut vaddr: m_uint32_t = (*cpu).gpr[rb as usize];

    if ra != 0 {
        vaddr = vaddr.wrapping_add((*cpu).gpr[ra as usize]);
    }
    let _ = vaddr;

    0
}

/// MFDCR - Move From Device Control Register (PowerPC 405)
unsafe extern "C" fn ppc32_exec_MFDCR(_cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let _rt: c_int = bits(insn, 21, 25);
    0
}

/// MTDCR - Move To Device Control Register (PowerPC 405)
unsafe extern "C" fn ppc32_exec_MTDCR(_cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let _rt = bits(insn, 21, 25);
    0
}

/// TLBRE - TLB Read Entry (PowerPC 405)
unsafe extern "C" fn ppc32_exec_TLBRE(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rt: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let ws: c_int = bits(insn, 11, 15);

    let index: m_uint32_t = (*cpu).gpr[ra as usize] & 0x3F;

    if ws == 1 {
        (*cpu).gpr[rt as usize] = (*cpu).ppc405_tlb[index as usize].tlb_lo;
    } else {
        (*cpu).gpr[rt as usize] = (*cpu).ppc405_tlb[index as usize].tlb_hi;
        (*cpu).ppc405_pid = (*cpu).ppc405_tlb[index as usize].tid;
    }

    0
}

/// TLBWE - TLB Write Entry (PowerPC 405)
unsafe extern "C" fn ppc32_exec_TLBWE(cpu: *mut cpu_ppc_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let ws: c_int = bits(insn, 11, 15);

    let index: m_uint32_t = (*cpu).gpr[ra as usize] & 0x3F;

    if ws == 1 {
        (*cpu).ppc405_tlb[index as usize].tlb_lo = (*cpu).gpr[rs as usize];
    } else {
        (*cpu).ppc405_tlb[index as usize].tlb_hi = (*cpu).gpr[rs as usize];
        (*cpu).ppc405_tlb[index as usize].tid = (*cpu).ppc405_pid;
    }

    0
}

/// PowerPC instruction array
static mut ppc32_exec_tags: [ppc32_insn_exec_tag; 195] = [
    ppc32_insn_exec_tag::new(cstr!("mflr"), Some(ppc32_exec_MFLR), 0xfc1fffff, 0x7c0802a6, 0),
    ppc32_insn_exec_tag::new(cstr!("mtlr"), Some(ppc32_exec_MTLR), 0xfc1fffff, 0x7c0803a6, 0),
    ppc32_insn_exec_tag::new(cstr!("mfctr"), Some(ppc32_exec_MFCTR), 0xfc1fffff, 0x7c0902a6, 0),
    ppc32_insn_exec_tag::new(cstr!("mtctr"), Some(ppc32_exec_MTCTR), 0xfc1fffff, 0x7c0903a6, 0),
    ppc32_insn_exec_tag::new(cstr!("add"), Some(ppc32_exec_ADD), 0xfc0007ff, 0x7c000214, 0),
    ppc32_insn_exec_tag::new(cstr!("add."), Some(ppc32_exec_ADD_dot), 0xfc0007ff, 0x7c000215, 0),
    ppc32_insn_exec_tag::new(cstr!("addo"), Some(ppc32_exec_ADDO), 0xfc0007ff, 0x7c000614, 0),
    ppc32_insn_exec_tag::new(cstr!("addo."), Some(ppc32_exec_ADDO_dot), 0xfc0007ff, 0x7c000615, 0),
    ppc32_insn_exec_tag::new(cstr!("addc"), Some(ppc32_exec_ADDC), 0xfc0007ff, 0x7c000014, 0),
    ppc32_insn_exec_tag::new(cstr!("addc."), Some(ppc32_exec_ADDC_dot), 0xfc0007ff, 0x7c000015, 0),
    ppc32_insn_exec_tag::new(cstr!("addco"), Some(ppc32_exec_ADDCO), 0xfc0007ff, 0x7c000414, 0),
    ppc32_insn_exec_tag::new(cstr!("addco."), Some(ppc32_exec_ADDCO_dot), 0xfc0007ff, 0x7c000415, 0),
    ppc32_insn_exec_tag::new(cstr!("adde"), Some(ppc32_exec_ADDE), 0xfc0007ff, 0x7c000114, 0),
    ppc32_insn_exec_tag::new(cstr!("adde."), Some(ppc32_exec_ADDE_dot), 0xfc0007ff, 0x7c000115, 0),
    ppc32_insn_exec_tag::new(cstr!("addeo"), Some(ppc32_exec_ADDEO), 0xfc0007ff, 0x7c000514, 0),
    ppc32_insn_exec_tag::new(cstr!("addeo."), Some(ppc32_exec_ADDEO_dot), 0xfc0007ff, 0x7c000515, 0),
    ppc32_insn_exec_tag::new(cstr!("addi"), Some(ppc32_exec_ADDI), 0xfc000000, 0x38000000, 0),
    ppc32_insn_exec_tag::new(cstr!("addic"), Some(ppc32_exec_ADDIC), 0xfc000000, 0x30000000, 0),
    ppc32_insn_exec_tag::new(cstr!("addic."), Some(ppc32_exec_ADDIC_dot), 0xfc000000, 0x34000000, 0),
    ppc32_insn_exec_tag::new(cstr!("addis"), Some(ppc32_exec_ADDIS), 0xfc000000, 0x3c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("addme"), Some(ppc32_exec_ADDME), 0xfc00ffff, 0x7c0001d4, 0),
    ppc32_insn_exec_tag::new(cstr!("addme."), Some(ppc32_exec_ADDME_dot), 0xfc00ffff, 0x7c0001d5, 0),
    ppc32_insn_exec_tag::new(cstr!("addze"), Some(ppc32_exec_ADDZE), 0xfc00ffff, 0x7c000194, 0),
    ppc32_insn_exec_tag::new(cstr!("addze."), Some(ppc32_exec_ADDZE_dot), 0xfc00ffff, 0x7c000195, 0),
    ppc32_insn_exec_tag::new(cstr!("and"), Some(ppc32_exec_AND), 0xfc0007ff, 0x7c000038, 0),
    ppc32_insn_exec_tag::new(cstr!("and."), Some(ppc32_exec_AND_dot), 0xfc0007ff, 0x7c000039, 0),
    ppc32_insn_exec_tag::new(cstr!("andc"), Some(ppc32_exec_ANDC), 0xfc0007ff, 0x7c000078, 0),
    ppc32_insn_exec_tag::new(cstr!("andc."), Some(ppc32_exec_ANDC_dot), 0xfc0007ff, 0x7c000079, 0),
    ppc32_insn_exec_tag::new(cstr!("andi."), Some(ppc32_exec_ANDI_dot), 0xfc000000, 0x70000000, 0),
    ppc32_insn_exec_tag::new(cstr!("andis."), Some(ppc32_exec_ANDIS_dot), 0xfc000000, 0x74000000, 0),
    ppc32_insn_exec_tag::new(cstr!("b"), Some(ppc32_exec_B), 0xfc000003, 0x48000000, 0),
    ppc32_insn_exec_tag::new(cstr!("ba"), Some(ppc32_exec_BA), 0xfc000003, 0x48000002, 0),
    ppc32_insn_exec_tag::new(cstr!("bl"), Some(ppc32_exec_BL), 0xfc000003, 0x48000001, 0),
    ppc32_insn_exec_tag::new(cstr!("bla"), Some(ppc32_exec_BLA), 0xfc000003, 0x48000003, 0),
    ppc32_insn_exec_tag::new(cstr!("bc"), Some(ppc32_exec_BC), 0xfc000003, 0x40000000, 0),
    ppc32_insn_exec_tag::new(cstr!("bca"), Some(ppc32_exec_BCA), 0xfc000003, 0x40000002, 0),
    ppc32_insn_exec_tag::new(cstr!("bcl"), Some(ppc32_exec_BCL), 0xfc000003, 0x40000001, 0),
    ppc32_insn_exec_tag::new(cstr!("bcla"), Some(ppc32_exec_BCLA), 0xfc000003, 0x40000003, 0),
    ppc32_insn_exec_tag::new(cstr!("bclr"), Some(ppc32_exec_BCLR), 0xfc00ffff, 0x4c000020, 0),
    ppc32_insn_exec_tag::new(cstr!("bclrl"), Some(ppc32_exec_BCLRL), 0xfc00ffff, 0x4c000021, 0),
    ppc32_insn_exec_tag::new(cstr!("bcctr"), Some(ppc32_exec_BCCTR), 0xfc00ffff, 0x4c000420, 0),
    ppc32_insn_exec_tag::new(cstr!("bcctrl"), Some(ppc32_exec_BCCTRL), 0xfc00ffff, 0x4c000421, 0),
    ppc32_insn_exec_tag::new(cstr!("cmp"), Some(ppc32_exec_CMP), 0xfc6007ff, 0x7c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("cmpi"), Some(ppc32_exec_CMPI), 0xfc600000, 0x2c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("cmpl"), Some(ppc32_exec_CMPL), 0xfc6007ff, 0x7c000040, 0),
    ppc32_insn_exec_tag::new(cstr!("cmpli"), Some(ppc32_exec_CMPLI), 0xfc600000, 0x28000000, 0),
    ppc32_insn_exec_tag::new(cstr!("cntlzw"), Some(ppc32_exec_CNTLZW), 0xfc00ffff, 0x7c000034, 0),
    ppc32_insn_exec_tag::new(cstr!("crand"), Some(ppc32_exec_CRAND), 0xfc0007ff, 0x4c000202, 0),
    ppc32_insn_exec_tag::new(cstr!("crandc"), Some(ppc32_exec_CRANDC), 0xfc0007ff, 0x4c000102, 0),
    ppc32_insn_exec_tag::new(cstr!("creqv"), Some(ppc32_exec_CREQV), 0xfc0007ff, 0x4c000242, 0),
    ppc32_insn_exec_tag::new(cstr!("crnand"), Some(ppc32_exec_CRNAND), 0xfc0007ff, 0x4c0001c2, 0),
    ppc32_insn_exec_tag::new(cstr!("crnor"), Some(ppc32_exec_CRNOR), 0xfc0007ff, 0x4c000042, 0),
    ppc32_insn_exec_tag::new(cstr!("cror"), Some(ppc32_exec_CROR), 0xfc0007ff, 0x4c000382, 0),
    ppc32_insn_exec_tag::new(cstr!("crorc"), Some(ppc32_exec_CRORC), 0xfc0007ff, 0x4c000342, 0),
    ppc32_insn_exec_tag::new(cstr!("crxor"), Some(ppc32_exec_CRXOR), 0xfc0007ff, 0x4c000182, 0),
    ppc32_insn_exec_tag::new(cstr!("dcbf"), Some(ppc32_exec_DCBF), 0xffe007ff, 0x7c0000ac, 0),
    ppc32_insn_exec_tag::new(cstr!("dcbi"), Some(ppc32_exec_DCBI), 0xffe007ff, 0x7c0003ac, 0),
    ppc32_insn_exec_tag::new(cstr!("dcbt"), Some(ppc32_exec_DCBT), 0xffe007ff, 0x7c00022c, 0),
    ppc32_insn_exec_tag::new(cstr!("dcbst"), Some(ppc32_exec_DCBST), 0xffe007ff, 0x7c00006c, 0),
    ppc32_insn_exec_tag::new(cstr!("divw"), Some(ppc32_exec_DIVW), 0xfc0007ff, 0x7c0003d6, 0),
    ppc32_insn_exec_tag::new(cstr!("divw."), Some(ppc32_exec_DIVW_dot), 0xfc0007ff, 0x7c0003d7, 0),
    ppc32_insn_exec_tag::new(cstr!("divwu"), Some(ppc32_exec_DIVWU), 0xfc0007ff, 0x7c000396, 0),
    ppc32_insn_exec_tag::new(cstr!("divwu."), Some(ppc32_exec_DIVWU_dot), 0xfc0007ff, 0x7c000397, 0),
    ppc32_insn_exec_tag::new(cstr!("eieio"), Some(ppc32_exec_EIEIO), 0xffffffff, 0x7c0006ac, 0),
    ppc32_insn_exec_tag::new(cstr!("eqv"), Some(ppc32_exec_EQV), 0xfc0007ff, 0x7c000238, 0),
    ppc32_insn_exec_tag::new(cstr!("extsb"), Some(ppc32_exec_EXTSB), 0xfc00ffff, 0x7c000774, 0),
    ppc32_insn_exec_tag::new(cstr!("extsb."), Some(ppc32_exec_EXTSB_dot), 0xfc00ffff, 0x7c000775, 0),
    ppc32_insn_exec_tag::new(cstr!("extsh"), Some(ppc32_exec_EXTSH), 0xfc00ffff, 0x7c000734, 0),
    ppc32_insn_exec_tag::new(cstr!("extsh."), Some(ppc32_exec_EXTSH_dot), 0xfc00ffff, 0x7c000735, 0),
    ppc32_insn_exec_tag::new(cstr!("icbi"), Some(ppc32_exec_ICBI), 0xffe007ff, 0x7c0007ac, 0),
    ppc32_insn_exec_tag::new(cstr!("isync"), Some(ppc32_exec_ISYNC), 0xffffffff, 0x4c00012c, 0),
    ppc32_insn_exec_tag::new(cstr!("lbz"), Some(ppc32_exec_LBZ), 0xfc000000, 0x88000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lbzu"), Some(ppc32_exec_LBZU), 0xfc000000, 0x8c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lbzux"), Some(ppc32_exec_LBZUX), 0xfc0007ff, 0x7c0000ee, 0),
    ppc32_insn_exec_tag::new(cstr!("lbzx"), Some(ppc32_exec_LBZX), 0xfc0007ff, 0x7c0000ae, 0),
    ppc32_insn_exec_tag::new(cstr!("lha"), Some(ppc32_exec_LHA), 0xfc000000, 0xa8000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lhau"), Some(ppc32_exec_LHAU), 0xfc000000, 0xac000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lhaux"), Some(ppc32_exec_LHAUX), 0xfc0007ff, 0x7c0002ee, 0),
    ppc32_insn_exec_tag::new(cstr!("lhax"), Some(ppc32_exec_LHAX), 0xfc0007ff, 0x7c0002ae, 0),
    ppc32_insn_exec_tag::new(cstr!("lhz"), Some(ppc32_exec_LHZ), 0xfc000000, 0xa0000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lhzu"), Some(ppc32_exec_LHZU), 0xfc000000, 0xa4000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lhzux"), Some(ppc32_exec_LHZUX), 0xfc0007ff, 0x7c00026e, 0),
    ppc32_insn_exec_tag::new(cstr!("lhzx"), Some(ppc32_exec_LHZX), 0xfc0007ff, 0x7c00022e, 0),
    ppc32_insn_exec_tag::new(cstr!("lmw"), Some(ppc32_exec_LMW), 0xfc000000, 0xb8000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lwbrx"), Some(ppc32_exec_LWBRX), 0xfc0007ff, 0x7c00042c, 0),
    ppc32_insn_exec_tag::new(cstr!("lwz"), Some(ppc32_exec_LWZ), 0xfc000000, 0x80000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lwzu"), Some(ppc32_exec_LWZU), 0xfc000000, 0x84000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lwzux"), Some(ppc32_exec_LWZUX), 0xfc0007ff, 0x7c00006e, 0),
    ppc32_insn_exec_tag::new(cstr!("lwzx"), Some(ppc32_exec_LWZX), 0xfc0007ff, 0x7c00002e, 0),
    ppc32_insn_exec_tag::new(cstr!("lwarx"), Some(ppc32_exec_LWARX), 0xfc0007ff, 0x7c000028, 0),
    ppc32_insn_exec_tag::new(cstr!("lfd"), Some(ppc32_exec_LFD), 0xfc000000, 0xc8000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lfdu"), Some(ppc32_exec_LFDU), 0xfc000000, 0xcc000000, 0),
    ppc32_insn_exec_tag::new(cstr!("lfdux"), Some(ppc32_exec_LFDUX), 0xfc0007ff, 0x7c0004ee, 0),
    ppc32_insn_exec_tag::new(cstr!("lfdx"), Some(ppc32_exec_LFDX), 0xfc0007ff, 0x7c0004ae, 0),
    ppc32_insn_exec_tag::new(cstr!("lswi"), Some(ppc32_exec_LSWI), 0xfc0007ff, 0x7c0004aa, 0),
    ppc32_insn_exec_tag::new(cstr!("lswx"), Some(ppc32_exec_LSWX), 0xfc0007ff, 0x7c00042a, 0),
    ppc32_insn_exec_tag::new(cstr!("mcrf"), Some(ppc32_exec_MCRF), 0xfc63ffff, 0x4c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("mfcr"), Some(ppc32_exec_MFCR), 0xfc1fffff, 0x7c000026, 0),
    ppc32_insn_exec_tag::new(cstr!("mfmsr"), Some(ppc32_exec_MFMSR), 0xfc1fffff, 0x7c0000a6, 0),
    ppc32_insn_exec_tag::new(cstr!("mfspr"), Some(ppc32_exec_MFSPR), 0xfc0007ff, 0x7c0002a6, 0),
    ppc32_insn_exec_tag::new(cstr!("mfsr"), Some(ppc32_exec_MFSR), 0xfc10ffff, 0x7c0004a6, 0),
    ppc32_insn_exec_tag::new(cstr!("mfsrin"), Some(ppc32_exec_MFSRIN), 0xfc1f07ff, 0x7c000526, 0),
    ppc32_insn_exec_tag::new(cstr!("mftbl"), Some(ppc32_exec_MFTBL), 0xfc1ff7ff, 0x7c0c42e6, 0),
    ppc32_insn_exec_tag::new(cstr!("mftbu"), Some(ppc32_exec_MFTBU), 0xfc1ff7ff, 0x7c0d42e6, 0),
    ppc32_insn_exec_tag::new(cstr!("mtcrf"), Some(ppc32_exec_MTCRF), 0xfc100fff, 0x7c000120, 0),
    ppc32_insn_exec_tag::new(cstr!("mtmsr"), Some(ppc32_exec_MTMSR), 0xfc1fffff, 0x7c000124, 0),
    ppc32_insn_exec_tag::new(cstr!("mtspr"), Some(ppc32_exec_MTSPR), 0xfc0007ff, 0x7c0003a6, 0),
    ppc32_insn_exec_tag::new(cstr!("mtsr"), Some(ppc32_exec_MTSR), 0xfc10ffff, 0x7c0001a4, 0),
    ppc32_insn_exec_tag::new(cstr!("mulhw"), Some(ppc32_exec_MULHW), 0xfc0007ff, 0x7c000096, 0),
    ppc32_insn_exec_tag::new(cstr!("mulhw."), Some(ppc32_exec_MULHW_dot), 0xfc0007ff, 0x7c000097, 0),
    ppc32_insn_exec_tag::new(cstr!("mulhwu"), Some(ppc32_exec_MULHWU), 0xfc0007ff, 0x7c000016, 0),
    ppc32_insn_exec_tag::new(cstr!("mulhwu."), Some(ppc32_exec_MULHWU_dot), 0xfc0007ff, 0x7c000017, 0),
    ppc32_insn_exec_tag::new(cstr!("mulli"), Some(ppc32_exec_MULLI), 0xfc000000, 0x1c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("mullw"), Some(ppc32_exec_MULLW), 0xfc0007ff, 0x7c0001d6, 0),
    ppc32_insn_exec_tag::new(cstr!("mullw."), Some(ppc32_exec_MULLW_dot), 0xfc0007ff, 0x7c0001d7, 0),
    ppc32_insn_exec_tag::new(cstr!("mullwo"), Some(ppc32_exec_MULLWO), 0xfc0007ff, 0x7c0005d6, 0),
    ppc32_insn_exec_tag::new(cstr!("mullwo."), Some(ppc32_exec_MULLWO_dot), 0xfc0007ff, 0x7c0005d7, 0),
    ppc32_insn_exec_tag::new(cstr!("nand"), Some(ppc32_exec_NAND), 0xfc0007ff, 0x7c0003b8, 0),
    ppc32_insn_exec_tag::new(cstr!("nand."), Some(ppc32_exec_NAND_dot), 0xfc0007ff, 0x7c0003b9, 0),
    ppc32_insn_exec_tag::new(cstr!("neg"), Some(ppc32_exec_NEG), 0xfc00ffff, 0x7c0000d0, 0),
    ppc32_insn_exec_tag::new(cstr!("neg."), Some(ppc32_exec_NEG_dot), 0xfc00ffff, 0x7c0000d1, 0),
    ppc32_insn_exec_tag::new(cstr!("nego"), Some(ppc32_exec_NEGO), 0xfc00ffff, 0x7c0004d0, 0),
    ppc32_insn_exec_tag::new(cstr!("nego."), Some(ppc32_exec_NEGO_dot), 0xfc00ffff, 0x7c0004d1, 0),
    ppc32_insn_exec_tag::new(cstr!("nor"), Some(ppc32_exec_NOR), 0xfc0007ff, 0x7c0000f8, 0),
    ppc32_insn_exec_tag::new(cstr!("nor."), Some(ppc32_exec_NOR_dot), 0xfc0007ff, 0x7c0000f9, 0),
    ppc32_insn_exec_tag::new(cstr!("or"), Some(ppc32_exec_OR), 0xfc0007ff, 0x7c000378, 0),
    ppc32_insn_exec_tag::new(cstr!("or."), Some(ppc32_exec_OR_dot), 0xfc0007ff, 0x7c000379, 0),
    ppc32_insn_exec_tag::new(cstr!("orc"), Some(ppc32_exec_ORC), 0xfc0007ff, 0x7c000338, 0),
    ppc32_insn_exec_tag::new(cstr!("orc."), Some(ppc32_exec_ORC_dot), 0xfc0007ff, 0x7c000339, 0),
    ppc32_insn_exec_tag::new(cstr!("ori"), Some(ppc32_exec_ORI), 0xfc000000, 0x60000000, 0),
    ppc32_insn_exec_tag::new(cstr!("oris"), Some(ppc32_exec_ORIS), 0xfc000000, 0x64000000, 0),
    ppc32_insn_exec_tag::new(cstr!("rfi"), Some(ppc32_exec_RFI), 0xffffffff, 0x4c000064, 0),
    ppc32_insn_exec_tag::new(cstr!("rlwimi"), Some(ppc32_exec_RLWIMI), 0xfc000001, 0x50000000, 0),
    ppc32_insn_exec_tag::new(cstr!("rlwimi."), Some(ppc32_exec_RLWIMI_dot), 0xfc000001, 0x50000001, 0),
    ppc32_insn_exec_tag::new(cstr!("rlwinm"), Some(ppc32_exec_RLWINM), 0xfc000001, 0x54000000, 0),
    ppc32_insn_exec_tag::new(cstr!("rlwinm."), Some(ppc32_exec_RLWINM_dot), 0xfc000001, 0x54000001, 0),
    ppc32_insn_exec_tag::new(cstr!("rlwnm"), Some(ppc32_exec_RLWNM), 0xfc000001, 0x5c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("rlwnm."), Some(ppc32_exec_RLWNM_dot), 0xfc000001, 0x5c000001, 0),
    ppc32_insn_exec_tag::new(cstr!("sc"), Some(ppc32_exec_SC), 0xffffffff, 0x44000002, 0),
    ppc32_insn_exec_tag::new(cstr!("slw"), Some(ppc32_exec_SLW), 0xfc0007ff, 0x7c000030, 0),
    ppc32_insn_exec_tag::new(cstr!("slw."), Some(ppc32_exec_SLW_dot), 0xfc0007ff, 0x7c000031, 0),
    ppc32_insn_exec_tag::new(cstr!("sraw"), Some(ppc32_exec_SRAW), 0xfc0007ff, 0x7c000630, 0),
    ppc32_insn_exec_tag::new(cstr!("srawi"), Some(ppc32_exec_SRAWI), 0xfc0007ff, 0x7c000670, 0),
    ppc32_insn_exec_tag::new(cstr!("srawi."), Some(ppc32_exec_SRAWI_dot), 0xfc0007ff, 0x7c000671, 0),
    ppc32_insn_exec_tag::new(cstr!("srw"), Some(ppc32_exec_SRW), 0xfc0007ff, 0x7c000430, 0),
    ppc32_insn_exec_tag::new(cstr!("srw."), Some(ppc32_exec_SRW_dot), 0xfc0007ff, 0x7c000431, 0),
    ppc32_insn_exec_tag::new(cstr!("stb"), Some(ppc32_exec_STB), 0xfc000000, 0x98000000, 0),
    ppc32_insn_exec_tag::new(cstr!("stbu"), Some(ppc32_exec_STBU), 0xfc000000, 0x9c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("stbux"), Some(ppc32_exec_STBUX), 0xfc0007ff, 0x7c0001ee, 0),
    ppc32_insn_exec_tag::new(cstr!("stbx"), Some(ppc32_exec_STBX), 0xfc0007ff, 0x7c0001ae, 0),
    ppc32_insn_exec_tag::new(cstr!("sth"), Some(ppc32_exec_STH), 0xfc000000, 0xb0000000, 0),
    ppc32_insn_exec_tag::new(cstr!("sthu"), Some(ppc32_exec_STHU), 0xfc000000, 0xb4000000, 0),
    ppc32_insn_exec_tag::new(cstr!("sthux"), Some(ppc32_exec_STHUX), 0xfc0007ff, 0x7c00036e, 0),
    ppc32_insn_exec_tag::new(cstr!("sthx"), Some(ppc32_exec_STHX), 0xfc0007ff, 0x7c00032e, 0),
    ppc32_insn_exec_tag::new(cstr!("stmw"), Some(ppc32_exec_STMW), 0xfc000000, 0xbc000000, 0),
    ppc32_insn_exec_tag::new(cstr!("stw"), Some(ppc32_exec_STW), 0xfc000000, 0x90000000, 0),
    ppc32_insn_exec_tag::new(cstr!("stwu"), Some(ppc32_exec_STWU), 0xfc000000, 0x94000000, 0),
    ppc32_insn_exec_tag::new(cstr!("stwux"), Some(ppc32_exec_STWUX), 0xfc0007ff, 0x7c00016e, 0),
    ppc32_insn_exec_tag::new(cstr!("stwx"), Some(ppc32_exec_STWX), 0xfc0007ff, 0x7c00012e, 0),
    ppc32_insn_exec_tag::new(cstr!("stwbrx"), Some(ppc32_exec_STWBRX), 0xfc0007ff, 0x7c00052c, 0),
    ppc32_insn_exec_tag::new(cstr!("stwcx."), Some(ppc32_exec_STWCX_dot), 0xfc0007ff, 0x7c00012d, 0),
    ppc32_insn_exec_tag::new(cstr!("stfd"), Some(ppc32_exec_STFD), 0xfc000000, 0xd8000000, 0),
    ppc32_insn_exec_tag::new(cstr!("stfdu"), Some(ppc32_exec_STFDU), 0xfc000000, 0xdc000000, 0),
    ppc32_insn_exec_tag::new(cstr!("stfdux"), Some(ppc32_exec_STFDUX), 0xfc0007ff, 0x7c0005ee, 0),
    ppc32_insn_exec_tag::new(cstr!("stfdx"), Some(ppc32_exec_STFDX), 0xfc0007ff, 0x7c0005ae, 0),
    ppc32_insn_exec_tag::new(cstr!("stswi"), Some(ppc32_exec_STSWI), 0xfc0007ff, 0x7c0005aa, 0),
    ppc32_insn_exec_tag::new(cstr!("stswx"), Some(ppc32_exec_STSWX), 0xfc0007ff, 0x7c00052a, 0),
    ppc32_insn_exec_tag::new(cstr!("subf"), Some(ppc32_exec_SUBF), 0xfc0007ff, 0x7c000050, 0),
    ppc32_insn_exec_tag::new(cstr!("subf."), Some(ppc32_exec_SUBF_dot), 0xfc0007ff, 0x7c000051, 0),
    ppc32_insn_exec_tag::new(cstr!("subfo"), Some(ppc32_exec_SUBFO), 0xfc0007ff, 0x7c000450, 0),
    ppc32_insn_exec_tag::new(cstr!("subfo."), Some(ppc32_exec_SUBFO_dot), 0xfc0007ff, 0x7c000451, 0),
    ppc32_insn_exec_tag::new(cstr!("subfc"), Some(ppc32_exec_SUBFC), 0xfc0007ff, 0x7c000010, 0),
    ppc32_insn_exec_tag::new(cstr!("subfc."), Some(ppc32_exec_SUBFC_dot), 0xfc0007ff, 0x7c000011, 0),
    #[cfg(if_0)]
    ppc32_insn_exec_tag::new(cstr!("subfco"), Some(ppc32_exec_SUBFCO), 0xfc0007ff, 0x7c000410, 0),
    #[cfg(if_0)]
    ppc32_insn_exec_tag::new(cstr!("subfco."), Some(ppc32_exec_SUBFCO_dot), 0xfc0007ff, 0x7c000411, 0),
    ppc32_insn_exec_tag::new(cstr!("subfe"), Some(ppc32_exec_SUBFE), 0xfc0007ff, 0x7c000110, 0),
    ppc32_insn_exec_tag::new(cstr!("subfic"), Some(ppc32_exec_SUBFIC), 0xfc000000, 0x20000000, 0),
    ppc32_insn_exec_tag::new(cstr!("subfze"), Some(ppc32_exec_SUBFZE), 0xfc00ffff, 0x7c000190, 0),
    ppc32_insn_exec_tag::new(cstr!("subfze."), Some(ppc32_exec_SUBFZE_dot), 0xfc00ffff, 0x7c000191, 0),
    ppc32_insn_exec_tag::new(cstr!("sync"), Some(ppc32_exec_SYNC), 0xffffffff, 0x7c0004ac, 0),
    ppc32_insn_exec_tag::new(cstr!("tlbia"), Some(ppc32_exec_TLBIA), 0xffffffff, 0x7c0002e4, 0),
    ppc32_insn_exec_tag::new(cstr!("tlbie"), Some(ppc32_exec_TLBIE), 0xffff07ff, 0x7c000264, 0),
    ppc32_insn_exec_tag::new(cstr!("tlbsync"), Some(ppc32_exec_TLBSYNC), 0xffffffff, 0x7c00046c, 0),
    ppc32_insn_exec_tag::new(cstr!("tw"), Some(ppc32_exec_TW), 0xfc0007ff, 0x7c000008, 0),
    ppc32_insn_exec_tag::new(cstr!("twi"), Some(ppc32_exec_TWI), 0xfc000000, 0x0c000000, 0),
    ppc32_insn_exec_tag::new(cstr!("xor"), Some(ppc32_exec_XOR), 0xfc0007ff, 0x7c000278, 0),
    ppc32_insn_exec_tag::new(cstr!("xor."), Some(ppc32_exec_XOR_dot), 0xfc0007ff, 0x7c000279, 0),
    ppc32_insn_exec_tag::new(cstr!("xori"), Some(ppc32_exec_XORI), 0xfc000000, 0x68000000, 0),
    ppc32_insn_exec_tag::new(cstr!("xoris"), Some(ppc32_exec_XORIS), 0xfc000000, 0x6c000000, 0),
    // PowerPC 405 specific instructions
    ppc32_insn_exec_tag::new(cstr!("dccci"), Some(ppc32_exec_DCCCI), 0xfc0007ff, 0x7c00038c, 0),
    ppc32_insn_exec_tag::new(cstr!("iccci"), Some(ppc32_exec_ICCCI), 0xfc0007ff, 0x7c00078c, 0),
    ppc32_insn_exec_tag::new(cstr!("mfdcr"), Some(ppc32_exec_MFDCR), 0xfc0007ff, 0x7c000286, 0),
    ppc32_insn_exec_tag::new(cstr!("mtdcr"), Some(ppc32_exec_MTDCR), 0xfc0007ff, 0x7c000386, 0),
    ppc32_insn_exec_tag::new(cstr!("tlbre"), Some(ppc32_exec_TLBRE), 0xfc0007ff, 0x7c000764, 0),
    ppc32_insn_exec_tag::new(cstr!("tlbwe"), Some(ppc32_exec_TLBWE), 0xfc0007ff, 0x7c0007a4, 0),
    // Unknown opcode fallback
    ppc32_insn_exec_tag::new(cstr!("unknown"), Some(ppc32_exec_unknown), 0x00000000, 0x00000000, 0),
    ppc32_insn_exec_tag::null(),
];
