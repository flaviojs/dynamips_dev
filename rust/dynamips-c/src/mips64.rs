//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! XXX TODO: proper context save/restore for CPUs.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips_common::*;
use crate::mips64_jit::*;
use crate::rbtree::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::utils::*;
use crate::vm::*;

pub type cpu_mips_t = cpu_mips;
pub type mips_cp0_t = mips_cp0;
pub type mips_cp1_t = mips_cp1;
pub type tlb_entry_t = tlb_entry;

/// MIPS General Purpose Registers
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

/// CP0 Context register
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_CONTEXT_VPN2_MASK: m_uint64_t = 0xffffe000_u64; // applied to addr
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_CONTEXT_BADVPN2_MASK: m_uint64_t = 0x7fffff_u64;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_CONTEXT_BADVPN2_SHIFT: c_int = 4;

/// CP0 XContext register
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_XCONTEXT_VPN2_MASK: m_uint64_t = 0xffffffe000_u64;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_XCONTEXT_RBADVPN2_MASK: m_uint64_t = 0x1ffffffff_u64;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_XCONTEXT_BADVPN2_SHIFT: c_int = 4;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_XCONTEXT_R_SHIFT: c_int = 31;

/// TLB masks and shifts
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_TLB_PAGE_MASK: m_uint64_t = 0x01ffe000;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_PAGE_MASK: m_uint64_t = 0x01ffe000_u64;
pub const MIPS_TLB_PAGE_SHIFT: c_int = 13;
pub const MIPS_TLB_VPN2_MASK_32: m_uint64_t = 0xffffe000_u64;
pub const MIPS_TLB_VPN2_MASK_64: m_uint64_t = 0xc00000ffffffe000_u64;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_TLB_PFN_MASK: m_uint32_t = 0x3fffffc0;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_PFN_MASK: m_uint64_t = 0x3fffffc0_u64;
pub const MIPS_TLB_ASID_MASK: m_uint32_t = 0x000000ff; // "asid" in EntryHi
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_TLB_G_MASK: m_uint32_t = 0x00001000; // "Global" in EntryHi
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_G_MASK: m_uint64_t = 0x00001000_u64; // "Global" in EntryHi
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_TLB_V_MASK: m_uint32_t = 0x2; // "Valid" in EntryLo
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_V_MASK: m_uint64_t = 0x2_u64; // "Valid" in EntryLo
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_TLB_D_MASK: m_uint32_t = 0x4; // "Dirty" in EntryLo
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_D_MASK: m_uint64_t = 0x4_u64; // "Dirty" in EntryLo
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_TLB_C_MASK: m_uint32_t = 0x38; // Page Coherency Attribute
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_C_MASK: m_uint64_t = 0x38_u64; // Page Coherency Attribute
pub const MIPS_TLB_C_SHIFT: c_int = 3;

#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_CP0_LO_G_MASK: m_uint64_t = 0x00000001; // "Global" in Lo0/1 reg
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_LO_G_MASK: m_uint64_t = 0x00000001_u64; // "Global" in Lo0/1 reg
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_CP0_HI_SAFE_MASK: m_uint64_t = 0xffffe0ff; // Safety mask for Hi reg
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_HI_SAFE_MASK: m_uint64_t = 0x3fffffff_u64; // Safety mask for Hi reg
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_CP0_LO_SAFE_MASK: m_uint64_t = 0x7fffffff; // Safety mask for Lo reg
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_CP0_LO_SAFE_MASK: m_uint64_t = 0xc00000ffffffe0ff_u64; // Safety mask for Lo reg

/// results for TLB lookups // TODO enum
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_LOOKUP_OK: c_int = 0; // Entry found
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_LOOKUP_INVALID: c_int = 1; // Invalid entry found
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_LOOKUP_MISS: c_int = 2; // No matching entry found
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_TLB_LOOKUP_MOD: c_int = 3; // Read-only

/// Exceptions vectors // TODO enum
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_EXCVECT_RST: c_int = 0; // Soft Reset, Reset, NMI
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_EXCVECT_TLB_REFILL: c_int = 1; // TLB Refill (32-bit)
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_EXCVECT_XTLB_REFILL: c_int = 2; // TLB Refill (64-bit)
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_EXCVECT_CACHE_ERR: c_int = 3; // Cache Error
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_EXCVECT_INT_IV0: c_int = 4; // Interrupt, IV=0
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_EXCVECT_INT_IV1: c_int = 5; // Interrupt, IV=1
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_EXCVECT_OTHERS: c_int = 6; // Other exceptions

/// MIPS "jr ra" instruction
pub const MIPS_INSN_JR_RA: mips_insn_t = 0x03e00008;

/// Minimum page size: 4 Kb
pub const MIPS_MIN_PAGE_SHIFT: c_int = 12;
pub const MIPS_MIN_PAGE_SIZE: size_t = 1 << MIPS_MIN_PAGE_SHIFT;
pub const MIPS_MIN_PAGE_IMASK: m_uint64_t = MIPS_MIN_PAGE_SIZE as m_uint64_t - 1;
pub const MIPS_MIN_PAGE_MASK: m_uint64_t = 0xfffffffffffff000_u64;

/// Addressing mode: Kernel, Supervisor and User
pub const MIPS_MODE_KERNEL: c_int = 00;

/// Segments in 32-bit User mode
pub const MIPS_USEG_BASE: m_uint32_t = 0x00000000;
pub const MIPS_USEG_SIZE: m_uint32_t = 0x80000000;

/// Segments in 32-bit Supervisor mode
pub const MIPS_SUSEG_BASE: m_uint32_t = 0x00000000;
pub const MIPS_SUSEG_SIZE: m_uint32_t = 0x80000000;
pub const MIPS_SSEG_BASE: m_uint32_t = 0xc0000000;
pub const MIPS_SSEG_SIZE: m_uint32_t = 0x20000000;

/// Segments in 32-bit Kernel mode
pub const MIPS_KUSEG_BASE: m_uint32_t = 0x00000000;
pub const MIPS_KUSEG_SIZE: m_uint32_t = 0x80000000;

pub const MIPS_KSEG0_BASE: m_uint32_t = 0x80000000;
pub const MIPS_KSEG0_SIZE: m_uint32_t = 0x20000000;

pub const MIPS_KSEG1_BASE: m_uint32_t = 0xa0000000;
pub const MIPS_KSEG1_SIZE: m_uint32_t = 0x20000000;

pub const MIPS_KSSEG_BASE: m_uint32_t = 0xc0000000;
pub const MIPS_KSSEG_SIZE: m_uint32_t = 0x20000000;

pub const MIPS_KSEG3_BASE: m_uint32_t = 0xe0000000;
pub const MIPS_KSEG3_SIZE: m_uint32_t = 0x20000000;

/// xkphys mask (36-bit physical address)
pub const MIPS64_XKPHYS_ZONE_MASK: m_uint64_t = 0xF800000000000000_u64;
pub const MIPS64_XKPHYS_PHYS_SIZE: m_uint64_t = 1_u64 << 36;
pub const MIPS64_XKPHYS_PHYS_MASK: m_uint64_t = MIPS64_XKPHYS_PHYS_SIZE - 1;
pub const MIPS64_XKPHYS_CCA_SHIFT: c_int = 59;

/// Initial Program Counter and Stack pointer for ROM
pub const MIPS_ROM_PC: m_uint64_t = 0xffffffffbfc00000_u64;
pub const MIPS_ROM_SP: m_uint64_t = 0xffffffff80004000_u64;

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

/// Number of instructions per page
pub const MIPS_INSN_PER_PAGE: size_t = MIPS_MIN_PAGE_SIZE / 4; // size_of::<mips_insn_t>();

/* MIPS CPU Identifiers */
pub const MIPS_PRID_R4600: m_uint32_t = 0x00002012;
pub const MIPS_PRID_R4700: m_uint32_t = 0x00002112;
pub const MIPS_PRID_R5000: m_uint32_t = 0x00002312;
pub const MIPS_PRID_R7000: m_uint32_t = 0x00002721;
pub const MIPS_PRID_R527x: m_uint32_t = 0x00002812;
pub const MIPS_PRID_BCM1250: m_uint32_t = 0x00040102;

/// Memory operations
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LOOKUP: u_int = 0;

#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LB: u_int = 1;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LBU: u_int = 2;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LH: u_int = 3;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LHU: u_int = 4;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LW: u_int = 5;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LWU: u_int = 6;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LD: u_int = 7;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SB: u_int = 8;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SH: u_int = 9;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SW: u_int = 10;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SD: u_int = 11;

#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LWL: u_int = 12;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LWR: u_int = 13;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LDL: u_int = 14;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LDR: u_int = 15;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SWL: u_int = 16;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SWR: u_int = 17;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SDL: u_int = 18;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SDR: u_int = 19;

#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LL: u_int = 20;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SC: u_int = 21;

#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_LDC1: u_int = 22;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_SDC1: u_int = 23;

#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_CACHE: u_int = 24;

#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MIPS_MEMOP_MAX: usize = 25;

/// Memory operations
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LOOKUP: u_int = 0;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_IFETCH: u_int = 1;

#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LB: u_int = 2;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LBU: u_int = 3;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LH: u_int = 4;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LHU: u_int = 5;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LW: u_int = 6;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LWU: u_int = 7;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LD: u_int = 8;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SB: u_int = 9;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SH: u_int = 10;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SW: u_int = 11;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SD: u_int = 12;

#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LWL: u_int = 13;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LWR: u_int = 14;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LDL: u_int = 15;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LDR: u_int = 16;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SWL: u_int = 17;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SWR: u_int = 18;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SDL: u_int = 19;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SDR: u_int = 20;

#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LL: u_int = 21;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SC: u_int = 22;

#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_LDC1: u_int = 23;
#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_SDC1: u_int = 24;

#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_CACHE: u_int = 25;

#[cfg(feature = "USE_UNSTABLE")]
pub const MIPS_MEMOP_MAX: usize = 26;

/// Maximum number of breakpoints
pub const MIPS64_MAX_BREAKPOINTS: usize = 8;

/// Memory operation function prototype
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
    pub translate: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, phys_page: *mut m_uint32_t) -> c_int>,

    /// Memory access functions
    pub mem_op_fn: [mips_memop_fn; MIPS_MEMOP_MAX],

    /// Memory lookup function (to load ELF image,...)
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub mem_op_lookup: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t) -> *mut c_void>,
    /// Memory lookup function (to load ELF image,...) and instruction fetch
    #[cfg(feature = "USE_UNSTABLE")]
    pub mem_op_lookup: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t) -> *mut c_void>,
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
    pub mts_map: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, paddr: m_uint64_t, len: m_uint32_t, cache_access: c_int, tlb_index: c_int)>,

    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub mts_unmap: Option<unsafe extern "C" fn(cpu: *mut cpu_mips_t, vaddr: m_uint64_t, len: m_uint32_t, val: m_uint32_t, tlb_index: c_int)>,

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
    pub sym_trace: c_int,
    pub sym_tree: *mut rbtree_tree,

    /// XXX
    #[cfg(feature = "USE_UNSTABLE")]
    pub current_tb: *mut cpu_tb_t,
}

#[no_mangle]
pub unsafe extern "C" fn MIPS64_IRQ_LOCK(cpu: *mut cpu_mips_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*cpu).irq_lock));
}
#[no_mangle]
pub unsafe extern "C" fn MIPS64_IRQ_UNLOCK(cpu: *mut cpu_mips_t) {
    libc::pthread_mutex_unlock(addr_of_mut!((*cpu).irq_lock));
}
