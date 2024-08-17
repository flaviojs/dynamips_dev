//! Cisco 3600 simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 3600 routines and definitions (EEPROM,...).

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dev_c3600_iofpga::*;
use crate::dynamips_common::*;
use crate::net::*;
use crate::nmc93cx6::*;
use crate::vm::*;

pub type c3600_t = c3600_router;

/// Default C3600 parameters
#[no_mangle]
pub static mut C3600_DEFAULT_CHASSIS: *mut c_char = cstr!("3640");
pub const C3600_DEFAULT_RAM_SIZE: c_int = 128;
pub const C3600_DEFAULT_ROM_SIZE: c_int = 2;
pub const C3600_DEFAULT_NVRAM_SIZE: c_int = 128;
pub const C3600_DEFAULT_CONF_REG: c_int = 0x2102;
pub const C3600_DEFAULT_CLOCK_DIV: c_int = 4;
pub const C3600_DEFAULT_RAM_MMAP: c_int = 1;
pub const C3600_DEFAULT_DISK0_SIZE: c_int = 0;
pub const C3600_DEFAULT_DISK1_SIZE: c_int = 0;
pub const C3600_DEFAULT_IOMEM_SIZE: c_int = 5; // Percents!

/// 6 NM slots for the 3660 + integrated FastEthernet ports
pub const C3600_MAX_NM_BAYS: usize = 7;

/// C3600 DUART Interrupt
pub const C3600_DUART_IRQ: c_int = 5;

/// C3600 Network I/O Interrupt
pub const C3600_NETIO_IRQ: c_int = 2;

/// C3600 GT64k DMA/Timer Interrupt
pub const C3600_GT64K_IRQ: c_int = 4;

/// C3600 External Interrupt
pub const C3600_EXT_IRQ: c_int = 6;

/// C3600 NM Management Interrupt handler
pub const C3600_NM_MGMT_IRQ: c_int = 3;

/// Network IRQ
pub const C3600_NETIO_IRQ_BASE: c_int = 32;
pub const C3600_NETIO_IRQ_PORT_BITS: c_int = 2;
pub const C3600_NETIO_IRQ_PORT_MASK: c_int = (1 << C3600_NETIO_IRQ_PORT_BITS) - 1;
pub const C3600_NETIO_IRQ_PER_SLOT: c_int = 1 << C3600_NETIO_IRQ_PORT_BITS;
pub const C3600_NETIO_IRQ_END: c_int = C3600_NETIO_IRQ_BASE + (C3600_MAX_NM_BAYS as c_int * C3600_NETIO_IRQ_PER_SLOT) - 1;

/// C3600 common device addresses
pub const C3600_GT64K_ADDR: m_uint64_t = 0x14000000_u64;
pub const C3600_IOFPGA_ADDR: m_uint64_t = 0x1e800000_u64;
pub const C3600_DUART_ADDR: m_uint64_t = 0x1e840000_u64;
pub const C3600_BITBUCKET_ADDR: m_uint64_t = 0x1ec00000_u64;
pub const C3600_NVRAM_ADDR: m_uint64_t = 0x1fe00000_u64;
pub const C3600_ROM_ADDR: m_uint64_t = 0x1fc00000_u64;
pub const C3600_BOOTFLASH_ADDR: m_uint64_t = 0x30000000_u64;
pub const C3600_PCI_IO_ADDR: m_uint64_t = 0x100000000_u64;

/// Reserved space for ROM in NVRAM
pub const C3600_NVRAM_ROM_RES_SIZE: size_t = 2048;

/// C3600 ELF Platform ID
pub const C3620_ELF_MACHINE_ID: c_int = 0x1e;
pub const C3640_ELF_MACHINE_ID: c_int = 0x1e;
pub const C3660_ELF_MACHINE_ID: c_int = 0x34;

#[no_mangle]
pub unsafe extern "C" fn VM_C3600(vm: *mut vm_instance_t) -> *mut c3600_t {
    (*vm).hw_data.cast::<_>()
}

/// Prototype of chassis driver initialization function
pub type c3600_chassis_init_fn = Option<unsafe extern "C" fn(router: *mut c3600_t) -> c_int>;

/// C3600 Chassis Driver
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c3600_chassis_driver {
    pub chassis_type: *mut c_char,
    pub chassis_id: c_int,
    pub supported: c_int,
    pub chassis_init: c3600_chassis_init_fn,
    pub eeprom: *mut cisco_eeprom,
}

/// C3600 router
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c3600_router {
    //// Chassis MAC address
    pub mac_addr: n_eth_addr_t,

    pub board_id: [c_char; 20],

    /// Associated VM instance
    pub vm: *mut vm_instance_t,

    /// I/O FPGA
    pub iofpga_data: *mut c3600_iofpga_data,

    /// Chassis information
    pub chassis_driver: *mut c3600_chassis_driver,
    pub oir_status: m_uint16_t,

    /// Mainboard EEPROM.
    /// It can be modified to change the chassis MAC address.
    pub mb_eeprom: cisco_eeprom,
    pub mb_eeprom_group: nmc93cX6_group,

    /// Network Module EEPROMs (3620/3640)
    pub nm_eeprom_group: nmc93cX6_group,

    //// Cisco 3660 NM EEPROMs
    pub c3660_nm_eeprom_group: [nmc93cX6_group; C3600_MAX_NM_BAYS],
}
