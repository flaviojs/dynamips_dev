//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 2600 routines and definitions (EEPROM,...).

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dev_c2600_iofpga::*;
use crate::dev_mpc860::*;
use crate::dynamips_common::*;
use crate::net::*;
use crate::nmc93cx6::*;
use crate::vm::*;

pub type c2600_t = c2600_router;

/// Default C2600 parameters
#[no_mangle]
pub static mut C2600_DEFAULT_MAINBOARD: *mut c_char = cstr!("2610");
pub const C2600_DEFAULT_RAM_SIZE: c_int = 64;
pub const C2600_DEFAULT_ROM_SIZE: c_int = 2;
pub const C2600_DEFAULT_NVRAM_SIZE: c_int = 128;
pub const C2600_DEFAULT_CONF_REG: c_int = 0x2102;
pub const C2600_DEFAULT_CLOCK_DIV: c_int = 8;
pub const C2600_DEFAULT_RAM_MMAP: c_int = 1;
pub const C2600_DEFAULT_DISK0_SIZE: c_int = 0;
pub const C2600_DEFAULT_DISK1_SIZE: c_int = 0;
pub const C2600_DEFAULT_IOMEM_SIZE: c_int = 15; // Percents!

/// 2600 characteristics: 1 NM + mainboard, 2 onboard WIC slots
pub const C2600_MAX_NM_BAYS: c_int = 2;
pub const C2600_MAX_WIC_BAYS: c_int = 2;

/// C2600 Virtual Timer Interrupt
pub const C2600_VTIMER_IRQ: c_int = 0;

/// C2600 DUART Interrupt
pub const C2600_DUART_IRQ: c_int = 1;

/// C2600 Network I/O Interrupt
pub const C2600_NETIO_IRQ: c_int = 2;

/// C2600 PA Management Interrupt
pub const C2600_PA_MGMT_IRQ: c_int = 3;

/// Network IRQ
pub const C2600_NETIO_IRQ_BASE: c_int = 32;
pub const C2600_NETIO_IRQ_PORT_BITS: c_int = 2;
pub const C2600_NETIO_IRQ_PORT_MASK: c_int = (1 << C2600_NETIO_IRQ_PORT_BITS) - 1;
pub const C2600_NETIO_IRQ_PER_SLOT: c_int = 1 << C2600_NETIO_IRQ_PORT_BITS;
pub const C2600_NETIO_IRQ_END: c_int = C2600_NETIO_IRQ_BASE + (C2600_MAX_NM_BAYS * C2600_NETIO_IRQ_PER_SLOT) - 1;

/// C2600 common device addresses
pub const C2600_FLASH_ADDR: m_uint64_t = 0x60000000_u64;
pub const C2600_WIC_ADDR: m_uint64_t = 0x67000000_u64;
pub const C2600_IOFPGA_ADDR: m_uint64_t = 0x67400000_u64;
pub const C2600_NVRAM_ADDR: m_uint64_t = 0x67c00000_u64;
pub const C2600_PCICTRL_ADDR: m_uint64_t = 0x68000000_u64;
pub const C2600_MPC860_ADDR: m_uint64_t = 0x68010000_u64;
pub const C2600_DUART_ADDR: m_uint64_t = 0xffe00000_u64;
pub const C2600_ROM_ADDR: m_uint64_t = 0xfff00000_u64;

/// WIC interval in address space
pub const C2600_WIC_SIZE: c_int = 0x400;

/// Reserved space for ROM in NVRAM
pub const C2600_NVRAM_ROM_RES_SIZE: size_t = 2048;

/// C2600 ELF Platform ID
pub const C2600_ELF_MACHINE_ID: c_int = 0x2b;

#[no_mangle]
pub unsafe extern "C" fn VM_C2600(vm: *mut vm_instance_t) -> *mut c2600_t {
    (*vm).hw_data.cast::<_>()
}

/// C2600 router
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c2600_router {
    /// Mainboard type (2610, 2611, etc)
    pub mainboard_type: *mut c_char,

    /// Is the router a XM model ?
    pub xm_model: c_int,

    /// Chassis MAC address
    pub mac_addr: n_eth_addr_t,

    pub board_id: [c_char; 20],

    /// Associated VM instance
    pub vm: *mut vm_instance_t,

    /// I/O FPGA
    pub iofpga_data: *mut c2600_iofpga_data,

    /// Mainboard EEPROM.
    /// It can be modified to change the chassis MAC address.
    pub mb_eeprom: cisco_eeprom,
    pub mb_eeprom_group: nmc93cX6_group,

    /// Network Module EEPROM
    pub nm_eeprom_group: nmc93cX6_group,

    /// MPC860 device private data
    pub mpc_data: *mut mpc860_data,
}
