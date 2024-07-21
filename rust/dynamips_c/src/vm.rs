//! Virtual machine abstraction.

use crate::_private::*;
use crate::cisco_card::*;
use crate::cpu::*;
use crate::dev_vtty::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::pci_dev::*;
use crate::pci_io::*;
use crate::rommon_var::*;
use crate::utils::*;

extern "C" {
    pub fn vm_ios_save_config(vm: *mut vm_instance_t) -> c_int;
    pub fn vm_object_dump(vm: *mut vm_instance_t);
    pub fn vm_resume(vm: *mut vm_instance_t) -> c_int;
    pub fn vm_suspend(vm: *mut vm_instance_t) -> c_int;
}

pub type vm_chunk_t = vm_chunk;
pub type vm_obj_t = vm_obj;
pub type vm_instance_t = vm_instance;
pub type vm_platform_t = vm_platform;

/// Maximum number of devices per VM
pub const VM_DEVICE_MAX: usize = 1 << 6;

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

/// Size of the PCI bus pool
pub const VM_PCI_POOL_SIZE: usize = 32;

/// Max slots per VM
pub const VM_MAX_SLOTS: usize = 16;

// VM instance status // TODO enum
/// VM is halted and no HW resources are used
pub const VM_STATUS_HALTED: c_int = 0;
/// Shutdown procedure engaged
pub const VM_STATUS_SHUTDOWN: c_int = 1;
/// VM is running
pub const VM_STATUS_RUNNING: c_int = 2;
/// VM is suspended
pub const VM_STATUS_SUSPENDED: c_int = 3;

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
    /// Platform specific helpers
    pub platform: *mut vm_platform_t,
    /// Instance status
    pub status: c_int,
    /// Instance Identifier
    pub instance_id: c_int,
    /// Lock file
    pub lock_file: *mut c_char,
    /// Log filename
    pub log_file: *mut c_char,
    /// Logging enabled
    pub log_file_enabled: c_int,
    /// RAM and ROM size in Mb
    pub ram_size: u_int,
    pub rom_size: u_int,
    /// RAM reserved space size
    pub ram_res_size: u_int,
    /// IOMEM size in Mb
    pub iomem_size: u_int,
    /// NVRAM size in Kb
    pub nvram_size: u_int,
    /// PCMCIA disk0 and disk1 sizes (in Mb)
    pub pcmcia_disk_size: [u_int; 2],
    /// Config register
    pub conf_reg: u_int,
    pub conf_reg_setup: u_int,
    /// Clock Divisor (see cp0.c)
    pub clock_divisor: u_int,
    /// Memory-mapped RAM ?
    pub ram_mmap: u_int,
    /// Restart IOS on reload ?
    pub restart_ios: u_int,
    /// ELF machine identifier
    pub elf_machine_id: u_int,
    /// Size of execution area for CPU
    pub exec_area_size: u_int,
    /// IOS entry point
    pub ios_entry_point: m_uint32_t,
    /// IOS image filename
    pub ios_image: *mut c_char,
    /// IOS configuration file for startup-config
    pub ios_startup_config: *mut c_char,
    /// IOS configuration file for private-config
    pub ios_private_config: *mut c_char,
    /// ROM filename
    pub rom_filename: *mut c_char,
    /// Symbol filename
    pub sym_filename: *mut c_char,
    /// Lock/Log file descriptors
    pub lock_fd: *mut libc::FILE,
    pub log_fd: *mut libc::FILE,
    /// Debugging Level
    pub debug_level: c_int,
    /// CPUs use JIT
    pub jit_use: c_int,
    /// Use sparse virtual memory
    pub sparse_mem: c_int,
    /// IO mem size to be passed to Smart Init
    pub nm_iomem_size: u_int,

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

/// Log a message
pub unsafe fn vm_flog(vm: *mut vm_instance_t, module: *mut c_char, format: *mut c_char, args: &[&dyn sprintf::Printf]) {
    if !(*vm).log_fd.is_null() {
        m_flog((*vm).log_fd, module, format, args);
    }
}

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
            libc::fprintf(c_stderr(), cstr!("%s '%s': %s"), vm_get_log_name(vm), (*vm).name, bytes.as_ptr());
        }
    };
}
pub use vm_error;

/// Get VM type
#[no_mangle]
pub unsafe extern "C" fn vm_get_type(vm: *mut vm_instance_t) -> *mut c_char {
    (*(*vm).platform).name
}

/// Get log name
pub unsafe fn vm_get_log_name(vm: *mut vm_instance_t) -> *mut c_char {
    if !(*(*vm).platform).log_name.is_null() {
        return (*(*vm).platform).log_name;
    }

    // default value
    cstr!("VM")
}

/// Clear an IRQ for a VM
#[no_mangle]
#[inline]
pub unsafe extern "C" fn vm_clear_irq(vm: *mut vm_instance_t, irq: u_int) {
    if (*vm).clear_irq.is_some() {
        (*vm).clear_irq.unwrap()(vm, irq);
    }
}

/// Set an IRQ for a VM
#[no_mangle]
#[inline]
pub unsafe extern "C" fn vm_set_irq(vm: *mut vm_instance_t, irq: u_int) {
    if (*vm).set_irq.is_some() {
        (*vm).set_irq.unwrap()(vm, irq);
    }
}
