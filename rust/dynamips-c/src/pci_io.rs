//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//!
//! PCI I/O space.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::pci_dev::*;
use crate::vm::*;

/// PCI I/O data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_io_data {
    pub dev: vdevice,
    pub dev_list: *mut pci_io_device,
}

/// Debugging flags
const DEBUG_ACCESS: u_int = 0;

/// Add a new PCI I/O device
#[no_mangle]
pub unsafe extern "C" fn pci_io_add(d: *mut pci_io_data, start: m_uint32_t, end: m_uint32_t, dev: *mut vdevice, handler: dev_handler_t) -> *mut pci_io_device {
    let p: *mut pci_io_device = libc::malloc(size_of::<pci_io_device>()).cast::<_>();
    if p.is_null() {
        libc::fprintf(c_stderr(), cstr!("pci_io_add: unable to create a new device.\n"));
        return null_mut();
    }

    (*p).start = start;
    (*p).end = end;
    (*p).real_dev = dev;
    (*p).handler = handler;

    (*p).next = (*d).dev_list;
    (*p).pprev = addr_of_mut!((*d).dev_list);

    if !(*d).dev_list.is_null() {
        (*(*d).dev_list).pprev = addr_of_mut!((*p).next);
    }

    (*d).dev_list = p;
    p
}

/// Remove a PCI I/O device
#[no_mangle]
pub unsafe extern "C" fn pci_io_remove(dev: *mut pci_io_device) {
    if !dev.is_null() {
        if !(*dev).next.is_null() {
            (*(*dev).next).pprev = (*dev).pprev;
        }

        *((*dev).pprev) = (*dev).next;
        libc::free(dev.cast::<_>());
    }
}

/// pci_io_access()
unsafe extern "C" fn pci_io_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut pci_io_data = (*dev).priv_data.cast::<_>();
    let mut p: *mut pci_io_device;

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, cstr!("PCI_IO"), cstr!("read request at pc=0x%llx, offset=0x%x\n"), cpu_get_pc(cpu), offset);
        } else {
            cpu_log!(cpu, cstr!("PCI_IO"), cstr!("write request (data=0x%llx) at pc=0x%llx, offset=0x%x\n"), *data, cpu_get_pc(cpu), offset);
        }
    }

    if op_type == MTS_READ {
        *data = 0;
    }

    p = (*d).dev_list;
    while !p.is_null() {
        if (offset >= (*p).start) && (offset <= (*p).end) {
            return (*p).handler.unwrap()(cpu, (*p).real_dev, offset - (*p).start, op_size, op_type, data);
        }
        p = (*p).next;
    }

    null_mut()
}

/// Remove PCI I/O space
#[no_mangle]
pub unsafe extern "C" fn pci_io_data_remove(vm: *mut vm_instance_t, d: *mut pci_io_data) {
    if !d.is_null() {
        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Free the structure itself
        libc::free(d.cast::<_>());
    }
}

/// Initialize PCI I/O space
#[no_mangle]
pub unsafe extern "C" fn pci_io_data_init(vm: *mut vm_instance_t, paddr: m_uint64_t) -> *mut pci_io_data {
    // Allocate the PCI I/O data structure
    let d: *mut pci_io_data = libc::malloc(size_of::<pci_io_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("PCI_IO: out of memory\n"));
        return null_mut();
    }

    libc::memset(d.cast::<_>(), 0, size_of::<pci_io_data>());
    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = cstr!("pci_io");
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = 2 * 1048576;
    (*d).dev.handler = Some(pci_io_access);

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    d
}
