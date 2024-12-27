use dynamips_c::net::*;
use libc::size_t;
use std::env;
use std::ffi::c_char;
use std::ffi::c_int;
use unixstring::UnixString;

const MAX_PKT_SIZE: usize = 2048;

fn main() {
    unsafe {
        let args: Vec<UnixString> = env::args_os().map(|arg| UnixString::from_os_string(arg).expect("arg")).collect();
        let mut pkt: [c_char; MAX_PKT_SIZE] = [0; MAX_PKT_SIZE];

        let fd: *mut libc::FILE = libc::fopen(args[1].as_ptr(), c"r".as_ptr());
        if fd.is_null() {
            libc::perror(c"fopen".as_ptr());
            libc::exit(libc::EXIT_FAILURE);
        }

        // Read packet from file
        let pkt_size: size_t = libc::fread(pkt.as_mut_ptr().cast::<_>(), 1, MAX_PKT_SIZE, fd);

        // Connect to remote port
        let sck: c_int = udp_connect(libc::atoi(args[2].as_ptr()), args[3].as_ptr().cast_mut(), libc::atoi(args[4].as_ptr()));
        if sck < 0 {
            libc::exit(libc::EXIT_FAILURE);
        }

        // Send it
        if libc::send(sck, pkt.as_ptr().cast::<_>(), pkt_size, 0) < 0 {
            libc::exit(libc::EXIT_FAILURE);
        }

        libc::close(sck);
        libc::exit(0);
    }
}
