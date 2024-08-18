//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 3725 routines and definitions (EEPROM,...).

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dev_c3725_iofpga::*;
use crate::dev_gt::*;
use crate::dynamips_common::*;
use crate::net::*;
use crate::nmc93cx6::*;
use crate::vm::*;

pub type c3725_t = c3725_router;

/// Default C3725 parameters
pub const C3725_DEFAULT_RAM_SIZE: c_int = 128;
pub const C3725_DEFAULT_ROM_SIZE: c_int = 2;
pub const C3725_DEFAULT_NVRAM_SIZE: c_int = 112;
pub const C3725_DEFAULT_CONF_REG: c_int = 0x2102;
pub const C3725_DEFAULT_CLOCK_DIV: c_int = 8;
pub const C3725_DEFAULT_RAM_MMAP: c_int = 1;
pub const C3725_DEFAULT_DISK0_SIZE: c_int = 16;
pub const C3725_DEFAULT_DISK1_SIZE: c_int = 0;
pub const C3725_DEFAULT_IOMEM_SIZE: c_int = 5; // Percents!

/// 3725 characteritics: 2 NM, 3 WIC, 2 AIM
pub const C3725_MAX_NM_BAYS: c_int = 3;
pub const C3725_MAX_WIC_BAYS: c_int = 3;

/// C3725 DUART Interrupt
pub const C3725_DUART_IRQ: c_int = 5;

/// C3725 Network I/O Interrupt
pub const C3725_NETIO_IRQ: c_int = 2;

/// C3725 GT64k DMA/Timer Interrupt
pub const C3725_GT96K_IRQ: c_int = 3;

/// C3725 External Interrupt
pub const C3725_EXT_IRQ: c_int = 6;

/// Network IRQ
pub const C3725_NETIO_IRQ_BASE: c_int = 32;
pub const C3725_NETIO_IRQ_PORT_BITS: c_int = 2;
pub const C3725_NETIO_IRQ_PORT_MASK: c_int = (1 << C3725_NETIO_IRQ_PORT_BITS) - 1;
pub const C3725_NETIO_IRQ_PER_SLOT: c_int = 1 << C3725_NETIO_IRQ_PORT_BITS;
pub const C3725_NETIO_IRQ_END: c_int = C3725_NETIO_IRQ_BASE + (C3725_MAX_NM_BAYS * C3725_NETIO_IRQ_PER_SLOT) - 1;

/// C3725 common device addresses
pub const C3725_GT96K_ADDR: m_uint64_t = 0x14000000_u64;
pub const C3725_IOFPGA_ADDR: m_uint64_t = 0x1e800000_u64;
pub const C3725_BITBUCKET_ADDR: m_uint64_t = 0x1ec00000_u64;
pub const C3725_ROM_ADDR: m_uint64_t = 0x1fc00000_u64;
pub const C3725_SLOT0_ADDR: m_uint64_t = 0x30000000_u64;
pub const C3725_SLOT1_ADDR: m_uint64_t = 0x32000000_u64;
pub const C3725_DUART_ADDR: m_uint64_t = 0x3c100000_u64;
pub const C3725_WIC_ADDR: m_uint64_t = 0x3c200000_u64;
pub const C3725_BSWAP_ADDR: m_uint64_t = 0xc0000000_u64;
pub const C3725_PCI_IO_ADDR: m_uint64_t = 0x100000000_u64;

/// WIC interval in address space
pub const C3725_WIC_SIZE: c_int = 0x2000;

/// Offset of simulated NVRAM in ROM flash
pub const C3725_NVRAM_OFFSET: size_t = 0xE0000;
pub const C3725_NVRAM_SIZE: size_t = 0x1C000; // with backup

/// Reserved space for ROM in NVRAM
pub const C3725_NVRAM_ROM_RES_SIZE: c_int = 0;

/// C3725 ELF Platform ID
pub const C3725_ELF_MACHINE_ID: c_int = 0x61;

#[no_mangle]
pub unsafe extern "C" fn VM_C3725(vm: *mut vm_instance_t) -> *mut c3725_t {
    (*vm).hw_data.cast::<_>()
}

/// C3725 router
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c3725_router {
    /// Chassis MAC address
    pub mac_addr: n_eth_addr_t,

    pub board_id: [c_char; 20],

    /// Associated VM instance
    pub vm: *mut vm_instance_t,

    /// GT96100 data
    pub gt_data: *mut gt_data,

    /// I/O FPGA
    pub iofpga_data: *mut c3725_iofpga_data,

    /// Chassis information
    pub oir_status: m_uint8_t,

    /// Mainboard EEPROM.
    /// It can be modified to change the chassis MAC address.
    pub mb_eeprom: cisco_eeprom,
    pub mb_eeprom_group: nmc93cX6_group,

    /// Network Module EEPROMs
    pub nm_eeprom_group: [nmc93cX6_group; 2],
}
