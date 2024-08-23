//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot.  All rights reserved.
//!
//! Cirrus Logic PD6729 PCI-to-PCMCIA host adapter.
//!
//! TODO: finish the code! (especially extended registers)

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::pci_dev::*;
use crate::pci_io::*;
use crate::vm::*;

const DEBUG_ACCESS: c_int = 0;

/// Cirrus Logic PD6729 PCI vendor/product codes
const CLPD6729_PCI_VENDOR_ID: u_int = 0x1013;
const CLPD6729_PCI_PRODUCT_ID: u_int = 0x1100;

const CLPD6729_REG_CHIP_REV: u_int = 0x00; // Chip Revision
const CLPD6729_REG_INT_STATUS: u_int = 0x01; // Interface Status
const CLPD6729_REG_POWER_CTRL: u_int = 0x02; // Power Control
const CLPD6729_REG_INTGEN_CTRL: u_int = 0x03; // Interrupt & General Control
const CLPD6729_REG_CARD_STATUS: u_int = 0x04; // Card Status Change
const CLPD6729_REG_FIFO_CTRL: u_int = 0x17; // FIFO Control
const CLPD6729_REG_EXT_INDEX: u_int = 0x2E; // Extended Index

/// CLPD6729 private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clpd6729_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub pci_dev: *mut pci_device,
    pub pci_io_dev: *mut pci_io_device,

    /// VM objects present in slots (typically, PCMCIA disks...)
    pub slot_obj: [*mut vm_obj_t; 2],

    /// Base registers
    pub base_index: m_uint8_t,
    pub base_regs: [m_uint8_t; 256],
}

/// Handle access to a base register
unsafe fn clpd6729_base_reg_access(cpu: *mut cpu_gen_t, d: *mut clpd6729_data, op_type: u_int, data: *mut m_uint64_t) {
    let slot_id: u_int;
    let reg: u_int;

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, cstr!("CLPD6729"), cstr!("reading reg 0x%2.2x at pc=0x%llx\n"), (*d).base_index, cpu_get_pc(cpu));
        } else {
            cpu_log!(cpu, cstr!("CLPD6729"), cstr!("writing reg 0x%2.2x, data=0x%llx at pc=0x%llx\n"), (*d).base_index, *data, cpu_get_pc(cpu));
        }
    }

    if op_type == MTS_READ {
        *data = 0;
    }

    // Reserved registers
    if (*d).base_index >= 0x80 {
        return;
    }

    // Socket A regs: 0x00 to 0x3f
    // Socket B regs: 0x40 to 0x7f
    if (*d).base_index >= 0x40 {
        slot_id = 1;
        reg = ((*d).base_index - 0x40) as u_int;
    } else {
        slot_id = 0;
        reg = (*d).base_index as u_int;
    }

    match reg {
        CLPD6729_REG_CHIP_REV => {
            if op_type == MTS_READ {
                *data = 0x48;
            }
        }

        CLPD6729_REG_INT_STATUS => {
            if op_type == MTS_READ {
                if !(*d).slot_obj[slot_id as usize].is_null() {
                    *data = 0xEF;
                } else {
                    *data = 0x80;
                }
            }
        }

        CLPD6729_REG_INTGEN_CTRL => {
            if op_type == MTS_READ {
                *data = 0x40;
            }
        }

        CLPD6729_REG_EXT_INDEX => {
            if op_type == MTS_WRITE {
                cpu_log!(cpu, cstr!("CLPD6729"), cstr!("ext reg index 0x%2.2llx at pc=0x%llx\n"), *data, cpu_get_pc(cpu));
            }
        }

        CLPD6729_REG_FIFO_CTRL => {
            if op_type == MTS_READ {
                *data = 0x80; // FIFO is empty
            }
        }

        _ => {
            if op_type == MTS_READ {
                *data = (*d).base_regs[(*d).base_index as usize] as m_uint64_t;
            } else {
                (*d).base_regs[(*d).base_index as usize] = (*data) as m_uint8_t;
            }
        }
    }
}

/// dev_clpd6729_io_access()
unsafe extern "C" fn dev_clpd6729_io_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut clpd6729_data = (*dev).priv_data.cast::<clpd6729_data>();

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*dev).name, cstr!("reading at offset 0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
        } else {
            cpu_log!(cpu, (*dev).name, cstr!("writing at offset 0x%x, pc=0x%llx, data=0x%llx\n"), offset, cpu_get_pc(cpu), *data);
        }
    }

    match offset {
        0 => {
            // Data register
            clpd6729_base_reg_access(cpu, d, op_type, data);
        }

        1 => {
            // Index register
            if op_type == MTS_READ {
                *data = (*d).base_index as m_uint64_t;
            } else {
                (*d).base_index = *data as m_uint8_t;
            }
        }

        _ => {}
    }

    null_mut()
}

/// Shutdown a CLPD6729 device
unsafe extern "C" fn dev_clpd6729_shutdown(_vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut clpd6729_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the PCI device
        pci_dev_remove((*d).pci_dev);

        // Remove the PCI I/O device
        pci_io_remove((*d).pci_io_dev);

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// dev_clpd6729_init()
#[no_mangle]
pub unsafe extern "C" fn dev_clpd6729_init(vm: *mut vm_instance_t, pci_bus: *mut pci_bus, pci_device: c_int, pci_io_data: *mut pci_io_data, io_start: m_uint32_t, io_end: m_uint32_t) -> c_int {
    // Allocate the private data structure
    let d: *mut clpd6729_data = libc::malloc(size_of::<clpd6729_data>()).cast::<_>();
    if !d.is_null() {
        libc::fprintf(c_stderr(), cstr!("CLPD6729: unable to create device.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<clpd6729_data>());
    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = cstr!("clpd6729");
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_clpd6729_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = cstr!("clpd6729");
    (*d).dev.priv_data = d.cast::<_>();

    (*d).pci_io_dev = pci_io_add(pci_io_data, io_start, io_end, addr_of_mut!((*d).dev), Some(dev_clpd6729_io_access));

    (*d).pci_dev = pci_dev_add(pci_bus, cstr!("clpd6729"), CLPD6729_PCI_VENDOR_ID, CLPD6729_PCI_PRODUCT_ID, pci_device, 0, -1, addr_of_mut!((*d).dev).cast::<_>(), None, None, None);

    if (*d).pci_io_dev.is_null() || (*d).pci_dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("CLPD6729: unable to create PCI devices.\n"));
        dev_clpd6729_shutdown(vm, d.cast::<_>());
        return -1;
    }

    vm_object_add(vm, addr_of_mut!((*d).vm_obj));

    if true {
        // PCMCIA disk test
        if (*vm).pcmcia_disk_size[0] != 0 {
            (*d).slot_obj[0] = dev_pcmcia_disk_init(vm, cstr!("disk0"), 0x40000000_u64, 0x200000, (*vm).pcmcia_disk_size[0], 0);
        }

        if (*vm).pcmcia_disk_size[1] != 0 {
            (*d).slot_obj[1] = dev_pcmcia_disk_init(vm, cstr!("disk1"), 0x44000000_u64, 0x200000, (*vm).pcmcia_disk_size[1], 0);
        }
    }

    if false {
        // PCMCIA disk test
        if (*vm).pcmcia_disk_size[0] != 0 {
            (*d).slot_obj[0] = dev_pcmcia_disk_init(vm, cstr!("disk0"), 0xd8000000_u64, 0x200000, (*vm).pcmcia_disk_size[0], 0);
        }

        if (*vm).pcmcia_disk_size[1] != 0 {
            (*d).slot_obj[1] = dev_pcmcia_disk_init(vm, cstr!("disk1"), 0xdc000000_u64, 0x200000, (*vm).pcmcia_disk_size[1], 0);
        }
    }

    0
}
