//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 1700 routines and definitions (EEPROM,...).

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dev_c1700_iofpga::*;
use crate::dev_mpc860::*;
use crate::dynamips_common::*;
use crate::net::*;
use crate::nmc93cx6::*;
use crate::vm::*;

pub type c1700_t = c1700_router;

/// Default C1700 parameters
#[no_mangle]
pub static mut C1700_DEFAULT_MAINBOARD: *mut c_char = cstr!("1720");
pub const C1700_DEFAULT_RAM_SIZE: c_int = 64;
pub const C1700_DEFAULT_ROM_SIZE: c_int = 2;
pub const C1700_DEFAULT_NVRAM_SIZE: c_int = 32;
pub const C1700_DEFAULT_CONF_REG: c_int = 0x2102;
pub const C1700_DEFAULT_CLOCK_DIV: c_int = 8;
pub const C1700_DEFAULT_RAM_MMAP: c_int = 1;
pub const C1700_DEFAULT_DISK0_SIZE: c_int = 0;
pub const C1700_DEFAULT_DISK1_SIZE: c_int = 0;
pub const C1700_DEFAULT_IOMEM_SIZE: c_int = 15; // Percents!

/// 1700 characteristics: only mainboard (considered as fake NM)
pub const C1700_MAX_NM_BAYS: c_int = 1;
pub const C1700_MAX_WIC_BAYS: c_int = 2;

/// C1700 Virtual Timer Interrupt
pub const C1700_VTIMER_IRQ: c_int = 0;

/// C1700 DUART Interrupt
pub const C1700_DUART_IRQ: c_int = 1;

/// C1700 Network I/O Interrupt
pub const C1700_NETIO_IRQ: c_int = 2;

/// C1700 PA Management Interrupt
pub const C1700_PA_MGMT_IRQ: c_int = 3;

/// Network IRQ
pub const C1700_NETIO_IRQ_BASE: c_int = 32;
pub const C1700_NETIO_IRQ_PORT_BITS: c_int = 2;
pub const C1700_NETIO_IRQ_PORT_MASK: c_int = (1 << C1700_NETIO_IRQ_PORT_BITS) - 1;
pub const C1700_NETIO_IRQ_PER_SLOT: c_int = 1 << C1700_NETIO_IRQ_PORT_BITS;
pub const C1700_NETIO_IRQ_END: c_int = C1700_NETIO_IRQ_BASE + (C1700_MAX_NM_BAYS * C1700_NETIO_IRQ_PER_SLOT) - 1;

/// C1700 common device addresses
pub const C1700_FLASH_ADDR: m_uint64_t = 0x60000000_u64;
pub const C1700_NVRAM_ADDR: m_uint64_t = 0x68000000_u64;
pub const C1700_IOFPGA_ADDR: m_uint64_t = 0x68020000_u64;
pub const C1700_WIC_ADDR: m_uint64_t = 0x68030000_u64;
pub const C1700_DUART_ADDR: m_uint64_t = 0x68050000_u64;
pub const C1700_MPC860_ADDR: m_uint64_t = 0xff000000_u64;
pub const C1700_ROM_ADDR: m_uint64_t = 0xfff00000_u64;

/// WIC interval in address space
pub const C1700_WIC_SIZE: c_int = 0x1000;

/// Reserved space for ROM in NVRAM
pub const C1700_NVRAM_ROM_RES_SIZE: size_t = 2048;

/// C1700 ELF Platform ID
pub const C1700_ELF_MACHINE_ID: c_int = 0x33;

#[no_mangle]
pub unsafe extern "C" fn VM_C1700(vm: *mut vm_instance_t) -> *mut c1700_t {
    (*vm).hw_data.cast::<_>()
}

/// C1700 router
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c1700_router {
    /// Mainboard type (2610, 2611, etc)
    pub mainboard_type: *mut c_char,

    /// Chassis MAC address
    pub mac_addr: n_eth_addr_t,

    pub board_id: [c_char; 20],

    /// Associated VM instance
    pub vm: *mut vm_instance_t,

    /// I/O FPGA
    pub iofpga_data: *mut c1700_iofpga_data,

    /// Mainboard EEPROM.
    /// It can be modified to change the chassis MAC address.
    pub mb_eeprom: cisco_eeprom,
    pub mb_eeprom_group: nmc93cX6_group,

    /// Network Module EEPROM
    pub nm_eeprom_group: nmc93cX6_group,

    /// MPC860 device private data
    pub mpc_data: *mut mpc860_data,
}
