//! Cisco router simulation platform.
//! Copyright (c) 2008 Christophe Fillot (cf@utc.fr)
//!
//! Translation Sharing Groups.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::utils::*;
use crate::vm::*;

pub type cpu_tb_t = cpu_tb;
pub type cpu_tc_t = cpu_tc;

/// Checksum type
pub type tsg_checksum_t = m_uint64_t;

/// Instruction jump patch
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct insn_patch {
    pub next: *mut insn_patch,
    pub jit_insn: *mut u_char,
    pub vaddr: m_uint64_t,
}

/// Instruction patch table
pub const INSN_PATCH_TABLE_SIZE: usize = 32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct insn_patch_table {
    pub next: *mut insn_patch_table,
    pub patches: [insn_patch; INSN_PATCH_TABLE_SIZE],
    pub cur_patch: u_int,
}

/// Flags for CPU Tranlation Blocks (TB)
pub const TB_FLAG_SMC: u_int = 0x01; // Self-modifying code
pub const TB_FLAG_RECOMP: u_int = 0x02; // Page being recompiled
pub const TB_FLAG_NOJIT: u_int = 0x04; // Page not supported for JIT
pub const TB_FLAG_VALID: u_int = 0x08;

/// Don't use translated code to execute the page
pub const TB_FLAG_NOTRANS: u_int = TB_FLAG_SMC | TB_FLAG_RECOMP | TB_FLAG_NOJIT;

/// CPU Translation Block
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_tb {
    pub flags: u_int,
    pub vaddr: m_uint64_t,
    pub exec_state: m_uint32_t,
    pub checksum: tsg_checksum_t,

    pub acc_count: m_uint64_t,
    pub target_code: *mut c_void,

    pub tb_pprev: *mut *mut cpu_tb_t,
    pub tb_next: *mut cpu_tb_t,

    /// Translated Code (can be shared among multiple CPUs)
    pub tc: *mut cpu_tc_t,
    pub tb_dl_pprev: *mut *mut cpu_tb_t,
    pub tb_dl_next: *mut cpu_tb_t,

    /// Virtual page hash
    pub virt_hash: m_uint32_t,

    /// Physical page information
    pub phys_page: m_uint32_t,
    pub phys_hash: m_uint32_t,
    pub phys_pprev: *mut *mut cpu_tb_t,
    pub phys_next: *mut cpu_tb_t,

    #[cfg(feature = "USE_DEBUG_BLOCK_TIMESTAMP")]
    tm_first_use: m_uint64_t,
    #[cfg(feature = "USE_DEBUG_BLOCK_TIMESTAMP")]
    tm_last_use: m_uint64_t,
}

/// Maximum exec pages per TC descriptor
pub const TC_MAX_CHUNKS: usize = 32;

/// TC descriptor flags
pub const TC_FLAG_REMOVAL: u_int = 0x01; // Descriptor marked for removal
pub const TC_FLAG_VALID: u_int = 0x02;

/// CPU Translated Code
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_tc {
    pub checksum: tsg_checksum_t,
    pub vaddr: m_uint64_t,
    pub exec_state: m_uint32_t,
    pub flags: u_int,

    /// Temporarily used during the translation
    pub target_code: *mut c_void,

    pub jit_insn_ptr: *mut *mut u_char,
    pub jit_chunk_pos: u_int,
    pub jit_chunks: [*mut insn_exec_page_t; TC_MAX_CHUNKS],

    /// Current JIT buffer
    pub jit_buffer: *mut insn_exec_page_t,
    pub jit_ptr: *mut u_char,

    /// Patch table
    pub patch_table: *mut insn_patch_table,

    /// Translation position in target code
    pub trans_pos: u_int,

    /// 1024 instructions per page, one bit per instruction
    pub target_bitmap: [m_uint32_t; 32],
    pub target_undef_cnt: m_uint32_t,

    /// Reference count
    pub ref_count: c_int,

    /// TB list referring to this translated code / exec pages
    pub tb_list: *mut cpu_tb_t,

    /// Linked list for hash table referencing
    pub hash_pprev: *mut *mut cpu_tc_t,
    pub hash_next: *mut cpu_tc_t,

    /// Linked list for single-CPU referencement (ref_count=1)
    pub sc_pprev: *mut *mut cpu_tc_t,
    pub sc_next: *mut cpu_tc_t,
}

unsafe fn TC_TARGET_BITMAP_INDEX(x: m_uint32_t) -> m_uint32_t {
    (x >> 7) & 0x1F
}
unsafe fn TC_TARGET_BITMAP_POS(x: m_uint32_t) -> m_uint32_t {
    (x >> 2) & 0x1F
}

/// Mark the specified vaddr as a target for further recompiling
#[inline]
#[no_mangle]
pub unsafe extern "C" fn tc_set_target_bit(tc: *mut cpu_tc_t, vaddr: m_uint32_t) {
    let index: c_int = TC_TARGET_BITMAP_INDEX(vaddr) as c_int;
    let pos: c_int = TC_TARGET_BITMAP_POS(vaddr) as c_int;

    (*tc).target_bitmap[index as usize] |= 1 << pos;
}

/// Returns TRUE if the specified vaddr is in the target bitmap
#[inline]
#[no_mangle]
pub unsafe extern "C" fn tc_get_target_bit(tc: *mut cpu_tc_t, vaddr: m_uint32_t) -> c_int {
    let index: c_int = TC_TARGET_BITMAP_INDEX(vaddr) as c_int;
    let pos: c_int = TC_TARGET_BITMAP_POS(vaddr) as c_int;

    ((*tc).target_bitmap[index as usize] & (1 << pos)) as c_int
}

/// Get the JIT instruction pointer in a translated code
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn tc_get_host_ptr(tc: *mut cpu_tc_t, vaddr: m_uint64_t) -> *mut u_char {
    let offset: m_uint32_t = ((vaddr & VM_PAGE_IMASK) >> 2) as m_uint32_t;
    *(*tc).jit_insn_ptr.add(offset as usize)
}

/// Get the JIT instruction pointer in a translated block */
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn tb_get_host_ptr(tb: *mut cpu_tb_t, vaddr: m_uint64_t) -> *mut u_char {
    tc_get_host_ptr((*tb).tc, vaddr)
}

/// Lookup return codes
pub const TSG_LOOKUP_NEW: c_int = 0;
pub const TSG_LOOKUP_SHARED: c_int = 1;

/// Translation sharing group statistics
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tsg_stats {
    pub total_tc: u_int,
    pub shared_tc: u_int,
    pub shared_pages: u_int,
}

// TODO enum
pub const CPU_JIT_DISABLE_CPU: c_int = 0;
pub const CPU_JIT_ENABLE_CPU: c_int = 1;
