//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::_extra::*;
use crate::cpu::*;
use crate::dynamips::*;
use crate::dynamips_common::*;
use std::arch::asm;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;
use std::ptr::null_mut;

// Device Flags
pub const VDEVICE_FLAG_NO_MTS_MMAP: c_int = 0x01; // Prevent MMAPed access by MTS
pub const VDEVICE_FLAG_CACHING: c_int = 0x02; // Device does support caching
pub const VDEVICE_FLAG_REMAP: c_int = 0x04; // Physical address remapping
pub const VDEVICE_FLAG_SYNC: c_int = 0x08; // Forced sync
pub const VDEVICE_FLAG_SPARSE: c_int = 0x10; // Sparse device
pub const VDEVICE_FLAG_GHOST: c_int = 0x20; // Ghost device

pub const VDEVICE_PTE_DIRTY: m_iptr_t = 0x01;

pub type dev_handler_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void>;

// Virtual Device
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vdevice {
    pub name: *mut c_char,
    pub id: u_int,
    pub phys_addr: m_uint64_t,
    pub phys_len: m_uint32_t,
    pub host_addr: m_iptr_t,
    pub priv_data: *mut c_void,
    pub flags: c_int,
    pub fd: c_int,
    pub handler: dev_handler_t,
    pub sparse_map: *mut m_iptr_t,
    pub next: *mut vdevice,
    pub pprev: *mut *mut vdevice,
}

// device access function
#[cfg(MAC64HACK)]
unsafe fn __dev_access_fast(cpu: *mut cpu_gen_t, dev_id: u_int, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let dev: *mut vdevice = (*(*cpu).vm).dev_array[dev_id as usize];

    if likely_stable::unlikely(dev.is_null()) {
        cpu_log(cpu, c"dev_access_fast".as_ptr().cast_mut(), c"null handler (dev_id=%u,offset=0x%x)\n".as_ptr().cast_mut(), dev_id, offset);
        return null_mut();
    }

    if DEBUG_DEV_PERF_CNT != 0 {
        (*cpu).dev_access_counter += 1;
    }

    (*dev).handler.unwrap()(cpu, dev, offset, op_size, op_type, data)
}

#[cfg(MAC64HACK)]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn dev_access_fast(cpu: *mut cpu_gen_t, dev_id: u_int, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    asm!("sub rsp, 8");
    let ret: *mut c_void = __dev_access_fast(cpu, dev_id, offset, op_size, op_type, data);
    asm!("add rsp, 8");
    ret
}

#[cfg(not(MAC64HACK))]
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn dev_access_fast(cpu: *mut cpu_gen_t, dev_id: u_int, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let dev: *mut vdevice = (*(*cpu).vm).dev_array[dev_id as usize];

    if likely_stable::unlikely(dev.is_null()) {
        cpu_log(cpu, c"dev_access_fast".as_ptr().cast_mut(), c"null handler (dev_id=%u,offset=0x%x)\n".as_ptr().cast_mut(), dev_id, offset);
        return null_mut();
    }

    if DEBUG_DEV_PERF_CNT != 0 {
        (*cpu).dev_access_counter += 1;
    }

    (*dev).handler.unwrap()(cpu, dev, offset, op_size, op_type, data)
}
