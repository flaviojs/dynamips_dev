//! Object Registry.

use crate::prelude::*;

pub const REGISTRY_HT_NAME_ENTRIES: c_int = 1024;
pub const REGISTRY_MAX_TYPES: c_int = 256;

// Object types for Registry // TODO enum
/// Virtual machine
pub const OBJ_TYPE_VM: c_int = 0;
/// Network IO descriptor
pub const OBJ_TYPE_NIO: c_int = 1;
/// Network IO bridge
pub const OBJ_TYPE_NIO_BRIDGE: c_int = 2;
/// Frame-Relay switch
pub const OBJ_TYPE_FRSW: c_int = 3;
/// ATM switch
pub const OBJ_TYPE_ATMSW: c_int = 4;
/// ATM bridge
pub const OBJ_TYPE_ATM_BRIDGE: c_int = 5;
/// Ethernet switch
pub const OBJ_TYPE_ETHSW: c_int = 6;
/// Hypervisor store
pub const OBJ_TYPE_STORE: c_int = 7;

/// Registry entry
pub type registry_entry_t = registry_entry;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct registry_entry {
    pub name: *mut c_char,
    pub data: *mut c_void,
    pub object_type: c_int,
    pub ref_count: c_int,
    /// Hash table for names
    pub hname_next: *mut registry_entry_t,
    pub hname_prev: *mut registry_entry_t,
    /// Hash table for types
    pub htype_next: *mut registry_entry_t,
    pub htype_prev: *mut registry_entry_t,
}

#[no_mangle]
pub extern "C" fn _export(_: *mut registry_entry_t) {}
