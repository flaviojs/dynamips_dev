//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! DEC21050/DEC21150 PCI bridges.
//! This is just a fake device.

use crate::_private::*;
use crate::pci_dev::*;

const PCI_VENDOR_DEC: u_int = 0x1011;
const PCI_PRODUCT_DEC_21050: u_int = 0x0001;
const PCI_PRODUCT_DEC_21052: u_int = 0x0021;
const PCI_PRODUCT_DEC_21150: u_int = 0x0023;
const PCI_PRODUCT_DEC_21152: u_int = 0x0024;
const PCI_PRODUCT_DEC_21154: u_int = 0x0026;

/// dev_dec21050_init()
#[no_mangle]
pub unsafe extern "C" fn dev_dec21050_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("dec21050"), PCI_VENDOR_DEC, PCI_PRODUCT_DEC_21050, pci_device, 0, sec_bus, None, None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}

/// dev_dec21052_init()
#[no_mangle]
pub unsafe extern "C" fn dev_dec21052_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("dec21052"), PCI_VENDOR_DEC, PCI_PRODUCT_DEC_21052, pci_device, 0, sec_bus, None, None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}

/// dev_dec21150_init()
#[no_mangle]
pub unsafe extern "C" fn dev_dec21150_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("dec21150"), PCI_VENDOR_DEC, PCI_PRODUCT_DEC_21150, pci_device, 0, sec_bus, None, None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}

/// dev_dec21152_init()
#[no_mangle]
pub unsafe extern "C" fn dev_dec21152_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("dec21152"), PCI_VENDOR_DEC, PCI_PRODUCT_DEC_21152, pci_device, 0, sec_bus, None, None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}

/// dev_dec21154_init()
#[no_mangle]
pub unsafe extern "C" fn dev_dec21154_init(pci_bus: *mut pci_bus, pci_device: c_int, sec_bus: *mut pci_bus) -> c_int {
    let dev: *mut pci_device = pci_bridge_create_dev(pci_bus, cstr!("dec21154"), PCI_VENDOR_DEC, PCI_PRODUCT_DEC_21154, pci_device, 0, sec_bus, None, None);
    if !dev.is_null() {
        0
    } else {
        -1
    }
}
