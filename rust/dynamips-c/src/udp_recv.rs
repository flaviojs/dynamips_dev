use crate::_private::*;
use crate::net::*;

const MAX_PKT_SIZE: usize = 2048;

pub unsafe fn udp_recv_main(_argc: c_int, argv: *mut *mut c_char) -> c_int {
    // FIXME will crash when there are not enough arguments
    let mut pkt: [c_char; MAX_PKT_SIZE] = [0; MAX_PKT_SIZE];
    let mut sck: c_int = -1;

    // Wait connection
    if ip_listen(null_mut(), libc::atoi(*argv.add(2)), libc::SOCK_DGRAM, 1, addr_of_mut!(sck)) < 1 {
        libc::perror(cstr!("ip_listen"));
        libc::exit(libc::EXIT_FAILURE);
    }

    // Receive packet and store it
    let pkt_size: ssize_t = libc::recvfrom(sck, pkt.as_c_void_mut(), pkt.len(), 0, null_mut(), null_mut());
    if pkt_size < 0 {
        libc::perror(cstr!("recvfrom"));
        libc::exit(libc::EXIT_FAILURE);
    }

    let fd: *mut libc::FILE = libc::fopen(*argv.add(1), cstr!("w"));
    if fd.is_null() {
        libc::perror(cstr!("fopen"));
        libc::exit(libc::EXIT_FAILURE);
    }

    libc::fwrite(pkt.as_c_void(), 1, pkt_size as size_t, fd);
    libc::fclose(fd);
    libc::close(sck);
    0
}
