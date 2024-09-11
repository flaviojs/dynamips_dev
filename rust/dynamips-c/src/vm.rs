//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Virtual machine abstraction.

use crate::_private::*;
use crate::cisco_card::*;
use crate::cpu::*;
use crate::dev_ram::*;
use crate::dev_vtty::*;
use crate::device::*;
use crate::dynamips_common::*;
#[cfg(feature = "USE_MIPS64_AMD64_TRANS")]
use crate::mips64_amd64_trans::*;
#[cfg(feature = "USE_MIPS64_NOJIT_TRANS")]
use crate::mips64_nojit_trans::*;
#[cfg(feature = "USE_MIPS64_PPC32_TRANS")]
use crate::mips64_ppc32_trans::*;
#[cfg(feature = "USE_MIPS64_X86_TRANS")]
use crate::mips64_x86_trans::*;
use crate::pci_dev::*;
use crate::pci_io::*;
use crate::registry::*;
use crate::rommon_var::*;
use crate::utils::*;

pub type vm_chunk_t = vm_chunk;
pub type vm_ghost_image_t = vm_ghost_image;
pub type vm_instance_t = vm_instance;
pub type vm_obj_t = vm_obj;
pub type vm_platform_t = vm_platform;

pub const VM_PAGE_SHIFT: c_int = 12;
pub const VM_PAGE_SIZE: usize = 1 << VM_PAGE_SHIFT;
pub const VM_PAGE_IMASK: m_uint64_t = VM_PAGE_SIZE as m_uint64_t - 1;
pub const VM_PAGE_MASK: m_uint64_t = !VM_PAGE_IMASK;

/// Number of pages in chunk area
pub const VM_CHUNK_AREA_SIZE: usize = 256;

/// VM memory chunk
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vm_chunk {
    pub area: *mut c_void,
    pub page_alloc: u_int,
    pub page_total: u_int,
    pub next: *mut vm_chunk_t,
}

/// VM ghost pool entry
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vm_ghost_image {
    pub filename: *mut c_char,
    pub ref_count: u_int,
    pub fd: c_int,
    pub file_size: libc::off_t,
    pub area_ptr: *mut u_char,
    pub next: *mut vm_ghost_image_t,
}

/// Maximum number of devices per VM
pub const VM_DEVICE_MAX: usize = 1 << 6;

/// Size of the PCI bus pool
pub const VM_PCI_POOL_SIZE: usize = 32;

/// VM instance status // TODO enum
pub const VM_STATUS_HALTED: c_int = 0; // VM is halted and no HW resources are used
pub const VM_STATUS_SHUTDOWN: c_int = 1; // Shutdown procedure engaged
pub const VM_STATUS_RUNNING: c_int = 2; // VM is running
pub const VM_STATUS_SUSPENDED: c_int = 3; // VM is suspended

/// Ghost RAM status // TODO enum
pub const VM_GHOST_RAM_NONE: c_int = 0;
pub const VM_GHOST_RAM_GENERATE: c_int = 1;
pub const VM_GHOST_RAM_USE: c_int = 2;

/// Timer IRQ check interval
pub const VM_TIMER_IRQ_CHECK_ITV: c_int = 1000;

/// Max slots per VM
pub const VM_MAX_SLOTS: usize = 16;

/// Shutdown function prototype for an object
pub type vm_shutdown_t = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, data: *mut c_void) -> *mut c_void>;

/// VM object, used to keep track of devices and various things
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vm_obj {
    pub name: *mut c_char,
    pub data: *mut c_void,
    pub next: *mut vm_obj,
    pub pprev: *mut *mut vm_obj,
    pub shutdown: vm_shutdown_t,
}

/// VM instance
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vm_instance {
    pub name: *mut c_char,
    pub platform: *mut vm_platform_t, // Platform specific helpers
    pub status: c_int,                // Instance status
    pub instance_id: c_int,           // Instance Identifier
    pub lock_file: *mut c_char,       // Lock file
    pub log_file: *mut c_char,        // Log filename
    pub log_file_enabled: c_int,      // Logging enabled
    pub ram_size: u_int,              // RAM and ROM size in Mb
    pub rom_size: u_int,
    pub ram_res_size: u_int,          // RAM reserved space size
    pub iomem_size: u_int,            // IOMEM size in Mb
    pub nvram_size: u_int,            // NVRAM size in Kb
    pub pcmcia_disk_size: [u_int; 2], // PCMCIA disk0 and disk1 sizes (in Mb)
    pub conf_reg: u_int,              // Config register
    pub conf_reg_setup: u_int,
    pub clock_divisor: u_int,            // Clock Divisor (see cp0.c)
    pub ram_mmap: u_int,                 // Memory-mapped RAM ?
    pub restart_ios: u_int,              // Restart IOS on reload ?
    pub elf_machine_id: u_int,           // ELF machine identifier
    pub exec_area_size: u_int,           // Size of execution area for CPU
    pub ios_entry_point: m_uint32_t,     // IOS entry point
    pub ios_image: *mut c_char,          // IOS image filename
    pub ios_startup_config: *mut c_char, // IOS configuration file for startup-config
    pub ios_private_config: *mut c_char, // IOS configuration file for private-config
    pub rom_filename: *mut c_char,       // ROM filename
    pub sym_filename: *mut c_char,       // Symbol filename
    pub lock_fd: *mut libc::FILE,        // Lock/Log file descriptors
    pub log_fd: *mut libc::FILE,
    pub debug_level: c_int,   // Debugging Level
    pub jit_use: c_int,       // CPUs use JIT
    pub sparse_mem: c_int,    // Use sparse virtual memory
    pub nm_iomem_size: u_int, // IO mem size to be passed to Smart Init

    /// ROMMON variables
    pub rommon_vars: rommon_var_list,

    /// Memory chunks
    pub chunks: *mut vm_chunk_t,

    /// Basic hardware: system CPU, PCI busses and PCI I/O space
    pub cpu_group: *mut cpu_group_t,
    pub boot_cpu: *mut cpu_gen_t,
    pub pci_bus: [*mut pci_bus; 2],
    pub pci_bus_pool: [*mut pci_bus; VM_PCI_POOL_SIZE],
    pub pci_io_space: *mut pci_io_data,

    /// Memory mapped devices
    pub dev_list: *mut vdevice,
    pub dev_array: [*mut vdevice; VM_DEVICE_MAX],

    /// IRQ routing
    pub set_irq: Option<unsafe extern "C" fn(vm: *mut vm_instance_t, irq: u_int)>,
    pub clear_irq: Option<unsafe extern "C" fn(vm: *mut vm_instance_t, irq: u_int)>,

    /// Slots for PA/NM/...
    pub nr_slots: u_int,
    pub slots_type: u_int,
    pub slots: [*mut cisco_card; VM_MAX_SLOTS],
    pub slots_drivers: *mut *mut cisco_card_driver,
    pub slots_pci_bus: [*mut pci_bus; VM_MAX_SLOTS],

    /// Filename for ghosted RAM
    pub ghost_ram_filename: *mut c_char,

    /// Ghost RAM image handling
    pub ghost_status: c_int,

    /// Timer IRQ interval check
    pub timer_irq_check_itv: u_int,

    /// Translation sharing group
    #[cfg(feature = "USE_UNSTABLE")]
    pub tsg: c_int,

    /// "idling" pointer counter
    pub idle_pc: m_uint64_t,

    /// JIT block direct jumps
    pub exec_blk_direct_jump: c_int,

    /// IRQ idling preemption
    pub irq_idle_preempt: [u_int; 256],

    /// Console and AUX port VTTY type and parameters
    pub vtty_con_type: c_int,
    pub vtty_aux_type: c_int,
    pub vtty_con_tcp_port: c_int,
    pub vtty_aux_tcp_port: c_int,
    pub vtty_con_serial_option: vtty_serial_option_t,
    pub vtty_aux_serial_option: vtty_serial_option_t,

    /// Virtual TTY for Console and AUX ports
    pub vtty_con: *mut vtty_t,
    pub vtty_aux: *mut vtty_t,

    /// Space reserved in NVRAM by ROM monitor
    pub nvram_rom_space: u_int,

    /// Chassis cookie (for c2600 and maybe other routers)
    pub chassis_cookie: [m_uint16_t; 64],

    /// Specific hardware data
    pub hw_data: *mut c_void,

    /// VM objects
    pub vm_object_list: *mut vm_obj,
}

/// VM Platform definition
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vm_platform {
    pub name: *mut c_char,
    pub log_name: *mut c_char,
    pub cli_name: *mut c_char,
    pub create_instance: Option<unsafe extern "C" fn(vm: *mut vm_instance_t) -> c_int>,
    pub delete_instance: Option<unsafe extern "C" fn(vm: *mut vm_instance_t) -> c_int>,
    pub init_instance: Option<unsafe extern "C" fn(vm: *mut vm_instance_t) -> c_int>,
    pub stop_instance: Option<unsafe extern "C" fn(vm: *mut vm_instance_t) -> c_int>,
    pub oir_start: Option<unsafe extern "C" fn(vm: *mut vm_instance_t, slot_id: u_int, subslot_id: u_int) -> c_int>,
    pub oir_stop: Option<unsafe extern "C" fn(vm: *mut vm_instance_t, slot_id: u_int, subslot_id: u_int) -> c_int>,
    pub nvram_extract_config: Option<unsafe extern "C" fn(vm: *mut vm_instance_t, startup_config: *mut *mut u_char, startup_len: *mut size_t, private_config: *mut *mut u_char, private_len: *mut size_t) -> c_int>,
    pub nvram_push_config: Option<unsafe extern "C" fn(vm: *mut vm_instance_t, startup_config: *mut u_char, startup_len: size_t, private_config: *mut u_char, private_len: size_t) -> c_int>,
    pub get_mac_addr_msb: Option<unsafe extern "C" fn() -> u_int>,
    pub save_config: Option<unsafe extern "C" fn(vm: *mut vm_instance_t, fd: *mut libc::FILE)>,
    pub cli_parse_options: Option<unsafe extern "C" fn(vm: *mut vm_instance_t, option: c_int) -> c_int>,
    pub cli_show_options: Option<unsafe extern "C" fn(vm: *mut vm_instance_t)>,
    pub show_spec_drivers: Option<unsafe extern "C" fn()>,
}

/// VM platform list item
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vm_platform_list {
    pub next: *mut vm_platform_list,
    pub platform: *mut vm_platform,
}

/// Set an IRQ for a VM
#[inline]
#[no_mangle]
pub unsafe extern "C" fn vm_set_irq(vm: *mut vm_instance_t, irq: u_int) {
    if (*vm).set_irq.is_some() {
        (*vm).set_irq.unwrap()(vm, irq);
    }
}

/// Clear an IRQ for a VM
#[inline]
#[no_mangle]
pub unsafe extern "C" fn vm_clear_irq(vm: *mut vm_instance_t, irq: u_int) {
    if (*vm).clear_irq.is_some() {
        (*vm).clear_irq.unwrap()(vm, irq);
    }
}

const DEBUG_VM: c_int = 1;

unsafe fn VM_GLOCK() {
    libc::pthread_mutex_lock(addr_of_mut!(vm_global_lock));
}
unsafe fn VM_GUNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!(vm_global_lock));
}

/// Type of VM file naming (0=use VM name, 1=use instance ID)
#[no_mangle]
pub static mut vm_file_naming_type: c_int = 0;

/// Platform list
static mut vm_platforms: *mut vm_platform_list = null_mut();

/// Pool of ghost images
static mut vm_ghost_pool: *mut vm_ghost_image_t = null_mut();

/// Global lock for VM manipulation
static mut vm_global_lock: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;

/// Initialize a VM object
#[no_mangle]
pub unsafe extern "C" fn vm_object_init(obj: *mut vm_obj_t) {
    libc::memset(obj.cast::<_>(), 0, size_of::<vm_obj_t>());
}

/// Add a VM object to an instance
#[no_mangle]
pub unsafe extern "C" fn vm_object_add(vm: *mut vm_instance_t, obj: *mut vm_obj_t) {
    (*obj).next = (*vm).vm_object_list;
    (*obj).pprev = addr_of_mut!((*vm).vm_object_list);

    if !(*vm).vm_object_list.is_null() {
        (*(*vm).vm_object_list).pprev = addr_of_mut!((*obj).next);
    }

    (*vm).vm_object_list = obj;
}

/// Remove a VM object from an instance
#[no_mangle]
pub unsafe extern "C" fn vm_object_remove(vm: *mut vm_instance_t, obj: *mut vm_obj_t) {
    if !(*obj).next.is_null() {
        (*(*obj).next).pprev = (*obj).pprev;
    }
    *((*obj).pprev) = (*obj).next;

    (*obj).shutdown.unwrap()(vm, (*obj).data);
}

/// Find an object given its name
#[no_mangle]
pub unsafe extern "C" fn vm_object_find(vm: *mut vm_instance_t, name: *mut c_char) -> *mut vm_obj_t {
    let mut obj: *mut vm_obj_t = (*vm).vm_object_list;
    while !obj.is_null() {
        if libc::strcmp((*obj).name, name) == 0 {
            return obj;
        }
        obj = (*obj).next;
    }

    null_mut()
}

/// Check that a mandatory object is present
#[no_mangle]
pub unsafe extern "C" fn vm_object_check(vm: *mut vm_instance_t, name: *mut c_char) -> c_int {
    if !vm_object_find(vm, name).is_null() {
        0
    } else {
        -1
    }
}

/// Shut down all objects of an instance
#[no_mangle]
pub unsafe extern "C" fn vm_object_free_list(vm: *mut vm_instance_t) {
    let mut obj: *mut vm_obj_t;
    let mut next: *mut vm_obj_t;

    obj = (*vm).vm_object_list;
    while !obj.is_null() {
        next = (*obj).next;

        if (*obj).shutdown.is_some() {
            if DEBUG_VM != 0 {
                vm_log!(vm, cstr!("VM_OBJECT"), cstr!("Shutdown of object \"%s\"\n"), (*obj).name);
            }
            (*obj).shutdown.unwrap()(vm, (*obj).data);
        }
        obj = next;
    }

    (*vm).vm_object_list = null_mut();
}

/// Rebuild the object list pointers
unsafe fn vm_object_rebuild_list(vm: *mut vm_instance_t) {
    let mut obj: *mut *mut vm_obj_t = addr_of_mut!((*vm).vm_object_list);
    while !obj.is_null() {
        (*(*obj)).pprev = obj;
        obj = addr_of_mut!((*(*obj)).next);
    }
}

/// Dump the object list of an instance
#[no_mangle]
pub unsafe extern "C" fn vm_object_dump(vm: *mut vm_instance_t) {
    let mut obj: *mut vm_obj_t;

    libc::printf(cstr!("VM \"%s\" (%u) object list:\n"), (*vm).name, (*vm).instance_id);

    obj = (*vm).vm_object_list;
    while !obj.is_null() {
        libc::printf(cstr!("  - %-15s [data=%p]\n"), (*obj).name, (*obj).data);
        obj = (*obj).next;
    }

    libc::printf(cstr!("\n"));
}

/// Get VM type
#[no_mangle]
pub unsafe extern "C" fn vm_get_type(vm: *mut vm_instance_t) -> *mut c_char {
    (*(*vm).platform).name
}

/// Get log name
#[no_mangle] // TODO private
pub unsafe extern "C" fn vm_get_log_name(vm: *mut vm_instance_t) -> *mut c_char {
    if !(*(*vm).platform).log_name.is_null() {
        return (*(*vm).platform).log_name;
    }

    // default value
    cstr!("VM")
}

/// Get MAC address MSB
#[no_mangle]
pub unsafe extern "C" fn vm_get_mac_addr_msb(vm: *mut vm_instance_t) -> u_int {
    if (*(*vm).platform).get_mac_addr_msb.is_some() {
        return (*(*vm).platform).get_mac_addr_msb.unwrap()();
    }

    // default value
    0xC6
}

/// Generate a filename for use by the instance
#[no_mangle]
pub unsafe extern "C" fn vm_build_filename(vm: *mut vm_instance_t, name: *mut c_char) -> *mut c_char {
    let machine: *mut c_char = vm_get_type(vm);

    #[allow(clippy::wildcard_in_or_patterns)]
    let filename: *mut c_char = match vm_file_naming_type {
        1 => dyn_sprintf!(cstr!("%s_i%u_%s"), machine, (*vm).instance_id, name),
        0 | _ => dyn_sprintf!(cstr!("%s_%s_%s"), machine, (*vm).name, name),
    };

    assert!(!filename.is_null());
    filename
}

/// Get the amount of host virtual memory used by a VM
#[no_mangle]
pub unsafe extern "C" fn vm_get_vspace_size(vm: *mut vm_instance_t) -> size_t {
    let mut dev: *mut vdevice;
    let mut hsize: size_t = 0;

    // Add memory used by CPU (exec area)
    // XXX TODO

    // Add memory used by devices
    dev = (*vm).dev_list;
    while !dev.is_null() {
        hsize += dev_get_vspace_size(dev);
        dev = (*dev).next;
    }

    hsize
}

/// Erase lock file
#[no_mangle]
pub unsafe extern "C" fn vm_release_lock(vm: *mut vm_instance_t, erase: c_int) {
    if !(*vm).lock_fd.is_null() {
        libc::fclose((*vm).lock_fd);
        (*vm).lock_fd = null_mut();
    }

    if !(*vm).lock_file.is_null() {
        if erase != 0 {
            libc::unlink((*vm).lock_file);
        }
        libc::free((*vm).lock_file.cast::<_>());
        (*vm).lock_file = null_mut();
    }
}

/// Check that an instance lock file doesn't already exist
#[no_mangle]
pub unsafe extern "C" fn vm_get_lock(vm: *mut vm_instance_t) -> c_int {
    let mut pid_str: [c_char; 32] = [0; 32];
    let mut lock: libc::flock = zeroed::<_>();

    (*vm).lock_file = vm_build_filename(vm, cstr!("lock"));

    (*vm).lock_fd = libc::fopen((*vm).lock_file, cstr!("w"));
    if (*vm).lock_fd.is_null() {
        libc::fprintf(c_stderr(), cstr!("Unable to create lock file \"%s\".\n"), (*vm).lock_file);
        return -1;
    }

    libc::memset(addr_of_mut!(lock).cast::<_>(), 0, size_of::<libc::flock>());
    lock.l_type = libc::F_WRLCK as _;
    lock.l_whence = libc::SEEK_SET as _;
    lock.l_start = 0;
    lock.l_len = 0;

    if libc::fcntl(libc::fileno((*vm).lock_fd), libc::F_SETLK, addr_of_mut!(lock)) == -1 {
        if libc::fcntl(libc::fileno((*vm).lock_fd), libc::F_GETLK, addr_of_mut!(lock)) == 0 {
            libc::snprintf(pid_str.as_c_mut(), pid_str.len(), cstr!("%ld"), lock.l_pid as c_long);
        } else {
            libc::strcpy(pid_str.as_c_mut(), cstr!("unknown"));
        }

        libc::fprintf(c_stderr(), cstr!("\nAn emulator instance (PID %s) is already running with identifier %u.\n", "If this is not the case, please erase file \"%s\".\n\n"), pid_str, (*vm).instance_id, (*vm).lock_file);
        vm_release_lock(vm, FALSE);
        return -1;
    }

    // write the emulator PID
    libc::fprintf((*vm).lock_fd, cstr!("%ld\n"), libc::getpid() as u_long);
    0
}

/// Log a message
pub unsafe fn vm_flog(vm: *mut vm_instance_t, module: *mut c_char, format: *mut c_char, args: &[&dyn sprintf::Printf]) {
    if !(*vm).log_fd.is_null() {
        m_flog((*vm).log_fd, module, format, args);
    }
}

/// Log a message
#[macro_export]
macro_rules! vm_log {
    ($vm:expr, $module:expr, $format: expr$(, $arg:expr)*) => {
        let vm: *mut vm_instance_t = $vm;
        let module: *mut c_char = $module;
        let format: *mut c_char = $format;
        let args: &[&dyn sprintf::Printf] = &[$(&CustomPrintf($arg)),*];

        if !(*vm).log_fd.is_null() {
            m_flog((*vm).log_fd, module, format, args);
        }
    };
}
pub use vm_log;

/// Close the log file
#[no_mangle]
pub unsafe extern "C" fn vm_close_log(vm: *mut vm_instance_t) -> c_int {
    if !(*vm).log_fd.is_null() {
        libc::fclose((*vm).log_fd);
    }

    libc::free((*vm).log_file.cast::<_>());

    (*vm).log_file = null_mut();
    (*vm).log_fd = null_mut();
    0
}

/// Create the log file
#[no_mangle]
pub unsafe extern "C" fn vm_create_log(vm: *mut vm_instance_t) -> c_int {
    if (*vm).log_file_enabled != 0 {
        vm_close_log(vm);

        (*vm).log_file = vm_build_filename(vm, cstr!("log.txt"));
        if (*vm).log_file.is_null() {
            return -1;
        }

        (*vm).log_fd = libc::fopen((*vm).log_file, cstr!("w"));
        if (*vm).log_fd.is_null() {
            libc::fprintf(c_stderr(), cstr!("VM %s: unable to create log file '%s'\n"), (*vm).name, (*vm).log_file);
            libc::free((*vm).log_file.cast::<_>());
            (*vm).log_file = null_mut();
            return -1;
        }
    }

    0
}

/// Reopen the log file
#[no_mangle]
pub unsafe extern "C" fn vm_reopen_log(vm: *mut vm_instance_t) -> c_int {
    if (*vm).log_file_enabled != 0 {
        vm_close_log(vm);

        (*vm).log_file = vm_build_filename(vm, cstr!("log.txt"));
        if (*vm).log_file.is_null() {
            return -1;
        }

        (*vm).log_fd = libc::fopen((*vm).log_file, cstr!("a"));
        if (*vm).log_fd.is_null() {
            libc::fprintf(c_stderr(), cstr!("VM %s: unable to reopen log file '%s'\n"), (*vm).name, (*vm).log_file);
            libc::free((*vm).log_file.cast::<_>());
            (*vm).log_file = null_mut();
            return -1;
        }
    }

    0
}

/// Error message
#[macro_export]
macro_rules! vm_error {
    ($vm:expr, $format:expr$(, $arg:expr)*) => {
        let vm: *mut vm_instance_t = $vm;
        let format: *mut c_char = $format;
        let args: &[&dyn sprintf::Printf] = &[$(&CustomPrintf($arg)),*];

        if let Ok(s) = sprintf::vsprintf(CStr::from_ptr(format).to_str().unwrap(), args) {
            let mut bytes = s.into_bytes();
            bytes.push(0);
            libc::fprintf(c_stderr(), cstr!("%s '%s': %s"), $crate::vm::vm_get_log_name(vm), (*vm).name, bytes.as_ptr());
        }
    };
}
pub use vm_error;

/// Create a new VM instance
unsafe fn vm_create(name: *mut c_char, instance_id: c_int, platform: *mut vm_platform_t) -> *mut vm_instance_t {
    let vm: *mut vm_instance_t = libc::malloc(size_of::<vm_instance_t>()).cast::<_>();
    if vm.is_null() {
        libc::fprintf(c_stderr(), cstr!("VM %s: unable to create new instance!\n"), name);
        return null_mut();
    }

    libc::memset(vm.cast::<_>(), 0, size_of::<vm_instance_t>());

    (*vm).name = libc::strdup(name);
    if (*vm).name.is_null() {
        libc::fprintf(c_stderr(), cstr!("VM %s: unable to store instance name!\n"), name);
        libc::free(vm.cast::<_>());
        return null_mut();
    }

    (*vm).instance_id = instance_id;
    (*vm).platform = platform;
    (*vm).status = VM_STATUS_HALTED;
    (*vm).jit_use = JIT_SUPPORT;
    (*vm).exec_blk_direct_jump = TRUE;
    (*vm).vtty_con_type = VTTY_TYPE_TERM;
    (*vm).vtty_aux_type = VTTY_TYPE_NONE;
    (*vm).timer_irq_check_itv = VM_TIMER_IRQ_CHECK_ITV as u_int;
    (*vm).log_file_enabled = TRUE;
    (*vm).rommon_vars.filename = vm_build_filename(vm, cstr!("rommon_vars"));

    if (*vm).rommon_vars.filename.is_null() {
        libc::free((*vm).name.cast::<_>());
        libc::free(vm.cast::<_>());
        return null_mut();
    }

    // XXX
    rommon_load_file(addr_of_mut!((*vm).rommon_vars));

    // create lock file
    if vm_get_lock(vm) == -1 {
        libc::free((*vm).rommon_vars.filename.cast::<_>());
        libc::free((*vm).name.cast::<_>());
        libc::free(vm.cast::<_>());
        return null_mut();
    }

    // create log file
    if vm_create_log(vm) == -1 {
        libc::free((*vm).lock_file.cast::<_>());
        libc::free((*vm).rommon_vars.filename.cast::<_>());
        libc::free((*vm).name.cast::<_>());
        libc::free(vm.cast::<_>());
        return null_mut();
    }

    if registry_add((*vm).name, OBJ_TYPE_VM, vm.cast::<_>()) == -1 {
        libc::fprintf(c_stderr(), cstr!("VM: Unable to store instance '%s' in registry!\n"), (*vm).name);
        vm_close_log(vm);
        libc::free((*vm).lock_file.cast::<_>());
        libc::free((*vm).rommon_vars.filename.cast::<_>());
        libc::free((*vm).name.cast::<_>());
        libc::free(vm.cast::<_>());
        return null_mut();
    }

    m_log!(cstr!("VM"), cstr!("VM %s created.\n"), (*vm).name);
    vm
}

/// Shutdown hardware resources used by a VM.
/// The CPU must have been stopped.
#[no_mangle]
pub unsafe extern "C" fn vm_hardware_shutdown(vm: *mut vm_instance_t) -> c_int {
    if ((*vm).status == VM_STATUS_HALTED) || !(*vm).cpu_group.is_null() {
        vm_log!(vm, cstr!("VM"), cstr!("trying to shutdown an inactive VM.\n"));
        return -1;
    }

    vm_log!(vm, cstr!("VM"), cstr!("shutdown procedure engaged.\n"));

    // Mark the VM as halted
    (*vm).status = VM_STATUS_HALTED;

    // Free the object list
    vm_object_free_list(vm);

    // Free resources used by PCI busses
    vm_log!(vm, cstr!("VM"), cstr!("removing PCI busses.\n"));
    pci_io_data_remove(vm, (*vm).pci_io_space);
    pci_bus_remove((*vm).pci_bus[0]);
    pci_bus_remove((*vm).pci_bus[1]);
    (*vm).pci_bus[0] = null_mut();
    (*vm).pci_bus[1] = null_mut();

    // Free the PCI bus pool
    for i in 0..VM_PCI_POOL_SIZE {
        if !(*vm).pci_bus_pool[i].is_null() {
            pci_bus_remove((*vm).pci_bus_pool[i]);
            (*vm).pci_bus_pool[i] = null_mut();
        }
    }

    // Remove the IRQ routing vectors
    (*vm).set_irq = None;
    (*vm).clear_irq = None;

    // Delete the VTTY for Console and AUX ports
    vm_log!(vm, cstr!("VM"), cstr!("deleting VTTY.\n"));
    vm_delete_vtty(vm);

    // Delete system CPU group
    vm_log!(vm, cstr!("VM"), cstr!("deleting system CPUs.\n"));
    cpu_group_delete((*vm).cpu_group);
    (*vm).cpu_group = null_mut();
    (*vm).boot_cpu = null_mut();

    vm_log!(vm, cstr!("VM"), cstr!("shutdown procedure completed.\n"));
    m_log!(cstr!("VM"), cstr!("VM %s shutdown.\n"), (*vm).name);
    0
}

/// Free resources used by a VM
#[no_mangle]
pub unsafe extern "C" fn vm_free(vm: *mut vm_instance_t) {
    if !vm.is_null() {
        // Free hardware resources
        vm_hardware_shutdown(vm);

        m_log!(cstr!("VM"), cstr!("VM %s destroyed.\n"), (*vm).name);

        // Close log file
        vm_close_log(vm);

        // Remove the lock file
        vm_release_lock(vm, TRUE);

        // Free all chunks
        vm_chunk_free_all(vm);

        // Free various elements
        #[cfg(not(feature = "USE_UNSTABLE"))]
        {
            rommon_var_clear(addr_of_mut!((*vm).rommon_vars));
        }
        libc::free((*vm).rommon_vars.filename.cast::<_>());
        libc::free((*vm).ghost_ram_filename.cast::<_>());
        libc::free((*vm).sym_filename.cast::<_>());
        libc::free((*vm).ios_image.cast::<_>());
        libc::free((*vm).ios_startup_config.cast::<_>());
        libc::free((*vm).ios_private_config.cast::<_>());
        libc::free((*vm).rom_filename.cast::<_>());
        libc::free((*vm).name.cast::<_>());
        libc::free(vm.cast::<_>());
    }
}

/// Get an instance given a name
#[no_mangle]
pub unsafe extern "C" fn vm_acquire(name: *mut c_char) -> *mut vm_instance_t {
    registry_find(name, OBJ_TYPE_VM).cast::<_>()
}

/// Release a VM (decrement reference count)
#[no_mangle]
pub unsafe extern "C" fn vm_release(vm: *mut vm_instance_t) -> c_int {
    registry_unref((*vm).name, OBJ_TYPE_VM)
}

/// Initialize RAM
#[no_mangle]
pub unsafe extern "C" fn vm_ram_init(vm: *mut vm_instance_t, paddr: m_uint64_t) -> c_int {
    let len: m_uint32_t = (*vm).ram_size * 1048576;

    if (*vm).ghost_status == VM_GHOST_RAM_USE {
        return dev_ram_ghost_init(vm, cstr!("ram"), (*vm).sparse_mem, (*vm).ghost_ram_filename, paddr, len);
    }

    dev_ram_init(vm, cstr!("ram"), (*vm).ram_mmap as c_int, ((*vm).ghost_status != VM_GHOST_RAM_GENERATE) as c_int, (*vm).ghost_ram_filename, (*vm).sparse_mem, paddr, len)
}

/// Initialize VTTY
#[no_mangle]
pub unsafe extern "C" fn vm_init_vtty(vm: *mut vm_instance_t) -> c_int {
    // Create Console and AUX ports
    (*vm).vtty_con = vtty_create(vm, cstr!("Console port"), (*vm).vtty_con_type, (*vm).vtty_con_tcp_port, addr_of_mut!((*vm).vtty_con_serial_option));

    (*vm).vtty_aux = vtty_create(vm, cstr!("AUX port"), (*vm).vtty_aux_type, (*vm).vtty_aux_tcp_port, addr_of_mut!((*vm).vtty_aux_serial_option));
    0
}

/// Delete VTTY
#[no_mangle]
pub unsafe extern "C" fn vm_delete_vtty(vm: *mut vm_instance_t) {
    vtty_delete((*vm).vtty_con);
    vtty_delete((*vm).vtty_aux);
    (*vm).vtty_con = null_mut();
    (*vm).vtty_aux = null_mut();
}

/// Bind a device to a virtual machine
#[no_mangle]
pub unsafe extern "C" fn vm_bind_device(vm: *mut vm_instance_t, dev: *mut vdevice) -> c_int {
    let mut cur: *mut *mut vdevice;
    let mut i: u_int;

    // Add this device to the device array. The index in the device array
    // is used by the MTS subsystem.
    i = 0;
    while i < VM_DEVICE_MAX as u_int {
        if (*vm).dev_array[i as usize].is_null() {
            break;
        }
        i += 1;
    }

    if i == VM_DEVICE_MAX as u_int {
        libc::fprintf(c_stderr(), cstr!("VM%u: vm_bind_device: device table full.\n"), (*vm).instance_id);
        return -1;
    }

    (*vm).dev_array[i as usize] = dev;
    (*dev).id = i;

    // Add it to the linked-list (devices are ordered by physical addresses).
    cur = addr_of_mut!((*vm).dev_list);
    while !(*cur).is_null() {
        if (*(*cur)).phys_addr > (*dev).phys_addr {
            break;
        }
        cur = addr_of_mut!((*(*cur)).next);
    }

    (*dev).next = *cur;
    if !(*cur).is_null() {
        (*(*cur)).pprev = addr_of_mut!((*dev).next);
    }
    (*dev).pprev = cur;
    *cur = dev;
    0
}

/// Unbind a device from a virtual machine
#[no_mangle]
pub unsafe extern "C" fn vm_unbind_device(vm: *mut vm_instance_t, dev: *mut vdevice) -> c_int {
    if dev.is_null() || (*dev).pprev.is_null() {
        return -1;
    }

    // Remove the device from the linked list
    if !(*dev).next.is_null() {
        (*(*dev).next).pprev = (*dev).pprev;
    }

    *((*dev).pprev) = (*dev).next;

    // Remove the device from the device array
    for i in 0..VM_DEVICE_MAX {
        if (*vm).dev_array[i] == dev {
            (*vm).dev_array[i] = null_mut();
            break;
        }
    }

    // Clear device list info
    (*dev).next = null_mut();
    (*dev).pprev = null_mut();
    0
}

/// Map a device at the specified physical address
#[no_mangle]
pub unsafe extern "C" fn vm_map_device(vm: *mut vm_instance_t, dev: *mut vdevice, base_addr: m_uint64_t) -> c_int {
    if false {
        // Suspend VM activity
        vm_suspend(vm);

        if cpu_group_sync_state((*vm).cpu_group) == -1 {
            libc::fprintf(c_stderr(), cstr!("VM%u: unable to sync with system CPUs.\n"), (*vm).instance_id);
            return -1;
        }
    }

    // Unbind the device if it was already active
    vm_unbind_device(vm, dev);

    // Map the device at the new base address and rebuild MTS
    (*dev).phys_addr = base_addr;
    vm_bind_device(vm, dev);
    cpu_group_rebuild_mts((*vm).cpu_group);

    if false {
        vm_resume(vm);
    }
    0
}

/// Suspend a VM instance
#[no_mangle]
pub unsafe extern "C" fn vm_suspend(vm: *mut vm_instance_t) -> c_int {
    if (*vm).status == VM_STATUS_RUNNING {
        cpu_group_save_state((*vm).cpu_group);
        cpu_group_set_state((*vm).cpu_group, CPU_STATE_SUSPENDED);
        (*vm).status = VM_STATUS_SUSPENDED;
    }
    0
}

/// Resume a VM instance
#[no_mangle]
pub unsafe extern "C" fn vm_resume(vm: *mut vm_instance_t) -> c_int {
    if (*vm).status == VM_STATUS_SUSPENDED {
        cpu_group_restore_state((*vm).cpu_group);
        (*vm).status = VM_STATUS_RUNNING;
    }
    0
}

/// Stop an instance
#[no_mangle]
pub unsafe extern "C" fn vm_stop(vm: *mut vm_instance_t) -> c_int {
    cpu_group_stop_all_cpu((*vm).cpu_group);
    (*vm).status = VM_STATUS_SHUTDOWN;
    0
}

/// Monitor an instance periodically
#[no_mangle]
pub unsafe extern "C" fn vm_monitor(vm: *mut vm_instance_t) {
    #[allow(clippy::while_immutable_condition)]
    while (*vm).status != VM_STATUS_SHUTDOWN {
        libc::usleep(200000);
    }
}

/// Create a new chunk
unsafe fn vm_chunk_create(vm: *mut vm_instance_t) -> *mut vm_chunk_t {
    let chunk: *mut vm_chunk_t = libc::malloc(size_of::<vm_chunk_t>()).cast::<_>();
    if chunk.is_null() {
        return null_mut();
    }

    let area_len: size_t = VM_CHUNK_AREA_SIZE * VM_PAGE_SIZE;

    (*chunk).area = m_memalign(VM_PAGE_SIZE, area_len);
    if (*chunk).area.is_null() {
        libc::free(chunk.cast::<_>());
        return null_mut();
    }

    (*chunk).page_alloc = 0;
    (*chunk).page_total = VM_CHUNK_AREA_SIZE as u_int;

    (*chunk).next = (*vm).chunks;
    (*vm).chunks = chunk;
    chunk
}

/// Free a chunk
unsafe fn vm_chunk_free(chunk: *mut vm_chunk_t) {
    libc::free((*chunk).area.cast::<_>());
    libc::free(chunk.cast::<_>());
}

/// Free all chunks used by a VM
unsafe fn vm_chunk_free_all(vm: *mut vm_instance_t) {
    let mut chunk: *mut vm_chunk_t;
    let mut next: *mut vm_chunk_t;

    chunk = (*vm).chunks;
    while !chunk.is_null() {
        next = (*chunk).next;
        vm_chunk_free(chunk);
        chunk = next;
    }

    (*vm).chunks = null_mut();
}

/// Allocate an host page
#[no_mangle]
pub unsafe extern "C" fn vm_alloc_host_page(vm: *mut vm_instance_t) -> *mut c_void {
    let mut chunk: *mut vm_chunk_t = (*vm).chunks;

    if chunk.is_null() || ((*chunk).page_alloc == (*chunk).page_total) {
        chunk = vm_chunk_create(vm);
        if chunk.is_null() {
            return null_mut();
        }
    }

    let ptr: *mut c_void = (*chunk).area.add((*chunk).page_alloc as usize * VM_PAGE_SIZE);
    (*chunk).page_alloc += 1;
    ptr
}

/// Free resources used by a ghost image
unsafe fn vm_ghost_image_free(img: *mut vm_ghost_image_t) {
    if !img.is_null() {
        if (*img).fd != -1 {
            libc::close((*img).fd);

            if !(*img).area_ptr.is_null() {
                memzone_unmap((*img).area_ptr.cast::<_>(), (*img).file_size as size_t);
            }
        }

        libc::free((*img).filename.cast::<_>());
        libc::free(img.cast::<_>());
    }
}

/// Find a specified ghost image in the pool
unsafe fn vm_ghost_image_find(filename: *mut c_char) -> *mut vm_ghost_image_t {
    let mut img: *mut vm_ghost_image_t = vm_ghost_pool;
    while !img.is_null() {
        if libc::strcmp((*img).filename, filename) == 0 {
            return img;
        }
        img = (*img).next;
    }

    null_mut()
}

/// Load a new ghost image
unsafe fn vm_ghost_image_load(filename: *mut c_char) -> *mut vm_ghost_image_t {
    let img: *mut vm_ghost_image_t = libc::calloc(1, size_of::<vm_ghost_image_t>()).cast::<_>();
    if img.is_null() {
        return null_mut();
    }

    (*img).fd = -1;

    (*img).filename = libc::strdup(filename);
    if (*img).filename.is_null() {
        vm_ghost_image_free(img);
        return null_mut();
    }

    (*img).fd = memzone_open_file_ro((*img).filename, addr_of_mut!((*img).area_ptr), addr_of_mut!((*img).file_size));

    if (*img).fd == -1 {
        vm_ghost_image_free(img);
        return null_mut();
    }

    m_log!(cstr!("GHOST"), cstr!("loaded ghost image %s (fd=%d) at addr=%p (size=0x%llx)\n"), (*img).filename, (*img).fd, (*img).area_ptr, (*img).file_size as c_longlong);

    img
}

/// Get a ghost image
#[no_mangle]
pub unsafe extern "C" fn vm_ghost_image_get(filename: *mut c_char, ptr: *mut *mut u_char, fd: *mut c_int) -> c_int {
    let mut img: *mut vm_ghost_image_t;

    VM_GLOCK();

    // Do we already have this image in the pool ?
    img = vm_ghost_image_find(filename);
    if !img.is_null() {
        (*img).ref_count += 1;
        *ptr = (*img).area_ptr;
        *fd = (*img).fd;
        VM_GUNLOCK();
        return 0;
    }

    // Load the ghost file and add it into the pool
    img = vm_ghost_image_load(filename);
    if img.is_null() {
        VM_GUNLOCK();
        libc::fprintf(c_stderr(), cstr!("Unable to load ghost image %s\n"), filename);
        return -1;
    }

    (*img).ref_count = 1;
    *ptr = (*img).area_ptr;
    *fd = (*img).fd;

    (*img).next = vm_ghost_pool;
    vm_ghost_pool = img;
    VM_GUNLOCK();

    m_log!(cstr!("GHOST"), cstr!("loaded image %s successfully.\n"), filename);
    0
}

/// Release a ghost image
#[no_mangle]
pub unsafe extern "C" fn vm_ghost_image_release(fd: c_int) -> c_int {
    let mut img: *mut *mut vm_ghost_image_t;

    VM_GLOCK();

    img = addr_of_mut!(vm_ghost_pool);
    while !img.is_null() {
        if (*(*img)).fd == fd {
            assert!((*(*img)).ref_count > 0);

            (*(*img)).ref_count -= 1;

            if (**img).ref_count == 0 {
                m_log!(cstr!("GHOST"), cstr!("unloaded ghost image %s (fd=%d) at addr=%p (size=0x%llx)\n"), (*(*img)).filename, (*(*img)).fd, (*(*img)).area_ptr, (*(*img)).file_size as c_longlong);

                let next: *mut vm_ghost_image_t = (*(*img)).next;
                vm_ghost_image_free(*img);
                *img = next;
            }

            VM_GUNLOCK();
            return 0;
        }
        img = addr_of_mut!((*(*img)).next);
    }

    VM_GUNLOCK();
    -1
}

/// Open a VM file and map it in memory
#[no_mangle]
pub unsafe extern "C" fn vm_mmap_open_file(vm: *mut vm_instance_t, name: *mut c_char, ptr: *mut *mut u_char, fsize: *mut libc::off_t) -> c_int {
    let filename: *mut c_char = vm_build_filename(vm, name);
    if filename.is_null() {
        libc::fprintf(c_stderr(), cstr!("vm_mmap_open_file: unable to create filename (%s)\n"), name);
        return -1;
    }

    let fd: c_int = memzone_open_file(filename, ptr, fsize);
    if fd == -1 {
        libc::fprintf(c_stderr(), cstr!("vm_mmap_open_file: unable to open file '%s' (%s)\n"), filename, libc::strerror(c_errno()));
    }

    libc::free(filename.cast::<_>());
    fd
}

/// Open/Create a VM file and map it in memory
#[no_mangle]
pub unsafe extern "C" fn vm_mmap_create_file(vm: *mut vm_instance_t, name: *mut c_char, len: size_t, ptr: *mut *mut u_char) -> c_int {
    let filename: *mut c_char = vm_build_filename(vm, name);
    if filename.is_null() {
        libc::fprintf(c_stderr(), cstr!("vm_mmap_create_file: unable to create filename (%s)\n"), name);
        return -1;
    }

    let fd: c_int = memzone_create_file(filename, len, ptr);
    if fd == -1 {
        libc::fprintf(c_stderr(), cstr!("vm_mmap_create_file: unable to open file '%s' (%s)\n"), filename, libc::strerror(c_errno()));
    }

    libc::free(filename.cast::<_>());
    fd
}

/// Close a memory mapped file
#[no_mangle]
pub unsafe extern "C" fn vm_mmap_close_file(fd: c_int, ptr: *mut u_char, len: size_t) -> c_int {
    if !ptr.is_null() {
        memzone_unmap(ptr.cast::<_>(), len);
    }

    if fd != -1 {
        libc::close(fd);
    }

    0
}

/// Save the Cisco IOS configuration from NVRAM
#[no_mangle]
pub unsafe extern "C" fn vm_ios_save_config(vm: *mut vm_instance_t) -> c_int {
    let output: *mut c_char = vm_build_filename(vm, cstr!("ios_cfg.txt"));
    if output.is_null() {
        return -1;
    }

    let res: c_int = vm_nvram_extract_config(vm, output);
    libc::free(output.cast::<_>());
    res
}

/// Set Cisco IOS image to use
#[no_mangle]
pub unsafe extern "C" fn vm_ios_set_image(vm: *mut vm_instance_t, ios_image: *mut c_char) -> c_int {
    let str_: *mut c_char = libc::strdup(ios_image);
    if str_.is_null() {
        return -1;
    }

    if !(*vm).ios_image.is_null() {
        libc::free((*vm).ios_image.cast::<_>());
        (*vm).ios_image = null_mut();
    }

    (*vm).ios_image = str_;
    0
}

/// Unset a Cisco IOS configuration file
#[no_mangle]
pub unsafe extern "C" fn vm_ios_unset_config(vm: *mut vm_instance_t) {
    libc::free((*vm).ios_startup_config.cast::<_>());
    (*vm).ios_startup_config = null_mut();

    libc::free((*vm).ios_private_config.cast::<_>());
    (*vm).ios_private_config = null_mut();
}

/// Set Cisco IOS configuration files to use (NULL to keep existing data)
#[no_mangle]
pub unsafe extern "C" fn vm_ios_set_config(vm: *mut vm_instance_t, startup_filename: *const c_char, private_filename: *const c_char) -> c_int {
    let mut startup_file: *mut c_char = null_mut();
    let mut private_file: *mut c_char = null_mut();

    if !startup_filename.is_null() {
        startup_file = libc::strdup(startup_filename);
        if startup_file.is_null() {
            libc::free(startup_file.cast::<_>());
            libc::free(private_file.cast::<_>());
            return -1;
        }
    }

    if !private_filename.is_null() {
        private_file = libc::strdup(private_filename);
        if private_file.is_null() {
            libc::free(startup_file.cast::<_>());
            libc::free(private_file.cast::<_>());
            return -1;
        }
    }

    vm_ios_unset_config(vm);
    (*vm).ios_startup_config = startup_file;
    (*vm).ios_private_config = private_file;
    0
}

/// Extract IOS configuration from NVRAM and write it to a file
#[no_mangle]
pub unsafe extern "C" fn vm_nvram_extract_config(vm: *mut vm_instance_t, filename: *mut c_char) -> c_int {
    let mut cfg_buffer: *mut u_char = null_mut();
    let mut cfg_len: size_t = 0;

    if (*(*vm).platform).nvram_extract_config.is_none() {
        return -1;
    }

    // Extract the IOS configuration
    if (*(*vm).platform).nvram_extract_config.unwrap()(vm, addr_of_mut!(cfg_buffer), addr_of_mut!(cfg_len), null_mut(), null_mut()) != 0 || cfg_buffer.is_null() {
        return -1;
    }

    // Write configuration to the specified filename
    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("w"));
    if fd.is_null() {
        vm_error!(vm, cstr!("unable to create file '%s'\n"), filename);
        libc::free(cfg_buffer.cast::<_>());
        return -1;
    }

    libc::fwrite(cfg_buffer.cast::<_>(), cfg_len, 1, fd);

    libc::fclose(fd);
    libc::free(cfg_buffer.cast::<_>());
    0
}

/// Read IOS configuraton from the files and push it to NVRAM (NULL to keep existing data)
#[no_mangle]
pub unsafe extern "C" fn vm_nvram_push_config(vm: *mut vm_instance_t, startup_filename: *const c_char, private_filename: *const c_char) -> c_int {
    let mut startup_config: *mut u_char = null_mut();
    let mut private_config: *mut u_char = null_mut();
    let mut startup_len: size_t = 0;
    let mut private_len: size_t = 0;
    let mut res: c_int = -1;

    // Read configuration
    if !startup_filename.is_null() {
        #[allow(clippy::collapsible_if)]
        if m_read_file(startup_filename, addr_of_mut!(startup_config), addr_of_mut!(startup_len)) != 0 {
            libc::free(startup_config.cast::<_>());
            libc::free(private_config.cast::<_>());
            return res;
        }
    }

    if !private_filename.is_null() {
        #[allow(clippy::collapsible_if)]
        if m_read_file(private_filename, addr_of_mut!(private_config), addr_of_mut!(private_len)) != 0 {
            libc::free(startup_config.cast::<_>());
            libc::free(private_config.cast::<_>());
            return res;
        }
    }

    // Push it!
    res = (*(*vm).platform).nvram_push_config.unwrap()(vm, startup_config, startup_len, private_config, private_len);

    libc::free(startup_config.cast::<_>());
    libc::free(private_config.cast::<_>());
    res
}

/// Save general VM configuration into the specified file
#[no_mangle]
pub unsafe extern "C" fn vm_save_config(vm: *mut vm_instance_t, fd: *mut libc::FILE) {
    libc::fprintf(fd, cstr!("vm create %s %u %s\n"), (*vm).name, (*vm).instance_id, (*(*vm).platform).name);

    if !(*vm).ios_image.is_null() {
        libc::fprintf(fd, cstr!("vm set_ios %s %s\n"), (*vm).name, (*vm).ios_image);
    }

    libc::fprintf(fd, cstr!("vm set_ram %s %u\n"), (*vm).name, (*vm).ram_size);
    libc::fprintf(fd, cstr!("vm set_nvram %s %u\n"), (*vm).name, (*vm).nvram_size);
    libc::fprintf(fd, cstr!("vm set_ram_mmap %s %u\n"), (*vm).name, (*vm).ram_mmap);
    libc::fprintf(fd, cstr!("vm set_clock_divisor %s %u\n"), (*vm).name, (*vm).clock_divisor);
    libc::fprintf(fd, cstr!("vm set_conf_reg %s 0x%4.4x\n"), (*vm).name, (*vm).conf_reg_setup);

    if (*vm).vtty_con_type == VTTY_TYPE_TCP {
        libc::fprintf(fd, cstr!("vm set_con_tcp_port %s %d\n"), (*vm).name, (*vm).vtty_con_tcp_port);
    }

    if (*vm).vtty_aux_type == VTTY_TYPE_TCP {
        libc::fprintf(fd, cstr!("vm set_aux_tcp_port %s %d\n"), (*vm).name, (*vm).vtty_aux_tcp_port);
    }

    // Save slot config
    vm_slot_save_all_config(vm, fd);
}

/// Find a platform
#[no_mangle]
pub unsafe extern "C" fn vm_platform_find(name: *mut c_char) -> *mut vm_platform_t {
    let mut p: *mut vm_platform_list = vm_platforms;
    while !p.is_null() {
        if libc::strcmp((*(*p).platform).name, name) == 0 {
            return (*p).platform;
        }
        p = (*p).next;
    }

    null_mut()
}

/// Find a platform given its CLI name
#[no_mangle]
pub unsafe extern "C" fn vm_platform_find_cli_name(name: *mut c_char) -> *mut vm_platform_t {
    let mut p: *mut vm_platform_list = vm_platforms;
    while !p.is_null() {
        if libc::strcmp((*(*p).platform).cli_name, name) == 0 {
            return (*p).platform;
        }
        p = (*p).next;
    }

    null_mut()
}

/// Destroy vm_platforms
extern "C" fn destroy_vm_platforms() {
    unsafe {
        let mut p: *mut vm_platform_list;
        let mut next: *mut vm_platform_list;

        p = vm_platforms;
        while !p.is_null() {
            next = (*p).next;
            libc::free(p.cast::<_>());
            p = next;
        }
        vm_platforms = null_mut();
    }
}

/// Register a platform
#[no_mangle]
pub unsafe extern "C" fn vm_platform_register(platform: *mut vm_platform_t) -> c_int {
    if !vm_platform_find((*platform).name).is_null() {
        libc::fprintf(c_stderr(), cstr!("vm_platform_register: platform '%s' already exists.\n"), (*platform).name);
        return -1;
    }

    let p: *mut vm_platform_list = libc::malloc(size_of::<vm_platform_list>()).cast::<_>();
    if p.is_null() {
        libc::fprintf(c_stderr(), cstr!("vm_platform_register: unable to record platform.\n"));
        return -1;
    }

    if vm_platforms.is_null() {
        libc::atexit(destroy_vm_platforms);
    }

    (*p).platform = platform;
    (*p).next = vm_platforms;
    vm_platforms = p;
    0
}

/// Create an instance of the specified type
#[no_mangle]
pub unsafe extern "C" fn vm_create_instance(name: *mut c_char, instance_id: c_int, type_: *mut c_char) -> *mut vm_instance_t {
    let mut vm: *mut vm_instance_t = null_mut();

    let platform: *mut vm_platform_t = vm_platform_find(type_);
    if platform.is_null() {
        libc::fprintf(c_stderr(), cstr!("VM %s: unknown platform '%s'\n"), name, type_);
        libc::fprintf(c_stderr(), cstr!("VM %s: unable to create instance!\n"), name);
        vm_free(vm);
        return null_mut();
    }

    // Create a generic VM instance
    vm = vm_create(name, instance_id, platform);
    if vm.is_null() {
        libc::fprintf(c_stderr(), cstr!("VM %s: unable to create instance!\n"), name);
        vm_free(vm);
        return null_mut();
    }

    // Initialize specific parts
    if (*(*vm).platform).create_instance.unwrap()(vm) == -1 {
        libc::fprintf(c_stderr(), cstr!("VM %s: unable to create instance!\n"), name);
        vm_free(vm);
        return null_mut();
    }

    vm
}

/// Free resources used by a VM instance
unsafe extern "C" fn vm_reg_delete_instance(data: *mut c_void, _arg: *mut c_void) -> c_int {
    let vm: *mut vm_instance_t = data.cast::<_>();
    (*(*vm).platform).delete_instance.unwrap()(vm)
}

/// Delete a VM instance
#[no_mangle]
pub unsafe extern "C" fn vm_delete_instance(name: *mut c_char) -> c_int {
    registry_delete_if_unused(name, OBJ_TYPE_VM, Some(vm_reg_delete_instance), null_mut())
}

/// Rename a VM instance
#[no_mangle]
pub unsafe extern "C" fn vm_rename_instance(vm: *mut vm_instance_t, name: *mut c_char) -> c_int {
    let mut old_lock_file: *mut c_char = null_mut();
    let mut old_lock_fd: *mut libc::FILE = null_mut();
    let mut globbuf: libc::glob_t = zeroed::<_>();
    let pattern: *mut c_char;
    let mut filename: *mut c_char;
    let mut do_rename: c_int = 0;

    if name.is_null() || vm.is_null() {
        return -1; // invalid argument
    }

    if (*vm).status != VM_STATUS_HALTED {
        return -1; // VM is not stopped
    }

    if libc::strcmp((*vm).name, name) == 0 {
        return 0; // same name, done
    }

    if !registry_exists(name, OBJ_TYPE_VM).is_null() {
        return -1; // name already exists
    }

    let old_name: *mut c_char = (*vm).name;
    (*vm).name = null_mut();

    (*vm).name = libc::strdup(name);
    if (*vm).name.is_null() {
        libc::free((*vm).name.cast::<_>());
        (*vm).name = old_name;

        if do_rename != 0 {
            vm_release_lock(vm, TRUE);
            (*vm).lock_file = old_lock_file;
            (*vm).lock_fd = old_lock_fd;
        }
        return -1; // out of memory
    }

    // get new lock
    do_rename = (vm_file_naming_type != 1) as c_int;
    if do_rename != 0 {
        old_lock_file = (*vm).lock_file;
        old_lock_fd = (*vm).lock_fd;
        (*vm).lock_file = null_mut();
        (*vm).lock_fd = null_mut();

        if vm_get_lock(vm) == -1 {
            libc::free((*vm).name.cast::<_>());
            (*vm).name = old_name;

            if do_rename != 0 {
                vm_release_lock(vm, TRUE);
                (*vm).lock_file = old_lock_file;
                (*vm).lock_fd = old_lock_fd;
            }
            return -1;
        }
    }

    if registry_rename(old_name, (*vm).name, OBJ_TYPE_VM) != 0 {
        // failed to rename
        libc::free((*vm).name.cast::<_>());
        (*vm).name = old_name;

        if do_rename != 0 {
            vm_release_lock(vm, TRUE);
            (*vm).lock_file = old_lock_file;
            (*vm).lock_fd = old_lock_fd;
        }
        return -1;
    }

    vm_log!(vm, cstr!("VM"), cstr!("renamed from '%s' to '%s'"), old_name, (*vm).name);

    // rename files (best effort)
    if do_rename != 0 {
        libc::fclose(old_lock_fd);
        libc::unlink(old_lock_file);
        libc::free(old_lock_file.cast::<_>());

        vm_close_log(vm);

        pattern = dyn_sprintf!(cstr!("%s_%s_*"), vm_get_type(vm), old_name);
        'rename: {
            if pattern.is_null() {
                break 'rename;
            }

            if libc::glob(pattern, libc::GLOB_NOSORT, None, addr_of_mut!(globbuf)) != 0 {
                break 'rename;
            }

            for i in 0..globbuf.gl_pathc {
                filename = dyn_sprintf!(cstr!("%s_%s_%s"), vm_get_type(vm), (*vm).name, *globbuf.gl_pathv.add(i).add(libc::strlen(pattern) - 1));
                if filename.is_null() {
                    break; // out of memory
                }

                if libc::rename(*globbuf.gl_pathv.add(i), filename) != 0 {
                    libc::fprintf(c_stderr(), cstr!("Warning: vm_rename_instance: rename(\"%s\",\"%s\"): %s\n"), *globbuf.gl_pathv.add(i), filename, libc::strerror(c_errno()));
                }
                libc::free(filename.cast::<_>());
            }
            libc::globfree(addr_of_mut!(globbuf));
        }
        libc::free(pattern.cast::<_>());

        vm_reopen_log(vm);
    }

    libc::free(old_name.cast::<_>());
    0 // done
}

/// Initialize a VM instance
#[no_mangle]
pub unsafe extern "C" fn vm_init_instance(vm: *mut vm_instance_t) -> c_int {
    (*(*vm).platform).init_instance.unwrap()(vm)
}

/// Stop a VM instance
#[no_mangle]
pub unsafe extern "C" fn vm_stop_instance(vm: *mut vm_instance_t) -> c_int {
    (*(*vm).platform).stop_instance.unwrap()(vm)
}

/// Delete all VM instances
#[no_mangle]
pub unsafe extern "C" fn vm_delete_all_instances() -> c_int {
    registry_delete_type(OBJ_TYPE_VM, Some(vm_reg_delete_instance), null_mut())
}

/// Save configurations of all VM instances */
unsafe extern "C" fn vm_reg_save_config(entry: *mut registry_entry_t, opt: *mut c_void, _err: *mut c_int) {
    let vm: *mut vm_instance_t = (*entry).data.cast::<_>();
    let fd: *mut libc::FILE = opt.cast::<_>();

    vm_save_config(vm, fd);

    // Save specific platform options
    if (*(*vm).platform).save_config.is_some() {
        (*(*vm).platform).save_config.unwrap()(vm, fd);
    }
}

/// Save all VM configs
#[no_mangle]
pub unsafe extern "C" fn vm_save_config_all(fd: *mut libc::FILE) -> c_int {
    registry_foreach_type(OBJ_TYPE_VM, Some(vm_reg_save_config), fd.cast::<_>(), null_mut());
    0
}

/// OIR to start a slot/subslot
#[no_mangle]
pub unsafe extern "C" fn vm_oir_start(vm: *mut vm_instance_t, slot: u_int, subslot: u_int) -> c_int {
    if (*(*vm).platform).oir_start.is_some() {
        return (*(*vm).platform).oir_start.unwrap()(vm, slot, subslot);
    }

    // OIR not supported
    -1
}

/// OIR to stop a slot/subslot
#[no_mangle]
pub unsafe extern "C" fn vm_oir_stop(vm: *mut vm_instance_t, slot: u_int, subslot: u_int) -> c_int {
    if (*(*vm).platform).oir_stop.is_some() {
        return (*(*vm).platform).oir_stop.unwrap()(vm, slot, subslot);
    }

    // OIR not supported
    -1
}

/// Set the JIT translation sharing group
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn vm_set_tsg(vm: *mut vm_instance_t, group: c_int) -> c_int {
    if (*vm).status == VM_STATUS_RUNNING {
        return -1;
    }

    (*vm).tsg = group;
    0
}
