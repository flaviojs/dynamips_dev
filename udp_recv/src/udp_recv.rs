use dynamips_c::net::*;
use libc::ssize_t;
use std::env;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ptr::addr_of_mut;
use std::ptr::null_mut;
use unixstring::UnixString;

const MAX_PKT_SIZE: usize = 2048;

fn main() {
    unsafe {
        let args: Vec<UnixString> = env::args_os().map(|arg| UnixString::from_os_string(arg).expect("arg")).collect();
        let mut pkt: [c_char; MAX_PKT_SIZE] = [0; MAX_PKT_SIZE];
        let mut sck: c_int = -1;

        // Wait connection
        if ip_listen(null_mut(), libc::atoi(args[2].as_ptr()), libc::SOCK_DGRAM, 1, addr_of_mut!(sck)) < 1 {
            libc::perror(c"ip_listen".as_ptr());
            libc::exit(libc::EXIT_FAILURE);
        }

        // Receive packet and store it
        let pkt_size: ssize_t = libc::recvfrom(sck, pkt.as_mut_ptr().cast::<_>(), pkt.len(), 0, null_mut(), null_mut());
        if pkt_size < 0 {
            libc::perror(c"recvfrom".as_ptr());
            libc::exit(libc::EXIT_FAILURE);
        }

        let fd: *mut libc::FILE = libc::fopen(args[1].as_ptr(), c"w".as_ptr());
        if fd.is_null() {
            libc::perror(c"fopen".as_ptr());
            libc::exit(libc::EXIT_FAILURE);
        }

        libc::fwrite(pkt.as_ptr().cast::<_>(), 1, pkt_size as _, fd);
        libc::fclose(fd);
        libc::close(sck);
        libc::exit(0);
    }
}
