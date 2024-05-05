//! Object Registry.

use crate::mempool::*;
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

/// Registry info
pub type registry_t = registry;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct registry {
    pub lock: libc::pthread_mutex_t,
    pub mp: mempool,
    pub ht_name_entries: c_int,
    pub ht_type_entries: c_int,
    pub ht_names: *mut registry_entry_t,
    pub ht_types: *mut registry_entry_t,
}

/// Registry "foreach" callback
pub type registry_foreach = Option<unsafe extern "C" fn(entry: *mut registry_entry_t, opt_arg: *mut c_void, err: *mut c_int)>;

/// Registry "exec" callback
pub type registry_exec = Option<unsafe extern "C" fn(data: *mut c_void, opt_arg: *mut c_void) -> c_int>;

#[no_mangle]
pub static mut registry: *mut registry_t = null_mut(); // TODO private

/// Terminate the registry
extern "C" fn registry_terminate() {
    unsafe {
        mp_free((*registry).ht_types.cast::<_>());
        mp_free((*registry).ht_names.cast::<_>());
        mp_free_pool(addr_of_mut!((*registry).mp));
        libc::pthread_mutex_destroy(addr_of_mut!((*registry).lock));
        libc::free(registry.cast::<_>());
        registry = null_mut();
    }
}

/// Initialize registry
#[no_mangle]
pub unsafe extern "C" fn registry_init() -> c_int {
    registry = libc::malloc(size_of::<registry_t>()).cast::<_>();
    assert!(!registry.is_null());

    let mut attr: libc::pthread_mutexattr_t = zeroed::<_>();
    libc::pthread_mutexattr_init(addr_of_mut!(attr));
    libc::pthread_mutexattr_settype(addr_of_mut!(attr), libc::PTHREAD_MUTEX_RECURSIVE);
    libc::pthread_mutex_init(addr_of_mut!((*registry).lock), addr_of_mut!(attr));
    libc::pthread_mutexattr_destroy(addr_of_mut!(attr));

    // initialize registry memory pool
    mp_create_fixed_pool(addr_of_mut!((*registry).mp), cstr!("registry"));

    (*registry).ht_name_entries = REGISTRY_HT_NAME_ENTRIES;
    (*registry).ht_type_entries = REGISTRY_MAX_TYPES;

    // initialize hash table for names, with sentinels
    let len: size_t = (*registry).ht_name_entries as usize * size_of::<registry_entry_t>();
    (*registry).ht_names = mp_alloc(addr_of_mut!((*registry).mp), len).cast::<_>();
    assert!(!(*registry).ht_names.is_null());

    for i in 0..(*registry).ht_name_entries as isize {
        let p: *mut registry_entry_t = (*registry).ht_names.offset(i);
        (*p).hname_next = p;
        (*p).hname_prev = p;
    }

    // initialize hash table for types, with sentinels
    let len: size_t = (*registry).ht_type_entries as usize * size_of::<registry_entry_t>();
    (*registry).ht_types = mp_alloc(addr_of_mut!((*registry).mp), len).cast::<_>();
    assert!(!(*registry).ht_types.is_null());

    for i in 0..(*registry).ht_type_entries as isize {
        let p: *mut registry_entry_t = (*registry).ht_types.offset(i);
        (*p).htype_next = p;
        (*p).htype_prev = p;
    }

    libc::atexit(registry_terminate);

    0
}

#[no_mangle]
pub extern "C" fn _export(_: *mut registry_entry_t, _: *mut registry_t, _: registry_foreach, _: registry_exec) {}
