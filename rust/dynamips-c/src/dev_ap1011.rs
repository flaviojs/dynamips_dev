//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot.  All rights reserved.
//!
//! AP1011 - Sturgeon HyperTransport-PCI Bridge.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips_common::*;
use crate::pci_dev::*;

const AP1011_PCI_VENDOR_ID: u_int = 0x14D9;
const AP1011_PCI_PRODUCT_ID: u_int = 0x0010;

/// pci_ap1011_read()
///
/// Read a PCI register.
unsafe extern "C" fn pci_ap1011_read(_cpu: *mut cpu_gen_t, _dev: *mut pci_device, reg: c_int) -> m_uint32_t {
    match reg {
        0x08 => 0x06040000,
        0x34 => 0x00000040,
        0x40 => 0x00210008,
        0x44 => 0x00000020,
        0x48 => 0x000000C0,
        _ => 0,
    }
}

/// Create an AP1011 Sturgeon HyperTransport-PCI Bridge
#[no_mangle]
pub unsafe extern "C" fn dev_ap1011_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("ap1011"), AP1011_PCI_VENDOR_ID, AP1011_PCI_PRODUCT_ID, pci_device, 0, sec_bus, Some(pci_ap1011_read), None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}
