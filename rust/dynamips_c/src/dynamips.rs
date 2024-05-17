//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Many thanks to Nicolas Szalay for his patch
//! for the command line parsing and virtual machine
//! settings (RAM, ROM, NVRAM, ...)

use crate::_private::*;

// Debugging flags
pub const DEBUG_BLOCK_SCAN: c_int = 0;
pub const DEBUG_BLOCK_COMPILE: c_int = 0;
pub const DEBUG_BLOCK_PATCH: c_int = 0;
pub const DEBUG_BLOCK_CHUNK: c_int = 0;
/// block timestamping (little overhead)
pub const DEBUG_BLOCK_TIMESTAMP: c_int = 0;
/// use symbol tree (slow)
pub const DEBUG_SYM_TREE: c_int = 0;
pub const DEBUG_MTS_MAP_DEV: c_int = 0;
pub const DEBUG_MTS_MAP_VIRT: c_int = 1;
/// undefined memory
pub const DEBUG_MTS_ACC_U: c_int = 1;
/// tlb exception
pub const DEBUG_MTS_ACC_T: c_int = 1;
/// address error exception
pub const DEBUG_MTS_ACC_AE: c_int = 1;
/// debugging for device access
pub const DEBUG_MTS_DEV: c_int = 0;
/// MTS cache performance
pub const DEBUG_MTS_STATS: c_int = 1;
/// Instruction performance counter
pub const DEBUG_INSN_PERF_CNT: c_int = 0;
/// Block performance counter
pub const DEBUG_BLOCK_PERF_CNT: c_int = 0;
/// Device performance counter
pub const DEBUG_DEV_PERF_CNT: c_int = 1;
pub const DEBUG_TLB_ACTIVITY: c_int = 0;
pub const DEBUG_SYSCALL: c_int = 0;
pub const DEBUG_CACHE: c_int = 0;
/// Debug register jumps to 0
pub const DEBUG_JR0: c_int = 0;

// Feature flags
/// Memlogger (fast memop must be off)
pub const MEMLOG_ENABLE: c_int = 0;
/// Virtual Breakpoints
pub const BREAKPOINT_ENABLE: c_int = 1;
/// Non-JIT mode stats (little overhead)
pub const NJM_STATS_ENABLE: c_int = 1;

/// Software version tag
#[no_mangle]
pub static mut sw_version_tag: *const c_char = cstr!("2023010200");

/// Binding address (NULL means any or 0.0.0.0)
#[no_mangle]
pub static mut binding_addr: *mut c_char = null_mut();

/// Console (vtty tcp) binding address (NULL means any or 0.0.0.0)
#[no_mangle]
pub static mut console_binding_addr: *mut c_char = null_mut();
