//! gen_eth.c: module used to send/receive Ethernet packets.
//!
//! Use libpcap (0.9+) or WinPcap (0.4alpha1+) to receive and send packets.

use crate::prelude::*;

extern "C" {
    pub fn gen_eth_close(p: *mut pcap_t);
    pub fn gen_eth_init(device: *mut c_char) -> *mut pcap_t;
    pub fn gen_eth_recv(p: *mut pcap_t, buffer: *mut c_char, len: size_t) -> ssize_t;
    pub fn gen_eth_send(p: *mut pcap_t, buffer: *mut c_char, len: size_t) -> ssize_t;
}
