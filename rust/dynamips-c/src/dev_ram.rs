//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot.  All rights reserved.
//!
//! RAM emulation.

use crate::_private::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::vm::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ram_data {
    pub vm_obj: vm_obj_t,
    pub dev: *mut vdevice,
    pub filename: *mut c_char,
    pub delete_file: c_int,
}

/// Shutdown a RAM device
unsafe extern "C" fn dev_ram_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut ram_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the device
        dev_remove(vm, (*d).dev);
        libc::free((*d).dev.cast::<_>());

        // Remove filename used to virtualize RAM
        if !(*d).filename.is_null() {
            if (*d).delete_file != 0 {
                libc::unlink((*d).filename);
            }
            libc::free((*d).filename.cast::<_>());
        }

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Initialize a RAM zone
#[no_mangle]
pub unsafe extern "C" fn dev_ram_init(vm: *mut vm_instance_t, name: *mut c_char, use_mmap: c_int, delete_file: c_int, alternate_name: *mut c_char, sparse: c_int, paddr: m_uint64_t, len: m_uint32_t) -> c_int {
    // allocate the private data structure
    let d: *mut ram_data = libc::malloc(size_of::<ram_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("RAM: unable to create device.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<ram_data>());
    (*d).delete_file = delete_file;

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_ram_shutdown);

    if use_mmap != 0 {
        if alternate_name.is_null() {
            (*d).filename = vm_build_filename(vm, name);
        } else {
            (*d).filename = libc::strdup(alternate_name);
        }

        if (*d).filename.is_null() {
            libc::fprintf(c_stderr(), cstr!("RAM: unable to create filename.\n"));
            libc::free(d.cast::<_>());
            return -1;
        }
    }

    (*d).dev = dev_create_ram(vm, name, sparse, (*d).filename, paddr, len);
    if (*d).dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("RAM: unable to create device.\n"));
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}

/// Initialize a ghosted RAM zone
#[no_mangle]
pub unsafe extern "C" fn dev_ram_ghost_init(vm: *mut vm_instance_t, name: *mut c_char, sparse: c_int, filename: *mut c_char, paddr: m_uint64_t, len: m_uint32_t) -> c_int {
    if filename.is_null() {
        vm_error!(vm, cstr!("RAM_ghost: unable to create device (filename=%s).\n"), filename);
        return -1;
    }

    // allocate the private data structure
    let d: *mut ram_data = libc::malloc(size_of::<ram_data>()).cast::<_>();
    if d.is_null() {
        vm_error!(vm, cstr!("RAM_ghost: unable to create device (filename=%s).\n"), filename);
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<ram_data>());
    (*d).delete_file = FALSE;

    (*d).filename = libc::strdup(filename);
    if (*d).filename.is_null() {
        libc::free(d.cast::<_>());
        return -1;
    }

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_ram_shutdown);

    (*d).dev = dev_create_ghost_ram(vm, name, sparse, (*d).filename, paddr, len);
    if (*d).dev.is_null() {
        vm_error!(vm, cstr!("RAM_ghost: unable to create device (filename=%s)\n"), (*d).filename);
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
