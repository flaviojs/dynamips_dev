//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::device::*;
use crate::pci_dev::*;

// PCI I/O data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_io_data {
    pub dev: vdevice,
    pub dev_list: *mut pci_io_device,
}
