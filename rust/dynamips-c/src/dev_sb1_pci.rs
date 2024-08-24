//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot.  All rights reserved.
//!
//! PCI configuration space for SB-1 processor.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::pci_dev::*;
use crate::vm::*;

const DEBUG_ACCESS: c_int = 0;

/// Sibyte PCI ID
const SB1_PCI_VENDOR_ID: u_int = 0x166D;

/// SB-1 PCI private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sb1_pci_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub pci_bus: *mut pci_bus,

    // PCI configuration (Bus 0, Device 0)
    pub pci_cfg_dev: *mut pci_device,

    // HyperTransport configuration (Bus 0, Device 1)
    pub ht_cfg_dev: *mut pci_device,
}

/// sb1_pci_cfg_read()
///
/// PCI Configuration (Bus 0, Device 0).
unsafe extern "C" fn sb1_pci_cfg_read(_cpu: *mut cpu_gen_t, _dev: *mut pci_device, reg: c_int) -> m_uint32_t {
    #[allow(clippy::needless_return)]
    match reg {
        0x08 => {
            return 0x06000002;
        }
        _ => {
            return 0;
        }
    }
}

/// sb1_ht_cfg_read()
///
/// HyperTransport Configuration (Bus 0, Device 1).
unsafe extern "C" fn sb1_ht_cfg_read(_cpu: *mut cpu_gen_t, _dev: *mut pci_device, reg: c_int) -> m_uint32_t {
    #[allow(clippy::needless_return)]
    match reg {
        0x08 => {
            return 0x06000002;
        }
        0x44 => {
            return 1 << 5; // HyperTransport OK
        }
        _ => {
            return 0;
        }
    }
}

/// dev_sb1_pci_access()
unsafe extern "C" fn dev_sb1_pci_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut sb1_pci_data = (*dev).priv_data.cast::<_>();

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*dev).name, cstr!("read  access to offset = 0x%x, pc = 0x%llx\n"), offset, cpu_get_pc(cpu));
        } else {
            cpu_log!(cpu, (*dev).name, cstr!("write access to vaddr = 0x%x, pc = 0x%llx, val = 0x%llx\n"), offset, cpu_get_pc(cpu), *data);
        }
    }

    if op_type == MTS_READ {
        *data = 0;
    }

    (*(*d).pci_bus).pci_addr = offset;
    pci_dev_data_handler(cpu, (*d).pci_bus, op_type, FALSE, data);
    null_mut()
}

/// Shutdown the PCI bus configuration zone
unsafe extern "C" fn dev_sb1_pci_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut sb1_pci_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

// Create the SB-1 PCI bus configuration zone
#[no_mangle]
pub unsafe extern "C" fn dev_sb1_pci_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t) -> c_int {
    // allocate the private data structure
    let d: *mut sb1_pci_data = libc::malloc(size_of::<sb1_pci_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("SB1_PCI: unable to create device.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<sb1_pci_data>());
    (*d).pci_bus = (*vm).pci_bus[0];

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_sb1_pci_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = 1 << 24;
    (*d).dev.handler = Some(dev_sb1_pci_access);

    // PCI configuration header on Bus 0, Device 0
    (*d).pci_cfg_dev = pci_dev_add((*d).pci_bus, cstr!("sb1_pci_cfg"), SB1_PCI_VENDOR_ID, 0x0001, 0, 0, -1, null_mut(), None, Some(sb1_pci_cfg_read), None);

    // Create the HyperTransport bus #1
    (*vm).pci_bus_pool[28] = pci_bus_create(cstr!("HT bus #1"), -1);

    // HyperTransport configuration header on Bus 0, Device 1
    (*d).ht_cfg_dev = pci_bridge_create_dev((*d).pci_bus, cstr!("sb1_ht_cfg"), SB1_PCI_VENDOR_ID, 0x0002, 1, 0, (*vm).pci_bus_pool[28], Some(sb1_ht_cfg_read), None);

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
