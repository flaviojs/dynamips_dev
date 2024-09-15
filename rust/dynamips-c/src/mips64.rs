//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! XXX TODO: proper context save/restore for CPUs.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::mips64_cp0::*;
use crate::mips64_jit::*;
use crate::ppc32::*;
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

/// MIPS general purpose registers names
#[rustfmt::skip]
#[no_mangle]
pub static mut mips64_gpr_reg_names: [*mut c_char; MIPS64_GPR_NR] = [
    cstr!("zr"), cstr!("at"), cstr!("v0"), cstr!("v1"), cstr!("a0"), cstr!("a1"), cstr!("a2"), cstr!("a3"),
    cstr!("t0"), cstr!("t1"), cstr!("t2"), cstr!("t3"), cstr!("t4"), cstr!("t5"), cstr!("t6"), cstr!("t7"),
    cstr!("s0"), cstr!("s1"), cstr!("s2"), cstr!("s3"), cstr!("s4"), cstr!("s5"), cstr!("s6"), cstr!("s7"),
    cstr!("t8"), cstr!("t9"), cstr!("k0"), cstr!("k1"), cstr!("gp"), cstr!("sp"), cstr!("fp"), cstr!("ra"),
];

/// Cacheability and Coherency Attribute
static cca_cache_status: [c_int; 8] = [1, 1, 0, 1, 0, 1, 0, 0];

/// Get register index given its name
#[no_mangle]
pub unsafe extern "C" fn mips64_get_reg_index(name: *mut c_char) -> c_int {
    for i in 0..MIPS64_GPR_NR as c_int {
        if libc::strcmp(mips64_gpr_reg_names[i as usize], name) == 0 {
            return i;
        }
    }

    -1
}

/// Get cacheability info
#[no_mangle]
pub unsafe extern "C" fn mips64_cca_cached(val: m_uint8_t) -> c_int {
    cca_cache_status[(val & 0x03) as usize]
}

/// Reset a MIPS64 CPU
#[no_mangle]
pub unsafe extern "C" fn mips64_reset(cpu: *mut cpu_mips_t) -> c_int {
    (*cpu).pc = MIPS_ROM_PC;
    (*cpu).gpr[MIPS_GPR_SP] = MIPS_ROM_SP;
    (*cpu).cp0.reg[MIPS_CP0_STATUS] = MIPS_CP0_STATUS_BEV as m_uint64_t;
    (*cpu).cp0.reg[MIPS_CP0_CAUSE] = 0;
    (*cpu).cp0.reg[MIPS_CP0_CONFIG] = 0x00c08ff0_u64;

    // Clear the complete TLB
    libc::memset((*cpu).cp0.tlb.as_c_void_mut(), 0, MIPS64_TLB_MAX_ENTRIES * size_of::<tlb_entry_t>());

    // Restart the MTS subsystem
    mips64_set_addr_mode(cpu, 32 /*64*/); // zzz
    (*(*cpu).gen).mts_rebuild.unwrap()((*cpu).gen);

    // Flush JIT structures
    mips64_jit_flush(cpu, 0);
    0
}

/// Initialize a MIPS64 processor
#[no_mangle]
pub unsafe extern "C" fn mips64_init(cpu: *mut cpu_mips_t) -> c_int {
    (*cpu).addr_bus_mask = 0xFFFFFFFFFFFFFFFF_u64;
    (*cpu).cp0.reg[MIPS_CP0_PRID] = MIPS_PRID_R4600 as m_uint64_t;
    (*cpu).cp0.tlb_entries = MIPS64_TLB_STD_ENTRIES as u_int;

    // Initialize idle timer
    (*(*cpu).gen).idle_max = 500;
    (*(*cpu).gen).idle_sleep_time = 30000;

    // Timer IRQ parameters (default frequency: 250 Hz <=> 4ms period)
    (*cpu).timer_irq_check_itv = 1000;
    (*cpu).timer_irq_freq = 250;

    // Enable fast memory operations
    (*cpu).fast_memop = TRUE as u_int;

    // Enable/Disable direct block jump
    (*cpu).exec_blk_direct_jump = (*(*cpu).vm).exec_blk_direct_jump as u_int;

    // Create the IRQ lock (for non-jit architectures)
    libc::pthread_mutex_init(addr_of_mut!((*cpu).irq_lock), null_mut());

    // Idle loop mutex and condition
    libc::pthread_mutex_init(addr_of_mut!((*(*cpu).gen).idle_mutex), null_mut());
    libc::pthread_cond_init(addr_of_mut!((*(*cpu).gen).idle_cond), null_mut());

    // Set the CPU methods
    (*(*cpu).gen).reg_set = Some(mips64_reg_set);
    (*(*cpu).gen).reg_dump = Some(mips64_dump_regs);
    (*(*cpu).gen).mmu_dump = Some(mips64_tlb_dump);
    (*(*cpu).gen).mmu_raw_dump = Some(mips64_tlb_raw_dump);
    (*(*cpu).gen).add_breakpoint = Some(mips64_add_breakpoint);
    (*(*cpu).gen).remove_breakpoint = Some(mips64_remove_breakpoint);
    (*(*cpu).gen).set_idle_pc = Some(mips64_set_idle_pc);
    (*(*cpu).gen).get_idling_pc = Some(mips64_get_idling_pc);

    // Set the startup parameters
    mips64_reset(cpu);
    0
}

/// Delete the symbol tree node
unsafe extern "C" fn mips64_delete_sym_tree_node(key: *mut c_void, _value: *mut c_void, _opt: *mut c_void) {
    libc::free(key);
}

/// Delete a MIPS64 processor
#[no_mangle]
pub unsafe extern "C" fn mips64_delete(cpu: *mut cpu_mips_t) {
    if !cpu.is_null() {
        mips64_mem_shutdown(cpu);
        mips64_jit_shutdown(cpu);
        if !(*cpu).sym_tree.is_null() {
            rbtree_foreach((*cpu).sym_tree, Some(mips64_delete_sym_tree_node), null_mut());
            rbtree_delete((*cpu).sym_tree);
            (*cpu).sym_tree = null_mut();
        }
    }
}

/// Set the CPU PRID register
#[no_mangle]
pub unsafe extern "C" fn mips64_set_prid(cpu: *mut cpu_mips_t, prid: m_uint32_t) {
    (*cpu).cp0.reg[MIPS_CP0_PRID] = prid as m_uint64_t;

    if (prid == MIPS_PRID_R7000) || (prid == MIPS_PRID_BCM1250) {
        (*cpu).cp0.tlb_entries = MIPS64_TLB_MAX_ENTRIES as u_int;
    }
}

/// Set idle PC value
#[no_mangle]
pub unsafe extern "C" fn mips64_set_idle_pc(cpu: *mut cpu_gen_t, addr: m_uint64_t) {
    (*CPU_MIPS64(cpu)).idle_pc.set(addr);
}

/// Timer IRQ
#[no_mangle]
pub unsafe extern "C" fn mips64_timer_irq_run(cpu: *mut cpu_mips_t) -> *mut c_void {
    let mut umutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
    let mut ucond: libc::pthread_cond_t = libc::PTHREAD_COND_INITIALIZER;
    let mut t_spc: libc::timespec = zeroed::<_>();
    let mut expire: m_tmcnt_t;

    let interval: u_int = 1000000 / (*cpu).timer_irq_freq;
    let threshold: u_int = (*cpu).timer_irq_freq * 10;
    expire = m_gettime_usec() + interval as m_tmcnt_t;

    while (*(*cpu).gen).state.get() != CPU_STATE_HALTED {
        libc::pthread_mutex_lock(addr_of_mut!(umutex));
        t_spc.tv_sec = (expire / 1000000) as _;
        t_spc.tv_nsec = ((expire % 1000000) * 1000) as _;
        libc::pthread_cond_timedwait(addr_of_mut!(ucond), addr_of_mut!(umutex), addr_of_mut!(t_spc));
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

pub const IDLE_HASH_SIZE: usize = 8192;

/// Idle PC hash item
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mips64_idle_pc_hash {
    pub pc: m_uint64_t,
    pub count: u_int,
    pub next: *mut mips64_idle_pc_hash,
}

/// Determine an "idling" PC
#[no_mangle]
pub unsafe extern "C" fn mips64_get_idling_pc(cpu: *mut cpu_gen_t) {
    unsafe fn mips64_get_idling_pc(cpu: *mut cpu_gen_t) -> c_int {
        let mcpu: *mut cpu_mips_t = CPU_MIPS64(cpu);
        let mut p: *mut mips64_idle_pc_hash;
        let mut res: *mut cpu_idle_pc;
        let mut h_index: u_int;
        let mut cur_pc: m_uint64_t;

        (*cpu).idle_pc_prop_count = 0;

        if (*mcpu).idle_pc.get() != 0 {
            libc::printf(cstr!("\nYou already use an idle PC, using the calibration would give incorrect results.\n"));
            return -1;
        }

        libc::printf(cstr!("\nPlease wait while gathering statistics...\n"));

        let pc_hash: *mut *mut mips64_idle_pc_hash = libc::calloc(IDLE_HASH_SIZE, size_of::<*mut mips64_idle_pc_hash>()).cast::<_>();
        if pc_hash.is_null() {
            libc::printf(cstr!("Out of memory."));
            return -1;
        }

        // Disable IRQ
        (*mcpu).irq_disable.set(TRUE as u_int);

        // Take 1000 measures, each mesure every 10ms
        for _ in 0..1000 {
            cur_pc = (*mcpu).pc;
            h_index = ((cur_pc >> 2) & (IDLE_HASH_SIZE as m_uint64_t - 1)) as u_int;

            p = *pc_hash.add(h_index as usize);
            while !p.is_null() {
                if (*p).pc == cur_pc {
                    (*p).count += 1;
                    break;
                }
                p = (*p).next;
            }

            if p.is_null() {
                p = libc::malloc(size_of::<mips64_idle_pc_hash>()).cast::<_>();
                if !p.is_null() {
                    (*p).pc = cur_pc;
                    (*p).count = 1;
                    (*p).next = *pc_hash.add(h_index as usize);
                    *pc_hash.add(h_index as usize) = p;
                }
            }

            libc::usleep(10000);
        }

        // Select PCs
        'select_pcs: for i in 0..IDLE_HASH_SIZE as c_int {
            p = *pc_hash.add(i as usize);
            while !p.is_null() {
                if ((*p).count >= 20) && ((*p).count <= if cfg!(not(feature = "USE_UNSTABLE")) { 80 } else { 180 }) {
                    res = addr_of_mut!((*cpu).idle_pc_prop[(*cpu).idle_pc_prop_count as usize]);
                    (*cpu).idle_pc_prop_count += 1;

                    (*res).pc = (*p).pc;
                    (*res).count = (*p).count;

                    if (*cpu).idle_pc_prop_count >= CPU_IDLE_PC_MAX_RES as u_int {
                        break 'select_pcs;
                    }
                }
                p = (*p).next;
            }
        }

        // Set idle PC
        if (*cpu).idle_pc_prop_count != 0 {
            libc::printf(cstr!("Done. Suggested idling PC:\n"));

            for i in 0..(*cpu).idle_pc_prop_count as c_int {
                libc::printf(cstr!("   0x%llx (count=%u)\n"), (*cpu).idle_pc_prop[i as usize].pc, (*cpu).idle_pc_prop[i as usize].count);
            }

            libc::printf(cstr!("Restart the emulator with \"--idle-pc=0x%llx\" (for example)\n"), (*cpu).idle_pc_prop[0].pc);
        } else {
            libc::printf(cstr!("Done. No suggestion for idling PC\n"));

            for i in 0..IDLE_HASH_SIZE as c_int {
                p = *pc_hash.add(i as usize);
                while !p.is_null() {
                    libc::printf(cstr!("  0x%16.16llx (%3u)\n"), (*p).pc, (*p).count);

                    if (*cpu).idle_pc_prop_count < CPU_IDLE_PC_MAX_RES as u_int {
                        res = addr_of_mut!((*cpu).idle_pc_prop[(*cpu).idle_pc_prop_count as usize]);
                        (*cpu).idle_pc_prop_count += 1;

                        (*res).pc = (*p).pc;
                        (*res).count = (*p).count;
                    }
                    p = (*p).next;
                }
            }

            libc::printf(cstr!("\n"));
        }

        // Re-enable IRQ
        (*mcpu).irq_disable.set(FALSE as u_int);
        libc::free(pc_hash.cast::<_>());
        0
    }
    mips64_get_idling_pc(cpu);
}

/// Set an IRQ (VM IRQ standard routing)
#[no_mangle]
pub unsafe extern "C" fn mips64_vm_set_irq(vm: *mut vm_instance_t, irq: u_int) {
    let boot_cpu: *mut cpu_mips_t = CPU_MIPS64((*vm).boot_cpu);

    if (*boot_cpu).irq_disable.get() != 0 {
        (*boot_cpu).irq_pending = 0;
        return;
    }

    mips64_set_irq(boot_cpu, irq as m_uint8_t);

    if (*boot_cpu).irq_idle_preempt[irq as usize] != 0 {
        cpu_idle_break_wait((*vm).boot_cpu);
    }
}

/// Clear an IRQ (VM IRQ standard routing)
#[no_mangle]
pub unsafe extern "C" fn mips64_vm_clear_irq(vm: *mut vm_instance_t, irq: u_int) {
    let boot_cpu: *mut cpu_mips_t = CPU_MIPS64((*vm).boot_cpu);
    mips64_clear_irq(boot_cpu, irq as m_uint8_t);
}

/// Update the IRQ flag (inline)
#[inline(always)]
pub unsafe extern "C" fn mips64_update_irq_flag_fast(cpu: *mut cpu_mips_t) -> c_int {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let imask: m_uint32_t;

    (*cpu).irq_pending = FALSE as u_int;

    let cause: m_uint32_t = ((*cp0).reg[MIPS_CP0_CAUSE] & !(MIPS_CP0_CAUSE_IMASK as m_uint64_t)) as m_uint32_t;
    (*cp0).reg[MIPS_CP0_CAUSE] = (cause | (*cpu).irq_cause) as m_uint64_t;

    let sreg_mask: m_uint32_t = MIPS_CP0_STATUS_IE | MIPS_CP0_STATUS_EXL | MIPS_CP0_STATUS_ERL;

    if ((*cp0).reg[MIPS_CP0_STATUS] & sreg_mask as m_uint64_t) == MIPS_CP0_STATUS_IE as m_uint64_t {
        imask = ((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_IMASK as m_uint64_t) as m_uint32_t;
        if unlikely(((*cp0).reg[MIPS_CP0_CAUSE] & imask as m_uint64_t) != 0) {
            (*cpu).irq_pending = TRUE as u_int;
            return TRUE;
        }
    }

    FALSE
}

/// Update the IRQ flag
#[no_mangle]
pub unsafe extern "C" fn mips64_update_irq_flag(cpu: *mut cpu_mips_t) {
    mips64_update_irq_flag_fast(cpu);
}

/// Generate an exception
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn mips64_trigger_exception(cpu: *mut cpu_mips_t, exc_code: u_int, bd_slot: c_int) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let mut cause: m_uint64_t;
    let vector: m_uint64_t;

    // we don't set EPC if EXL is set
    if ((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_EXL as m_uint64_t) == 0 {
        (*cp0).reg[MIPS_CP0_EPC] = (*cpu).pc;

        // keep IM, set exception code and bd slot
        cause = (*cp0).reg[MIPS_CP0_CAUSE] & MIPS_CP0_CAUSE_IMASK as m_uint64_t;

        if bd_slot != 0 {
            cause |= MIPS_CP0_CAUSE_BD_SLOT as m_uint64_t;
        } else {
            cause &= !(MIPS_CP0_CAUSE_BD_SLOT as m_uint64_t);
        }

        cause |= (exc_code << MIPS_CP0_CAUSE_SHIFT) as m_uint64_t;
        (*cp0).reg[MIPS_CP0_CAUSE] = cause;

        // XXX properly set vector
        vector = 0x180_u64;
    } else {
        // keep IM and set exception code
        cause = (*cp0).reg[MIPS_CP0_CAUSE] & MIPS_CP0_CAUSE_IMASK as m_uint64_t;
        cause |= (exc_code << MIPS_CP0_CAUSE_SHIFT) as m_uint64_t;
        (*cp0).reg[MIPS_CP0_CAUSE] = cause;

        // set vector
        vector = 0x180_u64;
    }

    // Set EXL bit in status register
    (*cp0).reg[MIPS_CP0_STATUS] |= MIPS_CP0_STATUS_EXL as m_uint64_t;

    // Use bootstrap vectors ?
    if ((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_BEV as m_uint64_t) != 0 {
        (*cpu).pc = 0xffffffffbfc00200_u64 + vector;
    } else {
        (*cpu).pc = 0xffffffff80000000_u64 + vector;
    }

    // Clear the pending IRQ flag
    (*cpu).irq_pending = 0;
}

/// Generate a general exception
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_general_exception(cpu: *mut cpu_mips_t, exc_code: u_int) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let mut cause: m_uint64_t;

    // Update cause register (set BD and ExcCode)
    cause = (*cp0).reg[MIPS_CP0_CAUSE] & MIPS_CP0_CAUSE_IMASK as m_uint64_t;

    if (*cpu).bd_slot != 0 {
        cause |= MIPS_CP0_CAUSE_BD_SLOT as m_uint64_t;
    } else {
        cause &= !(MIPS_CP0_CAUSE_BD_SLOT as m_uint64_t);
    }

    cause |= (exc_code << MIPS_CP0_CAUSE_SHIFT) as m_uint64_t;
    (*cp0).reg[MIPS_CP0_CAUSE] = cause;

    // If EXL bit is 0, set EPC and BadVaddr registers
    if likely(((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_EXL as m_uint64_t) == 0) {
        (*cp0).reg[MIPS_CP0_EPC] = (*cpu).pc - ((*cpu).bd_slot << 2) as m_uint64_t;
    }

    // Set EXL bit in status register
    (*cp0).reg[MIPS_CP0_STATUS] |= MIPS_CP0_STATUS_EXL as m_uint64_t;

    // Use bootstrap vectors ?
    if ((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_BEV as m_uint64_t) != 0 {
        (*cpu).pc = 0xffffffffbfc00200_u64 + 0x180;
    } else {
        (*cpu).pc = 0xffffffff80000000_u64 + 0x180;
    }

    // Clear the pending IRQ flag
    (*cpu).irq_pending = 0;
}

/// Generate a general exception that updates BadVaddr
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_gen_exception_badva(cpu: *mut cpu_mips_t, exc_code: u_int, bad_vaddr: m_uint64_t) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let mut cause: m_uint64_t;

    // Update cause register (set BD and ExcCode)
    cause = (*cp0).reg[MIPS_CP0_CAUSE] & MIPS_CP0_CAUSE_IMASK as m_uint64_t;

    if (*cpu).bd_slot != 0 {
        cause |= MIPS_CP0_CAUSE_BD_SLOT as m_uint64_t;
    } else {
        cause &= !(MIPS_CP0_CAUSE_BD_SLOT as m_uint64_t);
    }

    cause |= (exc_code << MIPS_CP0_CAUSE_SHIFT) as m_uint64_t;
    (*cp0).reg[MIPS_CP0_CAUSE] = cause;

    // If EXL bit is 0, set EPC and BadVaddr registers
    if likely(((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_EXL as m_uint64_t) == 0) {
        (*cp0).reg[MIPS_CP0_EPC] = (*cpu).pc - ((*cpu).bd_slot << 2) as m_uint64_t;
        (*cp0).reg[MIPS_CP0_BADVADDR] = bad_vaddr;
    }

    // Set EXL bit in status register
    (*cp0).reg[MIPS_CP0_STATUS] |= MIPS_CP0_STATUS_EXL as m_uint64_t;

    // Use bootstrap vectors ?
    if ((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_BEV as m_uint64_t) != 0 {
        (*cpu).pc = 0xffffffffbfc00200_u64 + 0x180;
    } else {
        (*cpu).pc = 0xffffffff80000000_u64 + 0x180;
    }

    // Clear the pending IRQ flag
    (*cpu).irq_pending = 0;
}

/// Generate a TLB/XTLB miss exception
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_tlb_miss_exception(cpu: *mut cpu_mips_t, exc_code: u_int, bad_vaddr: m_uint64_t) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);
    let mut cause: m_uint64_t;
    let vector: m_uint64_t;

    // Update cause register (set BD and ExcCode)
    cause = (*cp0).reg[MIPS_CP0_CAUSE] & MIPS_CP0_CAUSE_IMASK as m_uint64_t;

    if (*cpu).bd_slot != 0 {
        cause |= MIPS_CP0_CAUSE_BD_SLOT as m_uint64_t;
    } else {
        cause &= !(MIPS_CP0_CAUSE_BD_SLOT as m_uint64_t);
    }

    cause |= (exc_code << MIPS_CP0_CAUSE_SHIFT) as m_uint64_t;
    (*cp0).reg[MIPS_CP0_CAUSE] = cause;

    // If EXL bit is 0, set EPC and BadVaddr registers
    if likely(((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_EXL as m_uint64_t) == 0) {
        (*cp0).reg[MIPS_CP0_EPC] = (*cpu).pc - ((*cpu).bd_slot << 2) as m_uint64_t;
        (*cp0).reg[MIPS_CP0_BADVADDR] = bad_vaddr;

        // determine if TLB or XTLB exception, based on the current
        // addressing mode.
        if (*cpu).addr_mode == 64 {
            vector = 0x080;
        } else {
            vector = 0x000;
        }
    } else {
        // nested: handled as a general exception
        vector = 0x180;
    }

    // Set EXL bit in status register
    (*cp0).reg[MIPS_CP0_STATUS] |= MIPS_CP0_STATUS_EXL as m_uint64_t;

    // Use bootstrap vectors ?
    if ((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_BEV as m_uint64_t) != 0 {
        (*cpu).pc = 0xffffffffbfc00200_u64 + vector;
    } else {
        (*cpu).pc = 0xffffffff80000000_u64 + vector;
    }

    // Clear the pending IRQ flag
    (*cpu).irq_pending = 0;
}

/// Prepare a TLB exception
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn mips64_prepare_tlb_exception(cpu: *mut cpu_mips_t, vaddr: m_uint64_t) {
    // Update CP0 context and xcontext registers
    mips64_cp0_update_context_reg(cpu, vaddr);
    mips64_cp0_update_xcontext_reg(cpu, vaddr);

    // EntryHi also contains the VPN address
    let mask: m_uint64_t = mips64_cp0_get_vpn2_mask(cpu);
    let vpn2: m_uint64_t = vaddr & mask;
    (*cpu).cp0.reg[MIPS_CP0_TLB_HI] &= !mask;
    (*cpu).cp0.reg[MIPS_CP0_TLB_HI] |= vpn2;
}

/// Increment count register and trigger the timer IRQ if value in compare
/// register is the same.
#[no_mangle]
pub unsafe extern "C" fn mips64_exec_inc_cp0_cnt(cpu: *mut cpu_mips_t) {
    (*cpu).cp0_virt_cnt_reg += 1;

    if false {
        // TIMER_IRQ
        let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);

        if unlikely((*cpu).cp0_virt_cnt_reg == (*cpu).cp0_virt_cmp_reg) {
            (*cp0).reg[MIPS_CP0_COUNT] = (*cp0).reg[MIPS_CP0_COMPARE] as m_uint32_t as m_uint64_t;
            mips64_set_irq(cpu, 7);
            mips64_update_irq_flag_fast(cpu);
        }
    }
}

/// Trigger the Timer IRQ
#[no_mangle]
pub unsafe extern "C" fn mips64_trigger_timer_irq(cpu: *mut cpu_mips_t) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);

    (*cpu).timer_irq_count += 1;

    (*cp0).reg[MIPS_CP0_COUNT] = (*cp0).reg[MIPS_CP0_COMPARE] as m_uint32_t as m_uint64_t;
    mips64_set_irq(cpu, 7);
    mips64_update_irq_flag_fast(cpu);
}

/// Execute ERET instruction
#[no_mangle]
pub unsafe extern "C" fn mips64_exec_eret(cpu: *mut cpu_mips_t) {
    let cp0: *mut mips_cp0_t = addr_of_mut!((*cpu).cp0);

    if ((*cp0).reg[MIPS_CP0_STATUS] & MIPS_CP0_STATUS_ERL as m_uint64_t) != 0 {
        (*cp0).reg[MIPS_CP0_STATUS] &= !(MIPS_CP0_STATUS_ERL as m_uint64_t);
        (*cpu).pc = (*cp0).reg[MIPS_CP0_ERR_EPC];
    } else {
        (*cp0).reg[MIPS_CP0_STATUS] &= !(MIPS_CP0_STATUS_EXL as m_uint64_t);
        (*cpu).pc = (*cp0).reg[MIPS_CP0_EPC];
    }

    // We have to clear the LLbit
    (*cpu).ll_bit = 0;

    // Update the pending IRQ flag
    mips64_update_irq_flag_fast(cpu);
}

/// Execute SYSCALL instruction
#[no_mangle]
pub unsafe extern "C" fn mips64_exec_syscall(cpu: *mut cpu_mips_t) {
    if DEBUG_SYSCALL != 0 {
        libc::printf(cstr!("MIPS64: SYSCALL at PC=0x%llx (RA=0x%llx)\n   a0=0x%llx, a1=0x%llx, a2=0x%llx, a3=0x%llx\n"), (*cpu).pc, (*cpu).gpr[MIPS_GPR_RA], (*cpu).gpr[MIPS_GPR_A0], (*cpu).gpr[MIPS_GPR_A1], (*cpu).gpr[MIPS_GPR_A2], (*cpu).gpr[MIPS_GPR_A3]);
    }

    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        // XXX TODO: Branch Delay slot
        mips64_trigger_exception(cpu, MIPS_CP0_CAUSE_SYSCALL, 0);
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        mips64_general_exception(cpu, MIPS_CP0_CAUSE_SYSCALL);
    }
}

/// Execute BREAK instruction
#[no_mangle]
pub unsafe extern "C" fn mips64_exec_break(cpu: *mut cpu_mips_t, code: u_int) {
    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        libc::printf(cstr!("MIPS64: BREAK instruction (code=%u)\n"), code);
        mips64_dump_regs((*cpu).gen);

        // XXX TODO: Branch Delay slot
        mips64_trigger_exception(cpu, MIPS_CP0_CAUSE_BP, 0);
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        cpu_log!((*cpu).gen, cstr!("MIPS64"), cstr!("BREAK instruction (code=%u)\n"), code);
        mips64_general_exception(cpu, MIPS_CP0_CAUSE_BP);
    }
}

/// Trigger a Trap Exception
#[no_mangle]
pub unsafe extern "C" fn mips64_trigger_trap_exception(cpu: *mut cpu_mips_t) {
    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        // XXX TODO: Branch Delay slot
        libc::printf(cstr!("MIPS64: TRAP exception, CPU=%p\n"), cpu);
        mips64_trigger_exception(cpu, MIPS_CP0_CAUSE_TRAP, 0);
    }
    #[cfg(feature = "USE_UNSTABLE")]
    {
        cpu_log!((*cpu).gen, cstr!("MIPS64"), cstr!("TRAP exception\n"));
        mips64_general_exception(cpu, MIPS_CP0_CAUSE_TRAP);
    }
}

/// Trigger IRQs
#[no_mangle]
pub unsafe extern "C" fn mips64_trigger_irq(cpu: *mut cpu_mips_t) {
    if unlikely((*cpu).irq_disable.get() != 0) {
        (*cpu).irq_pending = 0;
        return;
    }

    (*cpu).irq_count += 1;
    if mips64_update_irq_flag_fast(cpu) != 0 {
        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            mips64_trigger_exception(cpu, MIPS_CP0_CAUSE_INTERRUPT, 0);
        }
        #[cfg(feature = "USE_UNSTABLE")]
        {
            mips64_general_exception(cpu, MIPS_CP0_CAUSE_INTERRUPT);
        }
    } else {
        (*cpu).irq_fp_count += 1;
    }
}

/// DMFC1
#[no_mangle]
pub unsafe extern "C" fn mips64_exec_dmfc1(cpu: *mut cpu_mips_t, gp_reg: u_int, cp1_reg: u_int) {
    (*cpu).gpr[gp_reg as usize] = (*cpu).fpu.reg[cp1_reg as usize];
}

/// DMTC1
#[no_mangle]
pub unsafe extern "C" fn mips64_exec_dmtc1(cpu: *mut cpu_mips_t, gp_reg: u_int, cp1_reg: u_int) {
    (*cpu).fpu.reg[cp1_reg as usize] = (*cpu).gpr[gp_reg as usize];
}

/// MFC1
#[no_mangle]
pub unsafe extern "C" fn mips64_exec_mfc1(cpu: *mut cpu_mips_t, gp_reg: u_int, cp1_reg: u_int) {
    let val: m_int64_t = ((*cpu).fpu.reg[cp1_reg as usize] & 0xffffffff) as m_int64_t;
    (*cpu).gpr[gp_reg as usize] = sign_extend(val, 32) as m_uint64_t;
}

/// MTC1
#[no_mangle]
pub unsafe extern "C" fn mips64_exec_mtc1(cpu: *mut cpu_mips_t, gp_reg: u_int, cp1_reg: u_int) {
    (*cpu).fpu.reg[cp1_reg as usize] = (*cpu).gpr[gp_reg as usize] & 0xffffffff;
}

/// Virtual breakpoint
#[no_mangle]
pub unsafe extern "C" fn mips64_run_breakpoint(cpu: *mut cpu_mips_t) {
    cpu_log!((*cpu).gen, cstr!("BREAKPOINT"), cstr!("Virtual breakpoint reached at PC=0x%llx\n"), (*cpu).pc);

    libc::printf(cstr!("[[[ Virtual Breakpoint reached at PC=0x%llx RA=0x%llx]]]\n"), (*cpu).pc, (*cpu).gpr[MIPS_GPR_RA]);

    mips64_dump_regs((*cpu).gen);
    memlog_dump((*cpu).gen);
}

/// Add a virtual breakpoint
#[no_mangle]
pub unsafe extern "C" fn mips64_add_breakpoint(cpu: *mut cpu_gen_t, pc: m_uint64_t) {
    unsafe fn mips64_add_breakpoint(cpu: *mut cpu_gen_t, pc: m_uint64_t) -> c_int {
        let mcpu: *mut cpu_mips_t = CPU_MIPS64(cpu);
        let mut i: c_int;

        i = 0;
        while i < MIPS64_MAX_BREAKPOINTS as c_int {
            if (*mcpu).breakpoints[i as usize] == 0 {
                break;
            }
            i += 1;
        }

        if i == MIPS64_MAX_BREAKPOINTS as c_int {
            return -1;
        }

        (*mcpu).breakpoints[i as usize] = pc;
        (*mcpu).breakpoints_enabled = TRUE as u_int;
        0
    }
    mips64_add_breakpoint(cpu, pc);
}

/// Remove a virtual breakpoint
#[no_mangle]
pub unsafe extern "C" fn mips64_remove_breakpoint(cpu: *mut cpu_gen_t, pc: m_uint64_t) {
    let mcpu: *mut cpu_mips_t = CPU_MIPS64(cpu);

    for i in 0..MIPS64_MAX_BREAKPOINTS as c_int {
        if (*mcpu).breakpoints[i as usize] == pc {
            for j in i..(MIPS64_MAX_BREAKPOINTS - 1) as c_int {
                (*mcpu).breakpoints[j as usize] = (*mcpu).breakpoints[j as usize + 1];
            }

            (*mcpu).breakpoints[MIPS64_MAX_BREAKPOINTS - 1] = 0;
        }
    }

    for i in 0..MIPS64_MAX_BREAKPOINTS as c_int {
        if (*mcpu).breakpoints[i as usize] != 0 {
            return;
        }
    }

    (*mcpu).breakpoints_enabled = FALSE as u_int;
}

/// Debugging for register-jump to address 0
#[no_mangle]
pub unsafe extern "C" fn mips64_debug_jr0(cpu: *mut cpu_mips_t) {
    libc::printf(cstr!("MIPS64: cpu %p jumping to address 0...\n"), cpu);
    mips64_dump_regs((*cpu).gen);
}

/// Set a register
#[no_mangle]
pub unsafe extern "C" fn mips64_reg_set(cpu: *mut cpu_gen_t, reg: u_int, val: m_uint64_t) {
    if reg < MIPS64_GPR_NR as u_int {
        (*CPU_MIPS64(cpu)).gpr[reg as usize] = val;
    }
}

/// Dump registers of a MIPS64 processor
#[no_mangle]
pub unsafe extern "C" fn mips64_dump_regs(cpu: *mut cpu_gen_t) {
    let mcpu: *mut cpu_mips_t = CPU_MIPS64(cpu);
    let insn: mips_insn_t;
    let mut buffer: [c_char; 80] = [0; 80];

    libc::printf(cstr!("MIPS64 Registers:\n"));

    for i in 0..(MIPS64_GPR_NR / 2) as c_int {
        libc::printf(cstr!("  %s ($%2d) = 0x%16.16llx   %s ($%2d) = 0x%16.16llx\n"), mips64_gpr_reg_names[(i * 2) as usize], i * 2, (*mcpu).gpr[(i * 2) as usize], mips64_gpr_reg_names[((i * 2) + 1) as usize], (i * 2) + 1, (*mcpu).gpr[((i * 2) + 1) as usize]);
    }

    libc::printf(cstr!("  lo = 0x%16.16llx, hi = 0x%16.16llx\n"), (*mcpu).lo, (*mcpu).hi);
    libc::printf(cstr!("  pc = 0x%16.16llx, ll_bit = %u\n"), (*mcpu).pc, (*mcpu).ll_bit);

    // Fetch the current instruction
    let ptr: *mut mips_insn_t = (*mcpu).mem_op_lookup.unwrap()(mcpu, (*mcpu).pc).cast::<_>();
    if !ptr.is_null() {
        insn = vmtoh32(*ptr);

        if mips64_dump_insn(buffer.as_c_mut(), buffer.len(), 1, (*mcpu).pc, insn) != -1 {
            libc::printf(cstr!("  Instruction: %s\n"), buffer);
        }
    }

    libc::printf(cstr!("\nCP0 Registers:\n"));

    for i in 0..(MIPS64_CP0_REG_NR / 2) as c_int {
        libc::printf(cstr!("  %-10s ($%2d) = 0x%16.16llx   %-10s ($%2d) = 0x%16.16llx\n"), mips64_cp0_reg_names[(i * 2) as usize], i * 2, mips64_cp0_get_reg(mcpu, (i * 2) as u_int), mips64_cp0_reg_names[((i * 2) + 1) as usize], (i * 2) + 1, mips64_cp0_get_reg(mcpu, ((i * 2) + 1) as u_int));
    }

    libc::printf(cstr!("\n  IRQ count: %llu, IRQ false positives: %llu, IRQ Pending: %u\n"), (*mcpu).irq_count, (*mcpu).irq_fp_count, (*mcpu).irq_pending);

    libc::printf(cstr!("  Timer IRQ count: %llu, pending: %u, timer drift: %u\n\n"), (*mcpu).timer_irq_count, (*mcpu).timer_irq_pending, (*mcpu).timer_drift);

    libc::printf(cstr!("  Device access count: %llu\n"), (*cpu).dev_access_counter);
    libc::printf(cstr!("\n"));
}

/// Dump a memory block
#[no_mangle]
pub unsafe extern "C" fn mips64_dump_memory(cpu: *mut cpu_mips_t, mut vaddr: m_uint64_t, count: u_int) {
    let mut haddr: *mut c_void;

    for i in 0..count {
        if (i & 3) == 0 {
            libc::printf(cstr!("\n  0x%16.16llx: "), vaddr);
        }

        haddr = (*cpu).mem_op_lookup.unwrap()(cpu, vaddr);

        if !haddr.is_null() {
            libc::printf(cstr!("0x%8.8x "), htovm32(*haddr.cast::<m_uint32_t>()));
        } else {
            libc::printf(cstr!("XXXXXXXXXX "));
        }
        vaddr += 4;
    }

    libc::printf(cstr!("\n\n"));
}

/// Dump the stack
#[no_mangle]
pub unsafe extern "C" fn mips64_dump_stack(cpu: *mut cpu_mips_t, count: u_int) {
    libc::printf(cstr!("MIPS Stack Dump at 0x%16.16llx:"), (*cpu).gpr[MIPS_GPR_SP]);
    mips64_dump_memory(cpu, (*cpu).gpr[MIPS_GPR_SP], count);
}

/// Save the CPU state into a file
#[no_mangle]
pub unsafe extern "C" fn mips64_save_state(cpu: *mut cpu_mips_t, filename: *mut c_char) -> c_int {
    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("w"));
    if fd.is_null() {
        libc::perror(cstr!("mips64_save_state: fopen"));
        return -1;
    }

    // pc, lo and hi
    libc::fprintf(fd, cstr!("pc: %16.16llx\n"), (*cpu).pc);
    libc::fprintf(fd, cstr!("lo: %16.16llx\n"), (*cpu).lo);
    libc::fprintf(fd, cstr!("hi: %16.16llx\n"), (*cpu).hi);

    // general purpose registers
    for i in 0..MIPS64_GPR_NR as c_int {
        libc::fprintf(fd, cstr!("%s: %16.16llx\n"), mips64_gpr_reg_names[i as usize], (*cpu).gpr[i as usize]);
    }

    libc::printf(cstr!("\n"));

    // cp0 registers
    for i in 0..MIPS64_CP0_REG_NR as c_int {
        libc::fprintf(fd, cstr!("%s: %16.16llx\n"), mips64_cp0_reg_names[i as usize], (*cpu).cp0.reg[i as usize]);
    }

    libc::printf(cstr!("\n"));

    // cp1 registers
    for i in 0..MIPS64_CP1_REG_NR as c_int {
        libc::fprintf(fd, cstr!("fpu%d: %16.16llx\n"), i, (*cpu).fpu.reg[i as usize]);
    }

    libc::printf(cstr!("\n"));

    // tlb entries
    for i in 0..(*cpu).cp0.tlb_entries as c_int {
        libc::fprintf(fd, cstr!("tlb%d_mask: %16.16llx\n"), i, (*cpu).cp0.tlb[i as usize].mask);
        libc::fprintf(fd, cstr!("tlb%d_hi: %16.16llx\n"), i, (*cpu).cp0.tlb[i as usize].hi);
        libc::fprintf(fd, cstr!("tlb%d_lo0: %16.16llx\n"), i, (*cpu).cp0.tlb[i as usize].lo0);
        libc::fprintf(fd, cstr!("tlb%d_lo1: %16.16llx\n"), i, (*cpu).cp0.tlb[i as usize].lo1);
    }

    libc::fclose(fd);
    0
}

/// Read a 64-bit unsigned integer
unsafe fn mips64_hex_u64(mut str_: *mut c_char, _err: *mut c_int) -> m_uint64_t {
    let mut res: m_uint64_t = 0;
    let mut c: u_char;

    // remove leading spaces
    while (*str_ == b' ' as c_char) || (*str_ == b'\t' as c_char) {
        str_ = str_.add(1);
    }

    while *str_ != 0 {
        c = *str_ as u_char;

        #[allow(clippy::manual_range_contains)]
        if (c >= b'0') && (c <= b'9') {
            res = (res << 4) + (c - b'0') as m_uint64_t;
        }

        #[allow(clippy::manual_range_contains)]
        if (c >= b'a') && (c <= b'f') {
            res = (res << 4) + ((c - b'a') + 10) as m_uint64_t;
        }

        #[allow(clippy::manual_range_contains)]
        if (c >= b'A') && (c <= b'F') {
            res = (res << 4) + ((c - b'A') + 10) as m_uint64_t;
        }

        str_ = str_.add(1);
    }

    res
}

/// Restore the CPU state from a file
#[no_mangle]
pub unsafe extern "C" fn mips64_restore_state(cpu: *mut cpu_mips_t, filename: *mut c_char) -> c_int {
    let mut buffer: [c_char; 4096] = [0; 4096];
    let mut sep: *mut c_char;
    let mut value: *mut c_char;
    let mut ep: *mut c_char;
    let mut field: *mut c_char;
    let mut len: size_t;
    let mut index: c_int;

    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("r"));
    if fd.is_null() {
        libc::perror(cstr!("mips64_restore_state: fopen"));
        return -1;
    }

    while libc::feof(fd) == 0 {
        buffer[0] = 0;
        libc::fgets(buffer.as_c_mut(), buffer.len() as c_int, fd);
        len = libc::strlen(buffer.as_c());

        if buffer[len - 1] == b'\n' as c_char {
            buffer[len - 1] = 0;
        }

        sep = libc::strchr(buffer.as_c(), b':' as c_int);
        if sep.is_null() {
            continue;
        }

        value = sep.add(1);
        *sep = 0;

        // gpr ?
        index = mips64_get_reg_index(buffer.as_c_mut());
        if index != -1 {
            (*cpu).gpr[index as usize] = mips64_hex_u64(value, null_mut());
            continue;
        }

        // cp0 register ?
        index = mips64_cp0_get_reg_index(buffer.as_c_mut());
        if index != -1 {
            (*cpu).cp0.reg[index as usize] = mips64_hex_u64(value, null_mut());
            continue;
        }

        // cp1 register ?
        if (len > 3) && libc::strncmp(buffer.as_c(), cstr!("fpu"), 3) == 0 {
            index = libc::atoi(buffer.as_c().add(3));
            (*cpu).fpu.reg[index as usize] = mips64_hex_u64(value, null_mut());
        }

        // tlb entry ?
        if (len > 3) && libc::strncmp(buffer.as_c(), cstr!("tlb"), 3) == 0 {
            ep = libc::strchr(buffer.as_c(), b'_' as c_int);

            if !ep.is_null() {
                index = libc::atoi(buffer.as_c().add(3));
                field = ep.add(1);

                if libc::strcmp(field, cstr!("mask")) == 0 {
                    (*cpu).cp0.tlb[index as usize].mask = mips64_hex_u64(value, null_mut());
                    continue;
                }

                if libc::strcmp(field, cstr!("hi")) == 0 {
                    (*cpu).cp0.tlb[index as usize].hi = mips64_hex_u64(value, null_mut());
                    continue;
                }

                if libc::strcmp(field, cstr!("lo0")) == 0 {
                    (*cpu).cp0.tlb[index as usize].lo0 = mips64_hex_u64(value, null_mut());
                    continue;
                }

                if libc::strcmp(field, cstr!("lo1")) == 0 {
                    (*cpu).cp0.tlb[index as usize].lo1 = mips64_hex_u64(value, null_mut());
                    continue;
                }
            }
        }

        // pc, lo, hi ?
        if libc::strcmp(buffer.as_c(), cstr!("pc")) == 0 {
            (*cpu).pc = mips64_hex_u64(value, null_mut());
            continue;
        }

        if libc::strcmp(buffer.as_c(), cstr!("lo")) == 0 {
            (*cpu).lo = mips64_hex_u64(value, null_mut());
            continue;
        }

        if libc::strcmp(buffer.as_c(), cstr!("hi")) == 0 {
            (*cpu).hi = mips64_hex_u64(value, null_mut());
            continue;
        }
    }

    #[cfg(not(feature = "USE_UNSTABLE"))]
    {
        mips64_cp0_map_all_tlb_to_mts(cpu);
    }

    mips64_dump_regs((*cpu).gen);
    mips64_tlb_dump((*cpu).gen);

    libc::fclose(fd);
    0
}

/// Load a raw image into the simulated memory
#[no_mangle]
pub unsafe extern "C" fn mips64_load_raw_image(cpu: *mut cpu_mips_t, filename: *mut c_char, mut vaddr: m_uint64_t) -> c_int {
    let mut file_info: libc::stat = zeroed::<_>();
    let mut len: size_t;
    let mut clen: size_t;
    let mut remain: m_uint32_t;
    let mut haddr: *mut c_void;

    let bfd: *mut libc::FILE = libc::fopen(filename, cstr!("r"));
    if bfd.is_null() {
        libc::perror(cstr!("fopen"));
        return -1;
    }

    if libc::fstat(libc::fileno(bfd), addr_of_mut!(file_info)) == -1 {
        libc::perror(cstr!("stat"));
        libc::fclose(bfd);
        return -1;
    }

    len = file_info.st_size as size_t;

    libc::printf(cstr!("Loading RAW file '%s' at virtual address 0x%llx (size=%lu)\n"), filename, vaddr, len as u_long);

    while len > 0 {
        haddr = (*cpu).mem_op_lookup.unwrap()(cpu, vaddr);

        if haddr.is_null() {
            libc::fprintf(c_stderr(), cstr!("load_raw_image: invalid load address 0x%llx\n"), vaddr);
            libc::fclose(bfd);
            return -1;
        }

        if len > MIPS_MIN_PAGE_SIZE {
            clen = MIPS_MIN_PAGE_SIZE;
        } else {
            clen = len;
        }

        remain = MIPS_MIN_PAGE_SIZE as m_uint32_t;
        remain -= (vaddr - (vaddr & MIPS_MIN_PAGE_MASK)) as m_uint32_t;

        clen = m_min(clen, remain as size_t);

        if libc::fread(haddr.cast::<u_char>().cast::<_>(), clen, 1, bfd) != 1 {
            break;
        }

        vaddr += clen as m_uint64_t;
        len -= clen;
    }

    libc::fclose(bfd);
    0
}

/// Load an ELF image into the simulated memory
#[no_mangle]
pub unsafe extern "C" fn mips64_load_elf_image(cpu: *mut cpu_mips_t, filename: *mut c_char, skip_load: c_int, entry_point: *mut m_uint32_t) -> c_int {
    let mut vaddr: m_uint64_t;
    let mut remain: m_uint32_t;
    let mut haddr: *mut c_void;
    let mut shdr: *mut libelf_sys::Elf32_Shdr;
    let mut scn: *mut libelf_sys::Elf_Scn;
    let mut len: size_t;
    let mut clen: size_t;
    let mut name: *mut c_char;
    let fd: c_int;

    if filename.is_null() {
        return -1;
    }

    #[cfg(if_0)]
    {
        // ifdef __CYGWIN__
        fd = libc::open(filename, libc::O_RDONLY | libc::O_BINARY);
    }
    #[cfg(not(if_0))]
    {
        fd = libc::open(filename, libc::O_RDONLY);
    }

    if fd == -1 {
        libc::perror(cstr!("load_elf_image: open"));
        return -1;
    }

    if libelf_sys::elf_version(libelf_sys::EV_CURRENT) == libelf_sys::EV_NONE {
        libc::fprintf(c_stderr(), cstr!("load_elf_image: library out of date\n"));
        libc::close(fd);
        return -1;
    }

    let img_elf: *mut libelf_sys::Elf = libelf_sys::elf_begin(fd, libelf_sys::Elf_Cmd::ELF_C_READ, null_mut());
    if img_elf.is_null() {
        libc::fprintf(c_stderr(), cstr!("load_elf_image: elf_begin: %s\n"), libelf_sys::elf_errmsg(libelf_sys::elf_errno()));
        libc::close(fd);
        return -1;
    }

    let ehdr: *mut libelf_sys::Elf32_Ehdr = libelf_sys::elf32_getehdr(img_elf);
    if ehdr.is_null() {
        libc::fprintf(c_stderr(), cstr!("load_elf_image: invalid ELF file\n"));
        libelf_sys::elf_end(img_elf);
        libc::close(fd);
        return -1;
    }

    libc::printf(cstr!("Loading ELF file '%s'...\n"), filename);
    let bfd: *mut libc::FILE = libc::fdopen(fd, cstr!("rb"));

    if bfd.is_null() {
        libc::perror(cstr!("load_elf_image: fdopen"));
        libelf_sys::elf_end(img_elf);
        libc::close(fd);
        return -1;
    }

    if skip_load == 0 {
        for i in 0..(*ehdr).e_shnum as c_int {
            scn = libelf_sys::elf_getscn(img_elf, i as size_t);

            shdr = libelf_sys::elf32_getshdr(scn);
            name = libelf_sys::elf_strptr(img_elf, (*ehdr).e_shstrndx as size_t, (*shdr).sh_name as size_t);
            len = (*shdr).sh_size as size_t;

            if ((*shdr).sh_flags & libelf_sys::SHF_ALLOC) == 0 || len == 0 {
                continue;
            }

            if libc::fseek(bfd, (*shdr).sh_offset as c_long, libc::SEEK_SET) != 0 {
                libc::perror(cstr!("load_elf_image: fseek"));
                libelf_sys::elf_end(img_elf);
                libc::fclose(bfd);
                return -1;
            }
            vaddr = sign_extend((*shdr).sh_addr as m_int64_t, 32) as m_uint64_t;

            if (*(*cpu).vm).debug_level > 0 {
                libc::printf(cstr!("   * Adding section at virtual address 0x%8.8llx (len=0x%8.8lx)\n"), vaddr & 0xFFFFFFFF, len as u_long);
            }

            while len > 0 {
                haddr = (*cpu).mem_op_lookup.unwrap()(cpu, vaddr);

                if haddr.is_null() {
                    libc::fprintf(c_stderr(), cstr!("load_elf_image: invalid load address 0x%llx\n"), vaddr);
                    libelf_sys::elf_end(img_elf);
                    libc::fclose(bfd);
                    return -1;
                }

                if len > MIPS_MIN_PAGE_SIZE {
                    clen = MIPS_MIN_PAGE_SIZE;
                } else {
                    clen = len;
                }

                remain = PPC32_MIN_PAGE_SIZE as m_uint32_t; // FIXME should be MIPS_MIN_PAGE_SIZE?
                remain -= (vaddr - (vaddr & PPC32_MIN_PAGE_MASK as m_uint64_t)) as m_uint32_t; // FIXME should be MIPS_MIN_PAGE_MASK?

                clen = m_min(clen, remain as size_t);

                if (*shdr).sh_type == libelf_sys::SHT_NOBITS {
                    // section with uninitialized data, zero it
                    libc::memset(haddr.cast::<u_char>().cast::<_>(), 0, clen);
                } else {
                    #[warn(clippy::collapsible_else_if)]
                    if libc::fread(haddr.cast::<u_char>().cast::<_>(), clen, 1, bfd) != 1 {
                        libc::perror(cstr!("load_elf_image: fread"));
                        libelf_sys::elf_end(img_elf);
                        libc::fclose(bfd);
                        return -1;
                    }
                }

                vaddr += clen as m_uint64_t;
                len -= clen;
            }
            let _ = name;
        }
    } else {
        libc::printf(cstr!("ELF loading skipped, using a ghost RAM file.\n"));
    }

    libc::printf(cstr!("ELF entry point: 0x%x\n"), (*ehdr).e_entry);

    if !entry_point.is_null() {
        *entry_point = (*ehdr).e_entry;
    }

    libelf_sys::elf_end(img_elf);
    libc::fclose(bfd);
    0
}

/// Symbol lookup
#[no_mangle]
pub unsafe extern "C" fn mips64_sym_lookup(cpu: *mut cpu_mips_t, mut addr: m_uint64_t) -> *mut symbol {
    rbtree_lookup((*cpu).sym_tree, addr_of_mut!(addr).cast::<_>()).cast::<_>()
}

/// Insert a new symbol
#[no_mangle]
pub unsafe extern "C" fn mips64_sym_insert(cpu: *mut cpu_mips_t, name: *mut c_char, addr: m_uint64_t) -> *mut symbol {
    if (*cpu).sym_tree.is_null() {
        return null_mut();
    }

    let len: size_t = libc::strlen(name);

    let sym: *mut symbol = libc::malloc(len + 1 + size_of::<symbol>()).cast::<_>();
    if sym.is_null() {
        return null_mut();
    }

    libc::memcpy((*sym).name.as_c_void_mut(), name.cast::<_>(), len + 1);
    (*sym).addr = addr;

    if rbtree_insert((*cpu).sym_tree, sym.cast::<_>(), sym.cast::<_>()) == -1 {
        libc::free(sym.cast::<_>());
        return null_mut();
    }

    sym
}

/// Symbol comparison function
unsafe extern "C" fn mips64_sym_compare(a1: *mut c_void, sym: *mut c_void, _: *mut c_void) -> c_int {
    let a1: *mut m_uint64_t = a1.cast::<_>();
    let sym: *mut symbol = sym.cast::<_>();
    if *a1 > (*sym).addr {
        return 1;
    }

    if *a1 < (*sym).addr {
        return -1;
    }

    0
}

/// Create the symbol tree
#[no_mangle]
pub unsafe extern "C" fn mips64_sym_create_tree(cpu: *mut cpu_mips_t) -> c_int {
    (*cpu).sym_tree = rbtree_create(Some(mips64_sym_compare), null_mut());
    if !(*cpu).sym_tree.is_null() {
        0
    } else {
        -1
    }
}

/// Load a symbol file
#[no_mangle]
pub unsafe extern "C" fn mips64_sym_load_file(cpu: *mut cpu_mips_t, filename: *mut c_char) -> c_int {
    let mut buffer: [c_char; 4096] = [0; 4096];
    let mut func_name: [c_char; 128] = [0; 128];
    let mut addr: m_uint64_t = 0;
    let mut sym_type: c_char = 0;

    if (*cpu).sym_tree.is_null() && (mips64_sym_create_tree(cpu) == -1) {
        libc::fprintf(c_stderr(), cstr!("CPU%u: Unable to create symbol tree.\n"), (*(*cpu).gen).id);
        return -1;
    }

    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("r"));
    if fd.is_null() {
        libc::perror(cstr!("load_sym_file: fopen"));
        return -1;
    }

    while libc::feof(fd) == 0 {
        libc::fgets(buffer.as_c_mut(), buffer.len() as c_int, fd);

        if libc::sscanf(buffer.as_c(), cstr!("%llx %c %s"), addr_of_mut!(addr), addr_of_mut!(sym_type), func_name) == 3 {
            mips64_sym_insert(cpu, func_name.as_c_mut(), addr);
        }
    }

    libc::fclose(fd);
    0
}
