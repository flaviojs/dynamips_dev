//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! Generic Cisco MSFC1 routines and definitions (EEPROM,...).

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dev_c6msfc1_mpfpga::*;
use crate::dev_ds1620::*;
use crate::dynamips_common::*;
use crate::net::*;
use crate::nmc93cx6::*;
use crate::vm::*;

pub type c6msfc1_t = c6msfc1_router;

/// Default MSFC1 parameters
pub const C6MSFC1_DEFAULT_RAM_SIZE: c_int = 256;
pub const C6MSFC1_DEFAULT_ROM_SIZE: c_int = 4;
pub const C6MSFC1_DEFAULT_NVRAM_SIZE: c_int = 128;
pub const C6MSFC1_DEFAULT_CONF_REG: c_int = 0x2102;
pub const C6MSFC1_DEFAULT_CLOCK_DIV: c_int = 4;
pub const C6MSFC1_DEFAULT_RAM_MMAP: c_int = 1;

/// EOBC + IBC
pub const C6MSFC1_MAX_PA_BAYS: c_int = 2;

/// MSFC1 Timer IRQ (virtual)
pub const C6MSFC1_VTIMER_IRQ: c_int = 0;

/// MSFC1 DUART Interrupt
pub const C6MSFC1_DUART_IRQ: c_int = 5;

/// MSFC1 Network I/O Interrupt
pub const C6MSFC1_NETIO_IRQ: c_int = 2;

/// MSFC1 PA Management Interrupt handler
pub const C6MSFC1_PA_MGMT_IRQ: c_int = 3;

/// MSFC1 GT64k DMA/Timer Interrupt
pub const C6MSFC1_GT64K_IRQ: c_int = 4;

/// MSFC1 Error/OIR Interrupt
pub const C6MSFC1_OIR_IRQ: c_int = 6;

/// Network IRQ
pub const C6MSFC1_NETIO_IRQ_BASE: c_int = 32;
pub const C6MSFC1_NETIO_IRQ_END: c_int = C6MSFC1_NETIO_IRQ_BASE + C6MSFC1_MAX_PA_BAYS - 1;

/// MSFC1 base ram limit (256 Mb)
pub const C6MSFC1_BASE_RAM_LIMIT: c_int = 256;

/// MSFC1 common device addresses
pub const C6MSFC1_GT64K_ADDR: m_uint64_t = 0x14000000_u64;
pub const C6MSFC1_GT64K_SEC_ADDR: m_uint64_t = 0x15000000_u64;
pub const C6MSFC1_BOOTFLASH_ADDR: m_uint64_t = 0x1a000000_u64;
pub const C6MSFC1_NVRAM_ADDR: m_uint64_t = 0x1e000000_u64;
pub const C6MSFC1_MPFPGA_ADDR: m_uint64_t = 0x1e800000_u64;
pub const C6MSFC1_IOFPGA_ADDR: m_uint64_t = 0x1e840000_u64;
pub const C6MSFC1_BITBUCKET_ADDR: m_uint64_t = 0x1f000000_u64;
pub const C6MSFC1_ROM_ADDR: m_uint64_t = 0x1fc00000_u64;
pub const C6MSFC1_IOMEM_ADDR: m_uint64_t = 0x20000000_u64;
pub const C6MSFC1_SRAM_ADDR: m_uint64_t = 0x4b000000_u64;
pub const C6MSFC1_BSWAP_ADDR: m_uint64_t = 0xc0000000_u64;
pub const C6MSFC1_PCI_IO_ADDR: m_uint64_t = 0x100000000_u64;

/// SRAM size
pub const C6MSFC1_SRAM_SIZE: size_t = 4096 * 1024;

/// Reserved space for ROM in NVRAM
pub const C6MSFC1_NVRAM_ROM_RES_SIZE: size_t = 2048;

/// MSFC1 physical address bus mask: keep only the lower 33 bits
pub const C6MSFC1_ADDR_BUS_MASK: m_uint64_t = 0x1ffffffff_u64;

/// MSFC1 ELF Platform ID
pub const C6MSFC1_ELF_MACHINE_ID: c_int = 0x19;

/// 2 temperature sensors in a MSFC1: chassis inlet and oulet
pub const C6MSFC1_TEMP_SENSORS: usize = 2;

#[no_mangle]
pub unsafe extern "C" fn VM_C6MSFC1(vm: *mut vm_instance_t) -> *mut c6msfc1_t {
    (*vm).hw_data.cast::<_>()
}

/// MSFC1 router
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c6msfc1_router {
    /// Chassis MAC address
    pub mac_addr: n_eth_addr_t,

    /// Associated VM instance
    pub vm: *mut vm_instance_t,

    /// Midplane FPGA
    pub mpfpga_data: *mut c6msfc1_mpfpga_data,

    /// Midplane EEPROM can be modified to change the chassis MAC address...
    pub cpu_eeprom: cisco_eeprom,
    pub mp_eeprom: cisco_eeprom,

    /// EEPROMs for CPU and Midplane
    pub sys_eeprom_g1: nmc93cX6_group,

    /// Temperature sensors
    pub ds1620_sensors: [ds1620_data; C6MSFC1_TEMP_SENSORS],

    /// Slot of this MSFC
    pub msfc_slot: u_int,
}
