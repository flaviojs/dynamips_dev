//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::_extra::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;

pub const PCI_BUS_ADDR: c_int = 0xcf8;
pub const PCI_BUS_DATA: c_int = 0xcfc;

// PCI ID (Vendor + Device) register
pub const PCI_REG_ID: c_int = 0x00;

// PCI Base Address Registers (BAR)
pub const PCI_REG_BAR0: c_int = 0x10;
pub const PCI_REG_BAR1: c_int = 0x14;
pub const PCI_REG_BAR2: c_int = 0x18;
pub const PCI_REG_BAR3: c_int = 0x1c;
pub const PCI_REG_BAR4: c_int = 0x20;
pub const PCI_REG_BAR5: c_int = 0x24;

// Forward declaration for PCI device
pub type pci_dev_t = pci_device;

// PCI function prototypes
pub type pci_init_t = Option<unsafe extern "C" fn(dev: *mut pci_dev_t)>;
pub type pci_reg_read_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, dev: *mut pci_dev_t, reg: c_int) -> m_uint32_t>;
pub type pci_reg_write_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, dev: *mut pci_dev_t, reg: c_int, value: m_uint32_t)>;
// PCI device
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_device {
    pub name: *mut c_char,
    pub vendor_id: u_int,
    pub product_id: u_int,
    pub device: c_int,
    pub function: c_int,
    pub irq: c_int,
    pub priv_data: *mut c_void,

    // Parent bus
    pub pci_bus: *mut pci_bus,

    pub init: pci_init_t,
    pub read_register: pci_reg_read_t,
    pub write_register: pci_reg_write_t,

    pub next: *mut pci_device,
    pub pprev: *mut *mut pci_device,
}

// PCI bus
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_bus {
    pub name: *mut c_char,
    pub pci_addr: m_uint32_t,

    // Bus number
    pub bus: c_int,

    pub dev_list: *mut pci_device,

    // PCI bridges to access other busses
    pub bridge_list: *mut pci_bridge,
}

// PCI bridge
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_bridge {
    pub pri_bus: c_int, // Primary Bus
    pub sec_bus: c_int, // Secondary Bus
    pub sub_bus: c_int, // Subordinate Bus

    pub skip_bus_check: c_int,

    // Bus configuration register
    pub cfg_reg_bus: m_uint32_t,

    // PCI bridge device
    pub pci_dev: *mut pci_device,

    // Secondary PCI bus
    pub pci_bus: *mut pci_bus,

    // Fallback handlers to read/write config registers
    pub fallback_read: pci_reg_read_t,
    pub fallback_write: pci_reg_write_t,

    pub next: *mut pci_bridge,
    pub pprev: *mut *mut pci_bridge,
}

// PCI IO device
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_io_device {
    pub start: m_uint32_t,
    pub end: m_uint32_t,
    pub real_dev: *mut vdevice,
    pub handler: dev_handler_t,
    pub next: *mut pci_io_device,
    pub pprev: *mut *mut pci_io_device,
}
