//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Management of CPU groups (for MP systems).

pub type cpu_gen_t = cpu_gen;
pub type cpu_group_t = cpu_group;

/// cbindgen:no-export
#[repr(C)]
pub struct cpu_gen {
    _todo: u8,
}

/// cbindgen:no-export
#[repr(C)]
pub struct cpu_group {
    _todo: u8,
}
