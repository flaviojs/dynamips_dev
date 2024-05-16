use dynamips_c::_private::*;
use dynamips_c::net::*;
use std::env;
use std::ffi::CString;

const MAX_PKT_SIZE: usize = 2048;

/// Usage: udp_send <input_file> <udp_port> <target_addr> <target_udp_port>
fn main() {
    unsafe {
        // FIXME will panic when there are not enough arguments
        let args: Vec<_> = env::args().map(|x| CString::new(x).unwrap()).collect();
        let mut pkt: [c_char; MAX_PKT_SIZE] = [0; MAX_PKT_SIZE];

        let fd: *mut libc::FILE = libc::fopen(args[1].as_c(), cstr!("r"));
        if fd.is_null() {
            libc::perror(cstr!("fopen"));
            libc::exit(libc::EXIT_FAILURE);
        }

        // Read packet from file
        let pkt_size: size_t = libc::fread(pkt.as_c_void_mut(), 1, MAX_PKT_SIZE, fd);

        // Connect to remote port
        let sck: c_int = udp_connect(libc::atoi(args[2].as_c()), args[3].as_c().cast_mut(), libc::atoi(args[4].as_c()));
        if sck < 0 {
            libc::exit(libc::EXIT_FAILURE);
        }

        // Send it
        if libc::send(sck, pkt.as_c_void(), pkt_size, 0) < 0 {
            libc::exit(libc::EXIT_FAILURE);
        }

        libc::close(sck);
    }
}
