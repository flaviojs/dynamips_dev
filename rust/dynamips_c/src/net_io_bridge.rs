//! NetIO bridge.

use crate::net_io::*;
use crate::prelude::*;
use crate::registry::*;

pub type netio_bridge_t = netio_bridge;

pub const NETIO_BRIDGE_MAX_NIO: usize = 32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct netio_bridge {
    pub name: *mut c_char,
    pub lock: libc::pthread_mutex_t,
    pub nio: [*mut netio_desc_t; NETIO_BRIDGE_MAX_NIO],
}

/// Create a virtual bridge
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_create(name: *mut c_char) -> *mut netio_bridge_t {
    // Allocate a new bridge structure
    let t: *mut netio_bridge_t = libc::malloc(size_of::<netio_bridge_t>()).cast::<_>();
    if t.is_null() {
        return null_mut();
    }

    libc::memset(t.cast::<_>(), 0, size_of::<netio_bridge_t>());
    libc::pthread_mutex_init(addr_of_mut!((*t).lock), null_mut());

    (*t).name = libc::strdup(name);
    if (*t).name.is_null() {
        libc::free(t.cast::<_>());
        return null_mut();
    }

    // Record this object in registry
    if registry_add((*t).name, OBJ_TYPE_NIO_BRIDGE, t.cast::<_>()) == -1 {
        libc::fprintf(c_stderr(), cstr!("netio_bridge_create: unable to register bridge '%s'\n"), name);
        libc::free((*t).name.cast::<_>());
        libc::free(t.cast::<_>());
        return null_mut();
    }

    t
}
