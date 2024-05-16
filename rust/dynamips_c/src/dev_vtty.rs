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

pub type vtty_serial_option_t = vtty_serial_option;
pub type vtty_t = virtual_tty;

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

/// cbindgen:no-export
#[repr(C)]
pub struct virtual_tty {
    _todo: u8,
}

#[no_mangle] // TODO private
pub static mut ctrl_code_ok: c_int = 1;

/// Allow the user to disable the CTRL code for the monitor interface
#[no_mangle]
pub unsafe extern "C" fn vtty_set_ctrlhandler(n: c_int) {
    ctrl_code_ok = n;
}
/// Parse serial interface descriptor string, return 0 if success
/// string takes the form "device:baudrate:databits:parity:stopbits:hwflow"
/// device is mandatory, other options are optional (default=9600,8,N,1,0).
#[no_mangle]
pub unsafe extern "C" fn vtty_parse_serial_option(option: *mut vtty_serial_option_t, optarg: *mut c_char) -> c_int {
    let mut array: [*mut c_char; 6] = [null_mut(); 6];
    let count: c_int = m_strtok(optarg, b':' as c_char, array.as_c_mut(), 6);
    if count < 1 {
        libc::fprintf(c_stderr(), cstr!("vtty_parse_serial_option: invalid string\n"));
        return -1;
    }

    (*option).device = libc::strdup(array[0]);
    if (*option).device.is_null() {
        libc::fprintf(c_stderr(), cstr!("vtty_parse_serial_option: unable to copy string\n"));
        return -1;
    }

    (*option).baudrate = if count > 1 { libc::atoi(array[1]) } else { 9600 };
    (*option).databits = if count > 2 { libc::atoi(array[2]) } else { 8 };

    if count > 3 {
        match *array[3] as u8 {
            b'o' | b'O' => (*option).parity = 1, // odd
            b'e' | b'E' => (*option).parity = 2, // even
            _ => (*option).parity = 0,           // none
        }
    } else {
        (*option).parity = 0;
    }

    (*option).stopbits = if count > 4 { libc::atoi(array[4]) } else { 1 };
    (*option).hwflow = if count > 5 { libc::atoi(array[5]) } else { 0 };
    0
}
