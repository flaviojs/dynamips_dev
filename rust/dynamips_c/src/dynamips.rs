//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Many thanks to Nicolas Szalay for his patch
//! for the command line parsing and virtual machine
//! settings (RAM, ROM, NVRAM, ...)

use crate::_private::*;

/// Software version tag
#[no_mangle]
pub static mut sw_version_tag: *const c_char = cstr!("2023010200");
