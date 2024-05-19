//! PCI devices.
//!
//! Very interesting docs:
//!   http://www.science.unitn.it/~fiorella/guidelinux/tlk/node72.html
//!   http://www.science.unitn.it/~fiorella/guidelinux/tlk/node76.html

use crate::vm::*;

extern "C" {
    pub fn dev_show_list(vm: *mut vm_instance_t);
    pub fn pci_dev_show_list(pci_bus: *mut pci_bus);
}

/// cbindgen:no-export
#[repr(C)]
pub struct pci_bus {
    _todo: u8,
}
