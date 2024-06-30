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
