//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! PPC32 JIT compiler.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips_common::*;
use crate::jit_op::*;
use crate::ppc32::*;
use crate::sbox::*;
use crate::utils::*;

pub type ppc32_jit_tcb_t = ppc32_jit_tcb;

/// Size of executable page area (in Mb)
pub const PPC_EXEC_AREA_SIZE: usize = 64;
#[cfg(if_0)]
pub const PPC_EXEC_AREA_SIZE: usize = 16; // FIXME this is the correct value for cygwin, but rust does not have a cygwin target yet

/// Buffer size for JIT code generation
pub const PPC_JIT_BUFSIZE: usize = 32768;

/// Maximum number of X86 chunks
pub const PPC_JIT_MAX_CHUNKS: usize = 64;

/// Size of hash for IA lookup
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const PPC_JIT_IA_HASH_BITS: c_int = 17;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const PPC_JIT_IA_HASH_MASK: m_uint32_t = (1 << PPC_JIT_IA_HASH_BITS) - 1;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const PPC_JIT_IA_HASH_SIZE: m_uint32_t = 1 << PPC_JIT_IA_HASH_BITS;

/// Size of hash for virtual address lookup
#[cfg(feature = "USE_UNSTABLE")]
pub const PPC_JIT_VIRT_HASH_BITS: c_int = 17;
#[cfg(feature = "USE_UNSTABLE")]
pub const PPC_JIT_VIRT_HASH_MASK: m_uint32_t = (1 << PPC_JIT_VIRT_HASH_BITS) - 1;
#[cfg(feature = "USE_UNSTABLE")]
pub const PPC_JIT_VIRT_HASH_SIZE: m_uint32_t = 1 << PPC_JIT_VIRT_HASH_BITS;

/// Size of hash for physical lookup
pub const PPC_JIT_PHYS_HASH_BITS: c_int = 16;
pub const PPC_JIT_PHYS_HASH_MASK: m_uint32_t = (1 << PPC_JIT_PHYS_HASH_BITS) - 1;
pub const PPC_JIT_PHYS_HASH_SIZE: m_uint32_t = 1 << PPC_JIT_PHYS_HASH_BITS;

#[no_mangle]
pub unsafe extern "C" fn PPC_JIT_TARGET_BITMAP_INDEX(x: m_uint32_t) -> m_uint32_t {
    (x >> 7) & 0x1F
}
#[no_mangle]
pub unsafe extern "C" fn PPC_JIT_TARGET_BITMAP_POS(x: m_uint32_t) -> m_uint32_t {
    (x >> 2) & 0x1F
}

/// Instruction jump patch
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_insn_patch {
    pub next: *mut ppc32_insn_patch,
    pub jit_insn: *mut u_char,
    pub ppc_ia: m_uint32_t,
}

/// Instruction patch table
pub const PPC32_INSN_PATCH_TABLE_SIZE: usize = 32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_jit_patch_table {
    pub next: *mut ppc32_jit_patch_table,
    pub patches: [ppc32_insn_patch; PPC32_INSN_PATCH_TABLE_SIZE],
    pub cur_patch: u_int,
}

#[cfg(feature = "USE_UNSTABLE")]
pub const PPC32_JIT_TCB_FLAG_SMC: c_int = 0x1; // Self-modifying code
pub const PPC32_JIT_TCB_FLAG_NO_FLUSH: c_int = 0x2; // No flushing

/// PPC32 translated code block
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_jit_tcb {
    pub flags: u_int,
    pub start_ia: m_uint32_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub exec_state: m_uint32_t,

    pub jit_insn_ptr: *mut *mut u_char,
    pub acc_count: m_uint64_t,
    pub ppc_code: *mut ppc_insn_t,
    pub ppc_trans_pos: u_int,
    pub jit_chunk_pos: u_int,
    pub jit_ptr: *mut u_char,
    pub jit_buffer: *mut insn_exec_page_t,
    pub jit_chunks: [*mut insn_exec_page_t; PPC_JIT_MAX_CHUNKS],
    pub patch_table: *mut ppc32_jit_patch_table,
    pub prev: *mut ppc32_jit_tcb_t,
    pub next: *mut ppc32_jit_tcb_t,

    /// Physical page information
    pub phys_page: m_uint32_t,
    pub phys_hash: m_uint32_t,
    pub phys_pprev: *mut *mut ppc32_jit_tcb_t,
    pub phys_next: *mut ppc32_jit_tcb_t,

    /// 1024 instructions per page, one bit per instruction
    pub target_bitmap: [m_uint32_t; 32],
    pub target_undef_cnt: m_uint32_t,

    #[cfg(feature = "USE_DEBUG_BLOCK_TIMESTAMP")]
    pub tm_first_use: m_uint64_t,
    #[cfg(feature = "USE_DEBUG_BLOCK_TIMESTAMP")]
    pub tm_last_use: m_uint64_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_insn_tag {
    pub emit: Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, arg1: *mut ppc32_jit_tcb_t, arg2: ppc_insn_t) -> c_int>,
    pub mask: m_uint32_t,
    pub value: m_uint32_t,
}
impl ppc32_insn_tag {
    pub const fn new(emit: unsafe extern "C" fn(cpu: *mut cpu_ppc_t, arg1: *mut ppc32_jit_tcb_t, arg2: ppc_insn_t) -> c_int, mask: m_uint32_t, value: m_uint32_t) -> Self {
        Self { emit: Some(emit), mask, value }
    }
    pub const fn null() -> Self {
        Self { emit: None, mask: 0x00000000, value: 0x00000000 }
    }
}

/// Mark the specified IA as a target for further recompiling
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_tcb_set_target_bit(b: *mut ppc32_jit_tcb_t, ia: m_uint32_t) {
    let index: c_int = PPC_JIT_TARGET_BITMAP_INDEX(ia) as c_int;
    let pos: c_int = PPC_JIT_TARGET_BITMAP_POS(ia) as c_int;

    (*b).target_bitmap[index as usize] |= 1 << pos;
}

/// Returns TRUE if the specified IA is in the target bitmap
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_tcb_get_target_bit(b: *mut ppc32_jit_tcb_t, ia: m_uint32_t) -> c_int {
    let index: c_int = PPC_JIT_TARGET_BITMAP_INDEX(ia) as c_int;
    let pos: c_int = PPC_JIT_TARGET_BITMAP_POS(ia) as c_int;

    ((*b).target_bitmap[index as usize] & (1 << pos)) as c_int
}

/// Get the JIT instruction pointer in a translated block
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_tcb_get_host_ptr(tcb: *mut ppc32_jit_tcb_t, vaddr: m_uint32_t) -> *mut u_char {
    let offset: m_uint32_t = (vaddr & PPC32_MIN_PAGE_IMASK) >> 2;
    *(*tcb).jit_insn_ptr.add(offset as usize)
}

/// Check if the specified address belongs to the specified block
#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_tcb_local_addr(tcb: *mut ppc32_jit_tcb_t, vaddr: m_uint32_t, jit_addr: *mut *mut u_char) -> c_int {
    if (vaddr & PPC32_MIN_PAGE_MASK) == (*tcb).start_ia {
        *jit_addr = ppc32_jit_tcb_get_host_ptr(tcb, vaddr);
        return 1;
    }

    0
}

/// Check if PC register matches the compiled block virtual address
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_tcb_match(cpu: *mut cpu_ppc_t, tcb: *mut ppc32_jit_tcb_t) -> c_int {
    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        let vpage: m_uint32_t = (*cpu).ia & !PPC32_MIN_PAGE_IMASK;
        ((*tcb).start_ia == vpage) as c_int
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        let vpage: m_uint32_t = (*cpu).ia & PPC32_MIN_PAGE_MASK;
        (((*tcb).start_ia == vpage) && ((*tcb).exec_state == (*cpu).exec_state)) as c_int
    }
}

/// Compute the hash index for the specified IA value
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_get_ia_hash(ia: m_uint32_t) -> m_uint32_t {
    let page_hash: m_uint32_t = sbox_u32(ia >> PPC32_MIN_PAGE_SHIFT);
    (page_hash ^ (page_hash >> 14)) & PPC_JIT_IA_HASH_MASK
}

/// Compute the hash index for the specified virtual address
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_get_virt_hash(vaddr: m_uint32_t) -> m_uint32_t {
    let page_hash: m_uint32_t = sbox_u32(vaddr >> PPC32_MIN_PAGE_SHIFT);
    (page_hash ^ (page_hash >> 14)) & PPC_JIT_VIRT_HASH_MASK
}

/// Compute the hash index for the specified physical page
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_get_phys_hash(phys_page: m_uint32_t) -> m_uint32_t {
    let page_hash: m_uint32_t = sbox_u32(phys_page);
    (page_hash ^ (page_hash >> 12)) & PPC_JIT_PHYS_HASH_MASK
}

/// Find a JIT block matching a physical page
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_jit_find_by_phys_page(cpu: *mut cpu_ppc_t, phys_page: m_uint32_t) -> *mut ppc32_jit_tcb_t {
    let page_hash: m_uint32_t = ppc32_jit_get_phys_hash(phys_page);

    #[cfg(not(feature = "USE_UNSTABLE"))]
    let mut tcb: *mut ppc32_jit_tcb_t = *(*cpu).exec_phys_map.add(page_hash as usize);
    #[cfg(feature = "USE_UNSTABLE")]
    let mut tcb: *mut ppc32_jit_tcb_t = *(*cpu).tcb_phys_hash.add(page_hash as usize);
    while !tcb.is_null() {
        if (*tcb).phys_page == phys_page {
            return tcb;
        }
        tcb = (*tcb).phys_next;
    }

    null_mut()
}

// ========================================================================
// JIT emit operations (generic).
// ========================================================================

/// Set opcode
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_set(cpu: *mut cpu_ppc_t, op: *mut jit_op_t) {
    let c: *mut cpu_gen_t = (*cpu).gen;
    *(*c).jit_op_current = op;
    (*c).jit_op_current = addr_of_mut!((*op).next);
}

/// EMIT_BASIC_OPCODE
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_basic_opcode(cpu: *mut cpu_ppc_t, opcode: u_int) {
    let op: *mut jit_op_t = jit_op_get((*cpu).gen, 0, opcode);
    ppc32_op_set(cpu, op);
}

/// Trash the specified host register
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_alter_host_reg(cpu: *mut cpu_ppc_t, host_reg: c_int) {
    let op: *mut jit_op_t = jit_op_get((*cpu).gen, 0, JIT_OP_ALTER_HOST_REG);
    (*op).param[0] = host_reg;
    ppc32_op_set(cpu, op);
}

/// EMIT_INSN_OUTPUT
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_insn_output(cpu: *mut cpu_ppc_t, size_index: u_int, insn_name: *mut c_char) -> *mut jit_op_t {
    let op: *mut jit_op_t = jit_op_get((*cpu).gen, size_index as c_int, JIT_OP_INSN_OUTPUT);
    (*op).arg_ptr = null_mut();
    (*op).insn_name = insn_name;
    ppc32_op_set(cpu, op);
    op
}

/// EMIT_LOAD_GPR
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_load_gpr(cpu: *mut cpu_ppc_t, host_reg: c_int, ppc_reg: c_int) {
    let op: *mut jit_op_t = jit_op_get((*cpu).gen, 0, JIT_OP_LOAD_GPR);
    (*op).param[0] = host_reg;
    (*op).param[1] = ppc_reg;
    (*op).param[2] = host_reg;
    ppc32_op_set(cpu, op);
}

/// EMIT_STORE_GPR
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_store_gpr(cpu: *mut cpu_ppc_t, ppc_reg: c_int, host_reg: c_int) {
    let op: *mut jit_op_t = jit_op_get((*cpu).gen, 0, JIT_OP_STORE_GPR);
    (*op).param[0] = host_reg;
    (*op).param[1] = ppc_reg;
    (*op).param[2] = host_reg;
    ppc32_op_set(cpu, op);
}

/// EMIT_UPDATE_FLAGS
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_update_flags(cpu: *mut cpu_ppc_t, field: c_int, is_signed: c_int) {
    let op: *mut jit_op_t = jit_op_get((*cpu).gen, 0, JIT_OP_UPDATE_FLAGS);

    (*op).param[0] = field;
    (*op).param[1] = is_signed;

    ppc32_op_set(cpu, op);
    ppc32_update_cr_set_altered_hreg(cpu);
}

/// EMIT_REQUIRE_FLAGS
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_require_flags(cpu: *mut cpu_ppc_t, field: c_int) {
    let op: *mut jit_op_t = jit_op_get((*cpu).gen, 0, JIT_OP_REQUIRE_FLAGS);
    (*op).param[0] = field;
    ppc32_op_set(cpu, op);
}

/// EMIT_BRANCH_TARGET
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_branch_target(cpu: *mut cpu_ppc_t, b: *mut ppc32_jit_tcb_t, ia: m_uint32_t) {
    if (ia & PPC32_MIN_PAGE_MASK) == (*b).start_ia {
        let c: *mut cpu_gen_t = (*cpu).gen;
        let op: *mut jit_op_t = jit_op_get(c, 0, JIT_OP_BRANCH_TARGET);
        let pos: u_int = (ia & PPC32_MIN_PAGE_IMASK) >> 2;

        // Insert in head
        (*op).next = *(*c).jit_op_array.add(pos as usize);
        *(*c).jit_op_array.add(pos as usize) = op;
    }
}

/// EMIT_SET_HOST_REG_IMM32
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_op_emit_set_host_reg_imm32(cpu: *mut cpu_ppc_t, reg: c_int, val: m_uint32_t) {
    let op: *mut jit_op_t = jit_op_get((*cpu).gen, 0, JIT_OP_SET_HOST_REG_IMM32);
    (*op).param[0] = reg;
    (*op).param[1] = val as c_int;
    ppc32_op_set(cpu, op);
}
