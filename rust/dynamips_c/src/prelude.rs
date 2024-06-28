//! Internal shared code to interact with C.
//!
//! cbindgen will ignore the contents of this module.

pub use crate::_ext::cfunc;
pub use crate::_ext::cstr;
#[cfg(feature = "ENABLE_GEN_ETH")]
pub use crate::_ext::pcap_dumper_t;
#[cfg(feature = "ENABLE_GEN_ETH")]
pub use crate::_ext::pcap_t;
pub use crate::_ext::str0;
pub use crate::_ext::u_char;
pub use crate::_ext::u_int;
pub use crate::_ext::u_long;
pub use crate::_ext::AsC;
pub use crate::_ext::AsCMut;
pub use crate::_ext::CArray;
pub use crate::_ext::Printf;
pub use crate::_ext::Volatile;
pub use function_name::named;
pub use libc;
pub use libc::size_t;
pub use libc::ssize_t;
pub use likely_stable::likely;
pub use likely_stable::unlikely;
#[cfg(target_os = "linux")]
pub use linux_raw_sys;
pub use setjmp;
pub use std::ffi::c_char;
pub use std::ffi::c_int;
pub use std::ffi::c_long;
pub use std::ffi::c_longlong;
pub use std::ffi::c_short;
pub use std::ffi::c_uchar;
pub use std::ffi::c_uint;
pub use std::ffi::c_ulong;
pub use std::ffi::c_ulonglong;
pub use std::ffi::c_ushort;
pub use std::ffi::c_void;
pub use std::ffi::CStr;
pub use std::ffi::CString;
pub use std::marker::PhantomData;
pub use std::mem::offset_of;
pub use std::mem::size_of;
pub use std::mem::zeroed;
pub use std::ptr::addr_of;
pub use std::ptr::addr_of_mut;
pub use std::ptr::null_mut;

extern "C" {
    // _ext.c
    pub fn c_errno() -> c_int;
    pub fn c_INET6_ADDRSTRLEN() -> libc::socklen_t;
    pub fn c_set_errno(x: c_int);
    pub fn c_stderr() -> *mut libc::FILE;
    pub fn c_stdout() -> *mut libc::FILE;
    pub fn c_timezone() -> c_long;
    // libc
    pub fn gethostbyname(name: *const c_char) -> *mut libc::hostent;
    pub fn htons(x: u16) -> u16;
    pub fn htonl(x: u32) -> u32;
    pub fn inet_addr(cp: *const libc::c_char) -> libc::in_addr_t;
    pub fn inet_aton(cp: *const c_char, inp: *mut libc::in_addr) -> c_int;
    pub fn inet_ntop(af: c_int, src: *const c_void, dst: *mut c_char, size: libc::socklen_t) -> *const c_char;
    pub fn inet_pton(af: c_int, src: *const c_char, dst: *mut c_void) -> c_int;
    pub fn ntohl(x: u32) -> u32;
    pub fn ntohs(x: u16) -> u16;
}

// libpcap stuff

#[cfg(feature = "ENABLE_GEN_ETH")]
pub const DLT_EN10MB: c_int = 1;

#[cfg(feature = "ENABLE_GEN_ETH")]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct pcap_pkthdr {
    pub ts: libc::timeval,
    pub caplen: c_uint,
    pub len: c_uint,
}

#[cfg(feature = "ENABLE_GEN_ETH")]
extern "C" {
    pub fn pcap_close(arg1: *mut pcap_t);
    pub fn pcap_datalink_name_to_val(arg1: *const c_char) -> c_int;
    pub fn pcap_dump_close(arg1: *mut pcap_dumper_t);
    pub fn pcap_dump_flush(arg1: *mut pcap_dumper_t) -> c_int;
    pub fn pcap_dump_open(arg1: *mut pcap_t, arg2: *const c_char) -> *mut pcap_dumper_t;
    pub fn pcap_dump(arg1: *mut c_uchar, arg2: *const pcap_pkthdr, arg3: *const c_uchar);
    pub fn pcap_open_dead(arg1: c_int, arg2: c_int) -> *mut pcap_t;
    pub fn pcap_snapshot(arg1: *mut pcap_t) -> c_int;
}
