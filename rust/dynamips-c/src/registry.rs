//! IPFlow Collector
//! Copyright (c) 2003 Christophe Fillot.
//! E-mail: cf@utc.fr
//!
//! Object Registry.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::hash::*;
use crate::mempool::*;

pub type registry_entry_t = registry_entry;
pub type registry_t = registry;

pub const REGISTRY_HT_NAME_ENTRIES: c_int = 1024;
pub const REGISTRY_MAX_TYPES: c_int = 256;

/// Object types for Registry // TODO enum
pub const OBJ_TYPE_VM: c_int = 0; // Virtual machine
pub const OBJ_TYPE_NIO: c_int = 1; // Network IO descriptor
pub const OBJ_TYPE_NIO_BRIDGE: c_int = 2; // Network IO bridge
pub const OBJ_TYPE_FRSW: c_int = 3; // Frame-Relay switch
pub const OBJ_TYPE_ATMSW: c_int = 4; // ATM switch
pub const OBJ_TYPE_ATM_BRIDGE: c_int = 5; // ATM bridge
pub const OBJ_TYPE_ETHSW: c_int = 6; // Ethernet switch
pub const OBJ_TYPE_STORE: c_int = 7; // Hypervisor store

/// Registry entry
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct registry_entry {
    pub name: *mut c_char,
    pub data: *mut c_void,
    pub object_type: c_int,
    pub ref_count: c_int,
    pub hname_next: *mut registry_entry_t,
    pub hname_prev: *mut registry_entry_t,
    pub htype_next: *mut registry_entry_t,
    pub htype_prev: *mut registry_entry_t,
}

/// Registry info
#[repr(C)]
#[derive(Copy, Clone)]
pub struct registry {
    pub lock: libc::pthread_mutex_t,
    pub mp: mempool_t,
    pub ht_name_entries: c_int,
    pub ht_type_entries: c_int,
    pub ht_names: *mut registry_entry_t, // Hash table for names
    pub ht_types: *mut registry_entry_t, // Hash table for types
}

/// Registry "foreach" callback
pub type registry_foreach = Option<unsafe extern "C" fn(entry: *mut registry_entry_t, opt_arg: *mut c_void, err: *mut c_int)>;

/// Registry "exec" callback
pub type registry_exec = Option<unsafe extern "C" fn(data: *mut c_void, opt_arg: *mut c_void) -> c_int>;

const DEBUG_REGISTRY: c_int = 0;

static mut registry: *mut registry_t = null_mut();

unsafe fn REGISTRY_LOCK() {
    libc::pthread_mutex_lock(addr_of_mut!((*registry).lock));
}
unsafe fn REGISTRY_UNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!((*registry).lock));
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
    let mut attr: libc::pthread_mutexattr_t = zeroed::<_>();
    let mut len: size_t;

    registry = libc::malloc(size_of::<registry_t>()).cast::<_>();
    assert!(!registry.is_null());

    libc::pthread_mutexattr_init(addr_of_mut!(attr));
    libc::pthread_mutexattr_settype(addr_of_mut!(attr), libc::PTHREAD_MUTEX_RECURSIVE);
    libc::pthread_mutex_init(addr_of_mut!((*registry).lock), addr_of_mut!(attr));
    libc::pthread_mutexattr_destroy(addr_of_mut!(attr));

    // initialize registry memory pool
    mp_create_fixed_pool(addr_of_mut!((*registry).mp), cstr!("registry"));

    (*registry).ht_name_entries = REGISTRY_HT_NAME_ENTRIES;
    (*registry).ht_type_entries = REGISTRY_MAX_TYPES;

    // initialize hash table for names, with sentinels
    len = (*registry).ht_name_entries as size_t * size_of::<registry_entry_t>();
    (*registry).ht_names = mp_alloc(addr_of_mut!((*registry).mp), len).cast::<_>();
    assert!(!(*registry).ht_names.is_null());

    for i in 0..(*registry).ht_name_entries {
        let p: *mut registry_entry_t = (*registry).ht_names.offset(i as isize);
        (*p).hname_next = p;
        (*p).hname_prev = p;
    }

    // initialize hash table for types, with sentinels
    len = (*registry).ht_type_entries as size_t * size_of::<registry_entry_t>();
    (*registry).ht_types = mp_alloc(addr_of_mut!((*registry).mp), len).cast::<_>();
    assert!(!(*registry).ht_types.is_null());

    for i in 0..(*registry).ht_type_entries {
        let p: *mut registry_entry_t = (*registry).ht_types.offset(i as isize);
        (*p).htype_next = p;
        (*p).htype_prev = p;
    }

    libc::atexit(registry_terminate);

    0
}

/// Insert a new entry
unsafe fn registry_insert_entry(entry: *mut registry_entry_t) {
    // insert new entry in hash table for names
    let h_index: u_int = str_hash((*entry).name.cast::<_>()) % (*registry).ht_name_entries as u_int;
    let mut bucket: *mut registry_entry_t = (*registry).ht_names.add(h_index as usize);

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
#[inline]
unsafe fn registry_find_entry(name: *mut c_char, object_type: c_int) -> *mut registry_entry_t {
    let h_index: u_int = str_hash(name.cast::<_>()) % (*registry).ht_name_entries as u_int;
    let bucket: *mut registry_entry_t = (*registry).ht_names.add(h_index as usize);

    let mut entry: *mut registry_entry_t = (*bucket).hname_next;
    while entry != bucket {
        if libc::strcmp((*entry).name, name) == 0 && (*entry).object_type == object_type {
            return entry;
        }
        entry = (*entry).hname_next;
    }

    null_mut()
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

    if DEBUG_REGISTRY != 0 {
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
        if DEBUG_REGISTRY != 0 {
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
        if DEBUG_REGISTRY != 0 {
            libc::printf(cstr!("Registry: object %s: ref_count = %d after find.\n"), (*entry).name, (*entry).ref_count);
        }
        (*entry).data
    } else {
        null_mut()
    };

    REGISTRY_UNLOCK();
    data
}

/// Check if entry exists (does not change reference count)
#[no_mangle]
pub unsafe extern "C" fn registry_exists(name: *mut c_char, object_type: c_int) -> *mut c_void {
    if name.is_null() {
        return null_mut();
    }

    REGISTRY_LOCK();
    let entry: *mut registry_entry_t = registry_find_entry(name, object_type);
    let mut data: *mut c_void = null_mut();
    if !entry.is_null() {
        data = (*entry).data;
    }
    REGISTRY_UNLOCK();
    data
}

/// Release a reference of an entry (decrement the reference count)
#[no_mangle]
pub unsafe extern "C" fn registry_unref(name: *mut c_char, object_type: c_int) -> c_int {
    let mut res: c_int = -1;

    if name.is_null() {
        return -1;
    }

    REGISTRY_LOCK();

    let entry: *mut registry_entry_t = registry_find_entry(name, object_type);
    if !entry.is_null() {
        (*entry).ref_count -= 1;

        if DEBUG_REGISTRY != 0 {
            libc::printf(cstr!("Registry: object %s: ref_count = %d after unref.\n"), name, (*entry).ref_count);
        }

        if (*entry).ref_count < 0 {
            libc::fprintf(c_stderr(), cstr!("Registry: object %s (type %d): negative ref_count.\n"), name, object_type);
        } else {
            res = 0;
        }
    }

    REGISTRY_UNLOCK();
    res
}

/// Execute action on an object if its reference count is less or equal to
/// the specified count.
#[no_mangle]
pub unsafe extern "C" fn registry_exec_refcount(name: *mut c_char, object_type: c_int, max_ref: c_int, reg_del: c_int, obj_action: registry_exec, opt_arg: *mut c_void) -> c_int {
    let mut res: c_int = -1;

    if name.is_null() {
        return -1;
    }

    REGISTRY_LOCK();

    let entry: *mut registry_entry_t = registry_find_entry(name, object_type);

    if !entry.is_null() {
        if (*entry).ref_count <= max_ref {
            let mut status: c_int = TRUE;

            if obj_action.is_some() {
                status = obj_action.unwrap()((*entry).data, opt_arg);
            }

            if reg_del != 0 && status != 0 {
                registry_remove_entry(entry);
            }

            res = 1;
        } else {
            res = 0;
        }
    }

    REGISTRY_UNLOCK();
    res
}

/// Delete object if unused
#[no_mangle]
pub unsafe extern "C" fn registry_delete_if_unused(name: *mut c_char, object_type: c_int, obj_destructor: registry_exec, opt_arg: *mut c_void) -> c_int {
    registry_exec_refcount(name, object_type, 0, TRUE, obj_destructor, opt_arg)
}

/// Execute a callback function for all objects of specified type
#[no_mangle]
pub unsafe extern "C" fn registry_foreach_type(object_type: c_int, cb: registry_foreach, opt: *mut c_void, err: *mut c_int) -> c_int {
    REGISTRY_LOCK();

    let bucket: *mut registry_entry_t = (*registry).ht_types.offset(object_type as isize);

    let mut count: c_int = 0;
    let mut p: *mut registry_entry_t = (*bucket).htype_next;
    while p != bucket {
        let next: *mut registry_entry_t = (*p).htype_next;
        if let Some(cb) = cb {
            cb(p, opt, err);
        }
        count += 1;
        p = next;
    }

    REGISTRY_UNLOCK();
    count
}

/// Delete all objects of the specified type
#[no_mangle]
pub unsafe extern "C" fn registry_delete_type(object_type: c_int, cb: registry_exec, opt: *mut c_void) -> c_int {
    REGISTRY_LOCK();

    let bucket: *mut registry_entry_t = (*registry).ht_types.offset(object_type as isize);

    let mut count: c_int = 0;
    let mut p: *mut registry_entry_t = (*bucket).htype_next;
    while p != bucket {
        let next: *mut registry_entry_t = (*p).htype_next;

        if (*p).ref_count == 0 {
            let mut status: c_int = TRUE;

            if let Some(cb) = cb {
                status = cb((*p).data, opt);
            }

            if status != 0 {
                registry_remove_entry(p);
                count += 1;
            }
        } else {
            libc::fprintf(c_stderr(), cstr!("registry_delete_type: object \"%s\" (type %d) still referenced (count=%d)\n"), (*p).name, object_type, (*p).ref_count);
        }
        p = next;
    }

    REGISTRY_UNLOCK();
    count
}

/// Dump the registry
#[no_mangle]
pub unsafe extern "C" fn registry_dump() {
    REGISTRY_LOCK();

    libc::printf(cstr!("Registry dump:\n"));

    libc::printf(cstr!("  Objects (from name hash table):\n"));

    // dump hash table of names
    for i in 0..(*registry).ht_name_entries {
        let bucket: *mut registry_entry_t = (*registry).ht_names.offset(i as isize);

        let mut p: *mut registry_entry_t = (*bucket).hname_next;
        while p != bucket {
            libc::printf(cstr!("     %s (type %d, ref_count=%d)\n"), (*p).name, (*p).object_type, (*p).ref_count);
            p = (*p).hname_next;
        }
    }

    libc::printf(cstr!("\n  Objects classed by types:\n"));

    // dump hash table of types
    for i in 0..(*registry).ht_type_entries {
        libc::printf(cstr!("     Type %d: "), i);

        let bucket = (*registry).ht_types.offset(i as isize);
        let mut p: *mut registry_entry_t = (*bucket).htype_next;
        while p != bucket {
            libc::printf(cstr!("%s(%d) "), (*p).name, (*p).ref_count);
            p = (*p).htype_next;
        }

        libc::printf(cstr!("\n"));
    }

    REGISTRY_UNLOCK();
}
