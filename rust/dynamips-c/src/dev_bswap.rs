//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! Byte-swapping device.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::utils::*;
use crate::vm::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bswap_data {
    /// VM object info
    pub vm_obj: vm_obj_t,

    /// VM instance
    pub vm: *mut vm_instance_t,

    /// Byte-swap device
    pub dev: vdevice,

    /// Physical address base for rewrite
    pub phys_base: m_uint64_t,
}

/// Byte swapped access.
unsafe extern "C" fn dev_bswap_access(_cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut bswap_data = (*dev).priv_data.cast::<_>();

    let paddr: m_uint64_t = (*d).phys_base + offset as m_uint64_t;

    match op_size {
        1 => {
            if op_type == MTS_READ {
                *data = physmem_copy_u8_from_vm((*d).vm, paddr ^ 0x03) as m_uint64_t;
            } else {
                physmem_copy_u8_to_vm((*d).vm, paddr ^ 0x03, *data as m_uint8_t);
            }
        }

        2 => {
            if op_type == MTS_READ {
                *data = swap16(physmem_copy_u16_from_vm((*d).vm, paddr ^ 0x02)) as m_uint64_t;
            } else {
                physmem_copy_u16_to_vm((*d).vm, paddr ^ 0x02, swap16(*data as m_uint16_t));
            }
        }

        4 => {
            if op_type == MTS_READ {
                *data = swap32(physmem_copy_u32_from_vm((*d).vm, paddr)) as m_uint64_t;
            } else {
                physmem_copy_u32_to_vm((*d).vm, paddr, swap32(*data as m_uint32_t));
            }
        }

        _ => {}
    }

    null_mut()
}

/// Shutdown an byte-swap device
unsafe extern "C" fn dev_bswap_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut bswap_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the alias, the byte-swapped and the main device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Initialized a byte-swap device
#[no_mangle]
pub unsafe extern "C" fn dev_bswap_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t, remap_addr: m_uint64_t) -> c_int {
    let d: *mut bswap_data = libc::malloc(size_of::<bswap_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("BSWAP: unable to create device.\n"));
        return -1;
    }

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm = vm;
    (*d).phys_base = remap_addr;
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_bswap_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_bswap_access);
    (*d).dev.priv_data = d.cast::<_>();

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
