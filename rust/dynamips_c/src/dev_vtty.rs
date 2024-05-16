//! Virtual console TTY.
//!
//! "Interactive" part idea by Mtve.
//! TCP console added by Mtve.
//! Serial console by Peter Ross (suxen_drol@hotmail.com)

use crate::prelude::*;
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

// Definitions for the TELNET protocol from arpa/telnet.h
/// interpret as command:
const IAC: u8 = 255;
/// you are not to use option
const DONT: u8 = 254;
/// please, you use option
const DO: u8 = 253;
/// I will use option
const WILL: u8 = 251;
/// echo
const TELOPT_ECHO: u8 = 1;
/// suppress go ahead
const TELOPT_SGA: u8 = 3;
/// terminal type
const TELOPT_TTYPE: u8 = 24;
/// Linemode option
const TELOPT_LINEMODE: u8 = 34;

#[no_mangle] // TODO private
pub static mut ctrl_code_ok: c_int = 1;

#[no_mangle] // TODO private
pub static mut telnet_message_ok: c_int = 1;

pub static mut tios: libc::termios = unsafe { zeroed::<_>() };
pub static mut tios_orig: libc::termios = unsafe { zeroed::<_>() };

/// Allow the user to disable the CTRL code for the monitor interface
#[no_mangle]
pub unsafe extern "C" fn vtty_set_ctrlhandler(n: c_int) {
    ctrl_code_ok = n;
}

/// Allow the user to disable the telnet message for AUX and CONSOLE
#[no_mangle]
pub unsafe extern "C" fn vtty_set_telnetmsg(n: c_int) {
    telnet_message_ok = n;
}

/// Send Telnet command: WILL TELOPT_ECHO
#[no_mangle] // TODO private
pub unsafe extern "C" fn vtty_telnet_will_echo(fd: c_int) {
    let cmd: [u8; 3] = [IAC, WILL, TELOPT_ECHO];
    libc::write(fd, cmd.as_ptr().cast::<_>(), cmd.len());
}

/* Send Telnet command: Suppress Go-Ahead */
#[no_mangle] // TODO private
pub unsafe extern "C" fn vtty_telnet_will_suppress_go_ahead(fd: c_int) {
    let cmd: [u8; 3] = [IAC, WILL, TELOPT_SGA];
    libc::write(fd, cmd.as_ptr().cast::<_>(), cmd.len());
}

/// Send Telnet command: Don't use linemode
#[no_mangle] // TODO private
pub unsafe extern "C" fn vtty_telnet_dont_linemode(fd: c_int) {
    let cmd: [u8; 3] = [IAC, DONT, TELOPT_LINEMODE];
    libc::write(fd, cmd.as_ptr().cast::<_>(), cmd.len());
}

/// Send Telnet command: does the client support terminal type message?
#[no_mangle] // TODO private
pub unsafe extern "C" fn vtty_telnet_do_ttype(fd: c_int) {
    let cmd: [u8; 3] = [IAC, DO, TELOPT_TTYPE];
    libc::write(fd, cmd.as_ptr().cast::<_>(), cmd.len());
}

/// Restore TTY original settings
extern "C" fn vtty_term_reset() {
    unsafe {
        libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, addr_of_mut!(tios_orig));
    }
}

/// Initialize real TTY
#[no_mangle] // TODO private
pub unsafe extern "C" fn vtty_term_init() {
    libc::tcgetattr(libc::STDIN_FILENO, addr_of_mut!(tios));

    libc::memcpy(addr_of_mut!(tios_orig).cast::<_>(), addr_of!(tios).cast::<_>(), size_of::<libc::termios>());
    libc::atexit(vtty_term_reset);

    tios.c_cc[libc::VTIME] = 0;
    tios.c_cc[libc::VMIN] = 1;

    // Disable Ctrl-C, Ctrl-S, Ctrl-Q and Ctrl-Z
    tios.c_cc[libc::VINTR] = 0;
    tios.c_cc[libc::VSTART] = 0;
    tios.c_cc[libc::VSTOP] = 0;
    tios.c_cc[libc::VSUSP] = 0;

    tios.c_lflag &= !(libc::ICANON | libc::ECHO);
    tios.c_iflag &= !libc::ICRNL;
    libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, addr_of_mut!(tios));
    libc::tcflush(libc::STDIN_FILENO, libc::TCIFLUSH);
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
