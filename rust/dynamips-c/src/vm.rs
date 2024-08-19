//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
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

/// Get log name
pub unsafe fn vm_get_log_name(vm: *mut vm_instance_t) -> *mut c_char {
    if !(*(*vm).platform).log_name.is_null() {
        return (*(*vm).platform).log_name;
    }

    // default value
    cstr!("VM")
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
