//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! ATM bridge (RFC1483)

use crate::_private::*;
use crate::atm::*;
use crate::atm_vsar::*;
use crate::dynamips_common::*;
use crate::net_io::*;
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

/// Release NIO used by an ATM bridge
unsafe fn atm_bridge_clear_config(t: *mut atm_bridge_t) {
    if !t.is_null() {
        // release ethernet NIO
        if !(*t).eth_nio.is_null() {
            netio_rxl_remove((*t).eth_nio);
            netio_release((*(*t).eth_nio).name);
        }

        // release ATM NIO
        if !(*t).atm_nio.is_null() {
            netio_rxl_remove((*t).atm_nio);
            netio_release((*(*t).atm_nio).name);
        }

        (*t).eth_nio = null_mut();
        (*t).atm_nio = null_mut();
    }
}

/// Unconfigure an ATM bridge
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_unconfigure(t: *mut atm_bridge_t) -> c_int {
    ATM_BRIDGE_LOCK(t);
    atm_bridge_clear_config(t);
    ATM_BRIDGE_UNLOCK(t);
    0
}

/// Free resources used by an ATM bridge
unsafe extern "C" fn atm_bridge_free(data: *mut c_void, _arg: *mut c_void) -> c_int {
    let t: *mut atm_bridge_t = data.cast::<_>();

    atm_bridge_clear_config(t);
    libc::free((*t).name.cast::<_>());
    libc::free(t.cast::<_>());
    TRUE
}

/// Delete an ATM bridge
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_delete(name: *mut c_char) -> c_int {
    registry_delete_if_unused(name, OBJ_TYPE_ATM_BRIDGE, Some(atm_bridge_free), null_mut())
}

/// Delete all ATM switches
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_delete_all() -> c_int {
    registry_delete_type(OBJ_TYPE_ATM_BRIDGE, Some(atm_bridge_free), null_mut())
}

/// Create a new interface
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_cfg_create_if(_t: *mut atm_bridge_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    let mut nio: *mut netio_desc_t = null_mut();

    // at least: IF, interface name, NetIO type
    if count < 3 {
        libc::fprintf(c_stderr(), cstr!("atmsw_cfg_create_if: invalid interface description\n"));
        return -1;
    }

    let nio_type: c_int = netio_get_type(*tokens.add(2));
    match nio_type as u_int {
        NETIO_TYPE_UNIX => 'block: {
            if count != 5 {
                libc::fprintf(c_stderr(), cstr!("ATMSW: invalid number of arguments for UNIX NIO '%s'\n"), *tokens.add(1));
                break 'block;
            }

            nio = netio_desc_create_unix(*tokens.add(1), *tokens.add(3), *tokens.add(4));
        }

        NETIO_TYPE_UDP => 'block: {
            if count != 6 {
                libc::fprintf(c_stderr(), cstr!("ATMSW: invalid number of arguments for UDP NIO '%s'\n"), *tokens.add(1));
                break 'block;
            }

            nio = netio_desc_create_udp(*tokens.add(1), libc::atoi(*tokens.add(3)), *tokens.add(4), libc::atoi(*tokens.add(5)));
        }

        NETIO_TYPE_TCP_CLI => 'block: {
            if count != 5 {
                libc::fprintf(c_stderr(), cstr!("ATMSW: invalid number of arguments for TCP CLI NIO '%s'\n"), *tokens.add(1));
                break 'block;
            }

            nio = netio_desc_create_tcp_cli(*tokens.add(1), *tokens.add(3), *tokens.add(4));
        }

        NETIO_TYPE_TCP_SER => 'block: {
            if count != 4 {
                libc::fprintf(c_stderr(), cstr!("ATMSW: invalid number of arguments for TCP SER NIO '%s'\n"), *tokens.add(1));
                break 'block;
            }

            nio = netio_desc_create_tcp_ser(*tokens.add(1), *tokens.add(3));
        }

        _ => {
            libc::fprintf(c_stderr(), cstr!("ATMSW: unknown/invalid NETIO type '%s'\n"), *tokens.add(2));
        }
    }

    if nio.is_null() {
        libc::fprintf(c_stderr(), cstr!("ATMSW: unable to create NETIO descriptor of interface %s\n"), *tokens.add(1));
        return -1;
    }

    netio_release((*nio).name);
    0
}

/// Bridge setup
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_cfg_setup(t: *mut atm_bridge_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    // 5 parameters: "BRIDGE", Eth_IF, ATM_IF, VPI, VCI
    if count != 5 {
        libc::fprintf(c_stderr(), cstr!("ATM Bridge: invalid VPC descriptor.\n"));
        return -1;
    }

    atm_bridge_configure(t, *tokens.add(1), *tokens.add(2), libc::atoi(*tokens.add(3)) as u_int, libc::atoi(*tokens.add(4)) as u_int)
}

const ATM_BRIDGE_MAX_TOKENS: usize = 16;

/// Handle an ATMSW configuration line
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_handle_cfg_line(t: *mut atm_bridge_t, str_: *mut c_char) -> c_int {
    let mut tokens: [*mut c_char; ATM_BRIDGE_MAX_TOKENS] = [null_mut(); ATM_BRIDGE_MAX_TOKENS];

    let count: c_int = m_strsplit(str_, b':' as c_char, tokens.as_c_mut(), ATM_BRIDGE_MAX_TOKENS as c_int);
    if count <= 1 {
        return -1;
    }

    if libc::strcmp(tokens[0], cstr!("IF")) == 0 {
        return atm_bridge_cfg_create_if(t, tokens.as_c_mut(), count);
    } else if libc::strcmp(tokens[0], cstr!("BRIDGE")) == 0 {
        return atm_bridge_cfg_setup(t, tokens.as_c_mut(), count);
    }

    libc::fprintf(c_stderr(), cstr!("ATM Bridge: Unknown statement \"%s\" (allowed: IF,BRIDGE)\n"), tokens[0]);
    -1
}

/// Read an ATM bridge configuration file
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_read_cfg_file(t: *mut atm_bridge_t, filename: *mut c_char) -> c_int {
    let mut buffer: [c_char; 1024] = [0; 1024];
    let mut ptr: *mut c_char;

    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("r"));
    if fd.is_null() {
        libc::perror(cstr!("fopen"));
        return -1;
    }

    while libc::feof(fd) == 0 {
        if libc::fgets(buffer.as_c_mut(), buffer.len() as c_int, fd).is_null() {
            break;
        }

        // skip comments and end of line
        ptr = libc::strpbrk(buffer.as_c(), cstr!("#\r\n"));
        if !ptr.is_null() {
            *ptr = 0;
        }

        // analyze non-empty lines
        if !libc::strchr(buffer.as_c(), b':' as c_int).is_null() {
            atm_bridge_handle_cfg_line(t, buffer.as_c_mut());
        }
    }

    libc::fclose(fd);
    0
}

/// Start a virtual ATM bridge
#[no_mangle]
pub unsafe extern "C" fn atm_bridge_start(filename: *mut c_char) -> c_int {
    let t: *mut atm_bridge_t = atm_bridge_create(cstr!("default"));
    if t.is_null() {
        libc::fprintf(c_stderr(), cstr!("ATM Bridge: unable to create virtual fabric table.\n"));
        return -1;
    }

    if atm_bridge_read_cfg_file(t, filename) == -1 {
        libc::fprintf(c_stderr(), cstr!("ATM Bridge: unable to parse configuration file.\n"));
        return -1;
    }

    atm_bridge_release(cstr!("default"));
    0
}
