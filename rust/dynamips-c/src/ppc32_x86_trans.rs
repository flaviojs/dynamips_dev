//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::_private::*;
use crate::dynamips_common::*;
use crate::jit_op::*;
use crate::memory::*;
use crate::ppc32::*;
use crate::ppc32_exec::*;
use crate::ppc32_jit::*;
use crate::utils::*;
use crate::x86_codegen::*;
use std::arch::asm;

#[cfg(not(all(target_arch = "x86", target_endian = "little")))]
compile_error!(r#"ppc32_x86_trans: expect all(target_arch = "x86", target_endian = "little")"#);

/// cbindgen:no-export
pub const JIT_SUPPORT: c_int = 1;

/// Manipulate bitmasks atomically
#[inline(always)]
pub unsafe fn atomic_or(v: *mut m_uint32_t, m: m_uint32_t) {
    asm! {
        "lock",
        "or {v:e}, {m:e}",
        v = inout(reg) *v,
        m = in(reg) m,
    };
}

#[inline(always)]
pub unsafe fn atomic_and(v: *mut m_uint32_t, m: m_uint32_t) {
    asm! {
         "lock",
         "and {v:e}, {m:e}",
         v = inout(reg) *v,
         m = in(reg) m,
    };
}

/// Wrappers to x86-codegen functions
#[macro_export]
macro_rules! ppc32_jit_tcb_set_patch {
    ($a:expr, $b:expr) => {
        x86_patch!($a, $b);
    };
}
pub use ppc32_jit_tcb_set_patch;
#[macro_export]
macro_rules! ppc32_jit_tcb_set_jump {
    ($a:expr, $b:expr) => {
        x86_jump_code!($a, $b);
    };
}
pub use ppc32_jit_tcb_set_jump;

/// Push epilog for an x86 instruction block
#[inline(always)]
pub unsafe fn ppc32_jit_tcb_push_epilog(ptr: *mut *mut u_char) {
    x86_ret!(&mut *ptr);
}

/// Execute JIT code
#[inline(always)]
pub unsafe fn ppc32_jit_tcb_exec(cpu: *mut cpu_ppc_t, block: *mut ppc32_jit_tcb_t) {
    let mut jit_code: insn_tblock_fptr;

    let offset: m_uint32_t = ((*cpu).ia & PPC32_MIN_PAGE_IMASK) >> 2;
    jit_code = std::mem::transmute::<*mut u_char, insn_tblock_fptr>(*(*block).jit_insn_ptr.add(offset as usize));

    if unlikely(jit_code.is_none()) {
        ppc32_jit_tcb_set_target_bit(block, (*cpu).ia);

        (*block).target_undef_cnt += 1;
        if (*block).target_undef_cnt == 16 {
            ppc32_jit_tcb_recompile(cpu, block);
            jit_code = std::mem::transmute::<*mut u_char, insn_tblock_fptr>(*(*block).jit_insn_ptr.add(offset as usize));
        } else {
            ppc32_exec_page(cpu);
            return;
        }
    }

    asm! {
        "mov edi, {cpu:e}",
        cpu = in(reg) cpu,
        // clobbers
        //out("esi") _, // XXX E0425 "cannot use register `si`: esi is used internally by LLVM and cannot be used as an operand for inline asm"
        out("edi") _,
        out("eax") _,
        out("ebx") _,
        out("ecx") _,
        out("edx") _,
    };
    jit_code.unwrap_unchecked()();
}

/// Keep the stack aligned on a 16-byte boundary for Darwin/x86 and gcc 4(.?):
/// %esp adjustment = 12 - 4*nregs + 16*k (lowest k that avoids a negative adjustment)
const STACK_ADJUST: c_int = 12;

// =======================================================================

/// Macros for CPU structure access
macro_rules! REG_OFFSET {
    ($reg:expr) => {
        OFFSET!(cpu_ppc_t, gpr) + size_of::<m_uint32_t>() as c_long * $reg as c_long
    };
}
macro_rules! MEMOP_OFFSET {
    ($op:expr) => {
        OFFSET!(cpu_ppc_t, mem_op_fn) + size_of::<ppc_memop_fn>() as c_long * $op as c_long
    };
}

/// EFLAGS to Condition Register (CR) field - signed
#[rustfmt::skip]
static eflags_to_cr_signed: [m_uint32_t; 64] = [
    0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 0x08, 0x02,
    0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 0x08, 0x02,
    0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 0x08, 0x02,
    0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 0x08, 0x02,
    0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 0x04, 0x02,
    0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 0x04, 0x02,
    0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 0x04, 0x02,
    0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 0x04, 0x02,
];

/// EFLAGS to Condition Register (CR) field - unsigned
#[rustfmt::skip]
static eflags_to_cr_unsigned: [m_uint32_t; 256] = [
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
    0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08,
];

/// Load a 32 bit immediate value
#[inline(always)]
unsafe fn ppc32_load_imm(ptr: *mut *mut u_char, reg: u_int, val: m_uint32_t) {
    if val != 0 {
        x86_mov_reg_imm!(&mut *ptr, reg, val);
    } else {
        x86_alu_reg_reg!(&mut *ptr, X86_XOR, reg, reg);
    }
}

/// Set the Instruction Address (IA) register
#[no_mangle]
pub unsafe extern "C" fn ppc32_set_ia(ptr: *mut *mut u_char, new_ia: m_uint32_t) {
    x86_mov_membase_imm!(&mut *ptr, X86_EDI, OFFSET!(cpu_ppc_t, ia), new_ia, 4);
}

/// Set the Link Register (LR)
unsafe fn ppc32_set_lr(iop: *mut jit_op_t, new_lr: m_uint32_t) {
    x86_mov_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, lr), new_lr, 4);
}

/// Try to branch directly to the specified JIT block without returning to
// main loop.
unsafe fn ppc32_try_direct_far_jump(cpu: *mut cpu_ppc_t, iop: *mut jit_op_t, new_ia: m_uint32_t) {
    let ia_hash: m_uint32_t;
    let test3: *mut u_char;
    #[cfg(feature = "USE_UNSTABLE")]
    let test4: *mut u_char;

    // Indicate that we throw %esi, %edx
    ppc32_op_emit_alter_host_reg(cpu, X86_ESI);
    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    let new_page: m_uint32_t = new_ia & PPC32_MIN_PAGE_MASK;
    let ia_offset: m_uint32_t = (new_ia & PPC32_MIN_PAGE_IMASK) >> 2;
    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        ia_hash = ppc32_jit_get_ia_hash(new_ia);
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        ia_hash = ppc32_jit_get_virt_hash(new_ia);
    }

    // Get JIT block info in %edx
    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_EBX, X86_EDI, OFFSET!(cpu_ppc_t, exec_blk_map), 4);
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_EBX, X86_EDI, OFFSET!(cpu_ppc_t, tcb_virt_hash), 4);
    }
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_EDX, X86_EBX, ia_hash * size_of::<*mut c_void>() as m_uint32_t, 4);

    // no JIT block found ?
    x86_test_reg_reg!(&mut (*iop).ob_ptr, X86_EDX, X86_EDX);
    let test1: *mut u_char = (*iop).ob_ptr;
    x86_branch8!(&mut (*iop).ob_ptr, X86_CC_Z, 0, 1);

    // Check block IA
    x86_mov_reg_imm!(&mut (*iop).ob_ptr, X86_ESI, new_page);
    x86_alu_reg_membase!(&mut (*iop).ob_ptr, X86_CMP, X86_ESI, X86_EDX, OFFSET!(ppc32_jit_tcb_t, start_ia));
    let test2: *mut u_char = (*iop).ob_ptr;
    x86_branch8!(&mut (*iop).ob_ptr, X86_CC_NE, 0, 1);

    // Jump to the code
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_ESI, X86_EDX, OFFSET!(ppc32_jit_tcb_t, jit_insn_ptr), 4);
    #[cfg(feature = "USE_UNSTABLE")]
    {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, X86_ESI, X86_ESI);
        test3 = (*iop).ob_ptr;
        x86_branch8!(&mut (*iop).ob_ptr, X86_CC_Z, 0, 1);
    }
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_EBX, X86_ESI, ia_offset * size_of::<*mut c_void>() as m_uint32_t, 4);

    x86_test_reg_reg!(&mut (*iop).ob_ptr, X86_EBX, X86_EBX);
    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        test3 = (*iop).ob_ptr;
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        test4 = (*iop).ob_ptr;
    }
    x86_branch8!(&mut (*iop).ob_ptr, X86_CC_Z, 0, 1);
    x86_jump_reg!(&mut (*iop).ob_ptr, X86_EBX);

    // Returns to caller...
    x86_patch!(test1, (*iop).ob_ptr);
    x86_patch!(test2, (*iop).ob_ptr);
    x86_patch!(test3, (*iop).ob_ptr);
    #[cfg(feature = "USE_UNSTABLE")]
    {
        x86_patch!(test4, (*iop).ob_ptr);
    }

    ppc32_set_ia(addr_of_mut!((*iop).ob_ptr), new_ia);
    ppc32_jit_tcb_push_epilog(addr_of_mut!((*iop).ob_ptr));
}

/// Set Jump
unsafe fn ppc32_set_jump(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, iop: *mut jit_op_t, new_ia: m_uint32_t, _local_jump: c_int) {
    let return_to_caller: c_int = FALSE;
    let mut jump_ptr: *mut u_char = null_mut();

    #[cfg(if_0)]
    if false {
        #[allow(clippy::collapsible_if)]
        if (*cpu).sym_trace != 0 && local_jump == 0 {
            return_to_caller = TRUE;
        }
    }

    if return_to_caller == 0 && ppc32_jit_tcb_local_addr(b, new_ia, addr_of_mut!(jump_ptr)) != 0 {
        ppc32_jit_tcb_record_patch(b, iop, (*iop).ob_ptr, new_ia);
        x86_jump32!(&mut (*iop).ob_ptr, 0);
    } else {
        #[allow(clippy::collapsible_if)]
        if (*cpu).exec_blk_direct_jump != 0 {
            // Block lookup optimization
            ppc32_try_direct_far_jump(cpu, iop, new_ia);
        } else {
            ppc32_set_ia(addr_of_mut!((*iop).ob_ptr), new_ia);
            ppc32_jit_tcb_push_epilog(addr_of_mut!((*iop).ob_ptr));
        }
    }
}

/// Jump to the next page
#[no_mangle]
pub unsafe extern "C" fn ppc32_set_page_jump(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t) {
    let mut op_list: *mut jit_op_t = null_mut();

    (*(*cpu).gen).jit_op_current = addr_of_mut!(op_list);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 4, cstr!("set_page_jump"));
    ppc32_set_jump(cpu, b, iop, (*b).start_ia + PPC32_MIN_PAGE_SIZE as m_uint32_t, FALSE);
    ppc32_op_insn_output(b, iop);

    jit_op_free_list((*cpu).gen, op_list);
    (*(*cpu).gen).jit_op_current = null_mut();
}

/// Load a GPR into the specified host register
#[inline(always)]
unsafe fn ppc32_load_gpr(ptr: *mut *mut u_char, host_reg: u_int, ppc_reg: u_int) {
    x86_mov_reg_membase!(&mut *ptr, host_reg, X86_EDI, REG_OFFSET!(ppc_reg), 4);
}

/// Store contents for a host register into a GPR register
#[inline(always)]
unsafe fn ppc32_store_gpr(ptr: *mut *mut u_char, ppc_reg: u_int, host_reg: u_int) {
    x86_mov_membase_reg!(&mut *ptr, X86_EDI, REG_OFFSET!(ppc_reg), host_reg, 4);
}

/// Apply an ALU operation on a GPR register and a host register
#[inline(always)]
unsafe fn ppc32_alu_gpr(ptr: *mut *mut u_char, op: u_int, host_reg: u_int, ppc_reg: u_int) {
    x86_alu_reg_membase!(&mut *ptr, op, host_reg, X86_EDI, REG_OFFSET!(ppc_reg));
}

/// Update CR from %eflags
// %eax, %edx, %esi are modified.
unsafe fn ppc32_update_cr(b: *mut ppc32_jit_tcb_t, field: c_int, is_signed: c_int) {
    // Get status bits from EFLAGS
    if is_signed == 0 {
        x86_mov_reg_imm!(&mut (*b).jit_ptr, X86_EAX, 0);
        x86_lahf!(&mut (*b).jit_ptr);
        x86_xchg_ah_al!(&mut (*b).jit_ptr);

        x86_mov_reg_imm!(&mut (*b).jit_ptr, X86_EDX, eflags_to_cr_unsigned.as_ptr());
    } else {
        x86_pushfd!(&mut (*b).jit_ptr);
        x86_pop_reg!(&mut (*b).jit_ptr, X86_EAX);
        x86_shift_reg_imm!(&mut (*b).jit_ptr, X86_SHR, X86_EAX, 6);
        x86_alu_reg_imm!(&mut (*b).jit_ptr, X86_AND, X86_EAX, 0x3F);

        x86_mov_reg_imm!(&mut (*b).jit_ptr, X86_EDX, eflags_to_cr_signed.as_ptr());
    }

    x86_mov_reg_memindex!(&mut (*b).jit_ptr, X86_EAX, X86_EDX, 0, X86_EAX, 2, 4);

    if false {
        // Check XER Summary of Overflow and report it
        x86_mov_reg_membase!(&mut (*b).jit_ptr, X86_EDX, X86_EDI, OFFSET!(cpu_ppc_t, xer), 4);
        x86_alu_reg_imm!(&mut (*b).jit_ptr, X86_AND, X86_ESI, PPC32_XER_SO);
        x86_shift_reg_imm!(&mut (*b).jit_ptr, X86_SHR, X86_ESI, PPC32_XER_SO_BIT);
        x86_alu_reg_reg!(&mut (*b).jit_ptr, X86_OR, X86_EAX, X86_ESI);
    }

    // Store modified CR field
    x86_mov_membase_reg!(&mut (*b).jit_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(field), X86_EAX, 4);
}

/// Update CR0 from %eflags
/// %eax, %edx, %esi are modified.
unsafe fn ppc32_update_cr0(b: *mut ppc32_jit_tcb_t) {
    ppc32_update_cr(b, 0, TRUE);
}

/// Indicate registers modified by ppc32_update_cr() functions
#[no_mangle]
pub unsafe extern "C" fn ppc32_update_cr_set_altered_hreg(cpu: *mut cpu_ppc_t) {
    // Throw %eax and %edx, which are modifed by ppc32_update_cr()
    ppc32_op_emit_alter_host_reg(cpu, X86_EAX);
    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);
}

/// Basic C call
#[inline(always)]
unsafe fn ppc32_emit_basic_c_call(ptr: *mut *mut u_char, f: *mut c_void) {
    x86_mov_reg_imm!(&mut *ptr, X86_EBX, f);
    x86_call_reg!(&mut *ptr, X86_EBX);
}

/// Emit a simple call to a C function without any parameter
unsafe fn ppc32_emit_c_call(b: *mut ppc32_jit_tcb_t, iop: *mut jit_op_t, f: *mut c_void) {
    ppc32_set_ia(addr_of_mut!((*iop).ob_ptr), (*b).start_ia + ((*b).ppc_trans_pos << 2));
    ppc32_emit_basic_c_call(addr_of_mut!((*iop).ob_ptr), f);
}

// ========================================================================

/// Initialize register mapping
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_init_hreg_mapping(cpu: *mut cpu_ppc_t) {
    let avail_hregs: [c_int; 5] = [X86_ESI, X86_EAX, X86_ECX, X86_EDX, -1];
    let mut map: *mut hreg_map;
    let mut i: c_int;
    let mut hreg: c_int;

    (*cpu).hreg_map_list = null_mut();
    (*cpu).hreg_lru = null_mut();

    // Add the available registers to the map list
    i = 0;
    while avail_hregs[i as usize] != -1 {
        hreg = avail_hregs[i as usize];
        map = addr_of_mut!((*cpu).hreg_map[hreg as usize]);

        // Initialize mapping. At the beginning, no PPC reg is mapped
        (*map).flags = 0;
        (*map).hreg = hreg;
        (*map).vreg = -1;
        ppc32_jit_insert_hreg_mru(cpu, map);
        i += 1;
    }

    // Clear PPC registers mapping
    for i in 0..PPC32_GPR_NR as c_int {
        (*cpu).ppc_reg_map[i as usize] = -1;
    }
}

/// Allocate a specific temp register
unsafe fn ppc32_jit_get_tmp_hreg(_cpu: *mut cpu_ppc_t) -> c_int {
    X86_EBX
}

// ========================================================================
// JIT operations (specific to target CPU).
// ========================================================================

/// INSN_OUTPUT
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_insn_output(b: *mut ppc32_jit_tcb_t, op: *mut jit_op_t) {
    (*op).ob_final = (*b).jit_ptr;
    libc::memcpy((*b).jit_ptr.cast::<_>(), (*op).ob_data.as_ptr().cast::<_>(), (*op).ob_ptr as usize - (*op).ob_data.as_ptr() as usize);
    (*b).jit_ptr = (*b).jit_ptr.add((*op).ob_ptr as usize - (*op).ob_data.as_ptr() as usize);

    if ((*op).ob_ptr as usize - (*op).ob_data.as_ptr() as usize) >= jit_op_blk_sizes[(*op).ob_size_index as usize] as usize {
        libc::printf(cstr!("ppc32_op_insn_output: FAILURE: count=%d, size=%d\n"), ((*op).ob_ptr as usize - (*op).ob_data.as_ptr() as usize) as c_int, jit_op_blk_sizes[(*op).ob_size_index as usize]);
    }
}

/// LOAD_GPR: p[0] = %host_reg, p[1] = %ppc_reg
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_load_gpr(b: *mut ppc32_jit_tcb_t, op: *mut jit_op_t) {
    if (*op).param[0] != JIT_OP_INV_REG {
        ppc32_load_gpr(addr_of_mut!((*b).jit_ptr), (*op).param[0] as u_int, (*op).param[1] as u_int);
    }
}

/// STORE_GPR: p[0] = %host_reg, p[1] = %ppc_reg
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_store_gpr(b: *mut ppc32_jit_tcb_t, op: *mut jit_op_t) {
    if (*op).param[0] != JIT_OP_INV_REG {
        ppc32_store_gpr(addr_of_mut!((*b).jit_ptr), (*op).param[1] as u_int, (*op).param[0] as u_int);
    }
}

/// UPDATE_FLAGS: p[0] = cr_field, p[1] = is_signed
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_update_flags(b: *mut ppc32_jit_tcb_t, op: *mut jit_op_t) {
    if (*op).param[0] != JIT_OP_INV_REG {
        ppc32_update_cr(b, (*op).param[0], (*op).param[1]);
    }
}

/// MOVE_HOST_REG: p[0] = %host_dst_reg, p[1] = %host_src_reg
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_move_host_reg(b: *mut ppc32_jit_tcb_t, op: *mut jit_op_t) {
    if (*op).param[0] != JIT_OP_INV_REG && (*op).param[1] != JIT_OP_INV_REG {
        x86_mov_reg_reg!(&mut (*b).jit_ptr, (*op).param[0], (*op).param[1], 4);
    }
}

/// SET_HOST_REG_IMM32: p[0] = %host_reg, p[1] = imm32
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_set_host_reg_imm32(b: *mut ppc32_jit_tcb_t, op: *mut jit_op_t) {
    if (*op).param[0] != JIT_OP_INV_REG {
        ppc32_load_imm(addr_of_mut!((*b).jit_ptr), (*op).param[0] as u_int, (*op).param[1] as u_int);
    }
}

// ========================================================================

/// Memory operation
unsafe fn ppc32_emit_memop(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, op: c_int, base: c_int, offset: c_int, target: c_int, update: c_int) {
    let val: m_uint32_t = sign_extend(offset as m_int64_t, 16) as m_uint32_t;

    // Since an exception can be triggered, clear JIT state. This allows
    // to use branch target tag (we can directly branch on this instruction).
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_BRANCH_TARGET);
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 5, cstr!("memop"));

    // Save PC for exception handling
    ppc32_set_ia(addr_of_mut!((*iop).ob_ptr), (*b).start_ia + ((*b).ppc_trans_pos << 2));

    // EDX = sign-extended offset
    ppc32_load_imm(addr_of_mut!((*iop).ob_ptr), X86_EDX as u_int, val);

    // EDX = GPR[base] + sign-extended offset
    if update != 0 || (base != 0) {
        ppc32_alu_gpr(addr_of_mut!((*iop).ob_ptr), X86_ADD as u_int, X86_EDX as u_int, base as u_int);
    }

    if update != 0 {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_ESI, X86_EDX, 4);
    }

    // ECX = target register
    x86_mov_reg_imm!(&mut (*iop).ob_ptr, X86_ECX, target);

    // EAX = CPU instance pointer
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EDI, 4);

    // Call memory function
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_SUB, X86_ESP, STACK_ADJUST - 12);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_ECX);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EDX);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EAX);
    x86_call_membase!(&mut (*iop).ob_ptr, X86_EDI, MEMOP_OFFSET!(op));
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, X86_ESP, STACK_ADJUST);

    if update != 0 {
        ppc32_store_gpr(addr_of_mut!((*iop).ob_ptr), base as u_int, X86_ESI as u_int);
    }
}

/// Memory operation (indexed)
unsafe fn ppc32_emit_memop_idx(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, op: c_int, ra: c_int, rb: c_int, target: c_int, update: c_int) {
    // Since an exception can be triggered, clear JIT state. This allows
    // to use branch target tag (we can directly branch on this instruction).
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_BRANCH_TARGET);
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 5, cstr!("memop_idx"));

    // Save PC for exception handling
    ppc32_set_ia(addr_of_mut!((*iop).ob_ptr), (*b).start_ia + ((*b).ppc_trans_pos << 2));

    // EDX = $rb
    ppc32_load_gpr(addr_of_mut!((*iop).ob_ptr), X86_EDX as u_int, rb as u_int);

    // EDX = $rb + $ra
    if update != 0 || (ra != 0) {
        ppc32_alu_gpr(addr_of_mut!((*iop).ob_ptr), X86_ADD as u_int, X86_EDX as u_int, ra as u_int);
    }

    if update != 0 {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_ESI, X86_EDX, 4);
    }

    // ECX = target register
    x86_mov_reg_imm!(&mut (*iop).ob_ptr, X86_ECX, target);

    // EAX = CPU instance pointer
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EDI, 4);

    // Call memory function
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_SUB, X86_ESP, STACK_ADJUST - 12);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_ECX);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EDX);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EAX);
    x86_call_membase!(&mut (*iop).ob_ptr, X86_EDI, MEMOP_OFFSET!(op));
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, X86_ESP, STACK_ADJUST);

    if update != 0 {
        ppc32_store_gpr(addr_of_mut!((*iop).ob_ptr), ra as u_int, X86_ESI as u_int);
    }
}

type memop_fast_access = Option<unsafe fn(iop: *mut jit_op_t, target: c_int)>;

/// Fast LBZ
unsafe fn ppc32_memop_fast_lbz(iop: *mut jit_op_t, target: c_int) {
    x86_clear_reg!(&mut (*iop).ob_ptr, X86_ECX);
    x86_mov_reg_memindex!(&mut (*iop).ob_ptr, X86_ECX, X86_EAX, 0, X86_EBX, 0, 1);
    ppc32_store_gpr(addr_of_mut!((*iop).ob_ptr), target as u_int, X86_ECX as u_int);
}

/// Fast STB
unsafe fn ppc32_memop_fast_stb(iop: *mut jit_op_t, target: c_int) {
    ppc32_load_gpr(addr_of_mut!((*iop).ob_ptr), X86_EDX as u_int, target as u_int);
    x86_mov_memindex_reg!(&mut (*iop).ob_ptr, X86_EAX, 0, X86_EBX, 0, X86_EDX, 1);
}

/// Fast LWZ
unsafe fn ppc32_memop_fast_lwz(iop: *mut jit_op_t, target: c_int) {
    x86_mov_reg_memindex!(&mut (*iop).ob_ptr, X86_EAX, X86_EAX, 0, X86_EBX, 0, 4);
    x86_bswap!(&mut (*iop).ob_ptr, X86_EAX);
    ppc32_store_gpr(addr_of_mut!((*iop).ob_ptr), target as u_int, X86_EAX as u_int);
}

/// Fast STW
unsafe fn ppc32_memop_fast_stw(iop: *mut jit_op_t, target: c_int) {
    ppc32_load_gpr(addr_of_mut!((*iop).ob_ptr), X86_EDX as u_int, target as u_int);
    x86_bswap!(&mut (*iop).ob_ptr, X86_EDX);
    x86_mov_memindex_reg!(&mut (*iop).ob_ptr, X86_EAX, 0, X86_EBX, 0, X86_EDX, 4);
}

/// Fast memory operation
#[allow(clippy::too_many_arguments)]
unsafe fn ppc32_emit_memop_fast(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, write_op: c_int, opcode: c_int, base: c_int, offset: c_int, target: c_int, op_handler: memop_fast_access) {
    let val: m_uint32_t = sign_extend(offset as m_int64_t, 16) as m_uint32_t;
    let mut test2: *mut u_char;
    #[cfg(not(feature = "USE_UNSTABLE"))]
    let mut p_fast_exit: *mut u_char = null_mut();

    // Since an exception can be triggered, clear JIT state. This allows
    // to use branch target tag (we can directly branch on this instruction).
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_BRANCH_TARGET);
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 5, cstr!("memop_fast"));

    test2 = null_mut();

    if val != 0 {
        // EBX = sign-extended offset
        ppc32_load_imm(addr_of_mut!((*iop).ob_ptr), X86_EBX as u_int, val);

        // EBX = GPR[base] + sign-extended offset
        if base != 0 {
            ppc32_alu_gpr(addr_of_mut!((*iop).ob_ptr), X86_ADD as u_int, X86_EBX as u_int, base as u_int);
        }
    } else {
        #[allow(clippy::collapsible_if)]
        if base != 0 {
            ppc32_load_gpr(addr_of_mut!((*iop).ob_ptr), X86_EBX as u_int, base as u_int);
        } else {
            ppc32_load_imm(addr_of_mut!((*iop).ob_ptr), X86_EBX as u_int, 0);
        }
    }

    #[cfg(not(feature = "USE_UNSTABLE"))]
    if false {
        // ======= zzz =======
        let testZ: *mut u_char;

        x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_ESI, X86_EBX, 4);
        x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, X86_ESI, PPC32_MIN_PAGE_MASK);
        x86_alu_reg_membase!(&mut (*iop).ob_ptr, X86_CMP, X86_ESI, X86_EDI, OFFSET!(cpu_ppc_t, vtlb) + size_of::<ppc32_vtlb_entry>() as c_long * base as c_long + OFFSET!(ppc32_vtlb_entry, vaddr));
        testZ = (*iop).ob_ptr;
        x86_branch8!(&mut (*iop).ob_ptr, X86_CC_NZ, 0, 1);

        x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EBX, PPC32_MIN_PAGE_IMASK);
        x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_EAX, X86_EDI, OFFSET!(cpu_ppc_t, vtlb) + size_of::<ppc32_vtlb_entry>() as c_long * base as c_long + OFFSET!(ppc32_vtlb_entry, haddr), 4);

        // Memory access
        op_handler.unwrap_unchecked()(iop, target);

        p_fast_exit = (*iop).ob_ptr;
        x86_jump8!(&mut (*iop).ob_ptr, 0);

        x86_patch!(testZ, (*iop).ob_ptr);
    }

    // EAX = mts32_entry index
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EBX, 4);
    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHR, X86_EAX, MTS32_HASH_SHIFT);
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_ESI, X86_EBX, 4);

        x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHR, X86_EAX, MTS32_HASH_SHIFT1);
        x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHR, X86_ESI, MTS32_HASH_SHIFT2);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, X86_EAX, X86_ESI);
    }
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EAX, MTS32_HASH_MASK);

    // EDX = mts32_entry
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_EDX, X86_EDI, OFFSET!(cpu_ppc_t, mts_cache) * size_of::<*mut mts32_entry_t>() as c_long * PPC32_MTS_DCACHE as c_long, 4);
    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, X86_EAX, 4);
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, X86_EDX, X86_EAX);

    // Compare virtual page address (ESI = vpage)
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_ESI, X86_EBX, 4);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, X86_ESI, PPC32_MIN_PAGE_MASK);

    x86_alu_reg_membase!(&mut (*iop).ob_ptr, X86_CMP, X86_ESI, X86_EDX, OFFSET!(mts32_entry_t, gvpa));
    let test1: *mut u_char = (*iop).ob_ptr;
    x86_branch8!(&mut (*iop).ob_ptr, X86_CC_NZ, 0, 1);

    // Test if we are writing to a COW page
    if write_op != 0 {
        x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDX, OFFSET!(mts32_entry_t, flags), MTS_FLAG_COW | MTS_FLAG_EXEC);
        test2 = (*iop).ob_ptr;
        x86_branch8!(&mut (*iop).ob_ptr, X86_CC_NZ, 0, 1);
    }

    // EBX = offset in page, EAX = Host Page Address
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EBX, PPC32_MIN_PAGE_IMASK);
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_EAX, X86_EDX, OFFSET!(mts32_entry_t, hpa), 4);

    #[cfg(not(feature = "USE_UNSTABLE"))]
    if false {
        // zzz
        x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, vtlb) + size_of::<ppc32_vtlb_entry>() as c_long * base as c_long + OFFSET!(ppc32_vtlb_entry, vaddr), X86_ESI, 4);
        x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, vtlb) + size_of::<ppc32_vtlb_entry>() as c_long * base as c_long + OFFSET!(ppc32_vtlb_entry, haddr), X86_EAX, 4);
    }

    // Memory access
    op_handler.unwrap_unchecked()(iop, target);

    let p_exit: *mut u_char = (*iop).ob_ptr;
    x86_jump8!(&mut (*iop).ob_ptr, 0);

    // === Slow lookup ===
    x86_patch!(test1, (*iop).ob_ptr);
    if !test2.is_null() {
        x86_patch!(test2, (*iop).ob_ptr);
    }

    // Update IA (EBX = vaddr)
    ppc32_set_ia(addr_of_mut!((*iop).ob_ptr), (*b).start_ia + ((*b).ppc_trans_pos << 2));

    // EDX = virtual address
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_EDX, X86_EBX, 4);

    // ECX = target register
    x86_mov_reg_imm!(&mut (*iop).ob_ptr, X86_ECX, target);

    // EAX = CPU instance pointer
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EDI, 4);

    // Call memory function
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_SUB, X86_ESP, STACK_ADJUST - 12);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_ECX);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EDX);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EAX);
    x86_call_membase!(&mut (*iop).ob_ptr, X86_EDI, MEMOP_OFFSET!(opcode));
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, X86_ESP, STACK_ADJUST);

    x86_patch!(p_exit, (*iop).ob_ptr);

    #[cfg(not(feature = "USE_UNSTABLE"))]
    if false {
        // zzz
        x86_patch!(p_fast_exit, (*iop).ob_ptr);
    }
}

/// Emit unhandled instruction code
unsafe extern "C" fn ppc32_emit_unknown(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, opcode: ppc_insn_t) -> c_int {
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("unknown"));

    // Update IA
    ppc32_set_ia(addr_of_mut!((*iop).ob_ptr), (*b).start_ia + ((*b).ppc_trans_pos << 2));

    // Fallback to non-JIT mode
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EDI, 4);
    x86_mov_reg_imm!(&mut (*iop).ob_ptr, X86_EDX, opcode);

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_SUB, X86_ESP, STACK_ADJUST - 8);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EDX);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EAX);
    ppc32_emit_basic_c_call(addr_of_mut!((*iop).ob_ptr), ppc32_exec_single_insn_ext as *mut c_void);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, X86_ESP, STACK_ADJUST);

    x86_test_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EAX);
    let test1: *mut u_char = (*iop).ob_ptr;
    x86_branch8!(&mut (*iop).ob_ptr, X86_CC_Z, 0, 1);
    ppc32_jit_tcb_push_epilog(addr_of_mut!((*iop).ob_ptr));

    x86_patch!(test1, (*iop).ob_ptr);

    // Signal this as an EOB to reset JIT state
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
    0
}

/// Virtual Breakpoint
#[no_mangle]
pub unsafe extern "C" fn ppc32_emit_breakpoint(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t) {
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("breakpoint"));

    x86_mov_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EDI, 4);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_SUB, X86_ESP, STACK_ADJUST - 4);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EAX);
    ppc32_emit_c_call(b, iop, ppc32_run_breakpoint as *mut c_void);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, X86_ESP, STACK_ADJUST);

    // Signal this as an EOB to to reset JIT state
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
}

/// Dump regs
unsafe fn ppc32_emit_dump_regs(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t) {
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("dump_regs"));

    x86_mov_reg_membase!(&mut (*iop).ob_ptr, X86_EAX, X86_EDI, OFFSET!(cpu_ppc_t, gen), 4);

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_SUB, X86_ESP, STACK_ADJUST - 4);
    x86_push_reg!(&mut (*iop).ob_ptr, X86_EAX);
    ppc32_emit_c_call(b, iop, ppc32_dump_regs as *mut c_void);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, X86_ESP, STACK_ADJUST);

    // Signal this as an EOB to to reset JIT state
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
}

/// Increment the number of executed instructions (performance debugging)
unsafe fn ppc32_inc_perf_counter(cpu: *mut cpu_ppc_t) {
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("perf_cnt"));
    x86_inc_membase!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, perf_counter));
}

// ========================================================================

/// BLR - Branch to Link Register
unsafe extern "C" fn ppc32_emit_BLR(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    ppc32_jit_start_hreg_seq(cpu, cstr!("blr"));
    let hreg: c_int = ppc32_jit_alloc_hreg(cpu, -1);
    ppc32_op_emit_alter_host_reg(cpu, hreg);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("blr"));

    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg, X86_EDI, OFFSET!(cpu_ppc_t, lr), 4);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, ia), hreg, 4);

    // set the return address
    if (insn & 1) != 0 {
        ppc32_set_lr(iop, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    }

    ppc32_jit_tcb_push_epilog(addr_of_mut!((*iop).ob_ptr));
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
    ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// BCTR - Branch to Count Register
unsafe extern "C" fn ppc32_emit_BCTR(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    ppc32_jit_start_hreg_seq(cpu, cstr!("bctr"));
    let hreg: c_int = ppc32_jit_alloc_hreg(cpu, -1);
    ppc32_op_emit_alter_host_reg(cpu, hreg);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("bctr"));

    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg, X86_EDI, OFFSET!(cpu_ppc_t, ctr), 4);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, ia), hreg, 4);

    // set the return address
    if (insn & 1) != 0 {
        ppc32_set_lr(iop, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    }

    ppc32_jit_tcb_push_epilog(addr_of_mut!((*iop).ob_ptr));
    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
    ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MFLR - Move From Link Register
unsafe extern "C" fn ppc32_emit_MFLR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mflr"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("mflr"));

    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_rd, X86_EDI, OFFSET!(cpu_ppc_t, lr), 4);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MTLR - Move To Link Register
unsafe extern "C" fn ppc32_emit_MTLR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mtlr"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("mtlr"));
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, lr), hreg_rs, 4);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MFCTR - Move From Counter Register
unsafe extern "C" fn ppc32_emit_MFCTR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mfctr"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("mfctr"));

    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_rd, X86_EDI, OFFSET!(cpu_ppc_t, ctr), 4);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MTCTR - Move To Counter Register
unsafe extern "C" fn ppc32_emit_MTCTR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mtctr"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("mtctr"));
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, ctr), hreg_rs, 4);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MFTBU - Move from Time Base (Up)
unsafe extern "C" fn ppc32_emit_MFTBU(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mftbu"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("mftbu"));

    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_rd, X86_EDI, OFFSET!(cpu_ppc_t, tb) + 4, 4);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

const PPC32_TB_INCREMENT: c_int = 50;

/// MFTBL - Move from Time Base (Lo)
unsafe extern "C" fn ppc32_emit_MFTBL(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mftbl"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("mftbl"));

    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_rd, X86_EDI, OFFSET!(cpu_ppc_t, tb), 4);

    // Increment the time base register
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_rd, X86_EDI, OFFSET!(cpu_ppc_t, tb), 4);
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_t0, X86_EDI, OFFSET!(cpu_ppc_t, tb) + 4, 4);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, PPC32_TB_INCREMENT);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADC, hreg_t0, 0);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, tb), hreg_rd, 4);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, tb) + 4, hreg_t0, 4);

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ADD
unsafe extern "C" fn ppc32_emit_ADD(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $rd = $ra + $rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("add"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("add"));

    if rd == ra {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, hreg_rb);
    } else if rd == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, hreg_ra);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_ra, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, hreg_rb);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ADDC
unsafe extern "C" fn ppc32_emit_ADDC(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $rd = $ra + $rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("addc"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    // store the carry flag
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("addc"));

    if rd == ra {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, hreg_rb);
    } else if rd == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, hreg_ra);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_ra, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, hreg_rb);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t0, FALSE);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x1);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t0, 4);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_rd);
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ADDE - Add Extended
unsafe extern "C" fn ppc32_emit_ADDE(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_jit_start_hreg_seq(cpu, cstr!("adde"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    let hreg_t0: c_int = ppc32_jit_alloc_hreg(cpu, -1);
    let hreg_t1: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_alter_host_reg(cpu, hreg_t0);
    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("adde"));

    // $t0 = $ra + carry
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t1, hreg_t1);
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_ra, 4);

    x86_alu_reg_membase!(&mut (*iop).ob_ptr, X86_ADD, hreg_t0, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t1, FALSE);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t1, 4);

    // $t0 += $rb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_t0, hreg_rb);
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t1, FALSE);
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t1);

    // update cr0
    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_t0);
    }

    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_t0, 4);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ADDI - ADD Immediate
unsafe extern "C" fn ppc32_emit_ADDI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let tmp: m_uint32_t = sign_extend_32(imm, 16) as m_uint32_t;
    let iop: *mut jit_op_t;

    // $rd = $ra + imm
    ppc32_jit_start_hreg_seq(cpu, cstr!("addi"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    if ra != 0 {
        let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
        ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

        iop = ppc32_op_emit_insn_output(cpu, 2, cstr!("addi"));

        if rd != ra {
            x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_ra, 4);
        }

        x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, tmp);
    } else {
        iop = ppc32_op_emit_insn_output(cpu, 1, cstr!("addi"));
        ppc32_load_imm(addr_of_mut!((*iop).ob_ptr), hreg_rd as u_int, tmp);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ADDIC - ADD Immediate with Carry
unsafe extern "C" fn ppc32_emit_ADDIC(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let tmp: m_uint32_t = sign_extend_32(imm, 16) as m_uint32_t;

    // $rd = $ra + imm
    ppc32_jit_start_hreg_seq(cpu, cstr!("addic"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("addic"));

    if rd != ra {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_ra, 4);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, tmp);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    x86_set_membase!(&mut (*iop).ob_ptr, X86_CC_C, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), FALSE);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ADDIC.
unsafe extern "C" fn ppc32_emit_ADDIC_dot(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: c_int = bits(insn, 0, 15);
    let tmp: m_uint32_t = sign_extend_32(imm, 16) as m_uint32_t;

    // $rd = $ra + imm
    ppc32_jit_start_hreg_seq(cpu, cstr!("addic."));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("addic."));

    if rd != ra {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_ra, 4);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, tmp);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    x86_set_membase!(&mut (*iop).ob_ptr, X86_CC_C, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), FALSE);

    ppc32_op_emit_update_flags(cpu, 0, TRUE);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ADDIS - ADD Immediate Shifted
unsafe extern "C" fn ppc32_emit_ADDIS(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;
    let tmp: m_uint32_t = imm << 16;
    let iop: *mut jit_op_t;

    // $rd = $ra + (imm << 16)
    ppc32_jit_start_hreg_seq(cpu, cstr!("addis"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    if ra != 0 {
        let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
        ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

        iop = ppc32_op_emit_insn_output(cpu, 1, cstr!("addis"));

        if rd != ra {
            x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_ra, 4);
        }

        x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, tmp);
    } else {
        if false {
            iop = ppc32_op_emit_insn_output(cpu, 1, cstr!("addis"));
            x86_mov_reg_imm!(&mut (*iop).ob_ptr, hreg_rd, tmp);
        }
        ppc32_op_emit_set_host_reg_imm32(cpu, hreg_rd, tmp);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ADDZE
unsafe extern "C" fn ppc32_emit_ADDZE(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    // $rd = $ra + xer_ca + set_carry
    ppc32_jit_start_hreg_seq(cpu, cstr!("addze"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("addze"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t0, hreg_t0);

    if rd != ra {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_ra, 4);
    }

    x86_alu_reg_membase!(&mut (*iop).ob_ptr, X86_ADD, hreg_rd, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca));

    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t0, FALSE);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t0, 4);

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// AND
unsafe extern "C" fn ppc32_emit_AND(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $ra = $rs & $rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("and"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("and"));

    if ra == rs {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, hreg_rb);
    } else if ra == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, hreg_rs);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, hreg_rb);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// ANDC
unsafe extern "C" fn ppc32_emit_ANDC(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $ra = $rs & ~$rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("andc"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("andc"));

    // $t0 = ~$rb
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rb, 4);
    x86_not_reg!(&mut (*iop).ob_ptr, hreg_t0);

    // $ra = $rs & $t0
    if ra == rs {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, hreg_t0);
    } else {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, hreg_rs);
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_t0, 4);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// AND Immediate
unsafe extern "C" fn ppc32_emit_ANDI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;
    let tmp: m_uint32_t = imm as m_uint32_t;

    // $ra = $rs & imm
    ppc32_jit_start_hreg_seq(cpu, cstr!("andi"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("andi"));

    if ra != rs {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, tmp);
    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    ppc32_op_emit_update_flags(cpu, 0, TRUE);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// AND Immediate Shifted
unsafe extern "C" fn ppc32_emit_ANDIS(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;
    let tmp: m_uint32_t = imm << 16;

    // $ra = $rs & imm
    ppc32_jit_start_hreg_seq(cpu, cstr!("andis"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("andis"));

    if ra != rs {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, tmp);
    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    ppc32_op_emit_update_flags(cpu, 0, TRUE);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// B - Branch
unsafe extern "C" fn ppc32_emit_B(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let offset: m_uint32_t = bits(insn, 2, 25) as m_uint32_t;
    let mut new_ia: m_uint32_t;

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 4, cstr!("b"));

    // compute the new ia
    new_ia = (*b).start_ia + ((*b).ppc_trans_pos << 2);
    new_ia += sign_extend((offset << 2) as m_int64_t, 26) as m_uint32_t;
    ppc32_set_jump(cpu, b, iop, new_ia, TRUE);

    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
    ppc32_op_emit_branch_target(cpu, b, new_ia);
    ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    0
}

/// BA - Branch Absolute
unsafe extern "C" fn ppc32_emit_BA(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let offset: m_uint32_t = bits(insn, 2, 25) as m_uint32_t;

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 4, cstr!("ba"));

    // compute the new ia
    let new_ia: m_uint32_t = sign_extend((offset << 2) as m_int64_t, 26) as m_uint32_t;
    ppc32_set_jump(cpu, b, iop, new_ia, TRUE);

    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
    ppc32_op_emit_branch_target(cpu, b, new_ia);
    ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    0
}

/// BL - Branch and Link
unsafe extern "C" fn ppc32_emit_BL(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let offset: m_uint32_t = bits(insn, 2, 25) as m_uint32_t;
    let mut new_ia: m_uint32_t;

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 4, cstr!("bl"));

    // compute the new ia
    new_ia = (*b).start_ia + ((*b).ppc_trans_pos << 2);
    new_ia += sign_extend((offset << 2) as m_int64_t, 26) as m_uint32_t;

    // set the return address
    ppc32_set_lr(iop, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    ppc32_set_jump(cpu, b, iop, new_ia, TRUE);

    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
    ppc32_op_emit_branch_target(cpu, b, new_ia);
    ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    0
}

/// BLA - Branch and Link Absolute
unsafe extern "C" fn ppc32_emit_BLA(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let offset: m_uint32_t = bits(insn, 2, 25) as m_uint32_t;

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 4, cstr!("bla"));

    // compute the new ia
    let new_ia: m_uint32_t = sign_extend((offset << 2) as m_int64_t, 26) as m_uint32_t;

    // set the return address
    ppc32_set_lr(iop, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    ppc32_set_jump(cpu, b, iop, new_ia, TRUE);

    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);
    ppc32_op_emit_branch_target(cpu, b, new_ia);
    ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    0
}

/// BC - Branch Conditional (Condition Check only)
unsafe extern "C" fn ppc32_emit_BCC(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);
    let bd: c_int = bits(insn, 2, 15);
    let mut new_ia: m_uint32_t;
    let mut jump_ptr: *mut u_char = null_mut();

    ppc32_op_emit_basic_opcode(cpu, JIT_OP_BRANCH_JUMP);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 5, cstr!("bcc"));

    // Get the wanted value for the condition bit
    let cond: c_int = (bo >> 3) & 0x1;

    // Set the return address
    if (insn & 1) != 0 {
        ppc32_set_lr(iop, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
        ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    }

    // Compute the new ia
    new_ia = sign_extend_32((bd << 2) as m_int32_t, 16) as m_uint32_t;
    if (insn & 0x02) == 0 {
        new_ia += (*b).start_ia + ((*b).ppc_trans_pos << 2);
    }

    /* Test the condition bit */
    let cr_field: u_int = ppc32_get_cr_field(bi as u_int);
    let cr_bit: u_int = ppc32_get_cr_bit(bi as u_int);

    ppc32_op_emit_require_flags(cpu, cr_field as c_int);

    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(cr_field as c_int), (1 << cr_bit));

    let local_jump: c_int = ppc32_jit_tcb_local_addr(b, new_ia, addr_of_mut!(jump_ptr));

    // Optimize the jump, depending if the destination is in the same
    // page or not.
    if local_jump != 0 {
        ppc32_jit_tcb_record_patch(b, iop, (*iop).ob_ptr, new_ia);
        x86_branch32!(&mut (*iop).ob_ptr, if cond != 0 { X86_CC_NZ } else { X86_CC_Z }, 0, FALSE);
    } else {
        jump_ptr = (*iop).ob_ptr;
        x86_branch32!(&mut (*iop).ob_ptr, if cond != 0 { X86_CC_Z } else { X86_CC_NZ }, 0, FALSE);
        ppc32_set_jump(cpu, b, iop, new_ia, TRUE);
        x86_patch!(jump_ptr, (*iop).ob_ptr);
    }

    ppc32_op_emit_branch_target(cpu, b, new_ia);
    0
}

/// BC - Branch Conditional
unsafe extern "C" fn ppc32_emit_BC(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);
    let bd: c_int = bits(insn, 2, 15);
    let cr_field: u_int;
    let cr_bit: u_int;
    let mut new_ia: m_uint32_t;
    let mut jump_ptr: *mut u_char = null_mut();

    ppc32_op_emit_basic_opcode(cpu, JIT_OP_BRANCH_JUMP);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 5, cstr!("bc"));

    ppc32_jit_start_hreg_seq(cpu, cstr!("bc"));
    let hreg_t0: c_int = ppc32_jit_alloc_hreg(cpu, -1);
    let hreg_t1: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_alter_host_reg(cpu, hreg_t0);

    // Get the wanted value for the condition bit and CTR value
    let cond: c_int = (bo >> 3) & 0x1;
    let ctr: c_int = (bo >> 1) & 0x1;

    // Set the return address
    if (insn & 1) != 0 {
        ppc32_set_lr(iop, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
        ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    }

    // Compute the new ia
    new_ia = sign_extend_32((bd << 2) as m_int32_t, 16) as m_uint32_t;
    if (insn & 0x02) == 0 {
        new_ia += (*b).start_ia + ((*b).ppc_trans_pos << 2);
    }

    x86_mov_reg_imm!(&mut (*iop).ob_ptr, hreg_t0, 1);

    // Decrement the count register
    if (bo & 0x04) == 0 {
        x86_dec_membase!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, ctr));
        x86_set_reg!(&mut (*iop).ob_ptr, if ctr != 0 { X86_CC_Z } else { X86_CC_NZ }, hreg_t1, FALSE);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, hreg_t1);
    }

    // Test the condition bit
    if ((bo >> 4) & 0x01) == 0 {
        cr_field = ppc32_get_cr_field(bi as u_int);
        cr_bit = ppc32_get_cr_bit(bi as u_int);

        ppc32_op_emit_require_flags(cpu, cr_field as c_int);

        x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(cr_field as c_int), (1 << cr_bit));

        x86_set_reg!(&mut (*iop).ob_ptr, if cond != 0 { X86_CC_NZ } else { X86_CC_Z }, hreg_t1, FALSE);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, hreg_t1);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    let local_jump: c_int = ppc32_jit_tcb_local_addr(b, new_ia, addr_of_mut!(jump_ptr));

    // Optimize the jump, depending if the destination is in the same
    // page or not.
    if local_jump != 0 {
        ppc32_jit_tcb_record_patch(b, iop, (*iop).ob_ptr, new_ia);
        x86_branch32!(&mut (*iop).ob_ptr, X86_CC_NZ, 0, FALSE);
    } else {
        jump_ptr = (*iop).ob_ptr;
        x86_branch32!(&mut (*iop).ob_ptr, X86_CC_Z, 0, FALSE);
        ppc32_set_jump(cpu, b, iop, new_ia, TRUE);
        x86_patch!(jump_ptr, (*iop).ob_ptr);
    }

    ppc32_op_emit_branch_target(cpu, b, new_ia);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// BCLR - Branch Conditional to Link register
unsafe extern "C" fn ppc32_emit_BCLR(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bo: c_int = bits(insn, 21, 25);
    let bi: c_int = bits(insn, 16, 20);
    let bd: c_int = bits(insn, 2, 15);
    let cr_field: u_int;
    let cr_bit: u_int;
    let mut _new_ia: m_uint32_t;

    ppc32_jit_start_hreg_seq(cpu, cstr!("bclr"));
    let hreg_t0: c_int = ppc32_jit_alloc_hreg(cpu, -1);
    let hreg_t1: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_alter_host_reg(cpu, hreg_t0);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 5, cstr!("bclr"));

    // Get the wanted value for the condition bit and CTR value
    let cond: c_int = (bo >> 3) & 0x1;
    let ctr: c_int = (bo >> 1) & 0x1;

    // Compute the new ia
    _new_ia = sign_extend_32(bd << 2, 16) as m_uint32_t;
    if (insn & 0x02) == 0 {
        _new_ia += (*b).start_ia + ((*b).ppc_trans_pos << 2);
    }

    x86_mov_reg_imm!(&mut (*iop).ob_ptr, hreg_t0, 1);

    // Decrement the count register
    if (bo & 0x04) == 0 {
        x86_dec_membase!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, ctr));
        x86_set_reg!(&mut (*iop).ob_ptr, if ctr != 0 { X86_CC_Z } else { X86_CC_NZ }, hreg_t1, FALSE);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, hreg_t1);
    }

    // Test the condition bit
    if ((bo >> 4) & 0x01) == 0 {
        cr_field = ppc32_get_cr_field(bi as u_int);
        cr_bit = ppc32_get_cr_bit(bi as u_int);

        ppc32_op_emit_require_flags(cpu, cr_field as c_int);

        x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(cr_field as c_int), (1 << cr_bit));

        x86_set_reg!(&mut (*iop).ob_ptr, if cond != 0 { X86_CC_NZ } else { X86_CC_Z }, hreg_t1, FALSE);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, hreg_t1);
    }

    // Set the return address
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_t1, X86_EDI, OFFSET!(cpu_ppc_t, lr), 4);

    if (insn & 1) != 0 {
        ppc32_set_lr(iop, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
        ppc32_op_emit_branch_target(cpu, b, (*b).start_ia + (((*b).ppc_trans_pos + 1) << 2));
    }

    // Branching
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    let jump_ptr: *mut u_char = (*iop).ob_ptr;
    x86_branch32!(&mut (*iop).ob_ptr, X86_CC_Z, 0, FALSE);

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t1, 0xFFFFFFFC_u32);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, ia), hreg_t1, 4);
    ppc32_jit_tcb_push_epilog(addr_of_mut!((*iop).ob_ptr));

    x86_patch!(jump_ptr, (*iop).ob_ptr);

    ppc32_op_emit_basic_opcode(cpu, JIT_OP_EOB);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CMP - Compare
unsafe extern "C" fn ppc32_emit_CMP(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_jit_start_hreg_seq(cpu, cstr!("cmp"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("cmp"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_CMP, hreg_ra, hreg_rb);
    ppc32_op_emit_update_flags(cpu, rd, TRUE);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CMPI - Compare Immediate
unsafe extern "C" fn ppc32_emit_CMPI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;
    let tmp: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    ppc32_jit_start_hreg_seq(cpu, cstr!("cmpi"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("cmpi"));

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_CMP, hreg_ra, tmp);
    ppc32_op_emit_update_flags(cpu, rd, TRUE);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CMPL - Compare Logical
unsafe extern "C" fn ppc32_emit_CMPL(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_jit_start_hreg_seq(cpu, cstr!("cmpl"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("cmpl"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_CMP, hreg_ra, hreg_rb);
    ppc32_op_emit_update_flags(cpu, rd, FALSE);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CMPLI - Compare Immediate
unsafe extern "C" fn ppc32_emit_CMPLI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;

    ppc32_jit_start_hreg_seq(cpu, cstr!("cmpli"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("cmpli"));

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_CMP, hreg_ra, imm);
    ppc32_op_emit_update_flags(cpu, rd, FALSE);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CRAND - Condition Register AND
unsafe extern "C" fn ppc32_emit_CRAND(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);

    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("crand"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);

    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(ba as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bb as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bd as u_int) as c_int);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("crand"));

    // test $ba bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba as u_int) as c_int), (1 << ppc32_get_cr_bit(ba as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, X86_EDX, FALSE);

    // test $bb bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb as u_int) as c_int), (1 << ppc32_get_cr_bit(bb as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, hreg_t0, FALSE);

    // result of AND between $ba and $bb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, X86_EDX);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    // set/clear $bd bit depending on the result
    x86_alu_membase_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), !(1 << ppc32_get_cr_bit(bd as u_int)));

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0, ppc32_get_cr_bit(bd as u_int));
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), hreg_t0);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CRANDC - Condition Register AND with Complement
unsafe extern "C" fn ppc32_emit_CRANDC(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);

    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("crandc"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);

    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(ba as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bb as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bd as u_int) as c_int);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("crandc"));

    // test $ba bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba as u_int) as c_int), (1 << ppc32_get_cr_bit(ba as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, X86_EDX, FALSE);

    // test $bb bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb as u_int) as c_int), (1 << ppc32_get_cr_bit(bb as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_Z, hreg_t0, FALSE);

    // result of AND between $ba and $bb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, X86_EDX);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    // set/clear $bd bit depending on the result
    x86_alu_membase_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), !(1 << ppc32_get_cr_bit(bd as u_int)));

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0, ppc32_get_cr_bit(bd as u_int));
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), hreg_t0);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CREQV - Condition Register EQV
unsafe extern "C" fn ppc32_emit_CREQV(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);

    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("creqv"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);

    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(ba as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bb as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bd as u_int) as c_int);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("creqv"));

    // test $ba bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba as u_int) as c_int), (1 << ppc32_get_cr_bit(ba as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, X86_EDX, FALSE);

    // test $bb bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb as u_int) as c_int), (1 << ppc32_get_cr_bit(bb as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, hreg_t0, FALSE);

    // result of XOR between $ba and $bb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t0, X86_EDX);
    x86_not_reg!(&mut (*iop).ob_ptr, hreg_t0);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    // set/clear $bd bit depending on the result
    x86_alu_membase_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), !(1 << ppc32_get_cr_bit(bd as u_int)));

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0, ppc32_get_cr_bit(bd as u_int));
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), hreg_t0);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CRNAND - Condition Register NAND
unsafe extern "C" fn ppc32_emit_CRNAND(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);

    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("crnand"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);

    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(ba as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bb as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bd as u_int) as c_int);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("crnand"));

    // test $ba bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba as u_int) as c_int), (1 << ppc32_get_cr_bit(ba as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, X86_EDX, FALSE);

    // test $bb bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb as u_int) as c_int), (1 << ppc32_get_cr_bit(bb as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, hreg_t0, FALSE);

    // result of NAND between $ba and $bb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, X86_EDX);
    x86_not_reg!(&mut (*iop).ob_ptr, hreg_t0);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    // set/clear $bd bit depending on the result
    x86_alu_membase_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), !(1 << ppc32_get_cr_bit(bd as u_int)));

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0, ppc32_get_cr_bit(bd as u_int));
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), hreg_t0);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CRNOR - Condition Register NOR
unsafe extern "C" fn ppc32_emit_CRNOR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);

    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("crnor"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);

    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(ba as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bb as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bd as u_int) as c_int);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("crnor"));

    // test $ba bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba as u_int) as c_int), (1 << ppc32_get_cr_bit(ba as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, X86_EDX, FALSE);

    // test $bb bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb as u_int) as c_int), (1 << ppc32_get_cr_bit(bb as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, hreg_t0, FALSE);

    // result of NOR between $ba and $bb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_t0, X86_EDX);
    x86_not_reg!(&mut (*iop).ob_ptr, hreg_t0);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    // set/clear $bd bit depending on the result
    x86_alu_membase_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), !(1 << ppc32_get_cr_bit(bd as u_int)));

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0, ppc32_get_cr_bit(bd as u_int));
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), hreg_t0);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CROR - Condition Register OR
unsafe extern "C" fn ppc32_emit_CROR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);

    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("cror"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);

    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(ba as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bb as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bd as u_int) as c_int);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("cror"));

    // test $ba bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba as u_int) as c_int), (1 << ppc32_get_cr_bit(ba as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, X86_EDX, FALSE);

    // test $bb bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb as u_int) as c_int), (1 << ppc32_get_cr_bit(bb as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, hreg_t0, FALSE);

    // result of OR between $ba and $bb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_t0, X86_EDX);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    // set/clear $bd bit depending on the result
    x86_alu_membase_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), !(1 << ppc32_get_cr_bit(bd as u_int)));

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0, ppc32_get_cr_bit(bd as u_int));
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), hreg_t0);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CRORC - Condition Register OR with Complement
unsafe extern "C" fn ppc32_emit_CRORC(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);

    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("crorc"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);

    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(ba as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bb as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bd as u_int) as c_int);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("crorc"));

    // test $ba bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba as u_int) as c_int), (1 << ppc32_get_cr_bit(ba as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, X86_EDX, FALSE);

    // test $bb bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb as u_int) as c_int), (1 << ppc32_get_cr_bit(bb as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_Z, hreg_t0, FALSE);

    // result of ORC between $ba and $bb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_t0, X86_EDX);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    // set/clear $bd bit depending on the result
    x86_alu_membase_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), !(1 << ppc32_get_cr_bit(bd as u_int)));

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0, ppc32_get_cr_bit(bd as u_int));
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), hreg_t0);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// CRXOR - Condition Register XOR
unsafe extern "C" fn ppc32_emit_CRXOR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let bd: c_int = bits(insn, 21, 25);
    let bb: c_int = bits(insn, 16, 20);
    let ba: c_int = bits(insn, 11, 15);

    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("crxor"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);

    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(ba as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bb as u_int) as c_int);
    ppc32_op_emit_require_flags(cpu, ppc32_get_cr_field(bd as u_int) as c_int);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("crxor"));

    // test $ba bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba as u_int) as c_int), (1 << ppc32_get_cr_bit(ba as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, X86_EDX, FALSE);

    // test $bb bit
    x86_test_membase_imm!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb as u_int) as c_int), (1 << ppc32_get_cr_bit(bb as u_int)));
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_NZ, hreg_t0, FALSE);

    // result of XOR between $ba and $bb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t0, X86_EDX);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x01);

    // set/clear $bd bit depending on the result
    x86_alu_membase_imm!(&mut (*iop).ob_ptr, X86_AND, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), !(1 << ppc32_get_cr_bit(bd as u_int)));

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0, ppc32_get_cr_bit(bd as u_int));
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd as u_int) as c_int), hreg_t0);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// DIVWU - Divide Word Unsigned
unsafe extern "C" fn ppc32_emit_DIVWU(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_jit_start_hreg_seq(cpu, cstr!("divwu"));
    ppc32_jit_alloc_hreg_forced(cpu, X86_EAX);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    // $rd = $ra / $rb
    ppc32_op_emit_load_gpr(cpu, X86_EAX, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("divwu"));
    ppc32_load_imm(addr_of_mut!((*iop).ob_ptr), X86_EDX as u_int, 0);

    x86_div_reg!(&mut (*iop).ob_ptr, hreg_rb, 0);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EAX);
    }

    ppc32_op_emit_store_gpr(cpu, rd, X86_EAX);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    // edx:eax are directly modified: throw them
    ppc32_op_emit_alter_host_reg(cpu, X86_EAX);
    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// EQV
unsafe extern "C" fn ppc32_emit_EQV(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $ra = ~($rs ^ $rb)
    ppc32_jit_start_hreg_seq(cpu, cstr!("eqv"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("eqv"));

    if ra == rs {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_ra, hreg_rb);
    } else if ra == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_ra, hreg_rs);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_ra, hreg_rb);
    }

    x86_not_reg!(&mut (*iop).ob_ptr, hreg_ra);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// EXTSB - Extend Sign Byte
unsafe extern "C" fn ppc32_emit_EXTSB(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    // $ra = extsb($rs)
    ppc32_jit_start_hreg_seq(cpu, cstr!("extsb"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("extsb"));

    if rs != ra {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_ra, 24);
    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SAR, hreg_ra, 24);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// EXTSH - Extend Sign Word
unsafe extern "C" fn ppc32_emit_EXTSH(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    // $ra = extsh($rs)
    ppc32_jit_start_hreg_seq(cpu, cstr!("extsh"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("extsh"));

    if rs != ra {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_ra, 16);
    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SAR, hreg_ra, 16);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// LBZ - Load Byte and Zero
unsafe extern "C" fn ppc32_emit_LBZ(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    if false {
        ppc32_emit_memop(cpu, b, PPC_MEMOP_LBZ, ra, offset as c_int, rs, 0);
    }
    ppc32_emit_memop_fast(cpu, b, 0, PPC_MEMOP_LBZ, ra, offset as c_int, rs, Some(ppc32_memop_fast_lbz));
    0
}

/// LBZU - Load Byte and Zero with Update
unsafe extern "C" fn ppc32_emit_LBZU(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_LBZ, ra, offset as c_int, rs, 1);
    0
}

/// LBZUX - Load Byte and Zero with Update Indexed
unsafe extern "C" fn ppc32_emit_LBZUX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_LBZ, ra, rb, rs, 1);
    0
}

/// LBZX - Load Byte and Zero Indexed
unsafe extern "C" fn ppc32_emit_LBZX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_LBZ, ra, rb, rs, 0);
    0
}

/// LHA - Load Half-Word Algebraic
unsafe extern "C" fn ppc32_emit_LHA(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_LHA, ra, offset as c_int, rs, 0);
    0
}

/// LHAU - Load Half-Word Algebraic with Update
unsafe extern "C" fn ppc32_emit_LHAU(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_LHA, ra, offset as c_int, rs, 1);
    0
}

/// LHAUX - Load Half-Word Algebraic with Update Indexed
unsafe extern "C" fn ppc32_emit_LHAUX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_LHA, ra, rb, rs, 1);
    0
}

/// LHAX - Load Half-Word Algebraic Indexed
unsafe extern "C" fn ppc32_emit_LHAX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_LHA, ra, rb, rs, 0);
    0
}

/// LHZ - Load Half-Word and Zero
unsafe extern "C" fn ppc32_emit_LHZ(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_LHZ, ra, offset as c_int, rs, 0);
    0
}

/// LHZU - Load Half-Word and Zero with Update
unsafe extern "C" fn ppc32_emit_LHZU(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_LHZ, ra, offset as c_int, rs, 1);
    0
}

/// LHZUX - Load Half-Word and Zero with Update Indexed
unsafe extern "C" fn ppc32_emit_LHZUX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_LHZ, ra, rb, rs, 1);
    0
}

/// LHZX - Load Half-Word and Zero Indexed
unsafe extern "C" fn ppc32_emit_LHZX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_LHZ, ra, rb, rs, 0);
    0
}

/// LWZ - Load Word and Zero
unsafe extern "C" fn ppc32_emit_LWZ(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    if false {
        ppc32_emit_memop(cpu, b, PPC_MEMOP_LWZ, ra, offset as c_int, rs, 0);
    }
    ppc32_emit_memop_fast(cpu, b, 0, PPC_MEMOP_LWZ, ra, offset as c_int, rs, Some(ppc32_memop_fast_lwz));
    0
}

/// LWZU - Load Word and Zero with Update
unsafe extern "C" fn ppc32_emit_LWZU(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_LWZ, ra, offset as c_int, rs, 1);
    0
}

/// LWZUX - Load Word and Zero with Update Indexed
unsafe extern "C" fn ppc32_emit_LWZUX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_LWZ, ra, rb, rs, 1);
    0
}

/// LWZX - Load Word and Zero Indexed
unsafe extern "C" fn ppc32_emit_LWZX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_LWZ, ra, rb, rs, 0);
    0
}

/// MCRF - Move Condition Register Field
unsafe extern "C" fn ppc32_emit_MCRF(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 23, 25);
    let rs: c_int = bits(insn, 18, 20);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mcrf"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_op_emit_require_flags(cpu, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("mcrf"));

    // Load "rs" field in %edx
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_t0, X86_EDI, PPC32_CR_FIELD_OFFSET(rs), 4);

    // Store it in "rd" field
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(rd), hreg_t0, 4);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MFCR - Move from Condition Register
unsafe extern "C" fn ppc32_emit_MFCR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mfcr"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    ppc32_op_emit_require_flags(cpu, JIT_OP_PPC_ALL_FLAGS);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("mfcr"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_rd, hreg_rd);

    for i in 0..8 as c_int {
        // load field in %edx
        x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_t0, X86_EDI, PPC32_CR_FIELD_OFFSET(i), 4);
        x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHL, hreg_rd, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_rd, hreg_t0);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MFMSR - Move from Machine State Register
unsafe extern "C" fn ppc32_emit_MFMSR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mfmsr"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("mfmsr"));
    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_rd, X86_EDI, OFFSET!(cpu_ppc_t, msr), 4);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MFSR - Move From Segment Register
unsafe extern "C" fn ppc32_emit_MFSR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let sr: c_int = bits(insn, 16, 19);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mfsr"));
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("mfsr"));

    x86_mov_reg_membase!(&mut (*iop).ob_ptr, hreg_rd, X86_EDI, (OFFSET!(cpu_ppc_t, sr) + (sr << 2)), 4);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MTCRF - Move to Condition Register Fields
unsafe extern "C" fn ppc32_emit_MTCRF(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let crm: c_int = bits(insn, 12, 19);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mtcrf"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("mtcrf"));

    for i in 0..8 as c_int {
        if (crm & (1 << (7 - i))) != 0 {
            x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rs, 4);

            if i != 7 {
                x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SHR, hreg_t0, 28 - (i << 2));
            }

            x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x0F);
            x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, PPC32_CR_FIELD_OFFSET(i), hreg_t0, 4);
        }
    }

    ppc32_op_emit_basic_opcode(cpu, JIT_OP_TRASH_FLAGS);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MULHW - Multiply High Word
unsafe extern "C" fn ppc32_emit_MULHW(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mulhw"));
    ppc32_jit_alloc_hreg_forced(cpu, X86_EAX);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, X86_EAX, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    // rd = hi(ra * rb)
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("mulhw"));
    x86_mul_reg!(&mut (*iop).ob_ptr, hreg_rb, 1);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, X86_EDX, X86_EDX);
    }

    ppc32_op_emit_store_gpr(cpu, rd, X86_EDX);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    // edx:eax are directly modified: throw them
    ppc32_op_emit_alter_host_reg(cpu, X86_EAX);
    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MULHWU - Multiply High Word Unsigned
unsafe extern "C" fn ppc32_emit_MULHWU(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mulhwu"));
    ppc32_jit_alloc_hreg_forced(cpu, X86_EAX);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, X86_EAX, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    // rd = hi(ra * rb)
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("mulhwu"));
    x86_mul_reg!(&mut (*iop).ob_ptr, hreg_rb, 0);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, X86_EDX, X86_EDX);
    }

    ppc32_op_emit_store_gpr(cpu, rd, X86_EDX);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    // edx:eax are directly modified: throw them
    ppc32_op_emit_alter_host_reg(cpu, X86_EAX);
    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MULLI - Multiply Low Immediate
unsafe extern "C" fn ppc32_emit_MULLI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;

    ppc32_jit_start_hreg_seq(cpu, cstr!("mulli"));
    ppc32_jit_alloc_hreg_forced(cpu, X86_EAX);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_load_gpr(cpu, X86_EAX, ra);

    // rd = lo(ra * imm)
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("mulli"));

    ppc32_load_imm(addr_of_mut!((*iop).ob_ptr), hreg_t0 as u_int, sign_extend_32(imm as m_int32_t, 16) as m_uint32_t);
    x86_mul_reg!(&mut (*iop).ob_ptr, hreg_t0, 1);
    ppc32_op_emit_store_gpr(cpu, rd, X86_EAX);

    // edx:eax are directly modified: throw them
    ppc32_op_emit_alter_host_reg(cpu, X86_EAX);
    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// MULLW - Multiply Low Word
unsafe extern "C" fn ppc32_emit_MULLW(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_jit_start_hreg_seq(cpu, cstr!("mullw"));
    ppc32_jit_alloc_hreg_forced(cpu, X86_EAX);
    ppc32_jit_alloc_hreg_forced(cpu, X86_EDX);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, X86_EAX, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    // rd = lo(ra * rb)
    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("mullw"));
    x86_mul_reg!(&mut (*iop).ob_ptr, hreg_rb, 1);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, X86_EAX, X86_EAX);
    }

    ppc32_op_emit_store_gpr(cpu, rd, X86_EAX);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    // edx:eax are directly modified: throw them
    ppc32_op_emit_alter_host_reg(cpu, X86_EAX);
    ppc32_op_emit_alter_host_reg(cpu, X86_EDX);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// NAND
unsafe extern "C" fn ppc32_emit_NAND(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $ra = ~($rs & $rb)
    ppc32_jit_start_hreg_seq(cpu, cstr!("nand"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("nand"));

    if ra == rs {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, hreg_rb);
    } else if ra == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, hreg_rs);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, hreg_rb);
    }

    x86_not_reg!(&mut (*iop).ob_ptr, hreg_ra);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// NEG
unsafe extern "C" fn ppc32_emit_NEG(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);

    // $rd = neg($ra)
    ppc32_jit_start_hreg_seq(cpu, cstr!("neg"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("neg"));

    if rd != ra {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_ra, 4);
    }

    x86_neg_reg!(&mut (*iop).ob_ptr, hreg_rd);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_rd);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// NOR
unsafe extern "C" fn ppc32_emit_NOR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $ra = ~($rs | $rb)
    ppc32_jit_start_hreg_seq(cpu, cstr!("nor"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("nor"));

    if ra == rs {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, hreg_rb);
    } else if ra == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, hreg_rs);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, hreg_rb);
    }

    x86_not_reg!(&mut (*iop).ob_ptr, hreg_ra);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// OR
unsafe extern "C" fn ppc32_emit_OR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let iop: *mut jit_op_t;

    // $ra = $rs | $rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("or"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    // special optimization for move/nop operation
    if rs == rb {
        ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
        iop = ppc32_op_emit_insn_output(cpu, 2, cstr!("or"));

        if ra != rs {
            x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
        }

        if (insn & 1) != 0 {
            x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
        }

        ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

        if (insn & 1) != 0 {
            ppc32_op_emit_update_flags(cpu, 0, TRUE);
        }

        ppc32_jit_close_hreg_seq(cpu);
        return 0;
    }

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    iop = ppc32_op_emit_insn_output(cpu, 2, cstr!("or"));

    if ra == rs {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, hreg_rb);
    } else if ra == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, hreg_rs);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, hreg_rb);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// OR with Complement
unsafe extern "C" fn ppc32_emit_ORC(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $ra = $rs & ~$rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("orc"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("orc"));

    // $t0 = ~$rb
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rb, 4);
    x86_not_reg!(&mut (*iop).ob_ptr, hreg_t0);

    // $ra = $rs | $t0
    if ra == rs {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, hreg_t0);
    } else {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_t0, hreg_rs);
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_t0, 4);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// OR Immediate
unsafe extern "C" fn ppc32_emit_ORI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;
    let tmp: m_uint32_t = imm as m_uint32_t;

    // $ra = $rs | imm
    ppc32_jit_start_hreg_seq(cpu, cstr!("ori"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("ori"));

    if ra != rs {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, tmp);
    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// OR Immediate Shifted
unsafe extern "C" fn ppc32_emit_ORIS(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;
    let tmp: m_uint32_t = (imm as m_uint32_t) << 16;

    // $ra = $rs | imm
    ppc32_jit_start_hreg_seq(cpu, cstr!("oris"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("oris"));

    if ra != rs {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, tmp);
    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// RLWIMI - Rotate Left Word Immediate then Mask Insert
unsafe extern "C" fn ppc32_emit_RLWIMI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    ppc32_jit_start_hreg_seq(cpu, cstr!("rlwimi"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("rlwimi"));

    // Apply inverse mask to $ra
    if mask != 0 {
        x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, !mask);
    }

    // Rotate $rs of "sh" bits and apply the mask
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rs, 4);

    if sh != 0 {
        x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_ROL, hreg_t0, sh);
    }

    if mask != 0xFFFFFFFF {
        x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, mask);
    }

    // Store the result
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_OR, hreg_ra, hreg_t0);
    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// RLWINM - Rotate Left Word Immediate AND with Mask
unsafe extern "C" fn ppc32_emit_RLWINM(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    ppc32_jit_start_hreg_seq(cpu, cstr!("rlwinm"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("rlwinm"));

    // Rotate $rs of "sh" bits and apply the mask
    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);

    if rs != ra {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    if sh != 0 {
        x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_ROL, hreg_ra, sh);
    }

    if mask != 0xFFFFFFFF {
        x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_ra, mask);
    }

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// RLWNM - Rotate Left Word then Mask Insert
unsafe extern "C" fn ppc32_emit_RLWNM(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);
    let mb: c_int = bits(insn, 6, 10);
    let me: c_int = bits(insn, 1, 5);

    // ecx is directly modified: throw it
    ppc32_op_emit_alter_host_reg(cpu, X86_ECX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("rlwnm"));
    ppc32_jit_alloc_hreg_forced(cpu, X86_ECX);

    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, X86_ECX, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("rlwnm"));

    // Load the shift register ("sh")
    let mask: m_uint32_t = ppc32_rotate_mask(mb as m_uint32_t, me as m_uint32_t);

    // Rotate $rs and apply the mask
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rs, 4);

    x86_shift_reg!(&mut (*iop).ob_ptr, X86_ROL, hreg_t0);

    if mask != 0xFFFFFFFF {
        x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, mask);
    }

    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_t0, 4);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// Shift Left Word
unsafe extern "C" fn ppc32_emit_SLW(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // ecx is directly modified: throw it
    ppc32_op_emit_alter_host_reg(cpu, X86_ECX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("slw"));
    ppc32_jit_alloc_hreg_forced(cpu, X86_ECX);
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    /* $ra = $rs << $rb. If count >= 32, then null result */
    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, X86_ECX, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("slw"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t0, hreg_t0);
    x86_test_reg_imm!(&mut (*iop).ob_ptr, X86_ECX, 0x20);
    let test1: *mut u_char = (*iop).ob_ptr;
    x86_branch8!(&mut (*iop).ob_ptr, X86_CC_NZ, 0, 1);

    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rs, 4);
    x86_shift_reg!(&mut (*iop).ob_ptr, X86_SHL, hreg_t0);

    // store the result
    x86_patch!(test1, (*iop).ob_ptr);
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_t0, 4);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// SRAWI - Shift Right Algebraic Word Immediate
unsafe extern "C" fn ppc32_emit_SRAWI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let sh: c_int = bits(insn, 11, 15);

    ppc32_jit_start_hreg_seq(cpu, cstr!("srawi"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    // $ra = (int32)$rs >> sh
    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("srawi"));
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rs, 4);

    if ra != rs {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }
    x86_shift_reg_imm!(&mut (*iop).ob_ptr, X86_SAR, hreg_ra, sh);

    // set XER_CA depending on the result
    let mask: m_uint32_t = !(0xFFFFFFFF_u32 << sh) | 0x80000000;

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, mask);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_CMP, hreg_t0, 0x80000000_u32);
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_A, hreg_t0, FALSE);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_AND, hreg_t0, 0x1);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t0, 4);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// Shift Right Word
unsafe extern "C" fn ppc32_emit_SRW(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // ecx is directly modified: throw it
    ppc32_op_emit_alter_host_reg(cpu, X86_ECX);

    ppc32_jit_start_hreg_seq(cpu, cstr!("srw"));
    ppc32_jit_alloc_hreg_forced(cpu, X86_ECX);
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    // $ra = $rs >> $rb. If count >= 32, then null result
    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, X86_ECX, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("srw"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t0, hreg_t0);
    x86_test_reg_imm!(&mut (*iop).ob_ptr, X86_ECX, 0x20);
    let test1: *mut u_char = (*iop).ob_ptr;
    x86_branch8!(&mut (*iop).ob_ptr, X86_CC_NZ, 0, 1);

    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rs, 4);
    x86_shift_reg!(&mut (*iop).ob_ptr, X86_SHR, hreg_t0);

    // store the result
    x86_patch!(test1, (*iop).ob_ptr);
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_t0, 4);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// STB - Store Byte
unsafe extern "C" fn ppc32_emit_STB(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    if false {
        ppc32_emit_memop(cpu, b, PPC_MEMOP_STB, ra, offset as c_int, rs, 0);
    }
    ppc32_emit_memop_fast(cpu, b, 1, PPC_MEMOP_STB, ra, offset as c_int, rs, Some(ppc32_memop_fast_stb));
    0
}

/// STBU - Store Byte with Update
unsafe extern "C" fn ppc32_emit_STBU(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_STB, ra, offset as c_int, rs, 1);
    0
}

/// STBUX - Store Byte with Update Indexed
unsafe extern "C" fn ppc32_emit_STBUX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_STB, ra, rb, rs, 1);
    0
}

/// STBUX - Store Byte Indexed
unsafe extern "C" fn ppc32_emit_STBX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_STB, ra, rb, rs, 0);
    0
}

/// STH - Store Half-Word
unsafe extern "C" fn ppc32_emit_STH(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_STH, ra, offset as c_int, rs, 0);
    0
}

/// STHU - Store Half-Word with Update
unsafe extern "C" fn ppc32_emit_STHU(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_STH, ra, offset as c_int, rs, 1);
    0
}

/// STHUX - Store Half-Word with Update Indexed
unsafe extern "C" fn ppc32_emit_STHUX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_STH, ra, rb, rs, 1);
    0
}

/// STHX - Store Half-Word Indexed
unsafe extern "C" fn ppc32_emit_STHX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_STH, ra, rb, rs, 0);
    0
}

/// STW - Store Word
unsafe extern "C" fn ppc32_emit_STW(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    if false {
        ppc32_emit_memop(cpu, b, PPC_MEMOP_STW, ra, offset as c_int, rs, 0);
    }
    ppc32_emit_memop_fast(cpu, b, 1, PPC_MEMOP_STW, ra, offset as c_int, rs, Some(ppc32_memop_fast_stw));
    0
}

/// STWU - Store Word with Update
unsafe extern "C" fn ppc32_emit_STWU(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let offset: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;

    ppc32_emit_memop(cpu, b, PPC_MEMOP_STW, ra, offset as c_int, rs, 1);
    0
}

/// STWUX - Store Word with Update Indexed
unsafe extern "C" fn ppc32_emit_STWUX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_STW, ra, rb, rs, 1);
    0
}

/// STWUX - Store Word Indexed
unsafe extern "C" fn ppc32_emit_STWX(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    ppc32_emit_memop_idx(cpu, b, PPC_MEMOP_STW, ra, rb, rs, 0);
    0
}

/// SUBF - Subtract From
unsafe extern "C" fn ppc32_emit_SUBF(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $rd = $rb - $ra
    ppc32_jit_start_hreg_seq(cpu, cstr!("subf"));
    let hreg_t0: c_int = ppc32_jit_get_tmp_hreg(cpu);

    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 2, cstr!("subf"));

    if rd == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_SUB, hreg_rd, hreg_ra);
    } else if rd == ra {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_rb, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_SUB, hreg_t0, hreg_ra);
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_t0, 4);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_rb, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_SUB, hreg_rd, hreg_ra);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// SUBFC - Subtract From Carrying
unsafe extern "C" fn ppc32_emit_SUBFC(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $rd = ~$ra + 1 + $rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("subfc"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    let hreg_t0: c_int = ppc32_jit_alloc_hreg(cpu, -1);
    let hreg_t1: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_alter_host_reg(cpu, hreg_t0);
    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("subfc"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t1, hreg_t1);

    // $t0 = ~$ra + 1
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_ra, 4);
    x86_not_reg!(&mut (*iop).ob_ptr, hreg_t0);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, hreg_t0, 1);
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t1, FALSE);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t1, 4);

    // $t0 += $rb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_t0, hreg_rb);
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t1, FALSE);
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t1);

    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_t0, 4);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_rd);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    // update cr0
    if (insn & 1) != 0 {
        ppc32_update_cr0(b);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// SUBFE - Subtract From Extended
unsafe extern "C" fn ppc32_emit_SUBFE(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $rd = ~$ra + $carry (xer_ca) + $rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("subfe"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    let hreg_t0: c_int = ppc32_jit_alloc_hreg(cpu, -1);
    let hreg_t1: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_alter_host_reg(cpu, hreg_t0);
    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("subfe"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t1, hreg_t1);

    // $t0 = ~$ra + $carry
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_ra, 4);
    x86_not_reg!(&mut (*iop).ob_ptr, hreg_t0);
    x86_alu_reg_membase!(&mut (*iop).ob_ptr, X86_ADD, hreg_t0, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca));

    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t1, FALSE);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t1, 4);

    // $t0 += $rb
    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_ADD, hreg_t0, hreg_rb);
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t1, FALSE);
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t1);

    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_t0, 4);

    if (insn & 1) != 0 {
        x86_test_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_rd);
    }

    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    // update cr0
    if (insn & 1) != 0 {
        ppc32_update_cr0(b);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// SUBFIC - Subtract From Immediate Carrying
unsafe extern "C" fn ppc32_emit_SUBFIC(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rd: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;
    let tmp: m_uint32_t = sign_extend_32(imm as m_int32_t, 16) as m_uint32_t;

    // $rd = ~$ra + 1 + sign_extend(imm,16)
    ppc32_jit_start_hreg_seq(cpu, cstr!("subfic"));
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rd: c_int = ppc32_jit_alloc_hreg(cpu, rd);

    let hreg_t0: c_int = ppc32_jit_alloc_hreg(cpu, -1);
    let hreg_t1: c_int = ppc32_jit_get_tmp_hreg(cpu);

    ppc32_op_emit_alter_host_reg(cpu, hreg_t0);
    ppc32_op_emit_load_gpr(cpu, hreg_ra, ra);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 3, cstr!("subfic"));

    x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_t1, hreg_t1);

    // $t0 = ~$ra + 1
    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_t0, hreg_ra, 4);
    x86_not_reg!(&mut (*iop).ob_ptr, hreg_t0);
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, hreg_t0, 1);

    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t1, FALSE);
    x86_mov_membase_reg!(&mut (*iop).ob_ptr, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t1, 4);

    // $t0 += sign_extend(imm,16)
    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_ADD, hreg_t0, tmp);
    x86_set_reg!(&mut (*iop).ob_ptr, X86_CC_C, hreg_t1, FALSE);
    x86_alu_membase_reg!(&mut (*iop).ob_ptr, X86_OR, X86_EDI, OFFSET!(cpu_ppc_t, xer_ca), hreg_t1);

    x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_rd, hreg_t0, 4);
    ppc32_op_emit_store_gpr(cpu, rd, hreg_rd);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// SYNC - Synchronize
unsafe extern "C" fn ppc32_emit_SYNC(_cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, _insn: ppc_insn_t) -> c_int {
    0
}

/// XOR
unsafe extern "C" fn ppc32_emit_XOR(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let rb: c_int = bits(insn, 11, 15);

    // $ra = $rs ^ $rb
    ppc32_jit_start_hreg_seq(cpu, cstr!("xor"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);
    let hreg_rb: c_int = ppc32_jit_alloc_hreg(cpu, rb);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);
    ppc32_op_emit_load_gpr(cpu, hreg_rb, rb);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("xor"));

    if ra == rs {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_ra, hreg_rb);
    } else if ra == rb {
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_ra, hreg_rs);
    } else {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
        x86_alu_reg_reg!(&mut (*iop).ob_ptr, X86_XOR, hreg_ra, hreg_rb);
    }

    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    if (insn & 1) != 0 {
        ppc32_op_emit_update_flags(cpu, 0, TRUE);
    }

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// XORI - XOR Immediate
unsafe extern "C" fn ppc32_emit_XORI(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint32_t = bits(insn, 0, 15) as m_uint32_t;

    // $ra = $rs ^ imm
    ppc32_jit_start_hreg_seq(cpu, cstr!("xori"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("xori"));

    if ra != rs {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_XOR, hreg_ra, imm);
    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// XORIS - XOR Immediate Shifted
unsafe extern "C" fn ppc32_emit_XORIS(cpu: *mut cpu_ppc_t, _b: *mut ppc32_jit_tcb_t, insn: ppc_insn_t) -> c_int {
    let rs: c_int = bits(insn, 21, 25);
    let ra: c_int = bits(insn, 16, 20);
    let imm: m_uint16_t = bits(insn, 0, 15) as m_uint16_t;
    let tmp: m_uint32_t = (imm as m_uint32_t) << 16;

    // $ra = $rs ^ (imm << 16)
    ppc32_jit_start_hreg_seq(cpu, cstr!("xoris"));
    let hreg_rs: c_int = ppc32_jit_alloc_hreg(cpu, rs);
    let hreg_ra: c_int = ppc32_jit_alloc_hreg(cpu, ra);

    ppc32_op_emit_load_gpr(cpu, hreg_rs, rs);

    let iop: *mut jit_op_t = ppc32_op_emit_insn_output(cpu, 1, cstr!("xoris"));

    if ra != rs {
        x86_mov_reg_reg!(&mut (*iop).ob_ptr, hreg_ra, hreg_rs, 4);
    }

    x86_alu_reg_imm!(&mut (*iop).ob_ptr, X86_XOR, hreg_ra, tmp);
    ppc32_op_emit_store_gpr(cpu, ra, hreg_ra);

    ppc32_jit_close_hreg_seq(cpu);
    0
}

/// PPC instruction array
#[no_mangle]
pub static mut ppc32_insn_tags: [ppc32_insn_tag; 103] = [
    ppc32_insn_tag::new(ppc32_emit_BLR, 0xfffffffe, 0x4e800020),
    ppc32_insn_tag::new(ppc32_emit_BCTR, 0xfffffffe, 0x4e800420),
    ppc32_insn_tag::new(ppc32_emit_MFLR, 0xfc1fffff, 0x7c0802a6),
    ppc32_insn_tag::new(ppc32_emit_MTLR, 0xfc1fffff, 0x7c0803a6),
    ppc32_insn_tag::new(ppc32_emit_MFCTR, 0xfc1fffff, 0x7c0902a6),
    ppc32_insn_tag::new(ppc32_emit_MTCTR, 0xfc1fffff, 0x7c0903a6),
    ppc32_insn_tag::new(ppc32_emit_MFTBL, 0xfc1ff7ff, 0x7c0c42e6),
    ppc32_insn_tag::new(ppc32_emit_MFTBU, 0xfc1ff7ff, 0x7c0d42e6),
    ppc32_insn_tag::new(ppc32_emit_ADD, 0xfc0007fe, 0x7c000214),
    ppc32_insn_tag::new(ppc32_emit_ADDC, 0xfc0007fe, 0x7c000014),
    ppc32_insn_tag::new(ppc32_emit_ADDE, 0xfc0007fe, 0x7c000114),
    ppc32_insn_tag::new(ppc32_emit_ADDI, 0xfc000000, 0x38000000),
    ppc32_insn_tag::new(ppc32_emit_ADDIC, 0xfc000000, 0x30000000),
    ppc32_insn_tag::new(ppc32_emit_ADDIC_dot, 0xfc000000, 0x34000000),
    ppc32_insn_tag::new(ppc32_emit_ADDIS, 0xfc000000, 0x3c000000),
    ppc32_insn_tag::new(ppc32_emit_ADDZE, 0xfc00fffe, 0x7c000194),
    ppc32_insn_tag::new(ppc32_emit_AND, 0xfc0007fe, 0x7c000038),
    ppc32_insn_tag::new(ppc32_emit_ANDC, 0xfc0007fe, 0x7c000078),
    ppc32_insn_tag::new(ppc32_emit_ANDI, 0xfc000000, 0x70000000),
    ppc32_insn_tag::new(ppc32_emit_ANDIS, 0xfc000000, 0x74000000),
    ppc32_insn_tag::new(ppc32_emit_B, 0xfc000003, 0x48000000),
    ppc32_insn_tag::new(ppc32_emit_BA, 0xfc000003, 0x48000002),
    ppc32_insn_tag::new(ppc32_emit_BL, 0xfc000003, 0x48000001),
    ppc32_insn_tag::new(ppc32_emit_BLA, 0xfc000003, 0x48000003),
    ppc32_insn_tag::new(ppc32_emit_BCC, 0xfe800000, 0x40800000),
    ppc32_insn_tag::new(ppc32_emit_BC, 0xfc000000, 0x40000000),
    ppc32_insn_tag::new(ppc32_emit_BCLR, 0xfc00fffe, 0x4c000020),
    ppc32_insn_tag::new(ppc32_emit_CMP, 0xfc6007ff, 0x7c000000),
    ppc32_insn_tag::new(ppc32_emit_CMPI, 0xfc600000, 0x2c000000),
    ppc32_insn_tag::new(ppc32_emit_CMPL, 0xfc6007ff, 0x7c000040),
    ppc32_insn_tag::new(ppc32_emit_CMPLI, 0xfc600000, 0x28000000),
    ppc32_insn_tag::new(ppc32_emit_CRAND, 0xfc0007ff, 0x4c000202),
    ppc32_insn_tag::new(ppc32_emit_CRANDC, 0xfc0007ff, 0x4c000102),
    ppc32_insn_tag::new(ppc32_emit_CREQV, 0xfc0007ff, 0x4c000242),
    ppc32_insn_tag::new(ppc32_emit_CRNAND, 0xfc0007ff, 0x4c0001c2),
    ppc32_insn_tag::new(ppc32_emit_CRNOR, 0xfc0007ff, 0x4c000042),
    ppc32_insn_tag::new(ppc32_emit_CROR, 0xfc0007ff, 0x4c000382),
    ppc32_insn_tag::new(ppc32_emit_CRORC, 0xfc0007ff, 0x4c000342),
    ppc32_insn_tag::new(ppc32_emit_CRXOR, 0xfc0007ff, 0x4c000182),
    ppc32_insn_tag::new(ppc32_emit_DIVWU, 0xfc0007fe, 0x7c000396),
    ppc32_insn_tag::new(ppc32_emit_EQV, 0xfc0007fe, 0x7c000238),
    ppc32_insn_tag::new(ppc32_emit_EXTSB, 0xfc00fffe, 0x7c000774),
    ppc32_insn_tag::new(ppc32_emit_EXTSH, 0xfc00fffe, 0x7c000734),
    ppc32_insn_tag::new(ppc32_emit_LBZ, 0xfc000000, 0x88000000),
    ppc32_insn_tag::new(ppc32_emit_LBZU, 0xfc000000, 0x8c000000),
    ppc32_insn_tag::new(ppc32_emit_LBZUX, 0xfc0007ff, 0x7c0000ee),
    ppc32_insn_tag::new(ppc32_emit_LBZX, 0xfc0007ff, 0x7c0000ae),
    ppc32_insn_tag::new(ppc32_emit_LHA, 0xfc000000, 0xa8000000),
    ppc32_insn_tag::new(ppc32_emit_LHAU, 0xfc000000, 0xac000000),
    ppc32_insn_tag::new(ppc32_emit_LHAUX, 0xfc0007ff, 0x7c0002ee),
    ppc32_insn_tag::new(ppc32_emit_LHAX, 0xfc0007ff, 0x7c0002ae),
    ppc32_insn_tag::new(ppc32_emit_LHZ, 0xfc000000, 0xa0000000),
    ppc32_insn_tag::new(ppc32_emit_LHZU, 0xfc000000, 0xa4000000),
    ppc32_insn_tag::new(ppc32_emit_LHZUX, 0xfc0007ff, 0x7c00026e),
    ppc32_insn_tag::new(ppc32_emit_LHZX, 0xfc0007ff, 0x7c00022e),
    ppc32_insn_tag::new(ppc32_emit_LWZ, 0xfc000000, 0x80000000),
    ppc32_insn_tag::new(ppc32_emit_LWZU, 0xfc000000, 0x84000000),
    ppc32_insn_tag::new(ppc32_emit_LWZUX, 0xfc0007ff, 0x7c00006e),
    ppc32_insn_tag::new(ppc32_emit_LWZX, 0xfc0007ff, 0x7c00002e),
    ppc32_insn_tag::new(ppc32_emit_MCRF, 0xfc63ffff, 0x4c000000),
    ppc32_insn_tag::new(ppc32_emit_MFCR, 0xfc1fffff, 0x7c000026),
    ppc32_insn_tag::new(ppc32_emit_MFMSR, 0xfc1fffff, 0x7c0000a6),
    ppc32_insn_tag::new(ppc32_emit_MFSR, 0xfc10ffff, 0x7c0004a6),
    ppc32_insn_tag::new(ppc32_emit_MTCRF, 0xfc100fff, 0x7c000120),
    ppc32_insn_tag::new(ppc32_emit_MULHW, 0xfc0007fe, 0x7c000096),
    ppc32_insn_tag::new(ppc32_emit_MULHWU, 0xfc0007fe, 0x7c000016),
    ppc32_insn_tag::new(ppc32_emit_MULLI, 0xfc000000, 0x1c000000),
    ppc32_insn_tag::new(ppc32_emit_MULLW, 0xfc0007fe, 0x7c0001d6),
    ppc32_insn_tag::new(ppc32_emit_NAND, 0xfc0007fe, 0x7c0003b8),
    ppc32_insn_tag::new(ppc32_emit_NEG, 0xfc00fffe, 0x7c0000d0),
    ppc32_insn_tag::new(ppc32_emit_NOR, 0xfc0007fe, 0x7c0000f8),
    ppc32_insn_tag::new(ppc32_emit_OR, 0xfc0007fe, 0x7c000378),
    ppc32_insn_tag::new(ppc32_emit_ORC, 0xfc0007fe, 0x7c000338),
    ppc32_insn_tag::new(ppc32_emit_ORI, 0xfc000000, 0x60000000),
    ppc32_insn_tag::new(ppc32_emit_ORIS, 0xfc000000, 0x64000000),
    ppc32_insn_tag::new(ppc32_emit_RLWIMI, 0xfc000000, 0x50000000),
    ppc32_insn_tag::new(ppc32_emit_RLWINM, 0xfc000000, 0x54000000),
    ppc32_insn_tag::new(ppc32_emit_RLWNM, 0xfc000000, 0x5c000000),
    ppc32_insn_tag::new(ppc32_emit_SLW, 0xfc0007fe, 0x7c000030),
    ppc32_insn_tag::new(ppc32_emit_SRAWI, 0xfc0007fe, 0x7c000670),
    ppc32_insn_tag::new(ppc32_emit_SRW, 0xfc0007fe, 0x7c000430),
    ppc32_insn_tag::new(ppc32_emit_STB, 0xfc000000, 0x98000000),
    ppc32_insn_tag::new(ppc32_emit_STBU, 0xfc000000, 0x9c000000),
    ppc32_insn_tag::new(ppc32_emit_STBUX, 0xfc0007ff, 0x7c0001ee),
    ppc32_insn_tag::new(ppc32_emit_STBX, 0xfc0007ff, 0x7c0001ae),
    ppc32_insn_tag::new(ppc32_emit_STH, 0xfc000000, 0xb0000000),
    ppc32_insn_tag::new(ppc32_emit_STHU, 0xfc000000, 0xb4000000),
    ppc32_insn_tag::new(ppc32_emit_STHUX, 0xfc0007ff, 0x7c00036e),
    ppc32_insn_tag::new(ppc32_emit_STHX, 0xfc0007ff, 0x7c00032e),
    ppc32_insn_tag::new(ppc32_emit_STW, 0xfc000000, 0x90000000),
    ppc32_insn_tag::new(ppc32_emit_STWU, 0xfc000000, 0x94000000),
    ppc32_insn_tag::new(ppc32_emit_STWUX, 0xfc0007ff, 0x7c00016e),
    ppc32_insn_tag::new(ppc32_emit_STWX, 0xfc0007ff, 0x7c00012e),
    ppc32_insn_tag::new(ppc32_emit_SUBF, 0xfc0007fe, 0x7c000050),
    ppc32_insn_tag::new(ppc32_emit_SUBFC, 0xfc0007fe, 0x7c000010),
    ppc32_insn_tag::new(ppc32_emit_SUBFE, 0xfc0007fe, 0x7c000110),
    ppc32_insn_tag::new(ppc32_emit_SUBFIC, 0xfc000000, 0x20000000),
    ppc32_insn_tag::new(ppc32_emit_SYNC, 0xffffffff, 0x7c0004ac),
    ppc32_insn_tag::new(ppc32_emit_XOR, 0xfc0007fe, 0x7c000278),
    ppc32_insn_tag::new(ppc32_emit_XORI, 0xfc000000, 0x68000000),
    ppc32_insn_tag::new(ppc32_emit_XORIS, 0xfc000000, 0x6c000000),
    ppc32_insn_tag::new(ppc32_emit_unknown, 0x00000000, 0x00000000),
    ppc32_insn_tag::null(),
];
