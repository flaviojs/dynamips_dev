//! Generic Cisco card routines and definitions.

use crate::prelude::*;
use crate::vm::*;

extern "C" {
    pub fn vm_slot_show_all_info(vm: *mut vm_instance_t) -> c_int;
}

/// cbindgen:no-export
#[repr(C)]
pub struct cisco_card_driver {
    _todo: u8,
}

/// cbindgen:no-export
#[repr(C)]
pub struct cisco_card {
    _todo: u8,
}
