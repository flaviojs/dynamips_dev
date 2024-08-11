//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! MIPS64 JIT compiler.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::mips64::*;
use crate::sbox::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::utils::*;

#[cfg(not(feature = "USE_UNSTABLE"))]
pub type mips64_jit_tcb_t = mips64_jit_tcb;
#[cfg(feature = "USE_UNSTABLE")]
pub type mips64_jit_tcb_t = c_void;

/// Size of executable page area (in Mb)
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_EXEC_AREA_SIZE: usize = 64;
#[cfg(all(not(feature = "USE_UNSTABLE"), if_0))]
pub const MIPS_EXEC_AREA_SIZE: usize = 16; // FIXME this is the correct value for cygwin, but rust does not have a cygwin target yet

/// Buffer size for JIT code generation
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_JIT_BUFSIZE: usize = 32768;

/// Maximum number of X86 chunks
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_JIT_MAX_CHUNKS: usize = 32;

/// Size of hash for PC lookup
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_JIT_PC_HASH_BITS: c_int = 16;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_JIT_PC_HASH_MASK: m_uint32_t = (1 << MIPS_JIT_PC_HASH_BITS) - 1;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_JIT_PC_HASH_SIZE: m_uint32_t = 1 << MIPS_JIT_PC_HASH_BITS;

/// Instruction jump patch
#[cfg(not(feature = "USE_UNSTABLE"))]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips64_insn_patch {
    pub jit_insn: *mut u_char,
    pub mips_pc: m_uint64_t,
}

/// Instruction patch table
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS64_INSN_PATCH_TABLE_SIZE: usize = 32;

#[cfg(not(feature = "USE_UNSTABLE"))]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips64_jit_patch_table {
    pub patches: [mips64_insn_patch; MIPS64_INSN_PATCH_TABLE_SIZE],
    pub cur_patch: u_int,
    pub next: *mut mips64_jit_patch_table,
}

/// MIPS64 translated code block
#[cfg(not(feature = "USE_UNSTABLE"))]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips64_jit_tcb {
    pub start_pc: m_uint64_t,
    pub jit_insn_ptr: *mut *mut u_char,
    pub acc_count: m_uint64_t,
    pub mips_code: *mut mips_insn_t,
    pub mips_trans_pos: u_int,
    pub jit_chunk_pos: u_int,
    pub jit_ptr: *mut u_char,
    pub jit_buffer: *mut insn_exec_page_t,
    pub jit_chunks: [*mut insn_exec_page_t; MIPS_JIT_MAX_CHUNKS],
    pub patch_table: *mut mips64_jit_patch_table,
    pub prev: *mut mips64_jit_tcb_t,
    pub next: *mut mips64_jit_tcb_t,
    #[cfg(feature = "USE_DEBUG_BLOCK_TIMESTAMP")]
    tm_first_use: m_uint64_t,
    #[cfg(feature = "USE_DEBUG_BLOCK_TIMESTAMP")]
    tm_last_use: m_uint64_t,
}

/// Size of hash for virtual address lookup
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_JIT_VIRT_HASH_BITS: c_int = 16;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_JIT_VIRT_HASH_MASK: m_uint32_t = (1 << MIPS_JIT_VIRT_HASH_BITS) - 1;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_JIT_VIRT_HASH_SIZE: m_uint32_t = 1 << MIPS_JIT_VIRT_HASH_BITS;

/// Size of hash for physical lookup
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_JIT_PHYS_HASH_BITS: c_int = 16;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_JIT_PHYS_HASH_MASK: m_uint32_t = (1 << MIPS_JIT_PHYS_HASH_BITS) - 1;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_JIT_PHYS_HASH_SIZE: m_uint32_t = 1 << MIPS_JIT_PHYS_HASH_BITS;

/// MIPS instruction recognition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips64_insn_tag {
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub emit: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, arg1: *mut mips64_jit_tcb_t, arg2: mips_insn_t) -> c_int>,
    #[cfg(feature = "USE_UNSTABLE")]
    pub emit: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, arg1: *mut cpu_tc_t, arg2: mips_insn_t) -> c_int>,
    pub mask: m_uint32_t,
    pub value: m_uint32_t,
    pub delay_slot: c_int,
}
impl mips64_insn_tag {
    pub const fn null() -> Self {
        Self { emit: None, mask: 0, value: 0, delay_slot: 0 }
    }
}

/// MIPS jump instruction (for block scan)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips64_insn_jump {
    pub name: *mut c_char,
    pub mask: m_uint32_t,
    pub value: m_uint32_t,
    pub offset_bits: c_int,
    pub relative: c_int,
}

/// Get the JIT instruction pointer in a translated block
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_get_host_ptr(b: *mut mips64_jit_tcb_t, vaddr: m_uint64_t) -> *mut u_char {
    let offset: m_uint32_t = (((vaddr as m_uint32_t as m_uint64_t) & MIPS_MIN_PAGE_IMASK) >> 2) as m_uint32_t;
    *(*b).jit_insn_ptr.add(offset as usize)
}

/// Get the JIT instruction pointer in a translated block
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tc_get_host_ptr(tc: *mut cpu_tc_t, vaddr: m_uint64_t) -> *mut u_char {
    let offset: m_uint32_t = (((vaddr as m_uint32_t as m_uint64_t) & MIPS_MIN_PAGE_IMASK) >> 2) as m_uint32_t;
    *(*tc).jit_insn_ptr.add(offset as usize)
}

/// Check if the specified address belongs to the specified block
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_local_addr(block: *mut mips64_jit_tcb_t, vaddr: m_uint64_t, jit_addr: *mut *mut u_char) -> c_int {
    if (vaddr & MIPS_MIN_PAGE_MASK) == (*block).start_pc {
        *jit_addr = mips64_jit_tcb_get_host_ptr(block, vaddr);
        return 1;
    }

    0
}

/// Check if the specified address belongs to the specified block
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_local_addr(tc: *mut cpu_tc_t, vaddr: m_uint64_t, jit_addr: *mut *mut u_char) -> c_int {
    if (vaddr & MIPS_MIN_PAGE_MASK) == (*tc).vaddr {
        *jit_addr = mips64_jit_tc_get_host_ptr(tc, vaddr);
        return 1;
    }

    0
}

/// Check if PC register matches the compiled block virtual address
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_match(cpu: *mut cpu_mips_t, block: *mut mips64_jit_tcb_t) -> c_int {
    let vpage: m_uint64_t = (*cpu).pc & !MIPS_MIN_PAGE_IMASK;
    ((*block).start_pc == vpage) as c_int
}

/// Check if PC register matches the compiled block virtual address
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_tcb_match(cpu: *mut cpu_mips_t, tb: *mut cpu_tb_t) -> c_int {
    let vpage: m_uint64_t = (*cpu).pc & MIPS_MIN_PAGE_MASK;
    (((*tb).vaddr == vpage) && ((*tb).exec_state == (*cpu).exec_state)) as c_int
}

/// Compute the hash index for the specified PC value
#[cfg(not(feature = "USE_UNSTABLE"))]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_get_pc_hash(pc: m_uint64_t) -> m_uint32_t {
    let page_hash: m_uint32_t = sbox_u32((pc >> MIPS_MIN_PAGE_SHIFT) as m_uint32_t);
    (page_hash ^ (page_hash >> 12)) & MIPS_JIT_PC_HASH_MASK
}

/// Compute the hash index for the specified virtual address
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_get_virt_hash(vaddr: m_uint64_t) -> m_uint32_t {
    let page_hash: m_uint32_t = sbox_u32((vaddr >> MIPS_MIN_PAGE_SHIFT) as m_uint32_t);
    (page_hash ^ (page_hash >> 12)) & MIPS_JIT_VIRT_HASH_MASK
}

/// Compute the hash index for the specified physical page
#[cfg(feature = "USE_UNSTABLE")]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_get_phys_hash(phys_page: m_uint32_t) -> m_uint32_t {
    let page_hash: m_uint32_t = sbox_u32(phys_page);
    (page_hash ^ (page_hash >> 12)) & MIPS_JIT_PHYS_HASH_MASK
}

/// Find a JIT block matching a physical page
#[cfg(feature = "USE_UNSTABLE")]
#[inline]
#[no_mangle]
pub unsafe extern "C" fn mips64_jit_find_by_phys_page(cpu: *mut cpu_mips_t, phys_page: m_uint32_t) -> *mut cpu_tb_t {
    let page_hash: m_uint32_t = mips64_jit_get_phys_hash(phys_page);

    let mut tb: *mut cpu_tb_t = *(*(*cpu).gen).tb_phys_hash.add(page_hash as usize);
    while !tb.is_null() {
        if (*tb).phys_page == phys_page {
            return tb;
        }
        tb = (*tb).phys_next;
    }

    null_mut()
}
