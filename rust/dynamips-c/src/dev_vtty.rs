//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Virtual console TTY.
//!
//! "Interactive" part idea by Mtve.
//! TCP console added by Mtve.
//! Serial console by Peter Ross (suxen_drol@hotmail.com)

use crate::_private::*;
use crate::utils::*;
use crate::vm::*;

pub type vtty_serial_option_t = vtty_serial_option;
pub type vtty_t = virtual_tty;

/// 4 Kb should be enough for a keyboard buffer
pub const VTTY_BUFFER_SIZE: usize = 4096;

/// Maximum listening socket number
pub const VTTY_MAX_FD: usize = 10;

/// VTTY connection types // TODO enum
pub const VTTY_TYPE_NONE: c_int = 0;
pub const VTTY_TYPE_TERM: c_int = 1;
pub const VTTY_TYPE_TCP: c_int = 2;
pub const VTTY_TYPE_SERIAL: c_int = 3;

/// VTTY connection states (for TCP) // TODO enum
pub const VTTY_STATE_TCP_INVALID: c_int = 0; // connection is not working
pub const VTTY_STATE_TCP_WAITING: c_int = 1; // waiting for incoming connection
pub const VTTY_STATE_TCP_RUNNING: c_int = 2; // character reading/writing ok

/// VTTY input states // TODO enum
pub const VTTY_INPUT_TEXT: c_int = 0;
pub const VTTY_INPUT_VT1: c_int = 1;
pub const VTTY_INPUT_VT2: c_int = 2;
pub const VTTY_INPUT_REMOTE: c_int = 3;
pub const VTTY_INPUT_TELNET: c_int = 4;
pub const VTTY_INPUT_TELNET_IYOU: c_int = 5;
pub const VTTY_INPUT_TELNET_SB1: c_int = 6;
pub const VTTY_INPUT_TELNET_SB2: c_int = 7;
pub const VTTY_INPUT_TELNET_SB_TTYPE: c_int = 8;
pub const VTTY_INPUT_TELNET_NEXT: c_int = 9;

/// Commmand line support utility
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vtty_serial_option {
    pub device: *mut c_char,
    pub baudrate: c_int,
    pub databits: c_int,
    pub parity: c_int,
    pub stopbits: c_int,
    pub hwflow: c_int,
}

/// Virtual TTY structure
#[repr(C)]
#[derive(Copy, Clone)]
pub struct virtual_tty {
    pub vm: *mut vm_instance_t,
    pub name: *mut c_char,
    pub r#type: c_int,
    pub fd_array: [c_int; VTTY_MAX_FD],
    pub fd_count: c_int,
    pub tcp_port: c_int,
    pub terminal_support: c_int,
    pub input_state: c_int,
    pub input_pending: c_int,
    pub telnet_cmd: c_int,
    pub telnet_opt: c_int,
    pub telnet_qual: c_int,
    pub managed_flush: c_int,
    pub buffer: [u_char; VTTY_BUFFER_SIZE],
    pub read_ptr: u_int,
    pub write_ptr: u_int,
    pub lock: libc::pthread_mutex_t,
    pub next: *mut vtty_t,
    pub pprev: *mut *mut vtty_t,
    pub priv_data: *mut c_void,
    pub user_arg: u_long,

    /// FD Pool (for TCP connections)
    pub fd_pool: fd_pool_t,

    /// Read notification
    pub read_notifier: Option<unsafe extern "C" fn(arg1: *mut vtty_t)>,

    /// Old text for replay
    pub replay_buffer: [u_char; VTTY_BUFFER_SIZE],
    pub replay_ptr: u_int,
    pub replay_full: u_char,
}

#[no_mangle]
pub unsafe extern "C" fn VTTY_LOCK(tty: *mut vtty_t) {
    libc::pthread_mutex_lock(addr_of_mut!((*tty).lock));
}
#[no_mangle]
pub unsafe extern "C" fn VTTY_UNLOCK(tty: *mut vtty_t) {
    libc::pthread_mutex_unlock(addr_of_mut!((*tty).lock));
}
