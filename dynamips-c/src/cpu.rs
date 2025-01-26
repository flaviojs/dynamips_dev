//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::_extra::*;
use crate::dynamips_common::*;
use crate::jit_op::*;
use crate::mips64::*;
use crate::ppc32::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::vm::*;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;
use std::ptr::addr_of_mut;

pub type cpu_gen_t = cpu_gen;

// Possible CPU types
pub type _CPU_TYPE_ENUM = u_int; // TODO enum
pub const CPU_TYPE_MIPS64: _CPU_TYPE_ENUM = 1;
pub const CPU_TYPE_PPC32: _CPU_TYPE_ENUM = 2;

// Virtual CPU states
pub type _CPU_STATE_ENUM = c_int; // TODO enum
pub const CPU_STATE_RUNNING: _CPU_STATE_ENUM = 0;
pub const CPU_STATE_HALTED: _CPU_STATE_ENUM = 1;
pub const CPU_STATE_SUSPENDED: _CPU_STATE_ENUM = 2;

// Maximum results for idle pc
pub const CPU_IDLE_PC_MAX_RES: usize = 10;

// Idle PC proposed value
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_idle_pc {
    pub pc: m_uint64_t,
    pub count: u_int,
}

// Number of recorded memory accesses (power of two)
pub const MEMLOG_COUNT: usize = 16;

pub type memlog_access_t = memlog_access;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct memlog_access {
    pub iaddr: m_uint64_t,
    pub vaddr: m_uint64_t,
    pub data: m_uint64_t,
    pub data_valid: m_uint32_t,
    pub op_size: m_uint32_t,
    pub op_type: m_uint32_t,
}

// Undefined memory access handler
pub type cpu_undefined_mem_handler_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, vaddr: m_uint64_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> c_int>;

// Generic CPU definition
#[repr(C)]
//#[derive(Copy, Clone)] // XXX setjmp::jmp_buf is not Copy or Clone
pub struct cpu_gen {
    // CPU type and identifier for MP systems
    pub r#type: u_int,
    pub id: u_int,

    // CPU states
    pub state: Volatile<u_int>,
    pub prev_state: Volatile<u_int>,
    pub seq_state: Volatile<m_uint64_t>,

    // Thread running this CPU
    pub cpu_thread: libc::pthread_t,
    pub cpu_thread_running: Volatile<c_int>,

    // Exception restore point
    pub exec_loop_env: setjmp::jmp_buf,

    // "Idle" loop management
    pub idle_count: u_int,
    pub idle_max: u_int,
    pub idle_sleep_time: u_int,
    pub idle_mutex: libc::pthread_mutex_t,
    pub idle_cond: libc::pthread_cond_t,

    // VM instance
    pub vm: *mut vm_instance_t,

    // Next CPU in group
    pub next: *mut cpu_gen_t,

    // Idle PC proposal
    pub idle_pc_prop: [cpu_idle_pc; CPU_IDLE_PC_MAX_RES],
    pub idle_pc_prop_count: u_int,

    // Specific CPU part
    pub sp: cpu_gen_sp,

    // Methods
    pub reg_set: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, reg_index: u_int, val: m_uint64_t)>,
    pub reg_dump: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t)>,
    pub mmu_dump: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t)>,
    pub mmu_raw_dump: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t)>,
    pub add_breakpoint: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, addr: m_uint64_t)>,
    pub remove_breakpoint: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, addr: m_uint64_t)>,
    pub set_idle_pc: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, addr: m_uint64_t)>,
    pub get_idling_pc: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t)>,
    pub mts_rebuild: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t)>,
    pub mts_show_stats: Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t)>,

    pub undef_mem_handler: cpu_undefined_mem_handler_t,

    // Memory access log for fault debugging
    pub memlog_pos: u_int,
    pub memlog_array: [memlog_access_t; MEMLOG_COUNT],

    // Statistics
    pub dev_access_counter: m_uint64_t,

    // JIT op array for current compiled pages
    pub jit_op_array_size: u_int,
    pub jit_op_array: *mut *mut jit_op_t,
    pub jit_op_current: *mut *mut jit_op_t,

    // Translation group ID and TCB descriptor local list
    #[cfg(feature = "USE_UNSTABLE")]
    pub tsg: c_int,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tc_local_list: *mut cpu_tc_t,

    // Current and free lists of TBs
    #[cfg(feature = "USE_UNSTABLE")]
    pub tb_list: *mut cpu_tb_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tb_free_list: *mut cpu_tb_t,

    // Virtual and Physical hash tables to retrieve TBs
    #[cfg(feature = "USE_UNSTABLE")]
    pub tb_virt_hash: *mut *mut cpu_tb_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tb_phys_hash: *mut *mut cpu_tb_t,

    // CPU List for a Translation Sharing Group
    #[cfg(feature = "USE_UNSTABLE")]
    pub tsg_pprev: *mut *mut cpu_gen_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tsg_next: *mut cpu_gen_t,

    // JIT op pool
    pub jit_op_pool: [*mut jit_op_t; JIT_OP_POOL_NR],
}
// Specific CPU part
#[repr(C)]
#[derive(Copy, Clone)]
pub union cpu_gen_sp {
    pub mips64_cpu: cpu_mips_t,
    pub ppc32_cpu: cpu_ppc_t,
}

// CPU group definition
pub type cpu_group_t = cpu_group;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_group {
    pub name: *mut c_char,
    pub cpu_list: *mut cpu_gen_t,
    pub priv_data: *mut c_void,
}

macro_rules! CPU_MIPS64 {
    ($cpu:expr) => {
        addr_of_mut!((*$cpu).sp.mips64_cpu)
    };
}
pub(crate) use CPU_MIPS64;
macro_rules! CPU_PPC32 {
    ($cpu:expr) => {
        addr_of_mut!((*$cpu).sp.ppc32_cpu)
    };
}
pub(crate) use CPU_PPC32;

// Get CPU instruction pointer
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn cpu_get_pc(cpu: *mut cpu_gen_t) -> m_uint64_t {
    #[allow(clippy::needless_return)]
    match (*cpu).r#type {
        CPU_TYPE_MIPS64 => {
            return (*CPU_MIPS64!(cpu)).pc;
        }
        CPU_TYPE_PPC32 => {
            return (*CPU_PPC32!(cpu)).ia as m_uint64_t;
        }
        _ => {
            return 0;
        }
    }
}

// Get CPU performance counter
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn cpu_get_perf_counter(cpu: *mut cpu_gen_t) -> m_uint32_t {
    #[allow(clippy::needless_return)]
    match (*cpu).r#type {
        CPU_TYPE_MIPS64 => {
            return (*CPU_MIPS64!(cpu)).perf_counter;
        }
        CPU_TYPE_PPC32 => {
            return (*CPU_PPC32!(cpu)).perf_counter;
        }
        _ => {
            return 0;
        }
    }
}

// Returns to the CPU exec loop
#[inline]
#[no_mangle]
pub unsafe extern "C" fn cpu_exec_loop_enter(cpu: *mut cpu_gen_t) {
    setjmp::longjmp(addr_of_mut!((*cpu).exec_loop_env), 1);
}

// Set the exec loop entry point
macro_rules! cpu_exec_loop_set {
    ($cpu:expr) => {
        setjmp::setjmp(addr_of_mut!((*$cpu).exec_loop_env))
    };
}
pub(crate) use cpu_exec_loop_set;

extern "C" {
    pub fn cpu_log(cpu: *mut cpu_gen_t, module: *mut c_char, format: *mut c_char, ...); // TODO replace
}
