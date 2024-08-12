//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! PowerPC (32-bit) generic routines.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips_common::*;
use crate::ppc32_jit::*;
use crate::utils::*;
use crate::vm::*;

pub type cpu_ppc_t = cpu_ppc;
pub type ppc_fpu_t = ppc_fpu;

/// CPU identifiers
pub const PPC32_PVR_405: m_uint32_t = 0x40110000;

/// Number of GPR (general purpose registers)
pub const PPC32_GPR_NR: usize = 32;

/// Number of registers in FPU
pub const PPC32_FPU_REG_NR: usize = 32;

/// Minimum page size: 4 Kb
pub const PPC32_MIN_PAGE_SHIFT: c_int = 12;
pub const PPC32_MIN_PAGE_SIZE: usize = 1 << PPC32_MIN_PAGE_SHIFT;
pub const PPC32_MIN_PAGE_IMASK: m_uint32_t = (PPC32_MIN_PAGE_SIZE - 1) as m_uint32_t;
pub const PPC32_MIN_PAGE_MASK: m_uint32_t = 0xFFFFF000;

/// Number of instructions per page
pub const PPC32_INSN_PER_PAGE: usize = PPC32_MIN_PAGE_SIZE / 4; // size_of::<ppc_insn_t>();

/// Starting point for ROM
pub const PPC32_ROM_START: m_uint32_t = 0xfff00100;
pub const PPC32_ROM_SP: m_uint32_t = 0x00006000;

/// Special Purpose Registers (SPR)
pub const PPC32_SPR_XER: u_int = 1;
pub const PPC32_SPR_LR: u_int = 8; // Link Register
pub const PPC32_SPR_CTR: u_int = 9; // Count Register
pub const PPC32_SPR_DSISR: u_int = 18;
pub const PPC32_SPR_DAR: u_int = 19;
pub const PPC32_SPR_DEC: u_int = 22; // Decrementer
pub const PPC32_SPR_SDR1: u_int = 25; // Page Table Address
pub const PPC32_SPR_SRR0: u_int = 26;
pub const PPC32_SPR_SRR1: u_int = 27;
pub const PPC32_SPR_TBL_READ: u_int = 268; // Time Base Low (read)
pub const PPC32_SPR_TBU_READ: u_int = 269; // Time Base Up (read)
pub const PPC32_SPR_SPRG0: u_int = 272;
pub const PPC32_SPR_SPRG1: u_int = 273;
pub const PPC32_SPR_SPRG2: u_int = 274;
pub const PPC32_SPR_SPRG3: u_int = 275;
pub const PPC32_SPR_TBL_WRITE: u_int = 284; // Time Base Low (write)
pub const PPC32_SPR_TBU_WRITE: u_int = 285; // Time Base Up (write)
pub const PPC32_SPR_PVR: u_int = 287; // Processor Version Register
pub const PPC32_SPR_HID0: u_int = 1008;
pub const PPC32_SPR_HID1: u_int = 1009;

pub const PPC405_SPR_PID: u_int = 945; // Process Identifier

/// Exception vectors
pub const PPC32_EXC_SYS_RST: m_uint32_t = 0x00000100; // System Reset
pub const PPC32_EXC_MC_CHK: m_uint32_t = 0x00000200; // Machine Check
pub const PPC32_EXC_DSI: m_uint32_t = 0x00000300; // Data memory access failure
pub const PPC32_EXC_ISI: m_uint32_t = 0x00000400; // Instruction fetch failure
pub const PPC32_EXC_EXT: m_uint32_t = 0x00000500; // External Interrupt
pub const PPC32_EXC_ALIGN: m_uint32_t = 0x00000600; // Alignment
pub const PPC32_EXC_PROG: m_uint32_t = 0x00000700; // FPU, Illegal instruction, ...
pub const PPC32_EXC_NO_FPU: m_uint32_t = 0x00000800; // FPU unavailable
pub const PPC32_EXC_DEC: m_uint32_t = 0x00000900; // Decrementer
pub const PPC32_EXC_SYSCALL: m_uint32_t = 0x00000C00; // System Call
pub const PPC32_EXC_TRACE: m_uint32_t = 0x00000D00; // Trace
pub const PPC32_EXC_FPU_HLP: m_uint32_t = 0x00000E00; // Floating-Point Assist

/// Condition Register (CR) is accessed through 8 fields of 4 bits
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_get_cr_field(n: u_int) -> u_int {
    n >> 2
}
#[inline]
#[no_mangle]
pub unsafe extern "C" fn ppc32_get_cr_bit(n: u_int) -> u_int {
    !n & 0x03
}

/// Positions of LT, GT, EQ and SO bits in CR fields
pub const PPC32_CR_LT_BIT: m_uint32_t = 3;
pub const PPC32_CR_GT_BIT: m_uint32_t = 2;
pub const PPC32_CR_EQ_BIT: m_uint32_t = 1;
pub const PPC32_CR_SO_BIT: m_uint32_t = 0;

/// CR0 (Condition Register Field 0) bits
pub const PPC32_CR0_LT_BIT: m_uint32_t = 31;
pub const PPC32_CR0_LT: m_uint32_t = 1 << PPC32_CR0_LT_BIT; // Negative
pub const PPC32_CR0_GT_BIT: m_uint32_t = 30;
pub const PPC32_CR0_GT: m_uint32_t = 1 << PPC32_CR0_GT_BIT; // Positive
pub const PPC32_CR0_EQ_BIT: m_uint32_t = 29;
pub const PPC32_CR0_EQ: m_uint32_t = 1 << PPC32_CR0_EQ_BIT; // Zero
pub const PPC32_CR0_SO_BIT: m_uint32_t = 28;
pub const PPC32_CR0_SO: m_uint32_t = 1 << PPC32_CR0_SO_BIT; // Summary overflow

/// XER register
pub const PPC32_XER_SO_BIT: m_uint32_t = 31;
pub const PPC32_XER_SO: m_uint32_t = 1 << PPC32_XER_SO_BIT; // Summary Overflow
pub const PPC32_XER_OV: m_uint32_t = 0x40000000; // Overflow
pub const PPC32_XER_CA_BIT: m_uint32_t = 29;
pub const PPC32_XER_CA: m_uint32_t = 1 << PPC32_XER_CA_BIT; // Carry
pub const PPC32_XER_BC_MASK: m_uint32_t = 0x0000007F; // Byte cnt (lswx/stswx)

/// MSR (Machine State Register)
pub const PPC32_MSR_POW_MASK: m_uint32_t = 0x00060000; // Power Management
pub const PPC32_MSR_ILE: m_uint32_t = 0x00010000; // Exception Little-Endian Mode
pub const PPC32_MSR_EE: m_uint32_t = 0x00008000; // External Interrupt Enable
pub const PPC32_MSR_PR: m_uint32_t = 0x00004000; // Privilege Level (0=supervisor)
pub const PPC32_MSR_PR_SHIFT: m_uint32_t = 14;
pub const PPC32_MSR_FP: m_uint32_t = 0x00002000; // Floating-Point Available
pub const PPC32_MSR_ME: m_uint32_t = 0x00001000; // Machine Check Enable
pub const PPC32_MSR_FE0: m_uint32_t = 0x00000800; // Floating-Point Exception Mode 0
pub const PPC32_MSR_SE: m_uint32_t = 0x00000400; // Single-step trace enable
pub const PPC32_MSR_BE: m_uint32_t = 0x00000200; // Branch Trace Enable
pub const PPC32_MSR_FE1: m_uint32_t = 0x00000100; // Floating-Point Exception Mode 1
pub const PPC32_MSR_IP: m_uint32_t = 0x00000040; // Exception Prefix
pub const PPC32_MSR_IR: m_uint32_t = 0x00000020; // Instruction address translation
pub const PPC32_MSR_DR: m_uint32_t = 0x00000010; // Data address translation
pub const PPC32_MSR_RI: m_uint32_t = 0x00000002; // Recoverable Exception
pub const PPC32_MSR_LE: m_uint32_t = 0x00000001; // Little-Endian mode enable

pub const PPC32_RFI_MSR_MASK: m_uint32_t = 0x87c0ff73;
pub const PPC32_EXC_SRR1_MASK: m_uint32_t = 0x0000ff73;
pub const PPC32_EXC_MSR_MASK: m_uint32_t = 0x0006ef32;

/// Number of BAT registers (8 for PowerPC 7448)
pub const PPC32_BAT_NR: usize = 8;

/// Number of segment registers
pub const PPC32_SR_NR: usize = 16;

/// Upper BAT register
pub const PPC32_UBAT_BEPI_MASK: m_uint32_t = 0xFFFE0000; // Block Effective Page Index
pub const PPC32_UBAT_BEPI_SHIFT: m_uint32_t = 17;
pub const PPC32_UBAT_BL_MASK: m_uint32_t = 0x00001FFC; // Block Length
pub const PPC32_UBAT_BL_SHIFT: m_uint32_t = 2;
pub const PPC32_UBAT_XBL_MASK: m_uint32_t = 0x0001FFFC; // Block Length
pub const PPC32_UBAT_XBL_SHIFT: m_uint32_t = 2;
pub const PPC32_UBAT_VS: m_uint32_t = 0x00000002; // Supervisor mode valid bit
pub const PPC32_UBAT_VP: m_uint32_t = 0x00000001; // User mode valid bit
pub const PPC32_UBAT_PROT_MASK: m_uint32_t = PPC32_UBAT_VS | PPC32_UBAT_VP;

/// Lower BAT register
pub const PPC32_LBAT_BRPN_MASK: m_uint32_t = 0xFFFE0000; // Physical address
pub const PPC32_LBAT_BRPN_SHIFT: m_uint32_t = 17;
pub const PPC32_LBAT_WIMG_MASK: m_uint32_t = 0x00000078; // Memory/cache access mode bits
pub const PPC32_LBAT_PP_MASK: m_uint32_t = 0x00000003; // Protection bits

pub const PPC32_BAT_ADDR_SHIFT: m_uint32_t = 17;

/// Segment Descriptor
pub const PPC32_SD_T: m_uint32_t = 0x80000000;
pub const PPC32_SD_KS: m_uint32_t = 0x40000000; // Supervisor-state protection key
pub const PPC32_SD_KP: m_uint32_t = 0x20000000; // User-state protection key
pub const PPC32_SD_N: m_uint32_t = 0x10000000; // No-execute protection bit
pub const PPC32_SD_VSID_MASK: m_uint32_t = 0x00FFFFFF; // Virtual Segment ID

/// SDR1 Register
pub const PPC32_SDR1_HTABORG_MASK: m_uint32_t = 0xFFFF0000; // Physical base address
pub const PPC32_SDR1_HTABEXT_MASK: m_uint32_t = 0x0000E000; // Extended base address
pub const PPC32_SDR1_HTABMASK: m_uint32_t = 0x000001FF; // Mask for page table address
pub const PPC32_SDR1_HTMEXT_MASK: m_uint32_t = 0x00001FFF; // Extended mask

/// Page Table Entry (PTE) size: 64-bits
pub const PPC32_PTE_SIZE: m_uint32_t = 8;

/// PTE entry (Up and Lo)
pub const PPC32_PTEU_V: m_uint32_t = 0x80000000; // Valid entry
pub const PPC32_PTEU_VSID_MASK: m_uint32_t = 0x7FFFFF80; // Virtual Segment ID
pub const PPC32_PTEU_VSID_SHIFT: m_uint32_t = 7;
pub const PPC32_PTEU_H: m_uint32_t = 0x00000040; // Hash function
pub const PPC32_PTEU_API_MASK: m_uint32_t = 0x0000003F; // Abbreviated Page index
pub const PPC32_PTEL_RPN_MASK: m_uint32_t = 0xFFFFF000; // Physical Page Number
pub const PPC32_PTEL_XPN_MASK: m_uint32_t = 0x00000C00; // Extended Page Number (0-2)
pub const PPC32_PTEL_XPN_SHIFT: m_uint32_t = 9;
pub const PPC32_PTEL_R: m_uint32_t = 0x00000100; // Referenced bit
pub const PPC32_PTEL_C: m_uint32_t = 0x00000080; // Changed bit
pub const PPC32_PTEL_WIMG_MASK: m_uint32_t = 0x00000078; // Mem/cache access mode bits
pub const PPC32_PTEL_WIMG_SHIFT: m_uint32_t = 3;
pub const PPC32_PTEL_X_MASK: m_uint32_t = 0x00000004; // Extended Page Number (3)
pub const PPC32_PTEL_X_SHIFT: m_uint32_t = 2;
pub const PPC32_PTEL_PP_MASK: m_uint32_t = 0x00000003; // Page Protection bits

/// DSISR register
pub const PPC32_DSISR_NOTRANS: m_uint32_t = 0x40000000; // No valid translation
pub const PPC32_DSISR_STORE: m_uint32_t = 0x02000000; // Store operation

/// PowerPC 405 TLB definitions
pub const PPC405_TLBHI_EPN_MASK: m_uint32_t = 0xFFFFFC00; // Effective Page Number
pub const PPC405_TLBHI_SIZE_MASK: m_uint32_t = 0x00000380; // Page Size
pub const PPC405_TLBHI_SIZE_SHIFT: m_uint32_t = 7;
pub const PPC405_TLBHI_V: m_uint32_t = 0x00000040; // Valid TLB entry
pub const PPC405_TLBHI_E: m_uint32_t = 0x00000020; // Endianness
pub const PPC405_TLBHI_U0: m_uint32_t = 0x00000010; // User-Defined Attribute

pub const PPC405_TLBLO_RPN_MASK: m_uint32_t = 0xFFFFFC00; // Real Page Number
pub const PPC405_TLBLO_EX: m_uint32_t = 0x00000200; // Execute Enable
pub const PPC405_TLBLO_WR: m_uint32_t = 0x00000100; // Write Enable
pub const PPC405_TLBLO_ZSEL_MASK: m_uint32_t = 0x000000F0; // Zone Select
pub const PPC405_TLBLO_ZSEL_SHIFT: m_uint32_t = 4;
pub const PPC405_TLBLO_W: m_uint32_t = 0x00000008; // Write-Through
pub const PPC405_TLBLO_I: m_uint32_t = 0x00000004; // Caching Inhibited
pub const PPC405_TLBLO_M: m_uint32_t = 0x00000002; // Memory Coherent
pub const PPC405_TLBLO_G: m_uint32_t = 0x00000001; // Guarded

/// Number of TLB entries for PPC405
pub const PPC405_TLB_ENTRIES: usize = 64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc405_tlb_entry {
    pub tlb_hi: m_uint32_t,
    pub tlb_lo: m_uint32_t,
    pub tid: m_uint32_t,
}

/// Memory operations
pub const PPC_MEMOP_LOOKUP: c_int = 0;

/// Instruction fetch operation
pub const PPC_MEMOP_IFETCH: c_int = 1;

/// Load operations
pub const PPC_MEMOP_LBZ: c_int = 2;
pub const PPC_MEMOP_LHZ: c_int = 3;
pub const PPC_MEMOP_LWZ: c_int = 4;

/// Load operation with sign-extend
pub const PPC_MEMOP_LHA: c_int = 5;

/// Store operations
pub const PPC_MEMOP_STB: c_int = 6;
pub const PPC_MEMOP_STH: c_int = 7;
pub const PPC_MEMOP_STW: c_int = 8;

/// Byte-Reversed operations
pub const PPC_MEMOP_LWBR: c_int = 9;
pub const PPC_MEMOP_STWBR: c_int = 10;

/// String operations
pub const PPC_MEMOP_LSW: c_int = 11;
pub const PPC_MEMOP_STSW: c_int = 12;

/// FPU operations
pub const PPC_MEMOP_LFD: c_int = 13;
pub const PPC_MEMOP_STFD: c_int = 14;

/// ICBI - Instruction Cache Block Invalidate
pub const PPC_MEMOP_ICBI: c_int = 15;

pub const PPC_MEMOP_MAX: usize = 16;

/// Memory operation function prototype
pub type ppc_memop_fn = Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t, reg: u_int)>;

/// BAT type indexes // TODO enum
pub const PPC32_IBAT_IDX: c_int = 0;
pub const PPC32_DBAT_IDX: c_int = 1;

/// BAT register
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_bat_reg {
    pub reg: [m_uint32_t; 2],
}

/// BAT register programming
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_bat_prog {
    pub r#type: c_int,
    pub index: c_int,
    pub hi: m_uint32_t,
    pub lo: m_uint32_t,
}

/// MTS Instruction Cache and Data Cache
pub const PPC32_MTS_ICACHE: c_int = PPC32_IBAT_IDX;
pub const PPC32_MTS_DCACHE: c_int = PPC32_DBAT_IDX;

/// FPU Coprocessor definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc_fpu {
    pub reg: [m_uint64_t; PPC32_FPU_REG_NR],
}

/// Maximum number of breakpoints
pub const PPC32_MAX_BREAKPOINTS: usize = 8;

/// zzz
#[cfg(not(feature = "USE_UNSTABLE"))]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ppc32_vtlb_entry {
    pub vaddr: m_uint32_t,
    pub haddr: m_uint32_t,
}

/// PowerPC CPU definition
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

    /// XER, Condition Register, Link Register, Count Register
    pub xer: m_uint32_t,
    pub lr: m_uint32_t,
    pub ctr: m_uint32_t,
    pub reserve: m_uint32_t,
    pub xer_ca: m_uint32_t,

    /// Condition Register (CR) fields
    pub cr_fields: [u_int; 8],

    /// MTS caches (Instruction+Data)
    pub mts_cache: [*mut mts32_entry_t; 2],

    /// Code page translation cache and physical page mapping
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub exec_blk_map: *mut *mut ppc32_jit_tcb_t,
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub exec_phys_map: *mut *mut ppc32_jit_tcb_t,
    /// Code page translation cache and physical page mapping
    #[cfg(feature = "USE_UNSTABLE")]
    pub tcb_virt_hash: *mut *mut ppc32_jit_tcb_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tcb_phys_hash: *mut *mut ppc32_jit_tcb_t,

    /// Virtual address to physical page translation
    pub translate: Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t, cid: u_int, phys_page: *mut m_uint32_t) -> c_int>,

    /// Memory access functions
    pub mem_op_fn: [ppc_memop_fn; PPC_MEMOP_MAX],

    /// Memory lookup function (to load ELF image,...)
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub mem_op_lookup: Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t, cid: u_int) -> *mut c_void>,
    /// Memory lookup function (to load ELF image,...) and Instruction fetch
    #[cfg(feature = "USE_UNSTABLE")]
    pub mem_op_lookup: Option<unsafe extern "C" fn(cpu: *mut cpu_ppc_t, vaddr: m_uint32_t, cid: u_int) -> *mut c_void>,
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

    /// Segment registers
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

#[no_mangle]
pub unsafe extern "C" fn PPC32_CR_FIELD_OFFSET(f: c_int) -> c_long {
    OFFSET!(cpu_ppc_t, cr_fields) + (f as usize * size_of::<u_int>()) as c_long
}

/// Get the full CR register
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_get_cr(cpu: *mut cpu_ppc_t) -> m_uint32_t {
    let mut cr: m_uint32_t = 0;

    for i in 0..8 {
        cr |= (*cpu).cr_fields[i] << (28 - (i << 2));
    }

    cr
}

/// Set the CR fields given a CR value
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_set_cr(cpu: *mut cpu_ppc_t, cr: m_uint32_t) {
    for i in 0..8 {
        (*cpu).cr_fields[i] = (cr >> (28 - (i << 2))) & 0x0F;
    }
}

/// Get a CR bit
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_read_cr_bit(cpu: *mut cpu_ppc_t, bit: u_int) -> m_uint32_t {
    let res: m_uint32_t = (*cpu).cr_fields[ppc32_get_cr_field(bit) as usize] >> ppc32_get_cr_bit(bit);
    res & 0x01
}

/// Set a CR bit
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_set_cr_bit(cpu: *mut cpu_ppc_t, bit: u_int) {
    (*cpu).cr_fields[ppc32_get_cr_field(bit) as usize] |= 1 << ppc32_get_cr_bit(bit);
}

/// Clear a CR bit
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn ppc32_clear_cr_bit(cpu: *mut cpu_ppc_t, bit: u_int) {
    (*cpu).cr_fields[ppc32_get_cr_field(bit) as usize] &= !(1 << ppc32_get_cr_bit(bit));
}
