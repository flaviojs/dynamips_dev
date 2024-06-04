//! PowerPC (32-bit) generic routines.

use crate::cpu::*;
use crate::dynamips_common::*;
use crate::ppc32_jit::*;
use crate::ppc32_mem::*;
use crate::prelude::*;
use crate::utils::*;
use crate::vm::*;

pub type cpu_ppc_t = cpu_ppc;

/// Number of GPR (general purpose registers)
pub const PPC32_GPR_NR: usize = 32;

/// Number of registers in FPU
pub const PPC32_FPU_REG_NR: usize = 32;

/// Starting point for ROM
pub const PPC32_ROM_START: m_uint32_t = 0xfff00100;
pub const PPC32_ROM_SP: m_uint32_t = 0x00006000;

// MSR (Machine State Register)
/// Power Management
pub const PPC32_MSR_POW_MASK: m_uint32_t = 0x00060000;
/// Exception Little-Endian Mode
pub const PPC32_MSR_ILE: m_uint32_t = 0x00010000;
/// External Interrupt Enable
pub const PPC32_MSR_EE: m_uint32_t = 0x00008000;
/// Privilege Level (0=supervisor)
pub const PPC32_MSR_PR: m_uint32_t = 0x00004000;
pub const PPC32_MSR_PR_SHIFT: c_int = 14;
/// Floating-Point Available
pub const PPC32_MSR_FP: m_uint32_t = 0x00002000;
/// Machine Check Enable
pub const PPC32_MSR_ME: m_uint32_t = 0x00001000;
/// Floating-Point Exception Mode 0
pub const PPC32_MSR_FE0: m_uint32_t = 0x00000800;
/// Single-step trace enable
pub const PPC32_MSR_SE: m_uint32_t = 0x00000400;
/// Branch Trace Enable
pub const PPC32_MSR_BE: m_uint32_t = 0x00000200;
/// Floating-Point Exception Mode 1
pub const PPC32_MSR_FE1: m_uint32_t = 0x00000100;
/// Exception Prefix
pub const PPC32_MSR_IP: m_uint32_t = 0x00000040;
/// Instruction address translation
pub const PPC32_MSR_IR: m_uint32_t = 0x00000020;
/// Data address translation
pub const PPC32_MSR_DR: m_uint32_t = 0x00000010;
/// Recoverable Exception
pub const PPC32_MSR_RI: m_uint32_t = 0x00000002;
/// Little-Endian mode enable
pub const PPC32_MSR_LE: m_uint32_t = 0x00000001;

/// Number of BAT registers (8 for PowerPC 7448)
pub const PPC32_BAT_NR: usize = 8;

/// Number of segment registers
pub const PPC32_SR_NR: usize = 16;

/// Number of TLB entries for PPC405
pub const PPC405_TLB_ENTRIES: usize = 64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc405_tlb_entry {
    pub tlb_hi: m_uint32_t,
    pub tlb_lo: m_uint32_t,
    pub tid: m_uint32_t,
}

// Memory operations
/// Instruction fetch operation
pub const PPC_MEMOP_LOOKUP: c_uint = 0;
#[cfg(feature = "USE_UNSTABLE")]
pub const PPC_MEMOP_IFETCH: c_uint = 1;

/// Load operations
#[cfg(feature = "USE_UNSTABLE")]
pub const PPC_MEMOP_LBZ: c_uint = 2;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const PPC_MEMOP_LBZ: c_uint = 1;
pub const PPC_MEMOP_LHZ: c_uint = PPC_MEMOP_LBZ + 1;
pub const PPC_MEMOP_LWZ: c_uint = PPC_MEMOP_LBZ + 2;

/// Load operation with sign-extend
pub const PPC_MEMOP_LHA: c_uint = PPC_MEMOP_LBZ + 3;

/// Store operations
pub const PPC_MEMOP_STB: c_uint = PPC_MEMOP_LBZ + 4;
pub const PPC_MEMOP_STH: c_uint = PPC_MEMOP_LBZ + 5;
pub const PPC_MEMOP_STW: c_uint = PPC_MEMOP_LBZ + 6;

/// Byte-Reversed operations
pub const PPC_MEMOP_LWBR: c_uint = PPC_MEMOP_LBZ + 7;
pub const PPC_MEMOP_STWBR: c_uint = PPC_MEMOP_LBZ + 8;

/// String operations
pub const PPC_MEMOP_LSW: c_uint = PPC_MEMOP_LBZ + 9;
pub const PPC_MEMOP_STSW: c_uint = PPC_MEMOP_LBZ + 10;

/// FPU operations
pub const PPC_MEMOP_LFD: c_uint = PPC_MEMOP_LBZ + 11;
pub const PPC_MEMOP_STFD: c_uint = PPC_MEMOP_LBZ + 12;

/// ICBI - Instruction Cache Block Invalidate
pub const PPC_MEMOP_ICBI: c_uint = PPC_MEMOP_LBZ + 13;

pub const PPC_MEMOP_MAX: usize = PPC_MEMOP_LBZ as usize + 14;

/// Memory operation function prototype
#[cfg_attr(feature = "fastcall", abi("fastcall"))]
pub type ppc_memop_fn = Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t, reg: u_int)>;

/// BAT register
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_bat_reg {
    pub reg: [m_uint32_t; 2],
}

/// FPU Coprocessor definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc_fpu_t {
    pub reg: [m_uint64_t; PPC32_FPU_REG_NR],
}

/// Maximum number of breakpoints
pub const PPC32_MAX_BREAKPOINTS: usize = 8;

/// zzz
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_vtlb_entry {
    pub vaddr: m_uint32_t,
    pub haddr: m_uint32_t,
}

/// PowerPC CPU
#[repr(C)]
#[derive(Copy, Clone)]
pub struct cpu_ppc {
    /// Execution state
    #[cfg(feature = "USE_UNSTABLE")]
    pub exec_state: m_uint32_t,

    /// Instruction address
    pub ia: m_uint32_t,

    /// General Purpose registers
    pub gpr: [m_uint32_t; PPC32_GPR_NR],

    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub vtlb: [ppc32_vtlb_entry; PPC32_GPR_NR],

    /// Pending IRQ
    pub irq_pending: Volatile<m_uint32_t>,
    pub irq_check: Volatile<m_uint32_t>,

    // XER, Condition Register, Link Register, Count Register
    pub xer: m_uint32_t,
    pub lr: m_uint32_t,
    pub ctr: m_uint32_t,
    pub reserve: m_uint32_t,
    pub xer_ca: m_uint32_t,

    /// Condition Register (CR) fields
    pub cr_fields: [u_int; 8],

    /// MTS caches (Instruction+Data)
    pub mts_cache: [*mut mts32_entry_t; 2],

    // Code page translation cache and physical page mapping
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub exec_blk_map: *mut *mut ppc32_jit_tcb_t,
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub exec_phys_map: *mut *mut ppc32_jit_tcb_t,

    #[cfg(feature = "USE_UNSTABLE")]
    pub tcb_virt_hash: *mut *mut ppc32_jit_tcb_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tcb_phys_hash: *mut *mut ppc32_jit_tcb_t,

    /// Virtual address to physical page translation
    #[cfg_attr(feature = "fastcall", abi("fastcall"))]
    pub translate: Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t, cid: u_int, phys_page: *mut m_uint32_t) -> c_int>,

    /// Memory access functions
    pub mem_op_fn: [ppc_memop_fn; PPC_MEMOP_MAX],

    /// Memory lookup function (to load ELF image,...)
    pub mem_op_lookup: Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t, cid: u_int) -> *mut c_void>,
    /// and Instruction fetch
    #[cfg(feature = "USE_UNSTABLE")]
    pub mem_op_ifetch: Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t) -> *mut c_void>,

    /// MTS slow lookup function
    pub mts_slow_lookup: Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t, cid: u_int, op_code: u_int, op_size: u_int, op_type: u_int, data: *mut m_uint64_t, alt_entry: *mut mts32_entry_t) -> *mut mts32_entry_t>,

    /// IRQ counters
    pub irq_count: m_uint64_t,
    pub timer_irq_count: m_uint64_t,
    pub irq_fp_count: m_uint64_t,
    pub irq_lock: libc::pthread_mutex_t,

    /// Current and free lists of translated code blocks
    pub tcb_list: *mut ppc32_jit_tcb_t,
    pub tcb_last: *mut ppc32_jit_tcb_t,
    pub tcb_free_list: *mut ppc32_jit_tcb_t,

    /// Executable page area
    pub exec_page_area: *mut c_void,
    pub exec_page_area_size: size_t,
    pub exec_page_count: size_t,
    pub exec_page_alloc: size_t,
    pub exec_page_free_list: *mut insn_exec_page_t,
    pub exec_page_array: *mut insn_exec_page_t,

    /// Idle PC value
    pub idle_pc: Volatile<m_uint32_t>,

    /// Timer IRQs
    pub timer_irq_pending: Volatile<u_int>,
    pub timer_irq_armed: Volatile<u_int>,
    pub timer_irq_freq: u_int,
    pub timer_irq_check_itv: u_int,
    pub timer_drift: u_int,

    /// IRQ disable flag
    pub irq_disable: Volatile<u_int>,

    /// IBAT (Instruction) and DBAT (Data) registers
    pub bat: [[ppc32_bat_reg; PPC32_BAT_NR]; 2],

    /* Segment registers */
    pub sr: [m_uint32_t; PPC32_SR_NR],

    /// Page Table Address
    pub sdr1: m_uint32_t,
    pub sdr1_hptr: *mut c_void,

    /// MSR (Machine state register)
    pub msr: m_uint32_t,

    /// Interrupt Registers (SRR0/SRR1)
    pub srr0: m_uint32_t,
    pub srr1: m_uint32_t,
    pub dsisr: m_uint32_t,
    pub dar: m_uint32_t,

    /// SPRG registers
    pub sprg: [m_uint32_t; 4],

    /// PVR (Processor Version Register)
    pub pvr: m_uint32_t,

    /// Time-Base register
    pub tb: m_uint64_t,

    /// Decrementer
    pub dec: m_uint32_t,

    /// Hardware Implementation Dependent Registers
    pub hid0: m_uint32_t,
    pub hid1: m_uint32_t,

    /// String instruction position (lswi/stswi)
    pub sw_pos: u_int,

    /// PowerPC 405 TLB
    pub ppc405_tlb: [ppc405_tlb_entry; PPC405_TLB_ENTRIES],
    pub ppc405_pid: m_uint32_t,

    /// MPC860 IMMR register
    pub mpc860_immr: m_uint32_t,

    /// FPU
    pub fpu: ppc_fpu_t,

    /// Generic CPU instance pointer
    pub gen: *mut cpu_gen_t,

    /// VM instance
    pub vm: *mut vm_instance_t,

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

    /// Current exec page (non-JIT) info
    pub njm_exec_page: m_uint64_t,
    pub njm_exec_ptr: *mut mips_insn_t,

    /// Performance counter (non-JIT)
    pub perf_counter: m_uint32_t,

    /// non-JIT mode instruction counter
    pub insn_exec_count: m_uint64_t,

    /// Breakpoints
    pub breakpoints: [m_uint32_t; PPC32_MAX_BREAKPOINTS],
    pub breakpoints_enabled: u_int,

    /// JIT host register allocation
    pub jit_hreg_seq_name: *mut c_char,
    pub ppc_reg_map: [c_int; PPC32_GPR_NR],
    pub hreg_map_list: *mut hreg_map,
    pub hreg_lru: *mut hreg_map,
    pub hreg_map: [hreg_map; JIT_HOST_NREG],
}

/// Reset a PowerPC CPU
#[no_mangle]
pub unsafe extern "C" fn ppc32_reset(cpu: *mut cpu_ppc_t) -> c_int {
    (*cpu).ia = PPC32_ROM_START;
    (*cpu).gpr[1] = PPC32_ROM_SP;
    (*cpu).msr = PPC32_MSR_IP;

    // Restart the MTS subsystem
    ppc32_mem_restart(cpu);

    // Flush JIT structures
    ppc32_jit_flush(cpu, 0);
    0
}
