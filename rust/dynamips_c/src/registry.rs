//! Object Registry.

use crate::hash::*;
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

const DEBUG_REGISTRY: bool = false;

#[no_mangle]
pub static mut registry: *mut registry_t = null_mut(); // TODO private

unsafe fn REGISTRY_LOCK() {
    libc::pthread_mutex_lock(addr_of_mut!((*registry).lock));
}
unsafe fn REGISTRY_UNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!((*registry).lock));
}

/// Insert a new entry
unsafe fn registry_insert_entry(entry: *mut registry_entry_t) {
    // insert new entry in hash table for names
    let h_index: usize = str_hash((*entry).name.cast::<_>()) as usize % (*registry).ht_name_entries as usize;
    let mut bucket: *mut registry_entry_t = (*registry).ht_names.add(h_index);

    (*entry).hname_next = (*bucket).hname_next;
    (*entry).hname_prev = bucket;
    (*(*bucket).hname_next).hname_prev = entry;
    (*bucket).hname_next = entry;

    // insert new entry in hash table for object types
    bucket = (*registry).ht_types.offset((*entry).object_type as isize);

    (*entry).htype_next = (*bucket).htype_next;
    (*entry).htype_prev = bucket;
    (*(*bucket).htype_next).htype_prev = entry;
    (*bucket).htype_next = entry;
}

/// Detach a registry entry
unsafe fn registry_detach_entry(entry: *mut registry_entry_t) {
    (*(*entry).hname_prev).hname_next = (*entry).hname_next;
    (*(*entry).hname_next).hname_prev = (*entry).hname_prev;

    (*(*entry).htype_prev).htype_next = (*entry).htype_next;
    (*(*entry).htype_next).htype_prev = (*entry).htype_prev;
}

/// Remove a registry entry
unsafe fn registry_remove_entry(entry: *mut registry_entry_t) {
    registry_detach_entry(entry);

    mp_free(entry.cast::<_>());
}

/// Locate an entry
unsafe fn registry_find_entry(name: *mut c_char, object_type: c_int) -> *mut registry_entry_t {
    let h_index: usize = str_hash(name.cast::<_>()) as usize % (*registry).ht_name_entries as usize;
    let bucket: *mut registry_entry_t = (*registry).ht_names.add(h_index);

    let mut entry: *mut registry_entry_t = (*bucket).hname_next;
    while entry != bucket {
        if libc::strcmp((*entry).name, name) == 0 && (*entry).object_type == object_type {
            return entry;
        }
        entry = (*entry).hname_next;
    }

    null_mut()
}

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

/// Add a new entry to the registry
#[no_mangle]
pub unsafe extern "C" fn registry_add(name: *mut c_char, object_type: c_int, data: *mut c_void) -> c_int {
    if name.is_null() {
        return -1;
    }

    REGISTRY_LOCK();

    // check if we have already a reference for this name
    let mut entry: *mut registry_entry_t = registry_find_entry(name, object_type);
    if !entry.is_null() {
        REGISTRY_UNLOCK();
        return -1;
    }

    // create a new entry
    entry = mp_alloc(addr_of_mut!((*registry).mp), size_of::<registry_entry_t>()).cast::<_>();
    if entry.is_null() {
        REGISTRY_UNLOCK();
        return -1;
    }

    (*entry).name = name;
    (*entry).data = data;
    (*entry).object_type = object_type;
    (*entry).ref_count = 1; // consider object is referenced by the caller
    registry_insert_entry(entry);

    if DEBUG_REGISTRY {
        libc::printf(cstr!("Registry: object %s: ref_count = %d after add.\n"), (*entry).name, (*entry).ref_count);
    }

    REGISTRY_UNLOCK();
    0
}

/// Delete an entry from the registry
#[no_mangle]
pub unsafe extern "C" fn registry_delete(name: *mut c_char, object_type: c_int) -> c_int {
    if name.is_null() {
        return -1;
    }

    REGISTRY_LOCK();

    let entry: *mut registry_entry_t = registry_find_entry(name, object_type);
    if entry.is_null() {
        REGISTRY_UNLOCK();
        return -1;
    }

    // if the entry is referenced, just decrement ref counter
    (*entry).ref_count -= 1;
    if (*entry).ref_count > 0 {
        if DEBUG_REGISTRY {
            libc::printf(cstr!("Registry: object %s: ref_count = %d after delete.\n"), (*entry).name, (*entry).ref_count);
        }
        REGISTRY_UNLOCK();
        return 0;
    }

    registry_remove_entry(entry);
    REGISTRY_UNLOCK();
    0
}

/// Rename an entry in the registry
#[no_mangle]
pub unsafe extern "C" fn registry_rename(name: *mut c_char, newname: *mut c_char, object_type: c_int) -> c_int {
    if name.is_null() || newname.is_null() {
        return -1;
    }

    REGISTRY_LOCK();

    let entry: *mut registry_entry_t = registry_find_entry(name, object_type);
    if entry.is_null() {
        REGISTRY_UNLOCK();
        return -1;
    }

    if !registry_find_entry(newname, object_type).is_null() {
        REGISTRY_UNLOCK();
        return -1;
    }

    registry_detach_entry(entry);
    (*entry).name = newname;
    registry_insert_entry(entry);

    REGISTRY_UNLOCK();
    0
}

/// Find an entry (increment the reference count)
#[no_mangle]
pub unsafe extern "C" fn registry_find(name: *mut c_char, object_type: c_int) -> *mut c_void {
    if name.is_null() {
        return null_mut();
    }

    REGISTRY_LOCK();

    let entry: *mut registry_entry_t = registry_find_entry(name, object_type);
    let data: *mut c_void = if !entry.is_null() {
        (*entry).ref_count += 1;
        if DEBUG_REGISTRY {
            libc::printf(cstr!("Registry: object %s: ref_count = %d after find.\n"), (*entry).name, (*entry).ref_count);
        }
        (*entry).data
    } else {
        null_mut()
    };

    REGISTRY_UNLOCK();
    data
}

#[no_mangle]
pub extern "C" fn _export(_: *mut registry_entry_t, _: *mut registry_t, _: registry_foreach, _: registry_exec) {}
