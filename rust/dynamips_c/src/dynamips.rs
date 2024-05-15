//! Cisco router simulation platform.

use crate::prelude::*;

/// Software version tag
#[no_mangle]
pub static mut sw_version_tag: *const c_char = cstr!("2023010200");
