//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//!
//! NetIO bridges.

use crate::_private::*;
use crate::dynamips_common::*;
use crate::net_io::*;
use crate::registry::*;
use crate::utils::*;

pub type netio_bridge_t = netio_bridge;

pub const NETIO_BRIDGE_MAX_NIO: usize = 32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct netio_bridge {
    pub name: *mut c_char,
    pub lock: libc::pthread_mutex_t,
    pub nio: [*mut netio_desc_t; NETIO_BRIDGE_MAX_NIO],
}

unsafe fn NETIO_BRIDGE_LOCK(t: *mut netio_bridge_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*t).lock));
}
unsafe fn NETIO_BRIDGE_UNLOCK(t: *mut netio_bridge_t) {
    libc::pthread_mutex_unlock(addr_of_mut!((*t).lock));
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

/// Acquire a reference to NetIO bridge from the registry (inc ref count)
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_acquire(name: *mut c_char) -> *mut netio_desc_t {
    registry_find(name, OBJ_TYPE_NIO_BRIDGE).cast::<_>()
}

/// Release a NetIO bridge (decrement reference count)
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_release(name: *mut c_char) -> c_int {
    registry_unref(name, OBJ_TYPE_NIO_BRIDGE)
}

/// Receive a packet
unsafe extern "C" fn netio_bridge_recv_pkt(nio: *mut netio_desc_t, pkt: *mut u_char, pkt_len: ssize_t, t: *mut c_void, _: *mut c_void) -> c_int {
    let t: *mut netio_bridge_t = t.cast::<_>();
    NETIO_BRIDGE_LOCK(t);

    for i in 0..NETIO_BRIDGE_MAX_NIO {
        if !(*t).nio[i].is_null() && (*t).nio[i] != nio {
            netio_send((*t).nio[i], pkt.cast::<_>(), pkt_len as size_t);
        }
    }

    NETIO_BRIDGE_UNLOCK(t);
    0
}

/// Add a NetIO descriptor to a virtual bridge
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_add_netio(t: *mut netio_bridge_t, nio_name: *mut c_char) -> c_int {
    NETIO_BRIDGE_LOCK(t);

    // Try to find a free slot in the NIO array
    let mut i: usize = 0;
    while i < NETIO_BRIDGE_MAX_NIO {
        if (*t).nio[i].is_null() {
            break;
        }
        i += 1;
    }

    // No free slot found ...
    if i == NETIO_BRIDGE_MAX_NIO {
        NETIO_BRIDGE_UNLOCK(t);
        return -1;
    }

    // Acquire the NIO descriptor and increment its reference count
    let nio: *mut netio_desc_t = netio_acquire(nio_name);
    if nio.is_null() {
        NETIO_BRIDGE_UNLOCK(t);
        return -1;
    }

    (*t).nio[i] = nio;
    netio_rxl_add(nio, Some(netio_bridge_recv_pkt), t.cast::<_>(), null_mut());
    NETIO_BRIDGE_UNLOCK(t);
    0
}

/// Free resources used by a NIO in a bridge
#[no_mangle] // TODO private
pub unsafe extern "C" fn netio_bridge_free_nio(nio: *mut netio_desc_t) {
    netio_rxl_remove(nio);
    netio_release((*nio).name);
}

/// Remove a NetIO descriptor from a virtual bridge
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_remove_netio(t: *mut netio_bridge_t, nio_name: *mut c_char) -> c_int {
    NETIO_BRIDGE_LOCK(t);

    let nio: *mut netio_desc_t = registry_exists(nio_name, OBJ_TYPE_NIO).cast::<_>();
    if nio.is_null() {
        NETIO_BRIDGE_UNLOCK(t);
        return -1;
    }

    // Try to find the NIO in the NIO array
    let mut i: usize = 0;
    while i < NETIO_BRIDGE_MAX_NIO {
        if (*t).nio[i] == nio {
            break;
        }
        i += 1;
    }

    if i == NETIO_BRIDGE_MAX_NIO {
        NETIO_BRIDGE_UNLOCK(t);
        return -1;
    }

    // Remove the NIO from the RX multiplexer
    netio_bridge_free_nio((*t).nio[i]);
    (*t).nio[i] = null_mut();

    NETIO_BRIDGE_UNLOCK(t);
    0
}

/// Save the configuration of a bridge
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_save_config(t: *mut netio_bridge_t, fd: *mut libc::FILE) {
    libc::fprintf(fd, cstr!("nio_bridge create %s\n"), (*t).name);

    for i in 0..NETIO_BRIDGE_MAX_NIO {
        // FIXME does not check if nio is null
        libc::fprintf(fd, cstr!("nio_bridge add_nio %s %s\n"), (*t).name, (*(*t).nio[i]).name);
    }

    libc::fprintf(fd, cstr!("\n"));
}

/// Save configurations of all NIO bridges
unsafe extern "C" fn netio_bridge_reg_save_config(entry: *mut registry_entry_t, opt: *mut c_void, _err: *mut c_int) {
    netio_bridge_save_config((*entry).data.cast::<netio_bridge_t>(), opt.cast::<libc::FILE>());
}

#[no_mangle]
pub unsafe extern "C" fn netio_bridge_save_config_all(fd: *mut libc::FILE) {
    registry_foreach_type(OBJ_TYPE_NIO_BRIDGE, Some(netio_bridge_reg_save_config), fd.cast::<_>(), null_mut());
}

/// Free resources used by a NIO bridge
unsafe extern "C" fn netio_bridge_free(data: *mut c_void, _arg: *mut c_void) -> c_int {
    let t: *mut netio_bridge_t = data.cast::<_>();

    NETIO_BRIDGE_LOCK(t);

    for i in 0..NETIO_BRIDGE_MAX_NIO {
        if (*t).nio[i].is_null() {
            continue;
        }

        netio_bridge_free_nio((*t).nio[i]);
    }

    NETIO_BRIDGE_UNLOCK(t);
    libc::free((*t).name.cast::<_>());
    libc::free(t.cast::<_>());
    TRUE
}

/// Delete a virtual bridge
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_delete(name: *mut c_char) -> c_int {
    registry_delete_if_unused(name, OBJ_TYPE_NIO_BRIDGE, Some(netio_bridge_free), null_mut())
}

/// Delete all virtual bridges
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_delete_all() -> c_int {
    registry_delete_type(OBJ_TYPE_NIO_BRIDGE, Some(netio_bridge_free), null_mut())
}

/// Create a new interface
unsafe fn netio_bridge_cfg_create_if(t: *mut netio_bridge_t, tokens: *mut *mut c_char, count: c_int) -> c_int {
    let mut nio: *mut netio_desc_t = null_mut();

    let nio_type: c_int = netio_get_type(*tokens.add(1));
    match nio_type as u_int {
        NETIO_TYPE_UNIX => 'block: {
            if count != 4 {
                libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: invalid number of arguments for UNIX NIO\n"));
                break 'block;
            }

            nio = netio_desc_create_unix(*tokens.add(0), *tokens.add(2), *tokens.add(3));
        }

        NETIO_TYPE_TAP => 'block: {
            if count != 3 {
                libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: invalid number of arguments for TAP NIO\n"));
                break 'block;
            }

            nio = netio_desc_create_tap(*tokens.add(0), *tokens.add(2));
        }

        NETIO_TYPE_UDP => 'block: {
            if count != 5 {
                libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: invalid number of arguments for UDP NIO\n"));
                break 'block;
            }

            nio = netio_desc_create_udp(*tokens.add(0), libc::atoi(*tokens.add(2)), *tokens.add(3), libc::atoi(*tokens.add(4)));
        }

        NETIO_TYPE_TCP_CLI => 'block: {
            if count != 4 {
                libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: invalid number of arguments for TCP CLI NIO\n"));
                break 'block;
            }

            nio = netio_desc_create_tcp_cli(*tokens.add(0), *tokens.add(2), *tokens.add(3));
        }

        NETIO_TYPE_TCP_SER => 'block: {
            if count != 3 {
                libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: invalid number of arguments for TCP SER NIO\n"));
                break 'block;
            }

            nio = netio_desc_create_tcp_ser(*tokens.add(0), *tokens.add(2));
        }

        #[cfg(feature = "ENABLE_GEN_ETH")]
        NETIO_TYPE_GEN_ETH => 'block: {
            if count != 3 {
                libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: invalid number of arguments for Generic Ethernet NIO\n"));
                break 'block;
            }

            nio = netio_desc_create_geneth(*tokens.add(0), *tokens.add(2));
        }

        #[cfg(feature = "ENABLE_LINUX_ETH")]
        NETIO_TYPE_LINUX_ETH => 'block: {
            if count != 3 {
                libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: invalid number of arguments for Linux Ethernet NIO\n"));
                break 'block;
            }

            nio = netio_desc_create_lnxeth(*tokens.add(0), *tokens.add(2));
        }

        _ => {
            libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: unknown/invalid NETIO type '%s'\n"), *tokens.add(1));
        }
    }

    if nio.is_null() {
        libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: unable to create NETIO descriptor\n"));
        return -1;
    }

    if netio_bridge_add_netio(t, *tokens.add(0)) == -1 {
        libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: unable to add NETIO descriptor.\n"));
        netio_release((*nio).name);
        return -1;
    }

    netio_release((*nio).name);
    0
}

const NETIO_BRIDGE_MAX_TOKENS: usize = 16;

/// Handle a configuration line
unsafe fn netio_bridge_handle_cfg_line(t: *mut netio_bridge_t, str_: *mut c_char) -> c_int {
    let mut tokens: [*mut c_char; NETIO_BRIDGE_MAX_TOKENS] = [null_mut(); NETIO_BRIDGE_MAX_TOKENS];

    let count: c_int = m_strsplit(str_, b':' as c_char, tokens.as_c_mut(), NETIO_BRIDGE_MAX_TOKENS as c_int);
    if count <= 2 {
        return -1;
    }

    netio_bridge_cfg_create_if(t, tokens.as_c_mut(), count)
}

/// Read a configuration file
unsafe fn netio_bridge_read_cfg_file(t: *mut netio_bridge_t, filename: *mut c_char) -> c_int {
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
            netio_bridge_handle_cfg_line(t, buffer.as_c_mut());
        }
    }

    libc::fclose(fd);
    0
}

/// Start a virtual bridge
#[no_mangle]
pub unsafe extern "C" fn netio_bridge_start(filename: *mut c_char) -> c_int {
    let t: *mut netio_bridge_t = netio_bridge_create(cstr!("default"));
    if t.is_null() {
        libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: unable to create virtual fabric table.\n"));
        return -1;
    }

    if netio_bridge_read_cfg_file(t, filename) == -1 {
        libc::fprintf(c_stderr(), cstr!("NETIO_BRIDGE: unable to parse configuration file.\n"));
        return -1;
    }

    netio_bridge_release(cstr!("default"));
    0
}
