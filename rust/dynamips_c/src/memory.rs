//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Memory.

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips_common::*;
use crate::vm::*;

extern "C" {
    pub fn memlog_dump(cpu: *mut cpu_gen_t);
    pub fn physmem_copy_from_vm(vm: *mut vm_instance_t, real_buffer: *mut c_void, paddr: m_uint64_t, len: size_t);
    pub fn physmem_copy_to_vm(vm: *mut vm_instance_t, real_buffer: *mut c_void, paddr: m_uint64_t, len: size_t);
    pub fn physmem_copy_u32_from_vm(vm: *mut vm_instance_t, paddr: m_uint64_t) -> m_uint32_t;
    pub fn physmem_dma_transfer(vm: *mut vm_instance_t, src: m_uint64_t, dst: m_uint64_t, len: size_t);
    pub fn physmem_copy_u32_to_vm(vm: *mut vm_instance_t, paddr: m_uint64_t, val: m_uint32_t);
}

/// MTS operation
pub const MTS_READ: u_int = 0;
pub const MTS_WRITE: u_int = 1;

/// Memory access flags
pub const MTS_ACC_AE: u_int = 0x00000002; // Address Error
pub const MTS_ACC_T: u_int = 0x00000004; // TLB Exception
pub const MTS_ACC_U: u_int = 0x00000006; // Unexistent
