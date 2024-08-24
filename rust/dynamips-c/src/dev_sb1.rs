//! Cisco router simulation platform.
//! Copyright (c) 2005 Christophe Fillot (cf@utc.fr)
//!
//! SB-1 system control devices.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::vm::*;

const DEBUG_UNKNOWN: c_int = 1;

/// SB-1 private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sb1_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,

    /// Virtual machine
    pub vm: *mut vm_instance_t,
}

/// dev_sb1_access()
unsafe extern "C" fn dev_sb1_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut sb1_data = (*dev).priv_data.cast::<_>();
    let _ = d;

    if op_type == MTS_READ {
        *data = 0;
    }

    match offset {
        0x20000 => {
            if op_type == MTS_READ {
                *data = 0x125020FF;
            }
        }

        // Seen on a real NPE-G1 :)
        0x20008 => {
            if op_type == MTS_READ {
                *data = 0x00800000FCDB0700_u64;
            }
        }

        _ => {
            if DEBUG_UNKNOWN != 0 {
                if op_type == MTS_READ {
                    cpu_log!(cpu, cstr!("SB1"), cstr!("read from addr 0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
                } else {
                    cpu_log!(cpu, cstr!("SB1"), cstr!("write to addr 0x%x, value=0x%llx, pc=0x%llx\n"), offset, *data, cpu_get_pc(cpu));
                }
            }
        }
    }

    null_mut()
}

/// Shutdown the SB-1 system control devices
unsafe extern "C" fn dev_sb1_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut sb1_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Create SB-1 system control devices
#[no_mangle]
pub unsafe extern "C" fn dev_sb1_init(vm: *mut vm_instance_t) -> c_int {
    // allocate private data structure
    let d: *mut sb1_data = libc::malloc(size_of::<sb1_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("SB1: out of memory\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<sb1_data>());

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = cstr!("sb1_sysctrl");
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_sb1_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = cstr!("sb1_sysctrl");
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = 0x10000000_u64;
    (*d).dev.phys_len = 0x60000;
    (*d).dev.handler = Some(dev_sb1_access);

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
