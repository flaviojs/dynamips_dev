//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Virtual device.

// TODO remove
#[no_mangle]
pub extern "C" fn _export_device(_: *mut vdevice) {}

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips_common::*;

/// Device Flags
pub const VDEVICE_FLAG_NO_MTS_MMAP: c_int = 0x01; // Prevent MMAPed access by MTS
pub const VDEVICE_FLAG_CACHING: c_int = 0x02; // Device does support caching
pub const VDEVICE_FLAG_REMAP: c_int = 0x04; // Physical address remapping
pub const VDEVICE_FLAG_SYNC: c_int = 0x08; // Forced sync
pub const VDEVICE_FLAG_SPARSE: c_int = 0x10; // Sparse device
pub const VDEVICE_FLAG_GHOST: c_int = 0x20; // Ghost device

pub const VDEVICE_PTE_DIRTY: usize = 0x01;

pub type dev_handler_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void>;

/// Virtual Device
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
