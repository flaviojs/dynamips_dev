//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Virtual console TTY.
//!
//! "Interactive" part idea by Mtve.
//! TCP console added by Mtve.
//! Serial console by Peter Ross (suxen_drol@hotmail.com)

use crate::_private::*;
use crate::cisco_card::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::mips64::*;
use crate::pci_dev::*;
use crate::ppc32::*;
use crate::utils::*;
use crate::vm::*;
use std::cmp::max;

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

// Definitions for the TELNET protocol from arpa/telnet.h
/// interpret as command:
const IAC: u_char = 255;
/// you are not to use option
const DONT: u_char = 254;
/// please, you use option
const DO: u_char = 253;
/// I won't use option
const WONT: u_char = 252;
/// I will use option
const WILL: u_char = 251;
/// interpret as subnegotiation
const SB: u_char = 250;
/// end sub negotiation
const SE: u_char = 240;
/// echo
const TELOPT_ECHO: u_char = 1;
/// suppress go ahead
const TELOPT_SGA: u_char = 3;
/// terminal type
const TELOPT_TTYPE: u_char = 24;
/// Linemode option
const TELOPT_LINEMODE: u_char = 34;
/// option is...
const TELQUAL_IS: u_char = 0;
/// send option
const TELQUAL_SEND: u_char = 1;

// VTTY list
static mut vtty_list_mutex: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
static mut vtty_list: *mut vtty_t = null_mut();
static mut vtty_thread: libc::pthread_t = 0;

unsafe fn VTTY_LIST_LOCK() {
    libc::pthread_mutex_lock(addr_of_mut!(vtty_list_mutex));
}
unsafe fn VTTY_LIST_UNLOCK() {
    libc::pthread_mutex_unlock(addr_of_mut!(vtty_list_mutex));
}

static mut tios: libc::termios = unsafe { zeroed::<_>() };
static mut tios_orig: libc::termios = unsafe { zeroed::<_>() };

static mut ctrl_code_ok: c_int = 1;
static mut telnet_message_ok: c_int = 1;

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
unsafe fn vtty_telnet_will_echo(fd: c_int) {
    let cmd: [u_char; 3] = [IAC, WILL, TELOPT_ECHO];
    libc::write(fd, cmd.as_ptr().cast::<_>(), cmd.len());
}

/* Send Telnet command: Suppress Go-Ahead */
unsafe fn vtty_telnet_will_suppress_go_ahead(fd: c_int) {
    let cmd: [u_char; 3] = [IAC, WILL, TELOPT_SGA];
    libc::write(fd, cmd.as_ptr().cast::<_>(), cmd.len());
}

/// Send Telnet command: Don't use linemode
unsafe fn vtty_telnet_dont_linemode(fd: c_int) {
    let cmd: [u_char; 3] = [IAC, DONT, TELOPT_LINEMODE];
    libc::write(fd, cmd.as_ptr().cast::<_>(), cmd.len());
}

/// Send Telnet command: does the client support terminal type message?
unsafe fn vtty_telnet_do_ttype(fd: c_int) {
    let cmd: [u_char; 3] = [IAC, DO, TELOPT_TTYPE];
    libc::write(fd, cmd.as_ptr().cast::<_>(), cmd.len());
}

/// Restore TTY original settings
extern "C" fn vtty_term_reset() {
    unsafe {
        libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, addr_of_mut!(tios_orig));
    }
}

/// Initialize real TTY
unsafe fn vtty_term_init() {
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

/// Wait for a TCP connection
unsafe fn vtty_tcp_conn_wait(vtty: *mut vtty_t) -> c_int {
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

        libc::snprintf(port_str.as_ptr().cast_mut(), port_str.len(), cstr!("%d"), (*vtty).tcp_port);

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
            libc::perror(cstr!("vtty_tcp_waitcon: getaddrinfo"));
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
                libc::perror(cstr!("vtty_tcp_waitcon: setsockopt(SO_REUSEADDR)"));
            }

            if libc::setsockopt((*vtty).fd_array[nsock], libc::SOL_SOCKET, libc::SO_KEEPALIVE, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
                libc::perror(cstr!("vtty_tcp_waitcon: setsockopt(SO_KEEPALIVE)"));
            }

            // Send telnet packets asap. Dont wait to fill packets up
            if libc::setsockopt((*vtty).fd_array[nsock], libc::IPPROTO_TCP, libc::TCP_NODELAY, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
                libc::perror(cstr!("vtty_tcp_waitcon: setsockopt(TCP_NODELAY)"));
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
            libc::perror(cstr!("vtty_tcp_waitcon: socket"));
            return -1;
        }

        if libc::setsockopt((*vtty).fd_array[0], libc::SOL_SOCKET, libc::SO_REUSEADDR, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
            libc::perror(cstr!("vtty_tcp_waitcon: setsockopt(SO_REUSEADDR)"));
            libc::close((*vtty).fd_array[0]);
            (*vtty).fd_array[0] = -1;
            return -1;
        }

        if libc::setsockopt((*vtty).fd_array[0], libc::SOL_SOCKET, libc::SO_KEEPALIVE, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
            libc::perror(cstr!("vtty_tcp_waitcon: setsockopt(SO_KEEPALIVE)"));
            libc::close((*vtty).fd_array[0]);
            (*vtty).fd_array[0] = -1;
            return -1;
        }

        // Send telnet packets asap. Dont wait to fill packets up
        if libc::setsockopt((*vtty).fd_array[0], libc::IPPROTO_TCP, libc::TCP_NODELAY, addr_of!(one).cast::<_>(), size_of::<c_int>() as libc::socklen_t) < 0 {
            libc::perror(cstr!("vtty_tcp_waitcon: setsockopt(TCP_NODELAY)"));
            libc::close((*vtty).fd_array[0]);
            (*vtty).fd_array[0] = -1;
            return -1;
        }

        libc::memset(addr_of_mut!(serv).cast::<_>(), 0, size_of::<libc::sockaddr_in>());
        serv.sin_family = libc::AF_INET as _;
        serv.sin_addr.s_addr = htonl(libc::INADDR_ANY);
        serv.sin_port = htons((*vtty).tcp_port as u16);

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
    if cfg!(feature = "ENABLE_IPV6") {
        vtty_tcp_conn_wait_ipv4_ipv6(vtty)
    } else {
        vtty_tcp_conn_wait_ipv4(vtty)
    }
}

/// Accept a TCP connection
unsafe fn vtty_tcp_conn_accept(vtty: *mut vtty_t, nsock: c_int) -> c_int {
    let mut i: size_t;

    let mut fd_slot: *mut c_int = null_mut();
    if fd_pool_get_free_slot(addr_of_mut!((*vtty).fd_pool), addr_of_mut!(fd_slot)) < 0 {
        vm_error!((*vtty).vm, cstr!("unable to create a new VTTY TCP connection\n"));
        return -1;
    }

    let fd: c_int = libc::accept((*vtty).fd_array[nsock as usize], null_mut(), null_mut());
    if fd < 0 {
        vm_error!((*vtty).vm, cstr!("vtty_tcp_conn_accept: accept on port %d failed %s\n"), (*vtty).tcp_port, libc::strerror(c_errno()));
        return -1;
    }

    // Register the new FD
    *fd_slot = fd;

    vm_log!((*vtty).vm, cstr!("VTTY"), cstr!("%s is now connected (accept_fd=%d,conn_fd=%d)\n"), (*vtty).name, (*vtty).fd_array[nsock as usize], fd);

    // Adapt Telnet settings
    if (*vtty).terminal_support != 0 {
        vtty_telnet_do_ttype(fd);
        vtty_telnet_will_echo(fd);
        vtty_telnet_will_suppress_go_ahead(fd);
        vtty_telnet_dont_linemode(fd);
        (*vtty).input_state = VTTY_INPUT_TEXT;
    }

    if telnet_message_ok == 1 {
        fd_printf!(fd, 0, cstr!("Connected to Dynamips VM \"%s\" (ID %u, type %s) - %s\r\nPress ENTER to get the prompt.\r\n"), (*(*vtty).vm).name, (*(*vtty).vm).instance_id, vm_get_type((*vtty).vm), (*vtty).name);
        // replay old text
        if (*vtty).replay_full != 0 {
            i = (*vtty).replay_ptr as size_t;
            while i < VTTY_BUFFER_SIZE {
                let n: ssize_t = libc::send(fd, addr_of_mut!((*vtty).replay_buffer[i]).cast::<_>(), VTTY_BUFFER_SIZE - i, 0);
                if n < 0 {
                    libc::perror(cstr!("vtty_tcp_conn_accept: send"));
                    break;
                }
                i += n as size_t;
            }
        }
        i = 0;
        while i < (*vtty).replay_ptr as size_t {
            let n: ssize_t = libc::send(fd, addr_of_mut!((*vtty).replay_buffer[i]).cast::<_>(), (*vtty).replay_ptr as usize - i, 0);
            if n < 0 {
                libc::perror(cstr!("vtty_tcp_conn_accept: send"));
                break;
            }
            i += n as size_t;
        }
        // warn if not running
        if (*(*vtty).vm).status != VM_STATUS_RUNNING {
            fd_printf!(fd, 0, cstr!("\r\n!!! WARNING - VM is not running, will be unresponsive (status=%d) !!!\r\n"), (*(*vtty).vm).status);
        }
        vtty_flush(vtty);
    }
    0
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

/// Setup serial port, return 0 if success.
unsafe fn vtty_serial_setup(vtty: *mut vtty_t, option: *const vtty_serial_option_t) -> c_int {
    let mut tio: libc::termios = zeroed::<_>();

    if libc::tcgetattr((*vtty).fd_array[0], addr_of_mut!(tio)) != 0 {
        libc::fprintf(c_stderr(), cstr!("error: tcgetattr failed\n"));
        return -1;
    }

    #[cfg(has_libc_cfmakeraw)]
    libc::cfmakeraw(addr_of_mut!(tio));
    #[cfg(not(has_libc_cfmakeraw))]
    {
        // if defined(__CYGWIN__) || defined(SUNOS)
        unsafe fn cfmakeraw(termios_p: *mut libc::termios) {
            (*termios_p).c_iflag &= !(libc::IGNBRK | libc::BRKINT | libc::PARMRK | libc::ISTRIP | libc::INLCR | libc::IGNCR | libc::ICRNL | libc::IXON);
            (*termios_p).c_oflag &= !libc::OPOST;
            (*termios_p).c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
            (*termios_p).c_cflag &= !(libc::CSIZE | libc::PARENB);
            (*termios_p).c_cflag |= libc::CS8;
        }
        cfmakeraw(addr_of_mut!(tio));
    }

    tio.c_cflag = libc::CLOCAL // ignore modem control lines
        ;

    tio.c_cflag &= !libc::CREAD;
    tio.c_cflag |= libc::CREAD;

    let tio_baudrate: libc::speed_t = match (*option).baudrate {
        50 => libc::B50,
        75 => libc::B75,
        110 => libc::B110,
        134 => libc::B134,
        150 => libc::B150,
        200 => libc::B200,
        300 => libc::B300,
        600 => libc::B600,
        1200 => libc::B1200,
        1800 => libc::B1800,
        2400 => libc::B2400,
        4800 => libc::B4800,
        9600 => libc::B9600,
        19200 => libc::B19200,
        38400 => libc::B38400,
        57600 => libc::B57600,
        #[cfg(has_libc_B76800)]
        76800 => libc::B76800,
        115200 => libc::B115200,
        #[cfg(has_libc_B230400)]
        230400 => libc::B230400,
        _ => {
            libc::fprintf(c_stderr(), cstr!("error: unsupported baudrate\n"));
            return -1;
        }
    };

    libc::cfsetospeed(addr_of_mut!(tio), tio_baudrate);
    libc::cfsetispeed(addr_of_mut!(tio), tio_baudrate);

    tio.c_cflag &= !libc::CSIZE; // clear size flag
    match (*option).databits {
        5 => tio.c_cflag |= libc::CS5,
        6 => tio.c_cflag |= libc::CS6,
        7 => tio.c_cflag |= libc::CS7,
        8 => tio.c_cflag |= libc::CS8,
        _ => {
            libc::fprintf(c_stderr(), cstr!("error: unsupported databits\n"));
            return -1;
        }
    }

    tio.c_iflag &= !libc::INPCK; // clear parity flag
    tio.c_cflag &= !(libc::PARENB | libc::PARODD);
    match (*option).parity {
        0 => {}
        2 => {
            // even
            tio.c_iflag |= libc::INPCK;
            tio.c_cflag |= libc::PARENB;
        }
        1 => {
            /* odd */
            tio.c_iflag |= libc::INPCK;
            tio.c_cflag |= libc::PARENB | libc::PARODD;
        }
        _ => {
            libc::fprintf(c_stderr(), cstr!("error: unsupported parity\n"));
            return -1;
        }
    }

    tio.c_cflag &= !libc::CSTOPB; // clear stop flag
    match (*option).stopbits {
        1 => {}
        2 => tio.c_cflag |= libc::CSTOPB,
        _ => {
            libc::fprintf(c_stderr(), cstr!("error: unsupported stopbits\n"));
            return -1;
        }
    }

    #[cfg(has_libc_CRTSCTS)]
    {
        tio.c_cflag &= !libc::CRTSCTS;
    }
    #[cfg(has_libc_CNEW_RTSCTS)]
    {
        tio.c_cflag &= !libc::CNEW_RTSCTS;
    }
    if (*option).hwflow != 0 {
        #[cfg(has_libc_CRTSCTS)]
        {
            tio.c_cflag |= libc::CRTSCTS;
        }
        #[cfg(has_libc_CNEW_RTSCTS)]
        {
            tio.c_cflag |= libc::CNEW_RTSCTS;
        }
    }

    tio.c_cc[libc::VTIME] = 0;
    tio.c_cc[libc::VMIN] = 1; // block read() until one character is available

    if false {
        // not neccessary unless O_NONBLOCK used
        if libc::fcntl((*vtty).fd_array[0], libc::F_SETFL, 0) != 0 {
            // enable blocking mode
            libc::fprintf(c_stderr(), cstr!("error: fnctl F_SETFL failed\n"));
            return -1;
        }
    }

    if libc::tcflush((*vtty).fd_array[0], libc::TCIOFLUSH) != 0 {
        libc::fprintf(c_stderr(), cstr!("error: tcflush failed\n"));
        return -1;
    }

    if libc::tcsetattr((*vtty).fd_array[0], libc::TCSANOW, addr_of!(tio)) != 0 {
        libc::fprintf(c_stderr(), cstr!("error: tcsetattr failed\n"));
        return -1;
    }

    0
}

/// Create a virtual tty
#[no_mangle]
pub unsafe extern "C" fn vtty_create(vm: *mut vm_instance_t, name: *mut c_char, type_: c_int, tcp_port: c_int, option: *const vtty_serial_option_t) -> *mut vtty_t {
    let vtty: *mut vtty_t = libc::malloc(size_of::<vtty_t>()).cast::<_>();
    if vtty.is_null() {
        libc::fprintf(c_stderr(), cstr!("VTTY: unable to create new virtual tty.\n"));
        return null_mut();
    }
    libc::memset(vtty.cast::<_>(), 0, size_of::<vtty_t>());
    (*vtty).name = name;
    (*vtty).r#type = type_;
    (*vtty).vm = vm;
    (*vtty).fd_count = 0;
    libc::pthread_mutex_init(addr_of_mut!((*vtty).lock), null_mut());
    (*vtty).terminal_support = 1;
    (*vtty).input_state = VTTY_INPUT_TEXT;
    fd_pool_init(addr_of_mut!((*vtty).fd_pool));
    for i in 0..VTTY_MAX_FD {
        (*vtty).fd_array[i] = -1;
    }

    match (*vtty).r#type {
        VTTY_TYPE_NONE => {}

        VTTY_TYPE_TERM => {
            vtty_term_init();
            (*vtty).fd_array[0] = libc::STDIN_FILENO;
        }

        VTTY_TYPE_TCP => {
            (*vtty).tcp_port = tcp_port;
            (*vtty).fd_count = vtty_tcp_conn_wait(vtty);
        }

        VTTY_TYPE_SERIAL => {
            (*vtty).fd_array[0] = libc::open((*option).device, libc::O_RDWR);
            if (*vtty).fd_array[0] < 0 {
                libc::fprintf(c_stderr(), cstr!("VTTY: open failed\n"));
                libc::free(vtty.cast::<_>());
                return null_mut();
            }
            if vtty_serial_setup(vtty, option) != 0 {
                libc::fprintf(c_stderr(), cstr!("VTTY: setup failed\n"));
                libc::close((*vtty).fd_array[0]);
                libc::free(vtty.cast::<_>());
                return null_mut();
            }
            (*vtty).terminal_support = 0;
        }

        _ => {
            libc::fprintf(c_stderr(), cstr!("tty_create: bad vtty type %d\n"), (*vtty).r#type);
            libc::free(vtty.cast::<_>());
            return null_mut();
        }
    }

    // Add this new VTTY to the list
    VTTY_LIST_LOCK();
    (*vtty).next = vtty_list;
    (*vtty).pprev = addr_of_mut!(vtty_list);

    if !vtty_list.is_null() {
        (*vtty_list).pprev = addr_of_mut!((*vtty).next);
    }

    vtty_list = vtty;
    VTTY_LIST_UNLOCK();
    vtty
}

/// Delete a virtual tty
#[no_mangle]
pub unsafe extern "C" fn vtty_delete(vtty: *mut vtty_t) {
    if !vtty.is_null() {
        VTTY_LIST_LOCK();
        if !(*vtty).pprev.is_null() {
            if !(*vtty).next.is_null() {
                (*(*vtty).next).pprev = (*vtty).pprev;
            }
            *(*vtty).pprev = (*vtty).next;
        }
        VTTY_LIST_UNLOCK();

        match (*vtty).r#type {
            VTTY_TYPE_TCP => {
                for i in 0..(*vtty).fd_count as usize {
                    if (*vtty).fd_array[i] != -1 {
                        vm_log!((*vtty).vm, cstr!("VTTY"), cstr!("%s: closing FD %d\n"), (*vtty).name, (*vtty).fd_array[i]);
                        libc::close((*vtty).fd_array[i]);
                    }
                }

                fd_pool_free(addr_of_mut!((*vtty).fd_pool));
                (*vtty).fd_count = 0;
            }

            _ => {
                // We don't close FD 0 since it is stdin
                if (*vtty).fd_array[0] > 0 {
                    vm_log!((*vtty).vm, cstr!("VTTY"), cstr!("%s: closing FD %d\n"), (*vtty).name, (*vtty).fd_array[0]);
                    libc::close((*vtty).fd_array[0]);
                }
            }
        }
        libc::free(vtty.cast::<_>());
    }
}

/// Store a character in the FIFO buffer
unsafe fn vtty_store(vtty: *mut vtty_t, c: u_char) -> c_int {
    VTTY_LOCK(vtty);
    let mut nwptr: u_int = (*vtty).write_ptr + 1;
    if nwptr == VTTY_BUFFER_SIZE as u_int {
        nwptr = 0;
    }

    if nwptr == (*vtty).read_ptr {
        VTTY_UNLOCK(vtty);
        return -1;
    }

    (*vtty).buffer[(*vtty).write_ptr as usize] = c;
    (*vtty).write_ptr = nwptr;
    VTTY_UNLOCK(vtty);
    0
}

/// Store arbritary data in the FIFO buffer
#[no_mangle]
pub unsafe extern "C" fn vtty_store_data(vtty: *mut vtty_t, data: *mut c_char, len: c_int) -> c_int {
    if vtty.is_null() || data.is_null() || len < 0 {
        return -1; // invalid argument
    }

    let mut bytes: c_int = 0;
    while bytes < len {
        if vtty_store(vtty, *data.offset(bytes as isize) as c_uchar) == -1 {
            break;
        }
        bytes += 1;
    }

    (*vtty).input_pending = TRUE;
    bytes
}

/// Store CTRL+C in buffer
#[no_mangle]
pub unsafe extern "C" fn vtty_store_ctrlc(vtty: *mut vtty_t) -> c_int {
    if !vtty.is_null() {
        vtty_store(vtty, 0x03);
    }
    0
}

/// Read a character from the terminal.
unsafe fn vtty_term_read(vtty: *mut vtty_t) -> c_int {
    let mut c: u_char = 0;

    if libc::read((*vtty).fd_array[0], addr_of_mut!(c).cast::<_>(), 1) == 1 {
        return c.into();
    }

    libc::perror(cstr!("read from vtty failed"));
    -1
}

/// Read a character from the TCP connection.
unsafe fn vtty_tcp_read(_vtty: *mut vtty_t, fd_slot: *mut c_int) -> c_int {
    let fd: c_int = *fd_slot;
    let mut c: u_char = 0;

    if libc::read(fd, addr_of_mut!(c).cast::<_>(), 1) == 1 {
        return c.into();
    }

    // problem with the connection
    libc::shutdown(fd, 2);
    libc::close(fd);
    *fd_slot = -1;

    // Shouldn't happen...
    -1
}

/// Read a character from the virtual TTY.
///
/// If the VTTY is a TCP connection, restart it in case of error.
unsafe fn vtty_read(vtty: *mut vtty_t, fd_slot: *mut c_int) -> c_int {
    match (*vtty).r#type {
        VTTY_TYPE_TERM | VTTY_TYPE_SERIAL => vtty_term_read(vtty),
        VTTY_TYPE_TCP => vtty_tcp_read(vtty, fd_slot),
        _ => {
            libc::fprintf(c_stderr(), cstr!("vtty_read: bad vtty type %d\n"), (*vtty).r#type);
            -1
        }
    }
}

/// Remote control for MIPS64 processors
unsafe fn remote_control_mips64(_vtty: *mut vtty_t, c: c_char, cpu: *mut cpu_mips_t) -> c_int {
    match c as u8 {
        // Show information about JIT compiled pages
        b'b' => {
            libc::printf(cstr!("\nCPU0: %u JIT compiled pages [Exec Area Pages: %lu/%lu]\n"), (*cpu).compiled_pages, (*cpu).exec_page_alloc as u_long, (*cpu).exec_page_count as u_long);
        }

        // Non-JIT mode statistics
        b'j' => {
            mips64_dump_stats(cpu);
        }

        _ => {
            return FALSE;
        }
    }

    TRUE
}

/// Remote control for PPC32 processors
unsafe fn remote_control_ppc32(_vtty: *mut vtty_t, c: c_char, cpu: *mut cpu_ppc_t) -> c_int {
    match c as u8 {
        // Show information about JIT compiled pages
        b'b' => {
            libc::printf(cstr!("\nCPU0: %u JIT compiled pages [Exec Area Pages: %lu/%lu]\n"), (*cpu).compiled_pages, (*cpu).exec_page_alloc as u_long, (*cpu).exec_page_count as u_long);
        }

        // Non-JIT mode statistics
        b'j' => {
            ppc32_dump_stats(cpu);
        }

        _ => {
            return FALSE;
        }
    }

    TRUE
}

/// Process remote control char
unsafe fn remote_control(vtty: *mut vtty_t, c: u_char) {
    let vm: *mut vm_instance_t = (*vtty).vm;
    let cpu0: *mut cpu_gen_t = (*vm).boot_cpu;

    // Specific commands for the different CPU models
    if !cpu0.is_null() {
        match (*cpu0).r#type {
            CPU_TYPE_MIPS64 => {
                if remote_control_mips64(vtty, c as c_char, CPU_MIPS64(cpu0)) != 0 {
                    return;
                }
            }
            CPU_TYPE_PPC32 => {
                if remote_control_ppc32(vtty, c as c_char, CPU_PPC32(cpu0)) != 0 {
                    return;
                }
            }
            _ => {}
        }
    }

    match c {
        // Show the object list
        b'o' => {
            vm_object_dump(vm);
        }

        // Stop the MIPS VM
        b'q' => {
            (*vm).status = VM_STATUS_SHUTDOWN;
        }

        // Reboot the C7200
        b'k' => {
            #[cfg(if_0)]
            {
                if (*vm).type_ == VM_TYPE_C7200 {
                    c7200_boot_ios(VM_C7200(vm));
                }
            }
        }

        // Show the device list
        b'd' => {
            dev_show_list(vm);
            pci_dev_show_list((*vm).pci_bus[0]);
            pci_dev_show_list((*vm).pci_bus[1]);
        }

        // Show info about Port Adapters or Network Modules
        b'p' => {
            vm_slot_show_all_info(vm);
        }

        // Dump the MIPS registers
        b'r' => {
            if !cpu0.is_null() {
                (*cpu0).reg_dump.unwrap()(cpu0);
            }
        }

        // Dump the latest memory accesses
        b'm' => {
            if !cpu0.is_null() {
                memlog_dump(cpu0);
            }
        }

        // Suspend CPU emulation
        b's' => {
            vm_suspend(vm);
        }

        // Resume CPU emulation
        b'u' => {
            vm_resume(vm);
        }

        // Dump the MMU information
        b't' => {
            if !cpu0.is_null() {
                (*cpu0).mmu_dump.unwrap()(cpu0);
            }
        }

        // Dump the MMU information (raw mode)
        b'z' => {
            if !cpu0.is_null() {
                (*cpu0).mmu_raw_dump.unwrap()(cpu0);
            }
        }

        // Memory translation cache statistics
        b'l' => {
            if !cpu0.is_null() {
                (*cpu0).mts_show_stats.unwrap()(cpu0);
            }
        }

        // Extract the configuration from the NVRAM
        b'c' => {
            vm_ios_save_config(vm);
        }

        // Determine an idle pointer counter
        b'i' => {
            if !cpu0.is_null() {
                (*cpu0).get_idling_pc.unwrap()(cpu0);
            }
        }

        // Experimentations / Tests
        b'x' => {
            #[cfg(if_0)]
            {
                if !cpu0.is_null() {
                    // IRQ triggering
                    vm_set_irq(vm, 6);
                    //(*CPU_MIPS64(cpu0)).irq_disable = TRUE;
                }
            }
            #[cfg(feature = "USE_UNSTABLE")]
            {
                tsg_show_stats();
            }
        }

        b'y' => {
            if !cpu0.is_null() {
                // IRQ clearing
                vm_clear_irq(vm, 6);
            }
        }

        // Twice Ctrl + ']' (0x1d, 29), or Alt-Gr + '*' (0xb3, 179)
        0x1d | 0xb3 => {
            vtty_store(vtty, c);
        }

        _ => {
            libc::printf(cstr!("\n\nInstance %s (ID %d)\n\n"), (*vm).name, (*vm).instance_id);

            #[rustfmt::skip]
            libc::printf(cstr!(concat!(
                "o     - Show the VM object list\n",
                "d     - Show the device list\n",
                "r     - Dump CPU registers\n",
                "t     - Dump MMU information\n",
                "z     - Dump MMU information (raw mode)\n",
                "m     - Dump the latest memory accesses\n",
                "s     - Suspend CPU emulation\n",
                "u     - Resume CPU emulation\n",
                "q     - Quit the emulator\n",
                "k     - Reboot the virtual machine\n",
                "b     - Show info about JIT compiled pages\n",
                "l     - MTS cache statistics\n",
                "c     - Write IOS configuration to disk\n",
                "j     - Non-JIT mode statistics\n",
                "i     - Determine an idling pointer counter\n",
                "x     - Experimentations (can crash the box!)\n",
                "^]    - Send ^]\n",
                "Other - This help\n"
            )));
        }
    }
}

/// Read a character (until one is available) and store it in buffer
unsafe fn vtty_read_and_store(vtty: *mut vtty_t, fd_slot: *mut c_int) {
    // wait until we get a character input
    let c: c_int = vtty_read(vtty, fd_slot);

    // if read error, do nothing
    if c < 0 {
        return;
    }

    // If something was read, make sure the handler is informed
    (*vtty).input_pending = TRUE;

    if (*vtty).terminal_support == 0 {
        vtty_store(vtty, c as u_char);
        return;
    }

    match (*vtty).input_state {
        VTTY_INPUT_TEXT => {
            match c {
                0x1b => {
                    (*vtty).input_state = VTTY_INPUT_VT1;
                    #[allow(clippy::needless_return)]
                    {
                        return;
                    }
                }

                // Ctrl + ']' (0x1d, 29), or Alt-Gr + '*' (0xb3, 179)
                0x1d | 0xb3 => {
                    if ctrl_code_ok == 1 {
                        (*vtty).input_state = VTTY_INPUT_REMOTE;
                    } else {
                        vtty_store(vtty, c as u_char);
                    }
                    #[allow(clippy::needless_return)]
                    {
                        return;
                    }
                }
                x if x == IAC as c_int => {
                    (*vtty).input_state = VTTY_INPUT_TELNET;
                    #[allow(clippy::needless_return)]
                    {
                        return;
                    }
                }
                // NULL - Must be ignored - generated by Linux telnet
                // LF (Line Feed) - Must be ignored on Windows platform
                0 | 10 => {
                    #[allow(clippy::needless_return)]
                    {
                        return;
                    }
                }
                // Store a standard character
                _ => {
                    vtty_store(vtty, c as u_char);
                    #[allow(clippy::needless_return)]
                    {
                        return;
                    }
                }
            }
        }

        VTTY_INPUT_VT1 => {
            match c {
                0x5b => {
                    (*vtty).input_state = VTTY_INPUT_VT2;
                    return;
                }
                _ => {
                    vtty_store(vtty, 0x1b);
                    vtty_store(vtty, c as u_char);
                }
            }
            (*vtty).input_state = VTTY_INPUT_TEXT;
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        VTTY_INPUT_VT2 => {
            match c {
                // Up Arrow
                0x41 => {
                    vtty_store(vtty, 16);
                }
                // Down Arrow
                0x42 => {
                    vtty_store(vtty, 14);
                }
                // Right Arrow
                0x43 => {
                    vtty_store(vtty, 6);
                }
                // Left Arrow
                0x44 => {
                    vtty_store(vtty, 2);
                }
                _ => {
                    vtty_store(vtty, 0x5b);
                    vtty_store(vtty, 0x1b);
                    vtty_store(vtty, c as u_char);
                }
            }
            (*vtty).input_state = VTTY_INPUT_TEXT;
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        VTTY_INPUT_REMOTE => {
            remote_control(vtty, c as u_char);
            (*vtty).input_state = VTTY_INPUT_TEXT;
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        VTTY_INPUT_TELNET => {
            (*vtty).telnet_cmd = c;
            match c {
                x if x == WILL as c_int || x == WONT as c_int || x == DO as c_int || x == DONT as c_int => {
                    (*vtty).input_state = VTTY_INPUT_TELNET_IYOU;
                    return;
                }
                x if x == SB as c_int => {
                    (*vtty).telnet_cmd = c;
                    (*vtty).input_state = VTTY_INPUT_TELNET_SB1;
                    return;
                }
                x if x == SE as c_int => {}
                x if x == IAC as c_int => {
                    vtty_store(vtty, IAC);
                }
                _ => {}
            }
            (*vtty).input_state = VTTY_INPUT_TEXT;
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        VTTY_INPUT_TELNET_IYOU => {
            (*vtty).telnet_opt = c;
            // if telnet client can support ttype, ask it to send ttype string
            if ((*vtty).telnet_cmd == WILL as c_int) && ((*vtty).telnet_opt == TELOPT_TTYPE as c_int) {
                vtty_put_char(vtty, IAC as c_char);
                vtty_put_char(vtty, SB as c_char);
                vtty_put_char(vtty, TELOPT_TTYPE as c_char);
                vtty_put_char(vtty, TELQUAL_SEND as c_char);
                vtty_put_char(vtty, IAC as c_char);
                vtty_put_char(vtty, SE as c_char);
            }
            (*vtty).input_state = VTTY_INPUT_TEXT;
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        VTTY_INPUT_TELNET_SB1 => {
            (*vtty).telnet_opt = c;
            (*vtty).input_state = VTTY_INPUT_TELNET_SB2;
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        VTTY_INPUT_TELNET_SB2 => {
            (*vtty).telnet_qual = c;
            if ((*vtty).telnet_opt == TELOPT_TTYPE as c_int) && ((*vtty).telnet_qual == TELQUAL_IS as c_int) {
                (*vtty).input_state = VTTY_INPUT_TELNET_SB_TTYPE;
            } else {
                (*vtty).input_state = VTTY_INPUT_TELNET_NEXT;
            }
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        VTTY_INPUT_TELNET_SB_TTYPE => {
            // parse ttype string: first char is sufficient
            // if client is xterm or vt, set the title bar
            if (c == b'x' as c_int) || (c == b'X' as c_int) || (c == b'v' as c_int) || (c == b'V' as c_int) {
                fd_printf!(*fd_slot, 0, cstr!("\033]0;%s\07"), (*(*vtty).vm).name);
            }
            (*vtty).input_state = VTTY_INPUT_TELNET_NEXT;
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        VTTY_INPUT_TELNET_NEXT => {
            // ignore all chars until next IAC
            if c == IAC as c_int {
                (*vtty).input_state = VTTY_INPUT_TELNET;
            }
            #[allow(clippy::needless_return)]
            {
                return;
            }
        }

        _ => {}
    }
}

/// Read a character from the buffer (-1 if the buffer is empty)
#[no_mangle]
pub unsafe extern "C" fn vtty_get_char(vtty: *mut vtty_t) -> c_int {
    VTTY_LOCK(vtty);

    if (*vtty).read_ptr == (*vtty).write_ptr {
        VTTY_UNLOCK(vtty);
        return -1;
    }

    let c: c_uchar = (*vtty).buffer[(*vtty).read_ptr as usize];
    (*vtty).read_ptr += 1;

    if (*vtty).read_ptr == VTTY_BUFFER_SIZE as u_int {
        (*vtty).read_ptr = 0;
    }

    VTTY_UNLOCK(vtty);
    c.into()
}

/// Returns TRUE if a character is available in buffer
#[no_mangle]
pub unsafe extern "C" fn vtty_is_char_avail(vtty: *mut vtty_t) -> c_int {
    VTTY_LOCK(vtty);
    let res: c_int = ((*vtty).read_ptr != (*vtty).write_ptr) as c_int;
    VTTY_UNLOCK(vtty);
    res
}

/// Put char to vtty
#[no_mangle]
pub unsafe extern "C" fn vtty_put_char(vtty: *mut vtty_t, mut ch: c_char) {
    match (*vtty).r#type {
        VTTY_TYPE_NONE => {}

        VTTY_TYPE_TERM | VTTY_TYPE_SERIAL => {
            if libc::write((*vtty).fd_array[0], addr_of!(ch).cast::<_>(), 1) != 1 {
                vm_log!((*vtty).vm, cstr!("VTTY"), cstr!("%s: put char 0x%x failed (%s)\n"), (*vtty).name, ch as c_int, libc::strerror(c_errno()));
            }
        }

        VTTY_TYPE_TCP => {
            fd_pool_send(addr_of_mut!((*vtty).fd_pool), addr_of_mut!(ch).cast::<_>(), 1, 0);
        }

        _ => {
            vm_error!((*vtty).vm, cstr!("vtty_put_char: bad vtty type %d\n"), (*vtty).r#type);
            libc::exit(1);
        }
    }

    // store char for replay
    (*vtty).replay_buffer[(*vtty).replay_ptr as usize] = ch as u_char;

    (*vtty).replay_ptr += 1;
    if (*vtty).replay_ptr == VTTY_BUFFER_SIZE as u_int {
        (*vtty).replay_ptr = 0;
        (*vtty).replay_full = 1;
    }
}

/// Put a buffer to vtty
#[no_mangle]
pub unsafe extern "C" fn vtty_put_buffer(vtty: *mut vtty_t, buf: *mut c_char, len: size_t) {
    for i in 0..len {
        vtty_put_char(vtty, *buf.add(i));
    }

    vtty_flush(vtty);
}

/// Flush VTTY output
#[no_mangle]
pub unsafe extern "C" fn vtty_flush(vtty: *mut vtty_t) {
    match (*vtty).r#type {
        VTTY_TYPE_TERM | VTTY_TYPE_SERIAL => {
            if (*vtty).fd_array[0] != -1 {
                libc::fsync((*vtty).fd_array[0]);
            }
        }
        _ => {}
    }
}

/// VTTY TCP input
unsafe extern "C" fn vtty_tcp_input(fd_slot: *mut c_int, opt: *mut c_void) {
    vtty_read_and_store(opt.cast::<_>(), fd_slot);
}

/// VTTY thread
extern "C" fn vtty_thread_main(_arg: *mut c_void) -> *mut c_void {
    unsafe {
        let mut vtty: *mut vtty_t;
        let mut tv: libc::timeval = zeroed::<_>();
        let mut fd_max: c_int;
        let mut fd_tcp: c_int;
        let mut res: c_int;
        let mut rfds: libc::fd_set = zeroed::<_>();

        loop {
            VTTY_LIST_LOCK();

            // Build the FD set
            libc::FD_ZERO(addr_of_mut!(rfds));
            fd_max = -1;
            vtty = vtty_list;
            while !vtty.is_null() {
                match (*vtty).r#type {
                    VTTY_TYPE_TCP => {
                        for i in 0..(*vtty).fd_count {
                            if (*vtty).fd_array[i as usize] != -1 {
                                libc::FD_SET((*vtty).fd_array[i as usize], addr_of_mut!(rfds));
                                if (*vtty).fd_array[i as usize] > fd_max {
                                    fd_max = (*vtty).fd_array[i as usize];
                                }
                            }
                        }

                        fd_tcp = fd_pool_set_fds(addr_of_mut!((*vtty).fd_pool), addr_of_mut!(rfds));
                        fd_max = max(fd_tcp, fd_max);
                    }

                    _ => {
                        if (*vtty).fd_array[0] != -1 {
                            libc::FD_SET((*vtty).fd_array[0], addr_of_mut!(rfds));
                            fd_max = max((*vtty).fd_array[0], fd_max);
                        }
                    }
                }

                vtty = (*vtty).next;
            }
            VTTY_LIST_UNLOCK();

            // Wait for incoming data
            tv.tv_sec = 0;
            tv.tv_usec = 50 * 1000; // 50 ms
            res = libc::select(fd_max + 1, addr_of_mut!(rfds), null_mut(), null_mut(), addr_of_mut!(tv));

            if res == -1 {
                if c_errno() != libc::EINTR {
                    libc::perror(cstr!("vtty_thread: select"));
                }
                continue;
            }

            // Examine active FDs and call user handlers
            VTTY_LIST_LOCK();
            vtty = vtty_list;
            while !vtty.is_null() {
                match (*vtty).r#type {
                    VTTY_TYPE_TCP => {
                        // check incoming connection
                        for i in 0..(*vtty).fd_count {
                            if (*vtty).fd_array[i as usize] == -1 {
                                continue;
                            }

                            if !libc::FD_ISSET((*vtty).fd_array[i as usize], addr_of!(rfds)) {
                                continue;
                            }

                            vtty_tcp_conn_accept(vtty, i);
                        }

                        // check established connection
                        fd_pool_check_input(addr_of_mut!((*vtty).fd_pool), addr_of_mut!(rfds), Some(vtty_tcp_input), vtty.cast::<_>());
                    }

                    // Term, Serial
                    _ => {
                        if (*vtty).fd_array[0] != -1 && libc::FD_ISSET((*vtty).fd_array[0], addr_of!(rfds)) {
                            vtty_read_and_store(vtty, addr_of_mut!((*vtty).fd_array[0]));
                            (*vtty).input_pending = TRUE;
                        }
                    }
                }

                if (*vtty).input_pending != 0 {
                    if (*vtty).read_notifier.is_some() {
                        (*vtty).read_notifier.unwrap()(vtty);
                    }

                    (*vtty).input_pending = FALSE;
                }

                // Flush any pending output
                if (*vtty).managed_flush == 0 {
                    vtty_flush(vtty);
                }

                vtty = (*vtty).next;
            }
            VTTY_LIST_UNLOCK();
        }
    }
}

/// Initialize the VTTY thread
#[no_mangle]
pub unsafe extern "C" fn vtty_init() -> c_int {
    if libc::pthread_create(addr_of_mut!(vtty_thread), null_mut(), vtty_thread_main, null_mut()) != 0 {
        libc::perror(cstr!("vtty: pthread_create"));
        return -1;
    }

    0
}
