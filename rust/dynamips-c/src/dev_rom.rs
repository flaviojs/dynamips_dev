//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot.  All rights reserved.
//!
//! ROM Emulation.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::vm::*;

/// Embedded MIPS64 ROM
#[no_mangle]
#[cfg(not(feature = "USE_UNSTABLE"))]
pub static mut mips64_microcode: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/mips64_microcode_dump_stable")).len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/mips64_microcode_dump_stable"));
#[no_mangle]
#[cfg(feature = "USE_UNSTABLE")]
pub static mut mips64_microcode: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/mips64_microcode_dump_unstable")).len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/mips64_microcode_dump_unstable"));

#[no_mangle]
pub static mut mips64_microcode_len: ssize_t = unsafe { mips64_microcode.len() as ssize_t };

/// Embedded PPC32 ROM
#[no_mangle]
#[cfg(not(feature = "USE_UNSTABLE"))]
pub static mut ppc32_microcode: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/ppc32_microcode_dump_stable")).len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/ppc32_microcode_dump_stable"));
#[no_mangle]
#[cfg(feature = "USE_UNSTABLE")]
pub static mut ppc32_microcode: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/ppc32_microcode_dump_unstable")).len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/ppc32_microcode_dump_unstable"));

#[no_mangle]
pub static mut ppc32_microcode_len: ssize_t = unsafe { ppc32_microcode.len() as ssize_t };

/// ROM private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rom_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub rom_ptr: *mut m_uint8_t,
    pub rom_size: m_uint32_t,
}

/// dev_rom_access()
unsafe extern "C" fn dev_rom_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut rom_data = (*dev).priv_data.cast::<_>();

    if op_type == MTS_WRITE {
        cpu_log!(cpu, cstr!("ROM"), cstr!("write attempt at address 0x%llx (data=0x%llx)\n"), (*dev).phys_addr + offset as m_uint64_t, *data);
        return null_mut();
    }

    if offset >= (*d).rom_size {
        *data = 0;
        return null_mut();
    }

    (*d).rom_ptr.add(offset as usize).cast::<_>()
}

/// Shutdown a ROM device
unsafe extern "C" fn dev_rom_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut rom_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Initialize a ROM zone
#[no_mangle]
pub unsafe extern "C" fn dev_rom_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t, rom_data: *mut m_uint8_t, rom_data_size: ssize_t) -> c_int {
    // allocate the private data structure
    let d: *mut rom_data = libc::malloc(size_of::<rom_data>()).cast::<_>();
    if !d.is_null() {
        libc::fprintf(c_stderr(), cstr!("ROM: unable to create device.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<rom_data>());
    (*d).rom_ptr = rom_data;
    (*d).rom_size = rom_data_size as m_uint32_t;

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_rom_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.flags = VDEVICE_FLAG_CACHING;
    (*d).dev.handler = Some(dev_rom_access);

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
