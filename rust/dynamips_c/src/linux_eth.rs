//! module used to send/receive Ethernet packets.
//!
//! Specific to the Linux operating system.

use crate::prelude::*;

extern "C" {
    pub fn lnx_eth_get_dev_index(name: *mut c_char) -> c_int;
    pub fn lnx_eth_init_socket(device: *mut c_char) -> c_int;
    pub fn lnx_eth_recv(sck: c_int, buffer: *mut c_char, len: size_t) -> ssize_t;
    pub fn lnx_eth_send(sck: c_int, dev_id: c_int, buffer: *mut c_char, len: size_t) -> ssize_t;
}
