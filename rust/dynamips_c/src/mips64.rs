//! XXX TODO: proper context save/restore for CPUs.

use crate::cpu::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::mips64_jit::*;
use crate::prelude::*;
use crate::rbtree::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::utils::*;
use crate::vm::*;

extern "C" {
    pub fn mips64_cca_cached(val: m_uint8_t) -> c_int;
    pub fn mips64_clear_irq(cpu: *mut cpu_mips_t, irq: m_uint8_t);
    pub fn mips64_dump_regs(cpu: *mut cpu_gen_t);
    #[cfg(feature = "USE_UNSTABLE")]
    pub fn mips64_general_exception(cpu: *mut cpu_mips_t, exc_code: u_int);
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub fn mips64_trigger_exception(cpu: *mut cpu_mips_t, exc_code: u_int, bd_slot: c_int);
    pub fn mips64_trigger_irq(cpu: *mut cpu_mips_t);
    pub fn mips64_trigger_timer_irq(cpu: *mut cpu_mips_t);
    pub fn mips64_update_irq_flag(cpu: *mut cpu_mips_t);
}

pub type tlb_entry_t = tlb_entry;
pub type mips_cp0_t = mips_cp0;
pub type mips_cp1_t = mips_cp1;
pub type cpu_mips_t = cpu_mips;

// MIPS General Purpose Registers
pub const MIPS_GPR_ZERO: usize = 0; //  zero
pub const MIPS_GPR_AT: usize = 1; //  at
pub const MIPS_GPR_V0: usize = 2; //  v0
pub const MIPS_GPR_V1: usize = 3; //  v1
pub const MIPS_GPR_A0: usize = 4; //  a0
pub const MIPS_GPR_A1: usize = 5; //  a1
pub const MIPS_GPR_A2: usize = 6; //  a2
pub const MIPS_GPR_A3: usize = 7; //  a3
pub const MIPS_GPR_T0: usize = 8; //  t0
pub const MIPS_GPR_T1: usize = 9; //  t1
pub const MIPS_GPR_T2: usize = 10; //  t2
pub const MIPS_GPR_T3: usize = 11; //  t3
pub const MIPS_GPR_T4: usize = 12; //  t4
pub const MIPS_GPR_T5: usize = 13; //  t5
pub const MIPS_GPR_T6: usize = 14; //  t6
pub const MIPS_GPR_T7: usize = 15; //  t7
pub const MIPS_GPR_S0: usize = 16; //  s0
pub const MIPS_GPR_S1: usize = 17; //  s1
pub const MIPS_GPR_S2: usize = 18; //  s2
pub const MIPS_GPR_S3: usize = 19; //  s3
pub const MIPS_GPR_S4: usize = 20; //  s4
pub const MIPS_GPR_S5: usize = 21; //  s5
pub const MIPS_GPR_S6: usize = 22; //  s6
pub const MIPS_GPR_S7: usize = 23; //  s7
pub const MIPS_GPR_T8: usize = 24; //  t8
pub const MIPS_GPR_T9: usize = 25; //  t9
pub const MIPS_GPR_K0: usize = 26; //  k0
pub const MIPS_GPR_K1: usize = 27; //  k1
pub const MIPS_GPR_GP: usize = 28; //  gp
pub const MIPS_GPR_SP: usize = 29; //  sp
pub const MIPS_GPR_FP: usize = 30; //  fp
pub const MIPS_GPR_RA: usize = 31; //  ra

/// Coprocessor 0 (System Coprocessor) Register definitions
pub const MIPS_CP0_INDEX: usize = 0; // TLB Index
pub const MIPS_CP0_RANDOM: usize = 1; // TLB Random
pub const MIPS_CP0_TLB_LO_0: usize = 2; // TLB Entry Lo0
pub const MIPS_CP0_TLB_LO_1: usize = 3; // TLB Entry Lo1
pub const MIPS_CP0_CONTEXT: usize = 4; // Kernel PTE pointer
pub const MIPS_CP0_PAGEMASK: usize = 5; // TLB Page Mask
pub const MIPS_CP0_WIRED: usize = 6; // TLB Wired
pub const MIPS_CP0_INFO: usize = 7; // Info (RM7000)
pub const MIPS_CP0_BADVADDR: usize = 8; // Bad Virtual Address
pub const MIPS_CP0_COUNT: usize = 9; // Count
pub const MIPS_CP0_TLB_HI: usize = 10; // TLB Entry Hi
pub const MIPS_CP0_COMPARE: usize = 11; // Timer Compare
pub const MIPS_CP0_STATUS: usize = 12; // Status
pub const MIPS_CP0_CAUSE: usize = 13; // Cause
pub const MIPS_CP0_EPC: usize = 14; // Exception PC
pub const MIPS_CP0_PRID: usize = 15; // Proc Rev ID
pub const MIPS_CP0_CONFIG: usize = 16; // Configuration
pub const MIPS_CP0_LLADDR: usize = 17; // Load/Link address
pub const MIPS_CP0_WATCHLO: usize = 18; // Low Watch address
pub const MIPS_CP0_WATCHHI: usize = 19; // High Watch address
pub const MIPS_CP0_XCONTEXT: usize = 20; // Extended context
pub const MIPS_CP0_ECC: usize = 26; // ECC and parity
pub const MIPS_CP0_CACHERR: usize = 27; // Cache Err/Status
pub const MIPS_CP0_TAGLO: usize = 28; // Cache Tag Lo
pub const MIPS_CP0_TAGHI: usize = 29; // Cache Tag Hi
pub const MIPS_CP0_ERR_EPC: usize = 30; // Error exception PC

/// CP0 Set 1 Registers (R7000)
pub const MIPS_CP0_S1_CONFIG: usize = 16; // Configuration Register
pub const MIPS_CP0_S1_IPLLO: usize = 18; // Priority level for IRQ [7:0]
pub const MIPS_CP0_S1_IPLHI: usize = 19; // Priority level for IRQ [15:8]
pub const MIPS_CP0_S1_INTCTL: usize = 20; // Interrupt Control
pub const MIPS_CP0_S1_DERRADDR0: usize = 26; // Imprecise Error Address
pub const MIPS_CP0_S1_DERRADDR1: usize = 27; // Imprecise Error Address

/// CP0 Status Register
pub const MIPS_CP0_STATUS_CU0: m_uint32_t = 0x10000000;
pub const MIPS_CP0_STATUS_CU1: m_uint32_t = 0x20000000;
pub const MIPS_CP0_STATUS_BEV: m_uint32_t = 0x00400000;
pub const MIPS_CP0_STATUS_TS: m_uint32_t = 0x00200000;
pub const MIPS_CP0_STATUS_SR: m_uint32_t = 0x00100000;
pub const MIPS_CP0_STATUS_CH: m_uint32_t = 0x00040000;
pub const MIPS_CP0_STATUS_CE: m_uint32_t = 0x00020000;
pub const MIPS_CP0_STATUS_DE: m_uint32_t = 0x00010000;
pub const MIPS_CP0_STATUS_RP: m_uint32_t = 0x08000000;
pub const MIPS_CP0_STATUS_FR: m_uint32_t = 0x04000000;
pub const MIPS_CP0_STATUS_RE: m_uint32_t = 0x02000000;
pub const MIPS_CP0_STATUS_KX: m_uint32_t = 0x00000080;
pub const MIPS_CP0_STATUS_SX: m_uint32_t = 0x00000040;
pub const MIPS_CP0_STATUS_UX: m_uint32_t = 0x00000020;
pub const MIPS_CP0_STATUS_KSU: m_uint32_t = 0x00000018;
pub const MIPS_CP0_STATUS_ERL: m_uint32_t = 0x00000004;
pub const MIPS_CP0_STATUS_EXL: m_uint32_t = 0x00000002;
pub const MIPS_CP0_STATUS_IE: m_uint32_t = 0x00000001;
pub const MIPS_CP0_STATUS_IMASK7: m_uint32_t = 0x00008000;
pub const MIPS_CP0_STATUS_IMASK6: m_uint32_t = 0x00004000;
pub const MIPS_CP0_STATUS_IMASK5: m_uint32_t = 0x00002000;
pub const MIPS_CP0_STATUS_IMASK4: m_uint32_t = 0x00001000;
pub const MIPS_CP0_STATUS_IMASK3: m_uint32_t = 0x00000800;
pub const MIPS_CP0_STATUS_IMASK2: m_uint32_t = 0x00000400;
pub const MIPS_CP0_STATUS_IMASK1: m_uint32_t = 0x00000200;
pub const MIPS_CP0_STATUS_IMASK0: m_uint32_t = 0x00000100;

pub const MIPS_CP0_STATUS_DS_MASK: m_uint32_t = 0x00770000;
pub const MIPS_CP0_STATUS_CU_MASK: m_uint32_t = 0xF0000000;
pub const MIPS_CP0_STATUS_IMASK: m_uint32_t = 0x0000FF00;

/// Addressing mode: Kernel, Supervisor and User
pub const MIPS_CP0_STATUS_KSU_SHIFT: u_int = 0x03;
pub const MIPS_CP0_STATUS_KSU_MASK: u_int = 0x03;

pub const MIPS_CP0_STATUS_KM: u_int = 0x00;
pub const MIPS_CP0_STATUS_SM: u_int = 0x01;
pub const MIPS_CP0_STATUS_UM: u_int = 0x10;

/// CP0 Cause register
pub const MIPS_CP0_CAUSE_BD_SLOT: m_uint32_t = 0x80000000;

pub const MIPS_CP0_CAUSE_MASK: m_uint32_t = 0x0000007C;
pub const MIPS_CP0_CAUSE_CEMASK: m_uint32_t = 0x30000000;
pub const MIPS_CP0_CAUSE_IMASK: m_uint32_t = 0x0000FF00;

pub const MIPS_CP0_CAUSE_SHIFT: c_int = 2;
pub const MIPS_CP0_CAUSE_CESHIFT: c_int = 28;
pub const MIPS_CP0_CAUSE_ISHIFT: c_int = 8;

pub const MIPS_CP0_CAUSE_INTERRUPT: u_int = 0;
pub const MIPS_CP0_CAUSE_TLB_MOD: u_int = 1;
pub const MIPS_CP0_CAUSE_TLB_LOAD: u_int = 2;
pub const MIPS_CP0_CAUSE_TLB_SAVE: u_int = 3;
pub const MIPS_CP0_CAUSE_ADDR_LOAD: u_int = 4; // ADEL
pub const MIPS_CP0_CAUSE_ADDR_SAVE: u_int = 5; // ADES
pub const MIPS_CP0_CAUSE_BUS_INSTR: u_int = 6;
pub const MIPS_CP0_CAUSE_BUS_DATA: u_int = 7;
pub const MIPS_CP0_CAUSE_SYSCALL: u_int = 8;
pub const MIPS_CP0_CAUSE_BP: u_int = 9;
pub const MIPS_CP0_CAUSE_ILLOP: u_int = 10;
pub const MIPS_CP0_CAUSE_CP_UNUSABLE: u_int = 11;
pub const MIPS_CP0_CAUSE_OVFLW: u_int = 12;
pub const MIPS_CP0_CAUSE_TRAP: u_int = 13;
pub const MIPS_CP0_CAUSE_VC_INSTR: u_int = 14; // Virtual Coherency
pub const MIPS_CP0_CAUSE_FPE: u_int = 15;
pub const MIPS_CP0_CAUSE_WATCH: u_int = 23;
pub const MIPS_CP0_CAUSE_VC_DATA: u_int = 31; // Virtual Coherency

pub const MIPS_CP0_CAUSE_IBIT7: m_uint32_t = 0x00008000;
pub const MIPS_CP0_CAUSE_IBIT6: m_uint32_t = 0x00004000;
pub const MIPS_CP0_CAUSE_IBIT5: m_uint32_t = 0x00002000;
pub const MIPS_CP0_CAUSE_IBIT4: m_uint32_t = 0x00001000;
pub const MIPS_CP0_CAUSE_IBIT3: m_uint32_t = 0x00000800;
pub const MIPS_CP0_CAUSE_IBIT2: m_uint32_t = 0x00000400;
pub const MIPS_CP0_CAUSE_IBIT1: m_uint32_t = 0x00000200;
pub const MIPS_CP0_CAUSE_IBIT0: m_uint32_t = 0x00000100;

/// TLB masks and shifts
pub const MIPS_TLB_PAGE_MASK: m_uint64_t = 0x01ffe000;
pub const MIPS_TLB_PAGE_SHIFT: c_int = 13;
pub const MIPS_TLB_VPN2_MASK_32: m_uint64_t = 0xffffe000;
pub const MIPS_TLB_VPN2_MASK_64: m_uint64_t = 0xc00000ffffffe000;
pub const MIPS_TLB_PFN_MASK: m_uint32_t = 0x3fffffc0;
pub const MIPS_TLB_ASID_MASK: m_uint32_t = 0x000000ff; // "asid" in EntryHi
pub const MIPS_TLB_G_MASK: m_uint32_t = 0x00001000; // "Global" in EntryHi
pub const MIPS_TLB_V_MASK: m_uint32_t = 0x2; // "Valid" in EntryLo
pub const MIPS_TLB_D_MASK: m_uint32_t = 0x4; // "Dirty" in EntryLo
pub const MIPS_TLB_C_MASK: m_uint32_t = 0x38; // Page Coherency Attribute
pub const MIPS_TLB_C_SHIFT: c_int = 3;

pub const MIPS_CP0_LO_G_MASK: m_uint64_t = 0x00000001; // "Global" in Lo0/1 reg
pub const MIPS_CP0_HI_SAFE_MASK: m_uint64_t = 0xffffe0ff; // Safety mask for Hi reg
pub const MIPS_CP0_LO_SAFE_MASK: m_uint64_t = 0x7fffffff; // Safety mask for Lo reg

/// results for TLB lookups // TODO enum
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_LOOKUP_OK: c_int = 0; // Entry found
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_LOOKUP_INVALID: c_int = 1; // Invalid entry found
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_LOOKUP_MISS: c_int = 2; // No matching entry found
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_LOOKUP_MOD: c_int = 3; // Read-only

/// Minimum page size: 4 Kb
pub const MIPS_MIN_PAGE_SHIFT: c_int = 12;
pub const MIPS_MIN_PAGE_SIZE: size_t = 1 << MIPS_MIN_PAGE_SHIFT;
pub const MIPS_MIN_PAGE_IMASK: m_uint64_t = MIPS_MIN_PAGE_SIZE as m_uint64_t - 1;
pub const MIPS_MIN_PAGE_MASK: m_uint64_t = 0xfffffffffffff000;

/// Number of GPR (general purpose registers)
pub const MIPS64_GPR_NR: usize = 32;

/// Number of registers in CP0
pub const MIPS64_CP0_REG_NR: usize = 32;

/// Number of registers in CP1
pub const MIPS64_CP1_REG_NR: usize = 32;

/// Number of TLB entries
pub const MIPS64_TLB_STD_ENTRIES: usize = 48;
pub const MIPS64_TLB_MAX_ENTRIES: usize = 64;
pub const MIPS64_TLB_IDX_MASK: m_uint64_t = 0x3f; // 6 bits

/// Enable the 64 TLB entries for R7000 CPU
pub const MIPS64_R7000_TLB64_ENABLE: m_uint32_t = 0x20000000;

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

/// MIPS general purpose registers names
#[rustfmt::skip]
#[no_mangle]
pub static mut mips64_gpr_reg_names: [*mut c_char; MIPS64_GPR_NR] = [
    cstr!("zr"), cstr!("at"), cstr!("v0"), cstr!("v1"), cstr!("a0"), cstr!("a1"), cstr!("a2"), cstr!("a3"),
    cstr!("t0"), cstr!("t1"), cstr!("t2"), cstr!("t3"), cstr!("t4"), cstr!("t5"), cstr!("t6"), cstr!("t7"),
    cstr!("s0"), cstr!("s1"), cstr!("s2"), cstr!("s3"), cstr!("s4"), cstr!("s5"), cstr!("s6"), cstr!("s7"),
    cstr!("t8"), cstr!("t9"), cstr!("k0"), cstr!("k1"), cstr!("gp"), cstr!("sp"), cstr!("fp"), cstr!("ra"),
];

/// Timer IRQ
#[no_mangle]
pub extern "C" fn mips64_timer_irq_run(cpu: *mut c_void) -> *mut c_void {
    unsafe {
        let cpu: *mut cpu_mips_t = cpu.cast::<_>();
        let mut umutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
        let mut ucond: libc::pthread_cond_t = libc::PTHREAD_COND_INITIALIZER;
        let mut t_spc: libc::timespec = zeroed::<_>();
        let mut expire: m_tmcnt_t;

        let interval: u_int = 1000000 / (*cpu).timer_irq_freq;
        let threshold: u_int = (*cpu).timer_irq_freq * 10;
        expire = m_gettime_usec() + interval as m_tmcnt_t;

        while (*(*cpu).gen).state.get() != CPU_STATE_HALTED {
            libc::pthread_mutex_lock(addr_of_mut!(umutex));
            t_spc.tv_sec = (expire / 1000000) as libc::time_t;
            t_spc.tv_nsec = ((expire % 1000000) * 1000) as _;
            libc::pthread_cond_timedwait(addr_of_mut!(ucond), addr_of_mut!(umutex), addr_of!(t_spc));
            libc::pthread_mutex_unlock(addr_of_mut!(umutex));

            if likely((*cpu).irq_disable.get() == 0) && likely((*(*cpu).gen).state.get() == CPU_STATE_RUNNING) {
                (*cpu).timer_irq_pending.set((*cpu).timer_irq_pending.get() + 1);

                if unlikely((*cpu).timer_irq_pending.get() > threshold) {
                    (*cpu).timer_irq_pending.set(0);
                    (*cpu).timer_drift += 1;
                    if false {
                        libc::printf(cstr!("Timer IRQ not accurate (%u pending IRQ): reduce the \"--timer-irq-check-itv\" parameter (current value: %u)\n"), (*cpu).timer_irq_pending, (*cpu).timer_irq_check_itv);
                    }
                }
            }

            expire += interval as m_tmcnt_t;
        }

        null_mut()
    }
}

/// Virtual breakpoint
#[no_mangle]
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub unsafe extern "C" fn mips64_run_breakpoint(cpu: *mut cpu_mips_t) {
    cpu_log!((*cpu).gen, cstr!("BREAKPOINT"), cstr!("Virtual breakpoint reached at PC=0x%llx\n"), (*cpu).pc);

    libc::printf(cstr!("[[[ Virtual Breakpoint reached at PC=0x%llx RA=0x%llx]]]\n"), (*cpu).pc, (*cpu).gpr[MIPS_GPR_RA]);

    mips64_dump_regs((*cpu).gen);
    memlog_dump((*cpu).gen);
}
