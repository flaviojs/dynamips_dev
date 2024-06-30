//! ATM bridge (RFC1483)

use crate::atm::*;
use crate::atm_vsar::*;
use crate::dynamips_common::*;
use crate::net_io::*;
use crate::prelude::*;
use crate::registry::*;
use crate::utils::*;

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

unsafe fn ATM_BRIDGE_LOCK(t: *mut atm_bridge_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*t).lock));
}
unsafe fn ATM_BRIDGE_UNLOCK(t: *mut atm_bridge_t) {
    libc::pthread_mutex_unlock(addr_of_mut!((*t).lock));
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

/// Receive an ATM cell
unsafe extern "C" fn atm_bridge_recv_cell(_nio: *mut netio_desc_t, atm_cell: *mut u_char, cell_len: ssize_t, t: *mut c_void, _: *mut c_void) -> c_int {
    let t: *mut atm_bridge_t = t.cast::<_>();
    let mut res: c_int = 0;

    if cell_len != ATM_CELL_SIZE as ssize_t {
        return -1;
    }

    ATM_BRIDGE_LOCK(t);

    // check the VPI/VCI
    let atm_hdr: m_uint32_t = m_ntoh32(atm_cell);

    let vpi: m_uint32_t = (atm_hdr & ATM_HDR_VPI_MASK) >> ATM_HDR_VPI_SHIFT;
    let vci: m_uint32_t = (atm_hdr & ATM_HDR_VCI_MASK) >> ATM_HDR_VCI_SHIFT;

    if (*t).vpi != vpi || (*t).vci != vci {
        ATM_BRIDGE_UNLOCK(t);
        return res;
    }

    let status: c_int = atm_aal5_recv(addr_of_mut!((*t).arc), atm_cell);
    if status == 1 {
        // Got AAL5 packet, check RFC1483b encapsulation
        if (*t).arc.len > ATM_RFC1483B_HLEN && libc::memcmp((*t).arc.buffer.as_c_void(), atm_rfc1483b_header.as_c_void(), ATM_RFC1483B_HLEN) == 0 {
            netio_send((*t).eth_nio, (*t).arc.buffer.as_c_mut().add(ATM_RFC1483B_HLEN).cast::<_>(), (*t).arc.len - ATM_RFC1483B_HLEN);
        }

        atm_aal5_recv_reset(addr_of_mut!((*t).arc));
    } else if status < 0 {
        atm_aal5_recv_reset(addr_of_mut!((*t).arc));
        res = -1;
    }

    ATM_BRIDGE_UNLOCK(t);
    res
}

/* Receive an Ethernet packet */
unsafe extern "C" fn atm_bridge_recv_pkt(_nio: *mut netio_desc_t, pkt: *mut u_char, len: ssize_t, t: *mut c_void, _: *mut c_void) -> c_int {
    let t: *mut atm_bridge_t = t.cast::<_>();
    ATM_BRIDGE_LOCK(t);
    let res: c_int = atm_aal5_send_rfc1483b((*t).atm_nio, (*t).vpi, (*t).vci, pkt.cast::<_>(), len as size_t);
    ATM_BRIDGE_UNLOCK(t);
    res
}

/// Configure an ATM bridge
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_configure(t: *mut atm_bridge_t, eth_nio: *mut c_char, atm_nio: *mut c_char, vpi: u_int, vci: u_int) -> c_int {
    ATM_BRIDGE_LOCK(t);

    if !(*t).eth_nio.is_null() || !(*t).atm_nio.is_null() {
        ATM_BRIDGE_UNLOCK(t);
        return -1;
    }

    let e_nio: *mut netio_desc_t = netio_acquire(eth_nio);
    let a_nio: *mut netio_desc_t = netio_acquire(atm_nio);

    if e_nio.is_null() || a_nio.is_null() {
        ATM_BRIDGE_UNLOCK(t);
        return -1;
    }

    (*t).eth_nio = e_nio;
    (*t).atm_nio = a_nio;
    (*t).vpi = vpi;
    (*t).vci = vci;

    // Add ATM RX listener
    if netio_rxl_add((*t).atm_nio, Some(atm_bridge_recv_cell), t.cast::<_>(), null_mut()) == -1 {
        ATM_BRIDGE_UNLOCK(t);
        return -1;
    }

    // Add Ethernet RX listener
    if netio_rxl_add((*t).eth_nio, Some(atm_bridge_recv_pkt), t.cast::<_>(), null_mut()) == -1 {
        ATM_BRIDGE_UNLOCK(t);
        return -1;
    }

    ATM_BRIDGE_UNLOCK(t);
    0
}
