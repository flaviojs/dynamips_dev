use dynamips_c::_private::*;
use dynamips_c::net::*;
use std::env;
use std::ffi::CString;

const MAX_PKT_SIZE: usize = 2048;

/// Usage: udp_recv <output_file> <udp_port>
fn main() {
    unsafe {
        // FIXME will panic when there are not enough arguments
        let args: Vec<_> = env::args().map(|x| CString::new(x).unwrap()).collect();
        let mut pkt: [u8; MAX_PKT_SIZE] = [0; MAX_PKT_SIZE];
        let mut sck: c_int = -1;

        // Wait connection
        if ip_listen(null_mut(), libc::atoi(args[2].as_c()), libc::SOCK_DGRAM, 1, addr_of_mut!(sck)) < 1 {
            libc::perror(cstr!("ip_listen"));
            libc::exit(libc::EXIT_FAILURE);
        }

        // Receive packet and store it
        let pkt_size: ssize_t = libc::recvfrom(sck, pkt.as_c_void_mut(), MAX_PKT_SIZE, 0, null_mut(), null_mut());
        if pkt_size < 0 {
            libc::perror(cstr!("recvfrom"));
            libc::exit(libc::EXIT_FAILURE);
        }

        let fd: *mut libc::FILE = libc::fopen(args[1].as_c(), cstr!("w"));
        if fd.is_null() {
            libc::perror(cstr!("fopen"));
            libc::exit(libc::EXIT_FAILURE);
        }

        libc::fwrite(pkt.as_c_void(), 1, pkt_size as size_t, fd);
        libc::fclose(fd);
        libc::close(sck);
    }
}
