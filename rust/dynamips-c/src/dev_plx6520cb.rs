//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! PLX6520CB PCI bridge.
//! This is just a fake device.

use crate::_private::*;
use crate::pci_dev::*;

const PCI_VENDOR_PLX: u_int = 0x10b5;
const PCI_PRODUCT_PLX_6520CB: u_int = 0x6520;

/// dev_plx6520cb_init()
#[no_mangle]
pub unsafe extern "C" fn dev_plx6520cb_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("plx6520cb"), PCI_VENDOR_PLX, PCI_PRODUCT_PLX_6520CB, pci_device, 0, sec_bus, None, None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}
