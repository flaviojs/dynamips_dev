//! ATM bridge (RFC1483)

use crate::atm_vsar::*;
use crate::net_io::*;
use crate::prelude::*;
use crate::registry::*;

pub type atm_bridge_t = atm_bridge;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct atm_bridge {
    pub name: *mut c_char,
    pub lock: libc::pthread_mutex_t,
    pub eth_nio: *mut netio_desc_t,
    pub atm_nio: *mut netio_desc_t,
    pub vpi: u_int,
    pub vci: u_int,
    pub arc: atm_reas_context,
}

/// Acquire a reference to an ATM bridge (increment reference count)
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_acquire(name: *mut c_char) -> *mut atm_bridge_t {
    registry_find(name, OBJ_TYPE_ATM_BRIDGE).cast::<_>()
}

/// Release an ATM switch (decrement reference count)
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_release(name: *mut c_char) -> c_int {
    registry_unref(name, OBJ_TYPE_ATM_BRIDGE)
}

/// Create a virtual ATM bridge
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_create(name: *mut c_char) -> *mut atm_bridge_t {
    // Allocate a new switch structure
    let t: *mut atm_bridge_t = libc::malloc(size_of::<atm_bridge_t>()).cast::<_>();
    if t.is_null() {
        return null_mut();
    }

    libc::memset(t.cast::<_>(), 0, size_of::<atm_bridge_t>());
    libc::pthread_mutex_init(addr_of_mut!((*t).lock), null_mut());
    atm_aal5_recv_reset(addr_of_mut!((*t).arc));

    (*t).name = libc::strdup(name);
    if (*t).name.is_null() {
        libc::free(t.cast::<_>());
        return null_mut();
    }

    // Record this object in registry
    if registry_add((*t).name, OBJ_TYPE_ATM_BRIDGE, t.cast::<_>()) == -1 {
        libc::fprintf(c_stderr(), cstr!("atm_bridge_create: unable to create bridge '%s'\n"), name);
        libc::free((*t).name.cast::<_>());
        libc::free(t.cast::<_>());
        return null_mut();
    }

    t
}
