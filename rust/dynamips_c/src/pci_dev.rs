//! PCI devices.
//!
//! Very interesting docs:
//!   http://www.science.unitn.it/~fiorella/guidelinux/tlk/node72.html
//!   http://www.science.unitn.it/~fiorella/guidelinux/tlk/node76.html

/// cbindgen:no-export
#[repr(C)]
pub struct pci_bus {
    _todo: u8,
}
