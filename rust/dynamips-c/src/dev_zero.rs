//! Cisco router simulation platform.
//! Copyright (C) 2005,2006 Christophe Fillot.  All rights reserved.
//!
//! Zeroed memory zone.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::vm::*;

/// Zero zone private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct zero_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
}

/// dev_zero_access()
unsafe extern "C" fn dev_zero_access(_cpu: *mut cpu_gen_t, _dev: *mut vdevice, _offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    if op_type == MTS_READ {
        *data = 0;
    }

    null_mut()
}

/// Shutdown a zeroed memory zone
unsafe extern "C" fn dev_zero_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut zero_data = d.cast::<_>();
    if !d.is_null() {
        dev_remove(vm, addr_of_mut!((*d).dev));
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Initialized a zeroed memory zone
#[no_mangle]
pub unsafe extern "C" fn dev_zero_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t) -> c_int {
    let d: *mut zero_data = libc::malloc(size_of::<zero_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("ZERO: unable to create device.\n"));
        return -1;
    }

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_zero_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_zero_access);

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
