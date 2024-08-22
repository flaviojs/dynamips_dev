//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Pericom PCI bridge.

use crate::_private::*;
use crate::pci_dev::*;

const PCI_VENDOR_PERICOM: u_int = 0x12d8;
const PCI_PRODUCT_PERICOM: u_int = 0x8150;

/// dev_pericom_init()
#[no_mangle]
pub unsafe extern "C" fn dev_pericom_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("pericom"), PCI_VENDOR_PERICOM, PCI_PRODUCT_PERICOM, pci_device, 0, sec_bus, None, None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}
