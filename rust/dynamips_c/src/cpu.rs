//! Management of CPU groups (for MP systems).

use crate::dynamips_common::*;
use crate::jit_op::*;
use crate::mips64::*;
use crate::ppc32::*;
use crate::prelude::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::vm::*;

pub type memlog_access_t = memlog_access;
pub type cpu_gen_t = cpu_gen;
pub type cpu_group_t = cpu_group;

/// Maximum results for idle pc
pub const CPU_IDLE_PC_MAX_RES: usize = 10;

/// Idle PC proposed value
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_idle_pc {
    pub pc: m_uint64_t,
    pub count: u_int,
}

/// Number of recorded memory accesses (power of two)
pub const MEMLOG_COUNT: usize = 16;

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

/// Undefined memory access handler
pub type cpu_undefined_mem_handler_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, vaddr: m_uint64_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> c_int>;

#[repr(C)]
#[derive(Copy, Clone)]
pub union cpu_gen_sp {
    pub mips64_cpu: cpu_mips_t,
    pub ppc32_cpu: cpu_ppc_t,
}

/// Generic CPU definition
#[repr(C)]
#[derive(Copy, Clone)]
pub struct cpu_gen {
    /// CPU type and identifier for MP systems
    pub type_: u_int,
    pub id: u_int,

    /// CPU states
    pub state: Volatile<u_int>,
    pub prev_state: Volatile<u_int>,
    pub seq_state: Volatile<m_uint64_t>,

    /// Thread running this CPU
    pub cpu_thread: libc::pthread_t,
    pub cpu_thread_running: Volatile<c_int>,

    /// Exception restore point
    pub exec_loop_env: setjmp::jmp_buf,

    /// "Idle" loop management
    pub idle_count: u_int,
    pub idle_max: u_int,
    pub idle_sleep_time: u_int,
    pub idle_mutex: libc::pthread_mutex_t,
    pub idle_cond: libc::pthread_cond_t,

    /// VM instance
    pub vm: *mut vm_instance_t,

    /// Next CPU in group
    pub next: *mut cpu_gen_t,

    /// Idle PC proposal
    pub idle_pc_prop: [cpu_idle_pc; CPU_IDLE_PC_MAX_RES],
    pub idle_pc_prop_count: u_int,

    /// Specific CPU part
    pub sp: cpu_gen_sp,

    /// Methods
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

    /// Memory access log for fault debugging
    pub memlog_pos: u_int,
    pub memlog_array: [memlog_access_t; MEMLOG_COUNT],

    /// Statistics
    pub dev_access_counter: m_uint64_t,

    /// JIT op data
    pub jit_op_data: jit_op_data_t,

    /// Translation group ID and TCB descriptor local list
    #[cfg(feature = "USE_UNSTABLE")]
    pub tsg: c_int,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tc_local_list: *mut cpu_tc_t,

    /// Current and free lists of TBs
    #[cfg(feature = "USE_UNSTABLE")]
    pub tb_list: *mut cpu_tb_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tb_free_list: *mut cpu_tb_t,

    /// Virtual and Physical hash tables to retrieve TBs
    #[cfg(feature = "USE_UNSTABLE")]
    pub tb_virt_hash: *mut *mut cpu_tb_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tb_phys_hash: *mut *mut cpu_tb_t,

    /// CPU List for a Translation Sharing Group
    #[cfg(feature = "USE_UNSTABLE")]
    pub tsg_pprev: *mut *mut cpu_gen_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub tsg_next: *mut cpu_gen_t,
}

/// CPU group
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_group {
    pub name: *mut c_char,
    pub cpu_list: *mut cpu_gen_t,
    pub priv_data: *mut c_void,
}

/// Find a CPU in a group given its ID
#[no_mangle]
pub unsafe extern "C" fn cpu_group_find_id(group: *mut cpu_group_t, id: u_int) -> *mut cpu_gen_t {
    if group.is_null() {
        return null_mut();
    }

    let mut cpu: *mut cpu_gen_t = (*group).cpu_list;
    while !cpu.is_null() {
        if (*cpu).id == id {
            return cpu;
        }
        cpu = (*cpu).next
    }

    null_mut()
}
