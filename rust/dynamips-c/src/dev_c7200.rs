//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 7200 routines and definitions (EEPROM,...).
//!
//! Notes on IRQs (see "show stack"):
//!
//!   - triggering IRQ 3: we get indefinitely (for each slot):
//!        "Error: Unexpected NM Interrupt received from slot: 6"
//!
//!   - triggering IRQ 4: GT64010 reg access: probably "DMA/Timer Interrupt"
//!
//!   - triggering IRQ 6: we get (probably "OIR/Error Interrupt")
//!        %ERR-1-PERR: PCI bus parity error
//!        %ERR-1-SERR: PCI bus system/parity error
//!        %ERR-1-FATAL: Fatal error interrupt, No reloading
//!        err_stat=0x0, err_enable=0x0, mgmt_event=0xFFFFFFFF

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dev_ds1620::*;
use crate::dev_mv64460::*;
use crate::dynamips_common::*;
use crate::net::*;
use crate::nmc93cx6::*;
use crate::pci_dev::*;
use crate::vm::*;

pub type c7200_t = c7200_router;

/// Default C7200 parameters
#[no_mangle]
pub static mut C7200_DEFAULT_NPE_TYPE: *mut c_char = cstr!("npe-400");
#[no_mangle]
pub static mut C7200_DEFAULT_MIDPLANE: *mut c_char = cstr!("vxr");
pub const C7200_DEFAULT_RAM_SIZE: c_int = 256;
pub const C7200_DEFAULT_ROM_SIZE: c_int = 4;
pub const C7200_DEFAULT_NVRAM_SIZE: c_int = 128;
pub const C7200_DEFAULT_CONF_REG: c_int = 0x2102;
pub const C7200_DEFAULT_CLOCK_DIV: c_int = 4;
pub const C7200_DEFAULT_RAM_MMAP: c_int = 1;
pub const C7200_DEFAULT_DISK0_SIZE: c_int = 64;
pub const C7200_DEFAULT_DISK1_SIZE: c_int = 0;

/// 6 slots + 1 I/O card.
/// Slot 8 is special: it is for the NPE-G2 ethernet ports, but doesn't
/// represent something real.
pub const C7200_MAX_PA_BAYS: c_int = 9;

/// C7200 Timer IRQ (virtual)
pub const C7200_VTIMER_IRQ: c_int = 0;

/// C7200 DUART Interrupt
pub const C7200_DUART_IRQ: c_int = 5;

/// C7200 Network I/O Interrupt
pub const C7200_NETIO_IRQ: c_int = 2;

/// C7200 PA Management Interrupt handler
pub const C7200_PA_MGMT_IRQ: c_int = 3;

/// C7200 GT64k DMA/Timer Interrupt
pub const C7200_GT64K_IRQ: c_int = 4;

/// C7200 Error/OIR Interrupt
pub const C7200_OIR_IRQ: c_int = 6;

/// Network IRQ
pub const C7200_NETIO_IRQ_BASE: c_int = 32;
pub const C7200_NETIO_IRQ_PORT_BITS: c_int = 3;
pub const C7200_NETIO_IRQ_PORT_MASK: c_int = (1 << C7200_NETIO_IRQ_PORT_BITS) - 1;
pub const C7200_NETIO_IRQ_PER_SLOT: c_int = 1 << C7200_NETIO_IRQ_PORT_BITS;
pub const C7200_NETIO_IRQ_END: c_int = C7200_NETIO_IRQ_BASE + (C7200_MAX_PA_BAYS * C7200_NETIO_IRQ_PER_SLOT) - 1;

/// C7200 base ram limit (256 Mb)
pub const C7200_BASE_RAM_LIMIT: c_int = 256;

/// C7200 common device addresses
pub const C7200_GT64K_ADDR: m_uint64_t = 0x14000000_u64;
pub const C7200_GT64K_SEC_ADDR: m_uint64_t = 0x15000000_u64;
pub const C7200_BOOTFLASH_ADDR: m_uint64_t = 0x1a000000_u64;
pub const C7200_NVRAM_ADDR: m_uint64_t = 0x1e000000_u64;
pub const C7200_MPFPGA_ADDR: m_uint64_t = 0x1e800000_u64;
pub const C7200_IOFPGA_ADDR: m_uint64_t = 0x1e840000_u64;
pub const C7200_BITBUCKET_ADDR: m_uint64_t = 0x1f000000_u64;
pub const C7200_ROM_ADDR: m_uint64_t = 0x1fc00000_u64;
pub const C7200_IOMEM_ADDR: m_uint64_t = 0x20000000_u64;
pub const C7200_SRAM_ADDR: m_uint64_t = 0x4b000000_u64;
pub const C7200_BSWAP_ADDR: m_uint64_t = 0xc0000000_u64;
pub const C7200_PCI_IO_ADDR: m_uint64_t = 0x100000000_u64;

/// NPE-G1 specific info
pub const C7200_G1_NVRAM_ADDR: m_uint64_t = 0x1e400000_u64;

/// NPE-G2 specific info
pub const C7200_G2_BSWAP_ADDR: m_uint64_t = 0xce000000_u64;
pub const C7200_G2_BOOTFLASH_ADDR: m_uint64_t = 0xe8000000_u64;
pub const C7200_G2_PCI_IO_ADDR: m_uint64_t = 0xf0000000_u64;
pub const C7200_G2_MV64460_ADDR: m_uint64_t = 0xf1000000_u64;
pub const C7200_G2_MPFPGA_ADDR: m_uint64_t = 0xfe000000_u64;
pub const C7200_G2_IOFPGA_ADDR: m_uint64_t = 0xfe040000_u64;
pub const C7200_G2_NVRAM_ADDR: m_uint64_t = 0xff000000_u64;
pub const C7200_G2_ROM_ADDR: m_uint64_t = 0xfff00000_u64;

/// NVRAM size for NPE-G2: 2 Mb
pub const C7200_G2_NVRAM_SIZE: c_int = 2 * 1048576;

/// Reserved space for ROM in NVRAM
pub const C7200_NVRAM_ROM_RES_SIZE: size_t = 2048;

/// C7200 physical address bus mask: keep only the lower 33 bits
pub const C7200_ADDR_BUS_MASK: m_uint64_t = 0x1ffffffff_u64;

/// C7200 ELF Platform ID
pub const C7200_ELF_MACHINE_ID: c_int = 0x19;

/// NPE families // TODO enum
pub const C7200_NPE_FAMILY_MIPS: c_int = 0;
pub const C7200_NPE_FAMILY_PPC: c_int = 1;

/// 4 temperature sensors in a C7200
pub const C7200_TEMP_SENSORS: usize = 4;

#[no_mangle]
pub unsafe extern "C" fn VM_C7200(vm: *mut vm_instance_t) -> *mut c7200_t {
    (*vm).hw_data.cast::<_>()
}

/// Prototype of NPE driver initialization function
pub type c7200_npe_init_fn = Option<unsafe extern "C" fn(router: *mut c7200_t) -> ::std::os::raw::c_int>;

/// C7200 NPE Driver
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c7200_npe_driver {
    pub npe_type: *mut c_char,
    pub npe_family: c_int,
    pub npe_init: c7200_npe_init_fn,
    pub max_ram_size: c_int,
    pub supported: c_int,
    pub nvram_addr: m_uint64_t,
    pub iocard_required: c_int,
    pub clpd6729_pci_bus: c_int,
    pub clpd6729_pci_dev: c_int,
    pub dec21140_pci_bus: c_int,
    pub dec21140_pci_dev: c_int,
}

/// C7200 router
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c7200_router {
    /// Midplane type (standard,VXR) and chassis MAC address
    pub midplane_type: *mut c_char,
    pub midplane_version: c_int,
    pub mac_addr: n_eth_addr_t,
    pub board_id: [c_char; 20],

    /// Associated VM instance
    pub vm: *mut vm_instance_t,

    /// RAM size for npe-400
    pub npe400_ram_size: m_uint32_t,

    /// MV64460 device for NPE-G2
    pub mv64460_sysctr: *mut mv64460_data,

    /// NPE and OIR status
    pub npe_driver: *mut c7200_npe_driver,
    pub oir_status: [m_uint32_t; 2],

    /// Hidden I/O bridge hack to support PCMCIA
    pub io_pci_bridge: *mut pci_bridge,
    pub pcmcia_bus: *mut pci_bus,

    /// PA and Network IRQ registers
    pub pa_status_reg: [m_uint32_t; 2],
    pub pa_ctrl_reg: [m_uint32_t; 2],
    pub net_irq_status: [m_uint32_t; 3],
    pub net_irq_mask: [m_uint32_t; 3],

    /// Temperature sensors
    pub ds1620_sensors: [ds1620_data; C7200_TEMP_SENSORS],

    /// Power supply status
    pub ps_status: u_int,

    /// Midplane EEPROM can be modified to change the chassis MAC address...
    pub cpu_eeprom: cisco_eeprom,
    pub mp_eeprom: cisco_eeprom,
    pub pem_eeprom: cisco_eeprom,

    pub sys_eeprom_g1: nmc93cX6_group, // EEPROMs for CPU and Midplane
    pub sys_eeprom_g2: nmc93cX6_group, // EEPROM for PEM
    pub pa_eeprom_g1: nmc93cX6_group,  // EEPROMs for bays 0, 1, 3, 4
    pub pa_eeprom_g2: nmc93cX6_group,  // EEPROMs for bays 2, 5, 6
    pub pa_eeprom_g3: nmc93cX6_group,  // EEPROM for bay 7
}
