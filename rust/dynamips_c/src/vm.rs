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

/// cbindgen:no-export
#[repr(C)]
pub struct vm_platform {
    _todo: u8,
}

/// Log a message
pub unsafe fn vm_flog(vm: *mut vm_instance_t, module: *mut c_char, format: *mut c_char, args: &[&dyn sprintf::Printf]) {
    if !(*vm).log_fd.is_null() {
        m_flog((*vm).log_fd, module, format, args);
    }
}
