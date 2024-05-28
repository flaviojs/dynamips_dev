//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Virtual console TTY.
//!
//! "Interactive" part idea by Mtve.
//! TCP console added by Mtve.
//! Serial console by Peter Ross (suxen_drol@hotmail.com)

use crate::_private::*;
use crate::dynamips::*;
use crate::utils::*;
use crate::vm::*;

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

/// 4 Kb should be enough for a keyboard buffer
pub const VTTY_BUFFER_SIZE: usize = 4096;

/// Maximum listening socket number
pub const VTTY_MAX_FD: usize = 10;

/// Virtual TTY structure
#[repr(C)]
#[derive(Copy, Clone)]
pub struct virtual_tty {
    pub vm: *mut vm_instance_t,
    pub name: *mut c_char,
    pub type_: c_int,
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
    pub fd_pool: fd_pool_t,
    pub read_notifier: Option<unsafe extern "C" fn(arg1: *mut vtty_t)>,
    pub replay_buffer: [u_char; VTTY_BUFFER_SIZE],
    pub replay_ptr: u_int,
    pub replay_full: u_char,
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

/// Wait for a TCP connection
unsafe fn vtty_tcp_conn_wait_ipv4_ipv6(vtty: *mut vtty_t) -> c_int {
    let mut hints: libc::addrinfo = zeroed::<_>();
    let port_str: [c_char; 20] = [0; 20];
    let one: c_int = 1;

    for i in 0..VTTY_MAX_FD {
        (*vtty).fd_array[i] = -1;
    }

    libc::memset(addr_of_mut!(hints).cast::<_>(), 0, size_of::<libc::addrinfo>());
    hints.ai_family = libc::PF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;
    hints.ai_flags = libc::AI_PASSIVE;

    libc::snprintf(port_str.as_ptr().cast_mut(), 20, cstr!("%d"), (*vtty).tcp_port);

    // Try to use the console binding address first, then fallback to the global binding address
    let addr: *const c_char = if !console_binding_addr.is_null() && libc::strlen(console_binding_addr) != 0 {
        console_binding_addr
    } else if !binding_addr.is_null() && libc::strlen(binding_addr) != 0 {
        binding_addr
    } else {
        cstr!("127.0.0.1")
    };

    let mut res0: *mut libc::addrinfo = null_mut();
    if libc::getaddrinfo(addr, port_str.as_ptr(), addr_of!(hints), addr_of_mut!(res0)) != 0 {
        libc::perror(cstr!("vtty_tcp_conn_wait_ipv4_ipv6: getaddrinfo"));
        return -1;
    }

    let mut nsock: usize = 0;
    let mut next_res: *mut libc::addrinfo = res0;
    while !next_res.is_null() && nsock < VTTY_MAX_FD {
        let res = next_res;
        next_res = (*res).ai_next;

        if (*res).ai_family != libc::PF_INET && (*res).ai_family != libc::PF_INET6 {
            continue;
        }

        (*vtty).fd_array[nsock] = libc::socket((*res).ai_family, (*res).ai_socktype, (*res).ai_protocol);

        if (*vtty).fd_array[nsock] < 0 {
            continue;
        }

        if libc::setsockopt((*vtty).fd_array[nsock], libc::SOL_SOCKET, libc::SO_REUSEADDR, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
            libc::perror(cstr!("vtty_tcp_conn_wait_ipv4_ipv6: setsockopt(SO_REUSEADDR)"));
        }

        if libc::setsockopt((*vtty).fd_array[nsock], libc::SOL_SOCKET, libc::SO_KEEPALIVE, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
            libc::perror(cstr!("vtty_tcp_conn_wait_ipv4_ipv6: setsockopt(SO_KEEPALIVE)"));
        }

        // Send telnet packets asap. Dont wait to fill packets up
        if libc::setsockopt((*vtty).fd_array[nsock], libc::IPPROTO_TCP, libc::TCP_NODELAY, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
            libc::perror(cstr!("vtty_tcp_conn_wait_ipv4_ipv6: setsockopt(TCP_NODELAY)"));
        }

        if libc::bind((*vtty).fd_array[nsock], (*res).ai_addr, (*res).ai_addrlen) < 0 || libc::listen((*vtty).fd_array[nsock], 1) < 0 {
            libc::close((*vtty).fd_array[nsock]);
            (*vtty).fd_array[nsock] = -1;
            continue;
        }

        let proto: *mut c_char = if (*res).ai_family == libc::PF_INET6 { cstr!("IPv6") } else { cstr!("IPv4") };
        vm_log!((*vtty).vm, cstr!("VTTY"), cstr!("%s: waiting connection on tcp port %d for protocol %s (FD %d)\n"), (*vtty).name, (*vtty).tcp_port, proto, (*vtty).fd_array[nsock]);

        nsock += 1;
    }

    libc::freeaddrinfo(res0);
    nsock as c_int
}

/// Wait for a TCP connection
unsafe fn vtty_tcp_conn_wait_ipv4(vtty: *mut vtty_t) -> c_int {
    let mut serv: libc::sockaddr_in = zeroed::<_>();
    let one: c_int = 1;

    for i in 0..VTTY_MAX_FD {
        (*vtty).fd_array[i] = -1;
    }

    (*vtty).fd_array[0] = libc::socket(libc::PF_INET, libc::SOCK_STREAM, 0);
    if (*vtty).fd_array[0] < 0 {
        libc::perror(cstr!("vtty_tcp_conn_wait_ipv4: socket"));
        return -1;
    }

    if libc::setsockopt((*vtty).fd_array[0], libc::SOL_SOCKET, libc::SO_REUSEADDR, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
        libc::perror(cstr!("vtty_tcp_conn_wait_ipv4: setsockopt(SO_REUSEADDR)"));
        libc::close((*vtty).fd_array[0]);
        (*vtty).fd_array[0] = -1;
        return -1;
    }

    if libc::setsockopt((*vtty).fd_array[0], libc::SOL_SOCKET, libc::SO_KEEPALIVE, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
        libc::perror(cstr!("vtty_tcp_conn_wait_ipv4: setsockopt(SO_KEEPALIVE)"));
        libc::close((*vtty).fd_array[0]);
        (*vtty).fd_array[0] = -1;
        return -1;
    }

    // Send telnet packets asap. Dont wait to fill packets up
    if libc::setsockopt((*vtty).fd_array[0], libc::IPPROTO_TCP, libc::TCP_NODELAY, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
        libc::perror(cstr!("vtty_tcp_conn_wait_ipv4: setsockopt(TCP_NODELAY)"));
        libc::close((*vtty).fd_array[0]);
        (*vtty).fd_array[0] = -1;
        return -1;
    }

    libc::memset(addr_of_mut!(serv).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
    serv.sin_family = libc::AF_INET.try_into().unwrap();
    serv.sin_addr.s_addr = libc::INADDR_ANY.to_be();
    serv.sin_port = ((*vtty).tcp_port as u16).to_be();

    if libc::bind((*vtty).fd_array[0], addr_of!(serv).cast::<_>(), size_of::<libc::sockaddr_in>() as libc::socklen_t) < 0 {
        libc::perror(cstr!("vtty_tcp_waitcon: bind"));
        libc::close((*vtty).fd_array[0]);
        (*vtty).fd_array[0] = -1;
        return -1;
    }

    if libc::listen((*vtty).fd_array[0], 1) < 0 {
        libc::perror(cstr!("vtty_tcp_waitcon: listen"));
        libc::close((*vtty).fd_array[0]);
        (*vtty).fd_array[0] = -1;
        return -1;
    }

    vm_log!((*vtty).vm, cstr!("VTTY"), cstr!("%s: waiting connection on tcp port %d (FD %d)\n"), (*vtty).name, (*vtty).tcp_port, (*vtty).fd_array[0]);

    1
}

/// Wait for a TCP connection
#[no_mangle] // TODO private
pub unsafe extern "C" fn vtty_tcp_conn_wait(vtty: *mut vtty_t) -> c_int {
    if cfg!(feature = "ENABLE_IPV6") {
        vtty_tcp_conn_wait_ipv4_ipv6(vtty)
    } else {
        vtty_tcp_conn_wait_ipv4(vtty)
    }
}
