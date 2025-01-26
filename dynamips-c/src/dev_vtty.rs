//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Virtual console TTY.

use crate::_extra::*;
use crate::utils::*;
use crate::vm::*;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;

// 4 Kb should be enough for a keyboard buffer
pub const VTTY_BUFFER_SIZE: usize = 4096;

// Maximum listening socket number
pub const VTTY_MAX_FD: usize = 10;

// VTTY connection types
pub type _VTTY_TYPE_ENUM = c_int; // TODO enum
pub const VTTY_TYPE_NONE: _VTTY_TYPE_ENUM = 0;
pub const VTTY_TYPE_TERM: _VTTY_TYPE_ENUM = 1;
pub const VTTY_TYPE_TCP: _VTTY_TYPE_ENUM = 2;
pub const VTTY_TYPE_SERIAL: _VTTY_TYPE_ENUM = 3;

// VTTY connection states (for TCP)
pub type _VTTY_STATE_TCP_ENUM = c_int; // TODO enum
pub const VTTY_STATE_TCP_INVALID: _VTTY_STATE_TCP_ENUM = 0; // connection is not working
pub const VTTY_STATE_TCP_WAITING: _VTTY_STATE_TCP_ENUM = 1; // waiting for incoming connection
pub const VTTY_STATE_TCP_RUNNING: _VTTY_STATE_TCP_ENUM = 2; // character reading/writing ok

// VTTY input states
pub type _VTTY_INPUT_ENUM = c_int; // TODO enum
pub const VTTY_INPUT_TEXT: _VTTY_INPUT_ENUM = 0;
pub const VTTY_INPUT_VT1: _VTTY_INPUT_ENUM = 1;
pub const VTTY_INPUT_VT2: _VTTY_INPUT_ENUM = 2;
pub const VTTY_INPUT_REMOTE: _VTTY_INPUT_ENUM = 3;
pub const VTTY_INPUT_TELNET: _VTTY_INPUT_ENUM = 4;
pub const VTTY_INPUT_TELNET_IYOU: _VTTY_INPUT_ENUM = 5;
pub const VTTY_INPUT_TELNET_SB1: _VTTY_INPUT_ENUM = 6;
pub const VTTY_INPUT_TELNET_SB2: _VTTY_INPUT_ENUM = 7;
pub const VTTY_INPUT_TELNET_SB_TTYPE: _VTTY_INPUT_ENUM = 8;
pub const VTTY_INPUT_TELNET_NEXT: _VTTY_INPUT_ENUM = 9;

// Commmand line support utility
pub type vtty_serial_option_t = vtty_serial_option;
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

// Virtual TTY structure
pub type vtty_t = virtual_tty; // TODO enum
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

    // FD Pool (for TCP connections)
    pub fd_pool: fd_pool_t,

    // Read notification
    pub read_notifier: Option<unsafe extern "C" fn(arg1: *mut vtty_t)>,

    // Old text for replay
    pub replay_buffer: [u_char; VTTY_BUFFER_SIZE],
    pub replay_ptr: u_int,
    pub replay_full: u_char,
}

macro_rules! VTTY_LOCK {
    ($tty:expr) => {
        libc::pthread_mutex_lock(std::ptr::addr_of_mut!((*$tty).lock))
    };
}
macro_rules! VTTY_UNLOCK {
    ($tty:expr) => {
        libc::pthread_mutex_unlock(std::ptr::addr_of_mut!((*$tty).lock))
    };
}
