//! Rust code that is available in C.
//!
//! cbindgen will parse this module.

use crate::_private::*;
use std::ptr::read_volatile;
use std::ptr::write_volatile;

/// Make sure cbindgen exports the types it needs.
#[rustfmt::skip]
#[no_mangle]
pub extern "C" fn _export(
    _: *mut crate::device::vdevice,
    _: *mut crate::dynamips_common::m_int16_t,
    _: *mut crate::dynamips::symbol,
    _: *mut crate::mips64_jit::mips64_insn_jump,
    _: *mut crate::mips64_jit::mips64_insn_tag,
    _: *mut crate::net::n_eth_dot1q_hdr_t,
    _: *mut crate::net::n_eth_hdr_t,
    _: *mut crate::net::n_eth_isl_hdr_t,
    _: *mut crate::pci_dev::pci_io_device,
    _: *mut crate::ppc32_jit::ppc32_insn_patch,
    _: *mut crate::ppc32_jit::ppc32_insn_tag,
    _: *mut crate::ppc32_jit::ppc32_jit_patch_table,
    _: *mut crate::ppc32_jit::ppc32_jit_tcb_t,
    _: *mut crate::ppc32::ppc32_bat_prog,
    _: *mut crate::timer::timer_entry_t,
    _: *mut crate::utils::insn_tblock_fptr,
    _: *mut crate::utils::mts_map_t,
    _: *mut crate::utils::ppc_insn_t,
    _: *mut crate::vm::vm_ghost_image_t,
    _: *mut crate::vm::vm_obj_t,
    _: *mut crate::vm::vm_platform_list,
) {}
#[cfg(feature = "USE_UNSTABLE")]
#[rustfmt::skip]
#[no_mangle]
pub extern "C" fn _export_unstable(
    _: *mut crate::tcb::tsg_stats,
) {}

// Non-standard unsigned integers
pub type u_char = c_uchar;
pub type u_int = c_uint;
pub type u_long = c_ulong;
pub type u_short = c_ushort;

/// Wrapper around a volatile type.
/// cbindgen:no-export
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Volatile<T>(pub T);
impl<T> Volatile<T> {
    pub fn get(&self) -> T {
        // SAFETY the pointer is valid if self is valid
        unsafe { read_volatile(addr_of!(self.0)) }
    }
    pub fn set(&mut self, x: T) {
        // SAFETY the pointer is valid if self is valid
        unsafe { write_volatile(addr_of_mut!(self.0), x) }
    }
}
