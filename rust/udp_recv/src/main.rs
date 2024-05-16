use dynamips_c::net::*;
use dynamips_c::prelude::*;
use std::env::args;
use std::ffi::CString;

const MAX_PKT_SIZE: usize = 2048;

fn main() {
    unsafe {
        let mut pkt: [u8; MAX_PKT_SIZE] = [0; MAX_PKT_SIZE];
        let mut sck: c_int = -1;

        let argv: Vec<CString> = args().map(|x| CString::new(x).unwrap()).collect();
        if argv.len() != 3 {
            eprintln!("Usage: udp_recv <output_file> <udp_port>");
            libc::exit(libc::EXIT_FAILURE);
        }

        // Wait connection
        if ip_listen(null_mut(), libc::atoi(argv[2].as_c()), libc::SOCK_DGRAM, 1, addr_of_mut!(sck)) < 1 {
            libc::perror(cstr!("ip_listen"));
            libc::exit(libc::EXIT_FAILURE);
        }

        // Receive packet and store it
        let pkt_size: ssize_t = libc::recvfrom(sck, pkt.as_c_void_mut(), MAX_PKT_SIZE, 0, null_mut(), null_mut());
        if pkt_size < 0 {
            libc::perror(cstr!("recvfrom"));
            libc::exit(libc::EXIT_FAILURE);
        }

        let fd: *mut libc::FILE = libc::fopen(argv[1].as_c(), cstr!("w"));
        if fd.is_null() {
            libc::perror(cstr!("fopen"));
            libc::exit(libc::EXIT_FAILURE);
        }

        libc::fwrite(pkt.as_c_void(), 1, pkt_size as size_t, fd);
        libc::fclose(fd);
        libc::close(sck);
        libc::exit(0);
    }
}
