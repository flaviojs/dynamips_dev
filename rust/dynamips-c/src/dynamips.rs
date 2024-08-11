//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Many thanks to Nicolas Szalay for his patch
//! for the command line parsing and virtual machine
//! settings (RAM, ROM, NVRAM, ...)

use crate::_private::*;
use crate::dynamips_common::*;

/// Debugging flags
pub const DEBUG_BLOCK_SCAN: c_int = 0;
pub const DEBUG_BLOCK_COMPILE: c_int = 0;
pub const DEBUG_BLOCK_PATCH: c_int = 0;
pub const DEBUG_BLOCK_CHUNK: c_int = 0;
#[cfg(not(feature = "USE_DEBUG_BLOCK_TIMESTAMP"))]
pub const DEBUG_BLOCK_TIMESTAMP: c_int = 0; // block timestamping (little overhead)
#[cfg(feature = "USE_DEBUG_BLOCK_TIMESTAMP")]
pub const DEBUG_BLOCK_TIMESTAMP: c_int = 1;
pub const DEBUG_SYM_TREE: c_int = 0; // use symbol tree (slow)
pub const DEBUG_MTS_MAP_DEV: c_int = 0;
pub const DEBUG_MTS_MAP_VIRT: c_int = 1;
pub const DEBUG_MTS_ACC_U: c_int = 1; // undefined memory
pub const DEBUG_MTS_ACC_T: c_int = 1; // tlb exception
pub const DEBUG_MTS_ACC_AE: c_int = 1; // address error exception
pub const DEBUG_MTS_DEV: c_int = 0; // debugging for device access
pub const DEBUG_MTS_STATS: c_int = 1; // MTS cache performance
pub const DEBUG_INSN_PERF_CNT: c_int = 0; // Instruction performance counter
pub const DEBUG_BLOCK_PERF_CNT: c_int = 0; // Block performance counter
pub const DEBUG_DEV_PERF_CNT: c_int = 1; // Device performance counter
pub const DEBUG_TLB_ACTIVITY: c_int = 0;
pub const DEBUG_SYSCALL: c_int = 0;
pub const DEBUG_CACHE: c_int = 0;
pub const DEBUG_JR0: c_int = 0; // Debug register jumps to 0

/// Feature flags
pub const MEMLOG_ENABLE: c_int = 0; // Memlogger (fast memop must be off)
pub const BREAKPOINT_ENABLE: c_int = 1; // Virtual Breakpoints
pub const NJM_STATS_ENABLE: c_int = 1; // Non-JIT mode stats (little overhead)

/// Symbol
#[repr(C)]
#[derive(Debug)]
pub struct symbol {
    pub addr: m_uint64_t,
    pub name: [c_char; 0], // XXX length determined by the C string NUL terminator
}

/// ROM identification tag
pub const ROM_ID: m_uint32_t = 0x1e94b3df;

/// Command Line long options
pub const OPT_DISK0_SIZE: c_int = 0x100;
pub const OPT_DISK1_SIZE: c_int = 0x101;
pub const OPT_EXEC_AREA: c_int = 0x102;
pub const OPT_IDLE_PC: c_int = 0x103;
pub const OPT_TIMER_ITV: c_int = 0x104;
pub const OPT_VM_DEBUG: c_int = 0x105;
pub const OPT_IOMEM_SIZE: c_int = 0x106;
pub const OPT_SPARSE_MEM: c_int = 0x107;
pub const OPT_NOCTRL: c_int = 0x120;
pub const OPT_NOTELMSG: c_int = 0x121;
pub const OPT_FILEPID: c_int = 0x122;
pub const OPT_STARTUP_CONFIG_FILE: c_int = 0x140;
pub const OPT_PRIVATE_CONFIG_FILE: c_int = 0x141;
pub const OPT_CONSOLE_BINDING_ADDR: c_int = 0x150;
