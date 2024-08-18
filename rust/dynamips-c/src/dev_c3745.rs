//! Cisco 3745 simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 3745 routines and definitions (EEPROM,...).

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dev_c3745_iofpga::*;
use crate::dev_gt::*;
use crate::dynamips_common::*;
use crate::net::*;
use crate::nmc93cx6::*;
use crate::vm::*;

pub type c3745_t = c3745_router;

/// Default C3745 parameters
pub const C3745_DEFAULT_RAM_SIZE: c_int = 128;
pub const C3745_DEFAULT_ROM_SIZE: c_int = 2;
pub const C3745_DEFAULT_NVRAM_SIZE: c_int = 304;
pub const C3745_DEFAULT_CONF_REG: c_int = 0x2102;
pub const C3745_DEFAULT_CLOCK_DIV: c_int = 8;
pub const C3745_DEFAULT_RAM_MMAP: c_int = 1;
pub const C3745_DEFAULT_DISK0_SIZE: c_int = 16;
pub const C3745_DEFAULT_DISK1_SIZE: c_int = 0;
pub const C3745_DEFAULT_IOMEM_SIZE: c_int = 5; // Percents!

/// 3745 characteritics: 4 NM (+ motherboard), 3 WIC, 2 AIM
pub const C3745_MAX_NM_BAYS: c_int = 5;
pub const C3745_MAX_WIC_BAYS: c_int = 3;

/// C3745 DUART Interrupt
pub const C3745_DUART_IRQ: c_int = 5;

/// C3745 Network I/O Interrupt
pub const C3745_NETIO_IRQ: c_int = 2;

/// C3745 GT64k DMA/Timer Interrupt
pub const C3745_GT96K_IRQ: c_int = 3;

/// C3745 External Interrupt
pub const C3745_EXT_IRQ: c_int = 6;

/// Network IRQ
pub const C3745_NETIO_IRQ_BASE: c_int = 32;
pub const C3745_NETIO_IRQ_PORT_BITS: c_int = 2;
pub const C3745_NETIO_IRQ_PORT_MASK: c_int = (1 << C3745_NETIO_IRQ_PORT_BITS) - 1;
pub const C3745_NETIO_IRQ_PER_SLOT: c_int = 1 << C3745_NETIO_IRQ_PORT_BITS;
pub const C3745_NETIO_IRQ_END: c_int = C3745_NETIO_IRQ_BASE + (C3745_MAX_NM_BAYS * C3745_NETIO_IRQ_PER_SLOT) - 1;

/// C3745 common device addresses
pub const C3745_BITBUCKET_ADDR: m_uint64_t = 0x1ec00000_u64;
pub const C3745_IOFPGA_ADDR: m_uint64_t = 0x1fa00000_u64;
pub const C3745_ROM_ADDR: m_uint64_t = 0x1fc00000_u64;
pub const C3745_GT96K_ADDR: m_uint64_t = 0x24000000_u64;
pub const C3745_SLOT0_ADDR: m_uint64_t = 0x30000000_u64;
pub const C3745_SLOT1_ADDR: m_uint64_t = 0x32000000_u64;
pub const C3745_DUART_ADDR: m_uint64_t = 0x3c100000_u64;
pub const C3745_WIC_ADDR: m_uint64_t = 0x3c200000_u64;
pub const C3745_BSWAP_ADDR: m_uint64_t = 0xc0000000_u64;
pub const C3745_PCI_IO_ADDR: m_uint64_t = 0x100000000_u64;

/// WIC interval in address space
pub const C3745_WIC_SIZE: c_int = 0x2000;

/// Offset of simulated NVRAM in ROM flash
pub const C3745_NVRAM_OFFSET: size_t = 0xB0000;
pub const C3745_NVRAM_SIZE: size_t = 0x4C000; // with backup

/// Reserved space for ROM in NVRAM
pub const C3745_NVRAM_ROM_RES_SIZE: c_int = 0;

/// C3745 ELF Platform ID
pub const C3745_ELF_MACHINE_ID: c_int = 0x69;

#[no_mangle]
pub unsafe extern "C" fn VM_C3745(vm: *mut vm_instance_t) -> *mut c3745_t {
    (*vm).hw_data.cast::<_>()
}

/* C3745 router */
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c3745_router {
    /// Chassis MAC address
    pub mac_addr: n_eth_addr_t,

    pub board_id: [c_char; 20],

    /// Associated VM instance
    pub vm: *mut vm_instance_t,

    /// GT96100 data
    pub gt_data: *mut gt_data,

    /// I/O FPGA
    pub iofpga_data: *mut c3745_iofpga_data,

    /// OIR status
    pub oir_status: m_uint8_t,

    /// System EEPROMs.
    /// It can be modified to change the chassis MAC address.
    pub sys_eeprom: [cisco_eeprom; 3],
    pub sys_eeprom_group: nmc93cX6_group,

    /// Network Module EEPROMs
    pub nm_eeprom_group: [nmc93cX6_group; 4],
}
