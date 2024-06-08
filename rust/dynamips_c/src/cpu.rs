//! Management of CPU groups (for MP systems).

use crate::dynamips_common::*;
use crate::jit_op::*;
use crate::mips64::*;
use crate::ppc32::*;
use crate::prelude::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::vm::*;

extern "C" {
    pub fn cpu_delete(cpu: *mut cpu_gen_t);
    pub fn cpu_idle_loop(cpu: *mut cpu_gen_t);
    pub fn cpu_stop(cpu: *mut cpu_gen_t);
}

pub type memlog_access_t = memlog_access;
pub type cpu_gen_t = cpu_gen;
pub type cpu_group_t = cpu_group;

/// Virtual CPU states // TODO enum
pub const CPU_STATE_RUNNING: u_int = 0;
pub const CPU_STATE_HALTED: u_int = 1;
pub const CPU_STATE_SUSPENDED: u_int = 2;

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

// Possible CPU types // TODO enum
pub const CPU_TYPE_MIPS64: u_int = 1;
pub const CPU_TYPE_PPC32: u_int = 2;

pub unsafe fn CPU_MIPS64(cpu: *mut cpu_gen_t) -> *mut cpu_mips_t {
    addr_of_mut!((*cpu).sp.mips64_cpu)
}

pub unsafe fn CPU_PPC32(cpu: *mut cpu_gen_t) -> *mut cpu_ppc_t {
    addr_of_mut!((*cpu).sp.ppc32_cpu)
}

/// Set the exec loop entry point
#[macro_export]
macro_rules! cpu_exec_loop_set {
    ($cpu:expr) => {
        let cpu: *mut cpu_gen_t = $cpu;
        setjmp::setjmp(addr_of_mut!((*cpu).exec_loop_env));
    };
}
pub use cpu_exec_loop_set;

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

/// Log a message for a CPU
#[macro_export]
macro_rules! cpu_log {
    ($cpu:expr, $module:expr, $format:expr$(, $arg:expr)*) => {
        let cpu: *mut cpu_gen_t = $cpu;
        let module: *mut c_char = $module;
        let format: *mut c_char = $format;
        let args: &[&dyn sprintf::Printf] = &[$(&Printf($arg)),*];

        let mut buffer: [c_char; 256] = [0; 256];
        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            buffer[0] = b'C' as c_char;
            buffer[1] = b'P' as c_char;
            buffer[2] = b'U' as c_char;

            match  (*cpu).id{
                0 => buffer[3] = b'0' as c_char,
                1 => buffer[3] = b'1' as c_char,
                2 => buffer[3] = b'2' as c_char,
                3 => buffer[3] = b'3' as c_char,
                4 => buffer[3] = b'4' as c_char,
                5 => buffer[3] = b'5' as c_char,
                6 => buffer[3] = b'6' as c_char,
                7 => buffer[3] = b'7' as c_char,
                8 => buffer[3] = b'8' as c_char,
                9 => buffer[3] = b'9' as c_char,
                _ => buffer[3] = b'-' as c_char,
            }

            buffer[4] = b':' as c_char;
            buffer[5] = b' ' as c_char;

            let mut buf: *mut c_char = buffer.as_c_mut();
            buf = buf.add(6);
            let mut i: *mut c_char = module;
            while *i != 0 {
                *buf = *i;
                buf = buf.add(1);
                i = i.add(1);
            }

            *buf = 0;
        }
        #[cfg(feature = "USE_UNSTABLE")]
        {
            libc::snprintf(buffer.as_c_mut(), buffer.len(), cstr!("CPU%u: %s"), (*cpu).id, module);
        }

        $crate::vm::vm_flog((*cpu).vm, buffer.as_c_mut(), format, args)
    };
}
pub use cpu_log;

/// Find the highest CPU ID in a CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_find_highest_id(group: *mut cpu_group_t, highest_id: *mut u_int) -> c_int {
    let mut cpu: *mut cpu_gen_t;
    let mut max_id: u_int = 0;

    if group.is_null() || !(*group).cpu_list.is_null() {
        return -1;
    }

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        if (*cpu).id >= max_id {
            max_id = (*cpu).id;
        }
        cpu = (*cpu).next;
    }

    *highest_id = max_id;
    0
}

/// Add a CPU in a CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_add(group: *mut cpu_group_t, cpu: *mut cpu_gen_t) -> c_int {
    if group.is_null() {
        return -1;
    }

    // check that we don't already have a CPU with this id
    if !cpu_group_find_id(group, (*cpu).id).is_null() {
        libc::fprintf(c_stderr(), cstr!("cpu_group_add: CPU%u already present in group.\n"), (*cpu).id);
        return -1;
    }

    (*cpu).next = (*group).cpu_list;
    (*group).cpu_list = cpu;
    0
}

/// Create a new CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_create(name: *mut c_char) -> *mut cpu_group_t {
    let group: *mut cpu_group_t = libc::malloc(size_of::<cpu_group_t>()).cast::<_>();
    if group.is_null() {
        return null_mut();
    }

    (*group).name = name;
    (*group).cpu_list = null_mut();
    group
}

/// Delete a CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_delete(group: *mut cpu_group_t) {
    let mut cpu: *mut cpu_gen_t;
    let mut next: *mut cpu_gen_t;

    if !group.is_null() {
        cpu = (*group).cpu_list;
        while !cpu.is_null() {
            next = (*cpu).next;
            cpu_delete(cpu);
            cpu = next;
        }

        libc::free(group.cast::<_>());
    }
}

/// Rebuild the MTS subsystem for a CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_rebuild_mts(group: *mut cpu_group_t) -> c_int {
    let mut cpu: *mut cpu_gen_t;

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        (*cpu).mts_rebuild.unwrap()(cpu);
        cpu = (*cpu).next;
    }

    0
}
