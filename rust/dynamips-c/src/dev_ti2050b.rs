//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Texas Instruments PCI205B PCI bridge.

use crate::_private::*;
use crate::pci_dev::*;

const PCI_VENDOR_TI: u_int = 0x104C;
const PCI_PRODUCT_PCI2050B: u_int = 0xAC28;

/// dev_ti2050b_init()
#[no_mangle]
pub unsafe extern "C" fn dev_ti2050b_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("ti2050b"), PCI_VENDOR_TI, PCI_PRODUCT_PCI2050B, pci_device, 0, sec_bus, None, None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}
