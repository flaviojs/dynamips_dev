//! XXX TODO: proper context save/restore for CPUs.

use crate::cpu::*;
use crate::dynamips_common::*;
use crate::mips64_jit::*;
use crate::prelude::*;
use crate::rbtree::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::utils::*;
use crate::vm::*;

pub type tlb_entry_t = tlb_entry;
pub type mips_cp0_t = mips_cp0;
pub type mips_cp1_t = mips_cp1;
pub type cpu_mips_t = cpu_mips;

/// Number of GPR (general purpose registers)
pub const MIPS64_GPR_NR: usize = 32;

/// Number of registers in CP0
pub const MIPS64_CP0_REG_NR: usize = 32;

/// Number of registers in CP1
pub const MIPS64_CP1_REG_NR: usize = 32;

pub const MIPS64_TLB_MAX_ENTRIES: usize = 64;

// Memory operations // TODO enum
pub const MIPS_MEMOP_LOOKUP: c_uint = 0;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_IFETCH: c_uint = 1;

#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LB: c_uint = 2;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LB: c_uint = 1;
pub const MIPS_MEMOP_LBU: c_uint = MIPS_MEMOP_LB + 1;
pub const MIPS_MEMOP_LH: c_uint = MIPS_MEMOP_LB + 2;
pub const MIPS_MEMOP_LHU: c_uint = MIPS_MEMOP_LB + 3;
pub const MIPS_MEMOP_LW: c_uint = MIPS_MEMOP_LB + 4;
pub const MIPS_MEMOP_LWU: c_uint = MIPS_MEMOP_LB + 5;
pub const MIPS_MEMOP_LD: c_uint = MIPS_MEMOP_LB + 6;
pub const MIPS_MEMOP_SB: c_uint = MIPS_MEMOP_LB + 7;
pub const MIPS_MEMOP_SH: c_uint = MIPS_MEMOP_LB + 8;
pub const MIPS_MEMOP_SW: c_uint = MIPS_MEMOP_LB + 9;
pub const MIPS_MEMOP_SD: c_uint = MIPS_MEMOP_LB + 10;

pub const MIPS_MEMOP_LWL: c_uint = MIPS_MEMOP_LB + 11;
pub const MIPS_MEMOP_LWR: c_uint = MIPS_MEMOP_LB + 12;
pub const MIPS_MEMOP_LDL: c_uint = MIPS_MEMOP_LB + 13;
pub const MIPS_MEMOP_LDR: c_uint = MIPS_MEMOP_LB + 14;
pub const MIPS_MEMOP_SWL: c_uint = MIPS_MEMOP_LB + 15;
pub const MIPS_MEMOP_SWR: c_uint = MIPS_MEMOP_LB + 16;
pub const MIPS_MEMOP_SDL: c_uint = MIPS_MEMOP_LB + 17;
pub const MIPS_MEMOP_SDR: c_uint = MIPS_MEMOP_LB + 18;

pub const MIPS_MEMOP_LL: c_uint = MIPS_MEMOP_LB + 19;
pub const MIPS_MEMOP_SC: c_uint = MIPS_MEMOP_LB + 20;

pub const MIPS_MEMOP_LDC1: c_uint = MIPS_MEMOP_LB + 21;
pub const MIPS_MEMOP_SDC1: c_uint = MIPS_MEMOP_LB + 22;

pub const MIPS_MEMOP_CACHE: c_uint = MIPS_MEMOP_LB + 23;

pub const MIPS_MEMOP_MAX: usize = MIPS_MEMOP_LB as usize + 24;

/// Maximum number of breakpoints
pub const MIPS64_MAX_BREAKPOINTS: usize = 8;

/// Memory operation function prototype
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub type mips_memop_fn = Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, reg: u_int)>;

/// TLB entry definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tlb_entry {
    pub mask: m_uint64_t,
    pub hi: m_uint64_t,
    pub lo0: m_uint64_t,
    pub lo1: m_uint64_t,
}

/// System Coprocessor (CP0) definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips_cp0 {
    pub reg: [m_uint64_t; MIPS64_CP0_REG_NR],
    pub tlb: [tlb_entry_t; MIPS64_TLB_MAX_ENTRIES],

    /// Number of TLB entries
    pub tlb_entries: u_int,

    /// Extensions for R7000 CP0 Set1
    pub ipl_lo: m_uint32_t,
    pub ipl_hi: m_uint32_t,
    pub int_ctl: m_uint32_t,
    pub derraddr0: m_uint32_t,
    pub derraddr1: m_uint32_t,
}

/// FPU Coprocessor (CP1) definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips_cp1 {
    pub reg: [m_uint64_t; MIPS64_CP1_REG_NR],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union cpu_mips_mts_u {
    pub mts32_cache: *mut mts32_entry_t,
    pub mts64_cache: *mut mts64_entry_t,
}

/// MIPS CPU definition
#[repr(C)]
#[derive(Copy, Clone)]
pub struct cpu_mips {
    /// MTS32/MTS64 caches
    pub mts_u: cpu_mips_mts_u,

    /// Virtual version of CP0 Compare Register
    pub cp0_virt_cnt_reg: m_uint32_t,
    pub cp0_virt_cmp_reg: m_uint32_t,

    /// General Purpose Registers, Pointer Counter, LO/HI, IRQ
    pub irq_pending: m_uint32_t,
    pub irq_cause: m_uint32_t,
    pub ll_bit: m_uint32_t,
    pub pc: m_uint64_t,
    pub gpr: [m_uint64_t; MIPS64_GPR_NR],
    pub lo: m_uint64_t,
    pub hi: m_uint64_t,
    pub ret_pc: m_uint64_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub exec_state: m_uint32_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub bd_slot: u_int,

    /// Code page translation cache
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub exec_blk_map: *mut *mut mips64_jit_tcb_t,

    /// Virtual address to physical page translation
    #[cfg_attr(feature = "fastcall", abi("fastcall"))]
    pub translate: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, phys_page: *mut m_uint32_t) -> c_int>,

    /// Memory access functions
    pub mem_op_fn: [mips_memop_fn; MIPS_MEMOP_MAX],

    /// Memory lookup function (to load ELF image,...)
    pub mem_op_lookup: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t) -> *mut c_void>,
    /// and instruction fetch
    #[cfg(feature = "USE_UNSTABLE")]
    pub mem_op_ifetch: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t) -> *mut c_void>,

    /// System coprocessor (CP0)
    pub cp0: mips_cp0_t,

    /// FPU (CP1)
    pub fpu: mips_cp1_t,

    /// Address bus mask for physical addresses
    pub addr_bus_mask: m_uint64_t,

    /// IRQ counters and cause
    pub irq_count: m_uint64_t,
    pub timer_irq_count: m_uint64_t,
    pub irq_fp_count: m_uint64_t,
    pub irq_lock: libc::pthread_mutex_t,

    /// Current and free lists of translated code blocks
    pub tcb_list: *mut mips64_jit_tcb_t,
    pub tcb_last: *mut mips64_jit_tcb_t,
    pub tcb_free_list: *mut mips64_jit_tcb_t,

    /// Executable page area
    pub exec_page_area: *mut ::std::os::raw::c_void,
    pub exec_page_area_size: size_t,
    pub exec_page_count: size_t,
    pub exec_page_alloc: size_t,
    pub exec_page_free_list: *mut insn_exec_page_t,
    pub exec_page_array: *mut insn_exec_page_t,

    /// Idle PC value
    pub idle_pc: Volatile<m_uint64_t>,

    /// Timer IRQs
    pub timer_irq_pending: Volatile<u_int>,
    pub timer_irq_freq: u_int,
    pub timer_irq_check_itv: u_int,
    pub timer_drift: u_int,

    /// IRQ disable flag
    pub irq_disable: Volatile<u_int>,

    /// IRQ idling preemption
    pub irq_idle_preempt: [u_int; 8],

    /// Generic CPU instance pointer
    pub gen: *mut cpu_gen_t,

    /// VM instance
    pub vm: *mut vm_instance_t,

    /// non-JIT mode instruction counter
    pub insn_exec_count: m_uint64_t,

    /// MTS map/unmap/rebuild operations
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub mts_map: ::std::option::Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, paddr: m_uint64_t, len: m_uint32_t, cache_access: ::std::os::raw::c_int, tlb_index: ::std::os::raw::c_int)>,

    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub mts_unmap: ::std::option::Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, len: m_uint32_t, val: m_uint32_t, tlb_index: ::std::os::raw::c_int)>,

    /// MTS invalidate/shutdown operations
    #[cfg(feature = "USE_UNSTABLE")]
    pub mts_invalidate: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t)>,

    pub mts_shutdown: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t)>,

    /// MTS cache statistics
    pub mts_misses: m_uint64_t,
    pub mts_lookups: m_uint64_t,

    /// JIT flush method
    pub jit_flush_method: u_int,

    /// Number of compiled pages
    pub compiled_pages: u_int,

    /// Fast memory operations use
    pub fast_memop: u_int,

    /// Direct block jump
    pub exec_blk_direct_jump: u_int,

    /// Address mode (32 or 64 bits)
    pub addr_mode: u_int,

    /// Current exec page (non-JIT) info
    pub njm_exec_page: m_uint64_t,
    pub njm_exec_ptr: *mut mips_insn_t,

    /// Performance counter (number of instructions executed by CPU)
    pub perf_counter: m_uint32_t,

    /// Breakpoints
    pub breakpoints: [m_uint64_t; MIPS64_MAX_BREAKPOINTS],
    pub breakpoints_enabled: u_int,

    /// Symtrace
    pub sym_trace: ::std::os::raw::c_int,
    pub sym_tree: *mut rbtree_tree,

    /// XXX
    #[cfg(feature = "USE_UNSTABLE")]
    pub current_tb: *mut cpu_tb_t,
}
