//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Management of CPU groups (for MP systems).

use crate::_private::*;
use crate::dynamips_common::*;
use crate::jit_op::*;
use crate::mips64::*;
use crate::ppc32::*;
#[cfg(feature = "USE_UNSTABLE")]
use crate::tcb::*;
use crate::utils::*;
use crate::vm::*;

pub type cpu_gen_t = cpu_gen;
pub type cpu_group_t = cpu_group;
pub type memlog_access_t = memlog_access;

/// Possible CPU types // TODO enum
pub const CPU_TYPE_MIPS64: u_int = 1;
pub const CPU_TYPE_PPC32: u_int = 2;

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

/// Generic CPU definition
#[repr(C)]
pub struct cpu_gen {
    /// CPU type and identifier for MP systems
    pub r#type: u_int,
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

    /// JIT op array for current compiled pages
    pub jit_op_array_size: u_int,
    pub jit_op_array: *mut *mut jit_op_t,
    pub jit_op_current: *mut *mut jit_op_t,

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

    /// JIT op pool
    pub jit_op_pool: [*mut jit_op_t; JIT_OP_POOL_NR],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union cpu_gen_sp {
    pub mips64_cpu: cpu_mips_t,
    pub ppc32_cpu: cpu_ppc_t,
}

/// CPU group definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_group {
    pub name: *mut c_char,
    pub cpu_list: *mut cpu_gen_t,
    pub priv_data: *mut c_void,
}

#[no_mangle]
pub unsafe extern "C" fn CPU_MIPS64(cpu: *mut cpu_gen_t) -> *mut cpu_mips_t {
    addr_of_mut!((*cpu).sp.mips64_cpu)
}
#[no_mangle]
pub unsafe extern "C" fn CPU_PPC32(cpu: *mut cpu_gen_t) -> *mut cpu_ppc_t {
    addr_of_mut!((*cpu).sp.ppc32_cpu)
}

/// Get CPU instruction pointer
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn cpu_get_pc(cpu: *mut cpu_gen_t) -> m_uint64_t {
    match (*cpu).r#type {
        CPU_TYPE_MIPS64 => (*CPU_MIPS64(cpu)).pc,
        CPU_TYPE_PPC32 => (*CPU_PPC32(cpu)).ia as m_uint64_t,
        _ => 0,
    }
}

/// Get CPU performance counter
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn cpu_get_perf_counter(cpu: *mut cpu_gen_t) -> m_uint32_t {
    match (*cpu).r#type {
        CPU_TYPE_MIPS64 => (*CPU_MIPS64(cpu)).perf_counter,
        CPU_TYPE_PPC32 => (*CPU_PPC32(cpu)).perf_counter,
        _ => 0,
    }
}

/// Returns to the CPU exec loop
#[inline]
#[no_mangle]
pub unsafe extern "C" fn cpu_exec_loop_enter(cpu: *mut cpu_gen_t) {
    setjmp::longjmp(addr_of_mut!((*cpu).exec_loop_env), 1);
}

/// Set the exec loop entry point
#[inline]
#[no_mangle]
pub unsafe extern "C" fn cpu_exec_loop_set(cpu: *mut cpu_gen_t) -> c_int {
    setjmp::setjmp(addr_of_mut!((*cpu).exec_loop_env))
}

/// Find a CPU in a group given its ID
#[no_mangle]
pub unsafe extern "C" fn cpu_group_find_id(group: *mut cpu_group_t, id: u_int) -> *mut cpu_gen_t {
    let mut cpu: *mut cpu_gen_t;

    if group.is_null() {
        return null_mut();
    }

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        if (*cpu).id == id {
            return cpu;
        }
        cpu = (*cpu).next
    }

    null_mut()
}

/// Find the highest CPU ID in a CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_find_highest_id(group: *mut cpu_group_t, highest_id: *mut u_int) -> c_int {
    let mut cpu: *mut cpu_gen_t;
    let mut max_id: u_int = 0;

    if group.is_null() || !(*group).cpu_list.is_null() {
        // FIXME when cpu_list has data it should advance to the loop
        return -1;
    }

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        if (*cpu).id >= max_id {
            max_id = (*cpu).id;
        }
        cpu = (*cpu).next
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

/// Log a message for a CPU
#[macro_export]
macro_rules! cpu_log {
    ($cpu:expr, $module:expr, $format:expr$(, $arg:expr)*) => {
        let cpu: *mut cpu_gen_t = $cpu;
        let module: *mut c_char = $module;
        let format: *mut c_char = $format;
        let args: &[&dyn sprintf::Printf] = &[$(&CustomPrintf($arg)),*];

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

            let mut buf: *mut c_char = buffer.as_c_mut().add(6);
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

/// Create a new CPU
#[no_mangle]
pub unsafe extern "C" fn cpu_create(vm: *mut vm_instance_t, type_: u_int, id: u_int) -> *mut cpu_gen_t {
    let mut cpu_run_fn: unsafe extern "C" fn(*mut cpu_gen_t) -> *mut c_void;

    let cpu: *mut cpu_gen_t = libc::malloc(size_of::<cpu_gen_t>()).cast::<_>();
    if cpu.is_null() {
        return null_mut();
    }

    libc::memset(cpu.cast::<_>(), 0, size_of::<cpu_gen_t>());
    (*cpu).vm = vm;
    (*cpu).id = id;
    (*cpu).r#type = type_;
    (*cpu).state.set(CPU_STATE_SUSPENDED);
    #[cfg(feature = "USE_UNSTABLE")]
    {
        (*cpu).tsg = (*vm).tsg;
    }

    match (*cpu).r#type {
        CPU_TYPE_MIPS64 => {
            (*cpu).jit_op_array_size = MIPS_INSN_PER_PAGE as u_int;
            (*CPU_MIPS64(cpu)).vm = vm;
            (*CPU_MIPS64(cpu)).gen = cpu;
            mips64_init(CPU_MIPS64(cpu));

            cpu_run_fn = mips64_jit_run_cpu;

            if (*(*cpu).vm).jit_use == 0 {
                cpu_run_fn = mips64_exec_run_cpu;
            } else {
                mips64_jit_init(CPU_MIPS64(cpu));
            }
        }

        CPU_TYPE_PPC32 => {
            (*cpu).jit_op_array_size = PPC32_INSN_PER_PAGE as u_int;
            (*CPU_PPC32(cpu)).vm = vm;
            (*CPU_PPC32(cpu)).gen = cpu;
            ppc32_init(CPU_PPC32(cpu));

            cpu_run_fn = ppc32_jit_run_cpu;

            if (*(*cpu).vm).jit_use == 0 {
                cpu_run_fn = ppc32_exec_run_cpu;
            } else {
                ppc32_jit_init(CPU_PPC32(cpu));
            }
        }

        _ => {
            libc::fprintf(c_stderr(), cstr!("CPU type %u is not supported yet\n"), (*cpu).r#type);
            libc::abort();
        }
    }

    // create the CPU thread execution
    let cpu_run_fn: extern "C" fn(*mut c_void) -> *mut c_void = std::mem::transmute::<unsafe extern "C" fn(*mut cpu_gen_t) -> *mut c_void, extern "C" fn(*mut c_void) -> *mut c_void>(cpu_run_fn);
    if libc::pthread_create(addr_of_mut!((*cpu).cpu_thread), null_mut(), cpu_run_fn, cpu.cast::<_>()) != 0 {
        libc::fprintf(c_stderr(), cstr!("cpu_create: unable to create thread for CPU%u\n"), id);
        libc::free(cpu.cast::<_>());
        return null_mut();
    }

    cpu
}

/// Delete a CPU
#[no_mangle]
pub unsafe extern "C" fn cpu_delete(cpu: *mut cpu_gen_t) {
    if !cpu.is_null() {
        // Stop activity of this CPU
        cpu_stop(cpu);
        libc::pthread_join((*cpu).cpu_thread, null_mut());

        // Free resources
        match (*cpu).r#type {
            CPU_TYPE_MIPS64 => mips64_delete(CPU_MIPS64(cpu)),
            CPU_TYPE_PPC32 => ppc32_delete(CPU_PPC32(cpu)),
            _ => {}
        }

        libc::free((*cpu).jit_op_array.cast::<_>());
        libc::free(cpu.cast::<_>());
    }
}

/// Start a CPU
#[no_mangle]
pub unsafe extern "C" fn cpu_start(cpu: *mut cpu_gen_t) {
    if !cpu.is_null() {
        cpu_log!(cpu, cstr!("CPU_STATE"), cstr!("Starting CPU (old state=%u)...\n"), (*cpu).state.get());
        (*cpu).state.set(CPU_STATE_RUNNING);
    }
}

/// Stop a CPU
#[no_mangle]
pub unsafe extern "C" fn cpu_stop(cpu: *mut cpu_gen_t) {
    if !cpu.is_null() {
        cpu_log!(cpu, cstr!("CPU_STATE"), cstr!("Halting CPU (old state=%u)...\n"), (*cpu).state.get());
        (*cpu).state.set(CPU_STATE_HALTED);
    }
}

/// Start all CPUs of a CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_start_all_cpu(group: *mut cpu_group_t) {
    let mut cpu: *mut cpu_gen_t;

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        cpu_start(cpu);
        cpu = (*cpu).next;
    }
}

/// Stop all CPUs of a CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_stop_all_cpu(group: *mut cpu_group_t) {
    let mut cpu: *mut cpu_gen_t;

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        cpu_stop(cpu);
        cpu = (*cpu).next;
    }
}

/// Set a state of all CPUs of a CPU group
#[no_mangle]
pub unsafe extern "C" fn cpu_group_set_state(group: *mut cpu_group_t, state: u_int) {
    let mut cpu: *mut cpu_gen_t;

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        (*cpu).state.set(state);
        cpu = (*cpu).next;
    }
}

/// Returns TRUE if all CPUs in a CPU group are inactive
unsafe fn cpu_group_check_activity(group: *mut cpu_group_t) -> c_int {
    let mut cpu: *mut cpu_gen_t;

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        if (*cpu).cpu_thread_running.get() == 0 {
            cpu = (*cpu).next;
            continue;
        }

        if ((*cpu).state.get() == CPU_STATE_RUNNING) || (*cpu).seq_state.get() == 0 {
            return FALSE;
        }
        cpu = (*cpu).next;
    }

    TRUE
}

/// Synchronize on CPUs (all CPUs must be inactive)
#[no_mangle]
pub unsafe extern "C" fn cpu_group_sync_state(group: *mut cpu_group_t) -> c_int {
    let mut cpu: *mut cpu_gen_t;
    let mut t2: m_tmcnt_t;

    // Check that CPU activity is really suspended
    let t1: m_tmcnt_t = m_gettime();

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        (*cpu).seq_state.set(0);
        cpu = (*cpu).next;
    }

    while cpu_group_check_activity(group) == 0 {
        t2 = m_gettime();

        if t2 > (t1 + 10000) {
            return -1;
        }

        libc::usleep(50000);
    }

    0
}

/// Save state of all CPUs
#[no_mangle]
pub unsafe extern "C" fn cpu_group_save_state(group: *mut cpu_group_t) -> c_int {
    let mut cpu: *mut cpu_gen_t;

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        (*cpu).prev_state = (*cpu).state;
        cpu = (*cpu).next;
    }

    TRUE
}

/// Restore state of all CPUs
#[no_mangle]
pub unsafe extern "C" fn cpu_group_restore_state(group: *mut cpu_group_t) -> c_int {
    let mut cpu: *mut cpu_gen_t;

    cpu = (*group).cpu_list;
    while !cpu.is_null() {
        (*cpu).state = (*cpu).prev_state;
        cpu = (*cpu).next;
    }

    TRUE
}

/// Virtual idle loop
#[no_mangle]
pub unsafe extern "C" fn cpu_idle_loop(cpu: *mut cpu_gen_t) {
    let mut t_spc: libc::timespec = zeroed::<_>();

    let expire: m_tmcnt_t = m_gettime_usec() + (*cpu).idle_sleep_time as m_tmcnt_t;

    libc::pthread_mutex_lock(addr_of_mut!((*cpu).idle_mutex));
    t_spc.tv_sec = (expire / 1000000) as libc::time_t;
    t_spc.tv_nsec = ((expire % 1000000) * 1000) as _;
    while libc::pthread_cond_timedwait(addr_of_mut!((*cpu).idle_cond), addr_of_mut!((*cpu).idle_mutex), addr_of_mut!(t_spc)) != libc::ETIMEDOUT {}
    libc::pthread_mutex_unlock(addr_of_mut!((*cpu).idle_mutex));
}

/// Break idle wait state
#[no_mangle]
pub unsafe extern "C" fn cpu_idle_break_wait(cpu: *mut cpu_gen_t) {
    libc::pthread_cond_signal(addr_of_mut!((*cpu).idle_cond));
    (*cpu).idle_count = 0;
}
