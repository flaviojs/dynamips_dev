//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Copyright (c) 2005,2006 Christophe Fillot.  All rights reserved.
//!
//! Utility functions.

use crate::_extra::*;
use crate::dynamips_common::*;
use libc::off_t;
use libc::size_t;
use libc::ssize_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_uchar;
use std::ffi::c_uint;
use std::ffi::c_void;
use std::mem::zeroed;
use std::ptr::addr_of_mut;
use std::ptr::null_mut;
use std::ptr::read_unaligned;
use std::ptr::write_unaligned;
use unixstring::UnixString;

// Host CPU Types
pub const CPU_x86: c_int = 0;
pub const CPU_amd64: c_int = 1;
pub const CPU_nojit: c_int = 2;

// Number of host registers available for JIT
#[cfg(feature = "DYNAMIPS_ARCH_x86")]
pub const JIT_HOST_NREG: usize = 8;
#[cfg(feature = "DYNAMIPS_ARCH_amd64")]
pub const JIT_HOST_NREG: usize = 16;
#[cfg(not(any(feature = "DYNAMIPS_ARCH_x86", feature = "DYNAMIPS_ARCH_amd64")))]
pub const JIT_HOST_NREG: usize = 0;

// Host to VM (big-endian) conversion functions
#[no_mangle]
pub extern "C" fn htovm16(x: m_uint16_t) -> m_uint16_t {
    if cfg!(target_endian = "big") {
        x
    } else {
        libc::htons(x)
    }
}
#[no_mangle]
pub extern "C" fn htovm32(x: m_uint32_t) -> m_uint32_t {
    if cfg!(target_endian = "big") {
        x
    } else {
        libc::htonl(x)
    }
}
#[no_mangle]
pub extern "C" fn htovm64(x: m_uint64_t) -> m_uint64_t {
    if cfg!(target_endian = "big") {
        x
    } else {
        swap64(x)
    }
}

#[no_mangle]
pub extern "C" fn vmtoh16(x: m_uint16_t) -> m_uint16_t {
    if cfg!(target_endian = "big") {
        x
    } else {
        libc::ntohs(x)
    }
}
#[no_mangle]
pub extern "C" fn vmtoh32(x: m_uint32_t) -> m_uint32_t {
    if cfg!(target_endian = "big") {
        x
    } else {
        libc::ntohl(x)
    }
}
#[no_mangle]
pub extern "C" fn vmtoh64(x: m_uint64_t) -> m_uint64_t {
    if cfg!(target_endian = "big") {
        x
    } else {
        swap64(x)
    }
}

// FD pool
pub const FD_POOL_MAX: usize = 16;

pub type fd_pool_t = fd_pool;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fd_pool {
    pub fd: [c_int; FD_POOL_MAX],
    pub next: *mut fd_pool,
}

// Translated block function pointer
pub type insn_tblock_fptr = Option<unsafe extern "C" fn()>;

// Host executable page
pub type insn_exec_page_t = insn_exec_page;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct insn_exec_page {
    pub ptr: *mut u_char,
    pub next: *mut insn_exec_page_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub flags: c_int,
}

// MIPS instruction
pub type mips_insn_t = m_uint32_t;

// PowerPC instruction
pub type ppc_insn_t = m_uint32_t;

// Macros for double linked list
#[cfg(feature = "USE_UNSTABLE")]
macro_rules! M_LIST_ADD {
    ($item:expr, $head:expr, $prefix:expr) => {
        paste::paste! {
            (*$item).[<$prefix _next>] = $head;
            (*$item).[<$prefix _pprev>] = std::ptr::addr_of_mut!($head);

            if !$head.is_null() {
                (*$head).[<$prefix _pprev>] = std::ptr::addr_of_mut!((*$item).[<$prefix _next>]);
            }

            $head = $item;
        }
    };
}
#[cfg(feature = "USE_UNSTABLE")]
pub(crate) use M_LIST_ADD;

#[cfg(feature = "USE_UNSTABLE")]
macro_rules! M_LIST_REMOVE {
    ($item:expr, $prefix:expr) => {
        paste::paste! {
            if !(*$item).[<$prefix _pprev>].is_null() {
                if !(*$item).[<$prefix _next>].is_null() {
                    (*(*$item).[<$prefix _next>]).[<$prefix _pprev>] = (*$item).[<$prefix _pprev>];
                }
                (*(*$item).[<$prefix _pprev>]) = (*$item).[<$prefix _next>];

                (*$item).[<$prefix _pprev>] = std::ptr::null_mut();
                (*$item).[<$prefix _next>] = std::ptr::null_mut();
            }
        }
    };
}
#[cfg(feature = "USE_UNSTABLE")]
pub(crate) use M_LIST_REMOVE;

// List item
pub type m_list_t = m_list;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct m_list {
    pub data: *mut c_void,
    pub next: *mut m_list_t,
}

// MTS mapping info
pub type mts_map_t = mts_map;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct mts_map {
    pub vaddr: m_uint64_t,
    pub paddr: m_uint64_t,
    pub len: m_uint64_t,
    pub cached: m_uint32_t,
    #[cfg(not(feature = "USE_UNSTABLE"))]
    pub tlb_index: m_uint32_t,
    pub offset: m_uint32_t,
    #[cfg(feature = "USE_UNSTABLE")]
    pub flags: m_uint32_t,
}

// Invalid VTLB entry
pub const MTS_INV_ENTRY_MASK: m_uint32_t = 0x00000001;

// MTS entry flags
pub const MTS_FLAG_DEV: m_uint32_t = 0x000000001; // Virtual device used
pub const MTS_FLAG_COW: m_uint32_t = 0x000000002; // Copy-On-Write
pub const MTS_FLAG_EXEC: m_uint32_t = 0x000000004; // Exec page
pub const MTS_FLAG_RO: m_uint32_t = 0x000000008; // Read-only page

pub const MTS_FLAG_WRCATCH: m_uint32_t = MTS_FLAG_RO | MTS_FLAG_COW; // Catch writes

// Virtual TLB entry (32-bit MMU)
pub type mts32_entry_t = mts32_entry;
#[repr(C)]
#[repr(align(16))]
#[derive(Debug, Copy, Clone)]
pub struct mts32_entry {
    pub gvpa: m_uint32_t,  // Guest Virtual Page Address
    pub gppa: m_uint32_t,  // Guest Physical Page Address
    pub hpa: m_iptr_t,     // Host Page Address
    pub flags: m_uint32_t, // Flags
}

// Virtual TLB entry (64-bit MMU)
pub type mts64_entry_t = mts64_entry;
#[repr(C)]
#[repr(align(16))]
#[derive(Debug, Copy, Clone)]
pub struct mts64_entry {
    pub gvpa: m_uint64_t,  // Guest Virtual Page Address
    pub gppa: m_uint64_t,  // Guest Physical Page Address
    pub hpa: m_iptr_t,     // Host Page Address
    pub flags: m_uint32_t, // Flags
}

// Host register allocation
pub const HREG_FLAG_ALLOC_LOCKED: c_int = 1;
pub const HREG_FLAG_ALLOC_FORCED: c_int = 2;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hreg_map {
    pub hreg: c_int,
    pub vreg: c_int,
    pub flags: c_int,
    pub prev: *mut hreg_map,
    pub next: *mut hreg_map,
}

// Check status of a bit
#[inline]
#[no_mangle]
pub extern "C" fn check_bit(old: u_int, new: u_int, bit: u_int) -> c_int {
    let mask: u_int = 1 << bit;

    if (old & mask) != 0 && (new & mask) == 0 {
        return 1; // bit unset
    }

    if (old & mask) == 0 && (new & mask) != 0 {
        return 2; // bit set
    }

    // no change
    0
}

// Sign-extension
#[inline(always)]
#[no_mangle]
pub extern "C" fn sign_extend(x: m_int64_t, mut len: c_int) -> m_int64_t {
    len = 64 - len;
    (x << len) >> len
}

// Sign-extension (32-bit)
#[inline(always)]
#[no_mangle]
pub extern "C" fn sign_extend_32(x: m_int32_t, mut len: c_int) -> m_int32_t {
    len = 32 - len;
    (x << len) >> len
}

// Extract bits from a 32-bit values
#[inline]
#[no_mangle]
pub extern "C" fn bits(val: m_uint32_t, start: c_int, end: c_int) -> c_int {
    ((val >> start) & ((1 << (end - start + 1)) - 1)) as c_int
}

// Normalize a size
#[inline]
#[no_mangle]
pub extern "C" fn normalize_size(val: u_int, nb: u_int, shift: c_int) -> u_int {
    ((val + nb - 1) & !(nb - 1)) >> shift
}

// Convert a 16-bit number between little and big endian
#[inline(always)]
#[no_mangle]
pub extern "C" fn swap16(value: m_uint16_t) -> m_uint16_t {
    (value >> 8) | ((value & 0xFF) << 8)
}

// Convert a 32-bit number between little and big endian
#[inline(always)]
#[no_mangle]
pub extern "C" fn swap32(value: m_uint32_t) -> m_uint32_t {
    let mut result: m_uint32_t;

    result = value >> 24;
    result |= ((value >> 16) & 0xff) << 8;
    result |= ((value >> 8) & 0xff) << 16;
    result |= (value & 0xff) << 24;
    result
}

// Convert a 64-bit number between little and big endian
#[inline(always)]
#[no_mangle]
pub extern "C" fn swap64(value: m_uint64_t) -> m_uint64_t {
    let mut result: m_uint64_t;

    result = (swap32((value & 0xffffffff) as m_uint32_t) as m_uint64_t) << 32;
    result |= swap32((value >> 32) as m_uint32_t) as m_uint64_t;
    result
}

// Get current time in number of msec since epoch
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn m_gettime() -> m_tmcnt_t {
    let mut tvp: libc::timeval = zeroed();

    libc::gettimeofday(addr_of_mut!(tvp), null_mut());
    ((tvp.tv_sec as m_tmcnt_t) * 1000) + ((tvp.tv_usec as m_tmcnt_t) / 1000)
}

// Get current time in number of usec since epoch
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_gettime_usec() -> m_tmcnt_t {
    let mut tvp: libc::timeval = zeroed();

    libc::gettimeofday(addr_of_mut!(tvp), null_mut());
    ((tvp.tv_sec as m_tmcnt_t) * 1000000) + (tvp.tv_usec as m_tmcnt_t)
}

// Get current time in number of ms (localtime)
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_gettime_adj() -> m_tmcnt_t {
    let mut tvp: libc::timeval = zeroed();
    let mut tmx: libc::tm = zeroed();
    let gmt_adjust: libc::time_t;
    let mut ct: libc::time_t;

    libc::gettimeofday(addr_of_mut!(tvp), null_mut());
    ct = tvp.tv_sec;
    libc::localtime_r(addr_of_mut!(ct), addr_of_mut!(tmx));

    #[cfg(has_libc_tm_tm_gmtoff)]
    {
        gmt_adjust = tmx.tm_gmtoff;
    }
    #[cfg(not(has_libc_tm_tm_gmtoff))]
    {
        compile_error!("FIXME implement alternative to tm_gmtoff");
        /*
        #ifdef __CYGWIN__
        #define GET_TIMEZONE _timezone
        #else
        #define GET_TIMEZONE timezone
        #endif
        #if defined(__CYGWIN__) || defined(SUNOS)
        gmt_adjust = -(tmx.tm_isdst ? GET_TIMEZONE - 3600 : GET_TIMEZONE);
        #endif
        */
    }

    tvp.tv_sec += gmt_adjust;
    ((tvp.tv_sec as m_tmcnt_t) * 1000) + ((tvp.tv_usec as m_tmcnt_t) / 1000)
}

// Get a byte-swapped 16-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_ntoh16(ptr: *mut m_uint8_t) -> m_uint16_t {
    let val: m_uint16_t = ((*ptr as m_uint16_t) << 8) | (*ptr.add(1) as m_uint16_t);
    val
}

// Get a byte-swapped 32-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_ntoh32(ptr: *mut m_uint8_t) -> m_uint32_t {
    let val: m_uint32_t = ((*ptr as m_uint32_t) << 24) | ((*ptr.add(1) as m_uint32_t) << 16) | ((*ptr.add(2) as m_uint32_t) << 8) | (*ptr.add(3) as m_uint32_t);
    val
}

// Set a byte-swapped 16-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_hton16(ptr: *mut m_uint8_t, val: m_uint16_t) {
    *ptr = (val >> 8) as m_uint8_t;
    *ptr.add(1) = val as m_uint8_t;
}

// Set a byte-swapped 32-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_hton32(ptr: *mut m_uint8_t, val: m_uint32_t) {
    *ptr = (val >> 24) as m_uint8_t;
    *ptr.add(1) = (val >> 16) as m_uint8_t;
    *ptr.add(2) = (val >> 8) as m_uint8_t;
    *ptr.add(3) = val as m_uint8_t;
}

// Global log file
#[no_mangle]
pub static mut log_file: *mut libc::FILE = null_mut();

// Add an element to a list
#[no_mangle]
pub unsafe extern "C" fn m_list_add(head: *mut *mut m_list_t, data: *mut c_void) -> *mut m_list_t {
    let item: *mut m_list_t = libc::malloc(size_of::<m_list_t>()).cast::<_>();
    if !item.is_null() {
        (*item).data = data;
        (*item).next = *head;
        *head = item;
    }

    item
}

// Dynamic sprintf
macro_rules! dyn_sprintf {
    ($fmt:expr $(, $arg:expr)*) => {
        {
            let fmt: &str = $fmt;
            let args: &[&dyn sprintf::Printf] = &[$(&crate::_extra::Printf($arg)),*];
            match sprintf::vsprintf(fmt, args) {
                Ok(s) => {
                    let p = libc::malloc(s.len() + 1);
                    if p.is_null() {
                        libc::perror(c"dyn_sprintf: malloc".as_ptr());
                        null_mut()
                    } else {
                        libc::memcpy(p, s.as_str().as_ptr().cast::<_>(), s.len());
                        *p.cast::<u8>().add(s.len()) = 0;
                        p.cast::<c_char>()
                    }
                }
                Err(err) => {
                    eprintln!("dyn_sprintf({:?} {}): {}", fmt, args.len(), err);
                    null_mut()
                }
           }
        }
    }
}
pub(crate) use dyn_sprintf;

// Split a string
#[no_mangle]
pub unsafe extern "C" fn m_strsplit(mut str_: *mut c_char, delim: c_char, array: *mut *mut c_char, max_count: c_int) -> c_int {
    let mut pos: c_int = 0;
    let mut len: size_t;
    let mut ptr: *mut c_char;

    for i in 0..max_count {
        *array.add(i as usize) = null_mut();
    }

    let error = || {
        for i in 0..max_count {
            libc::free((*array.add(i as usize)).cast::<_>());
        }
        -1
    };

    loop {
        if pos == max_count {
            return error();
        }

        ptr = libc::strchr(str_, delim as c_int);
        if ptr.is_null() {
            ptr = str_.add(libc::strlen(str_));
        }

        len = ptr.offset_from(str_) as _;

        *array.add(pos as usize) = libc::malloc(len + 1).cast::<_>();
        if (*array.add(pos as usize)).is_null() {
            return error();
        }

        libc::memcpy((*array.add(pos as usize)).cast::<_>(), str_.cast::<_>(), len);
        *(*array.add(pos as usize)).add(len) = 0;

        str_ = ptr.add(1);
        pos += 1;
        if *ptr != 0 {
            continue;
        }
        break;
    }

    pos
}

// Tokenize a string
#[no_mangle]
pub unsafe extern "C" fn m_strtok(mut str_: *mut c_char, delim: c_char, array: *mut *mut c_char, max_count: c_int) -> c_int {
    let mut pos: c_int = 0;
    let mut len: size_t;
    let mut ptr: *mut c_char;

    for i in 0..max_count {
        *array.add(i as usize) = null_mut();
    }

    let error = || {
        for i in 0..max_count {
            libc::free((*array.add(i as usize)).cast::<_>());
        }
        -1
    };

    loop {
        if pos == max_count {
            return error();
        }

        ptr = libc::strchr(str_, delim as c_int);
        if ptr.is_null() {
            ptr = str_.add(libc::strlen(str_));
        }

        len = ptr.offset_from(str_) as size_t;

        *array.add(pos as usize) = libc::malloc(len + 1).cast::<_>();
        if (*array.add(pos as usize)).is_null() {
            return error();
        }

        libc::memcpy((*array.add(pos as usize)).cast::<_>(), str_.cast::<_>(), len);
        *(*array.add(pos as usize)).add(len) = 0;

        while *ptr == delim {
            ptr = ptr.add(1);
        }

        str_ = ptr;
        pos += 1;
        if *ptr != 0 {
            continue;
        }
        break;
    }

    pos
}

// Quote a string
#[no_mangle]
pub unsafe extern "C" fn m_strquote(buffer: *mut c_char, buf_len: size_t, str_: *mut c_char) -> *mut c_char {
    let p: *mut c_char = libc::strpbrk(str_, c" \t\"'".as_ptr());
    if p.is_null() {
        return str_;
    }

    libc::snprintf(buffer, buf_len, c"\"%s\"".as_ptr(), str_);
    buffer
}

// Decode from hex.
//
// hex to raw bytes, returning count of bytes.
// maxlen limits output buffer size.
#[no_mangle]
pub unsafe extern "C" fn hex_decode(mut out: *mut c_uchar, mut in_: *const c_uchar, maxlen: c_int) -> c_int {
    const BAD: c_char = 0xff_u8 as c_char;
    #[rustfmt::skip]
    static hexval: [c_char; 112] = [
        BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
          0,  1,  2,  3,   4,  5,  6,  7,   8,  9,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD, 10, 11, 12,  13, 14, 15,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD, 10, 11, 12,  13, 14, 15,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
    ];
    let mut len: c_int = 0;
    let mut empty: bool = true;

    while len < maxlen {
        if *in_ as usize >= hexval.len() || hexval[*in_ as usize] == BAD {
            break;
        }

        if empty {
            *out = (hexval[*in_ as usize] as c_uchar) << 4;
        } else {
            *out |= hexval[*in_ as usize] as c_uchar;
            out = out.add(1);
            len += 1;
        }
        in_ = in_.add(1);
        empty = !empty;
    }

    len
}

// Ugly function that dumps a structure in hexa and ascii.
#[no_mangle]
pub unsafe extern "C" fn mem_dump(f_output: *mut libc::FILE, pkt: *mut u_char, len: u_int) {
    let mut x: u_int;
    let mut i: u_int = 0;

    while i < len {
        if (len - i) > 16 {
            x = 16;
        } else {
            x = len - i;
        }

        libc::fprintf(f_output, c"%4.4x: ".as_ptr(), i);

        for tmp in 0..x {
            libc::fprintf(f_output, c"%2.2x ".as_ptr(), *pkt.add((i + tmp) as usize) as c_int);
        }
        for _ in x..16 {
            libc::fprintf(f_output, c"   ".as_ptr());
        }

        for tmp in 0..x {
            let c: c_char = *pkt.add((i + tmp) as usize) as c_char;

            if ((c >= b'A' as c_char) && (c <= b'Z' as c_char)) || ((c >= b'a' as c_char) && (c <= b'z' as c_char)) || ((c >= b'0' as c_char) && (c <= b'9' as c_char)) {
                libc::fprintf(f_output, c"%c".as_ptr(), c as c_int);
            } else {
                libc::fputs(c".".as_ptr(), f_output);
            }
        }

        i += x;
        libc::fprintf(f_output, c"\n".as_ptr());
    }

    libc::fprintf(f_output, c"\n".as_ptr());
    libc::fflush(f_output);
}

// Logging function
pub unsafe fn m_flog(fd: *mut libc::FILE, module: *const c_char, fmt: *const c_char, args: &[&dyn sprintf::Printf]) {
    let mut spec: libc::timespec = zeroed();
    let mut tmn: libc::tm = zeroed();

    if !fd.is_null() {
        libc::clock_gettime(libc::CLOCK_REALTIME, addr_of_mut!(spec));
        libc::gmtime_r(addr_of_mut!(spec.tv_sec), addr_of_mut!(tmn));

        // NOTE never use strftime for timestamps, it is crashy
        libc::fprintf(
            fd,
            c"%d-%02d-%02dT%02d:%02d:%02d.%03dZ %s: ".as_ptr(),
            tmn.tm_year + 1900,
            tmn.tm_mon + 1,
            tmn.tm_mday,
            tmn.tm_hour,
            tmn.tm_min,
            tmn.tm_sec,
            (spec.tv_nsec / 1000000) as c_int,
            module,
        );
        let fmt = UnixString::from_ptr(fmt);
        match sprintf::vsprintf(fmt.as_c_str().to_str().expect("fmt"), args) {
            Ok(s) => {
                let s = UnixString::from_string(s).expect("UnixString").into_cstring();
                libc::fprintf(fd, c"%s".as_ptr(), s.as_ptr());
            }
            Err(err) => {
                panic!("m_flog({:?} {}): {}", fmt, args.len(), err);
            }
        }
        libc::fflush(fd);
    }
}

// Logging function
macro_rules! m_log {
    ($module:expr, $fmt:expr$(, $arg:expr)*) => {{
        let module: *const c_char = $module;
        println!("module {:?}", module);
        let fmt: *const c_char = $fmt;
        println!("fmt {:?}", fmt);
        let args: &[&dyn sprintf::Printf] = &[$(&crate::_extra::Printf($arg)),*];
        $( println!("arg {:?}", $arg); )*
        m_flog(log_file, module, fmt, args);
    }};
}
pub(crate) use m_log;

// Write an array of string to a logfile
#[no_mangle]
pub unsafe extern "C" fn m_flog_str_array(fd: *mut libc::FILE, count: c_int, str_: *mut *mut c_char) {
    for i in 0..count {
        libc::fprintf(fd, c"%s ".as_ptr(), *str_.add(i as usize));
    }

    libc::fprintf(fd, c"\n".as_ptr());
    libc::fflush(fd);
}

// Returns a line from specified file (remove trailing '\n')
#[no_mangle]
pub unsafe extern "C" fn m_fgets(buffer: *mut c_char, size: c_int, fd: *mut libc::FILE) -> *mut c_char {
    *buffer = b'\0' as c_char;
    libc::fgets(buffer, size, fd);

    let len = libc::strlen(buffer);
    if len == 0 {
        return null_mut();
    }

    // remove trailing '\n'
    if *buffer.add(len - 1) == b'\n' as c_char {
        *buffer.add(len - 1) = b'\0' as c_char;
    }

    buffer
}

// Read a file and returns it in a buffer
#[no_mangle]
pub unsafe extern "C" fn m_read_file(filename: *const c_char, buffer: *mut *mut u_char, length: *mut size_t) -> c_int {
    // open
    let fd: *mut libc::FILE = libc::fopen(filename, c"rb".as_ptr());
    if fd.is_null() {
        return -1;
    }

    // len
    libc::fseek(fd, 0, libc::SEEK_END);
    let len = libc::ftell(fd);
    libc::fseek(fd, 0, libc::SEEK_SET);
    if len < 0 || libc::ferror(fd) != 0 {
        libc::fclose(fd);
        return -1;
    }

    if !length.is_null() {
        *length = len as size_t;
    }

    // data
    if !buffer.is_null() {
        *buffer = libc::malloc(len as size_t).cast::<_>();
        if (*buffer).is_null() || libc::fread((*buffer).cast::<_>(), len as size_t, 1, fd) != 1 {
            libc::free((*buffer).cast::<_>());
            *buffer = null_mut();
            libc::fclose(fd);
            return -1;
        }
    }

    // close
    libc::fclose(fd);
    0
}

// Allocate aligned memory
#[no_mangle]
pub unsafe extern "C" fn m_memalign(boundary: size_t, size: size_t) -> *mut c_void {
    let mut p: *mut c_void = null_mut();

    #[cfg(has_libc_posix_memalign)]
    {
        if libc::posix_memalign(addr_of_mut!(p), boundary, size) != 0 {
            return null_mut();
        }
    }
    #[cfg(all(not(has_libc_posix_memalign), has_libc_memalign))]
    {
        p = libc::memalign(boundary, size);
        if p.is_null() {
            return null_mut();
        }
    }
    #[cfg(all(not(has_libc_posix_memalign), not(has_libc_memalign)))]
    {
        p = libc::malloc(size);
        if p.is_null() {
            return null_mut();
        }
    }

    assert!((p as m_iptr_t) & (boundary as m_iptr_t - 1) == 0);
    p
}

// Block specified signal for calling thread
#[no_mangle]
pub unsafe extern "C" fn m_signal_block(sig: c_int) -> c_int {
    let mut sig_mask: libc::sigset_t = zeroed();
    libc::sigemptyset(addr_of_mut!(sig_mask));
    libc::sigaddset(addr_of_mut!(sig_mask), sig);
    libc::pthread_sigmask(libc::SIG_BLOCK, addr_of_mut!(sig_mask), null_mut())
}

// Unblock specified signal for calling thread
#[no_mangle]
pub unsafe extern "C" fn m_signal_unblock(sig: c_int) -> c_int {
    let mut sig_mask: libc::sigset_t = zeroed();
    libc::sigemptyset(addr_of_mut!(sig_mask));
    libc::sigaddset(addr_of_mut!(sig_mask), sig);
    libc::pthread_sigmask(libc::SIG_UNBLOCK, addr_of_mut!(sig_mask), null_mut())
}

// Set non-blocking mode on a file descriptor
#[no_mangle]
pub unsafe extern "C" fn m_fd_set_non_block(fd: c_int) -> c_int {
    let flags: c_int = libc::fcntl(fd, libc::F_GETFL, 0);

    if flags < 0 {
        return -1;
    }

    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK)
}

// Sync a memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_sync(addr: *mut c_void, len: size_t) -> c_int {
    libc::msync(addr, len, libc::MS_SYNC)
}

// Sync all mappings of a memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_sync_all(addr: *mut c_void, len: size_t) -> c_int {
    libc::msync(addr, len, libc::MS_SYNC | libc::MS_INVALIDATE)
}

// Unmap a memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_unmap(addr: *mut c_void, len: size_t) -> c_int {
    libc::munmap(addr, len)
}

// Return a memory zone or NULL on error
unsafe fn mmap_or_null(addr: *mut c_void, length: size_t, prot: c_int, flags: c_int, fd: c_int, offset: off_t) -> *mut c_void {
    let ptr: *mut c_void = libc::mmap(addr, length, prot, flags, fd, offset);
    if ptr == libc::MAP_FAILED {
        return null_mut();
    }

    ptr
}

// Map a memory zone as an executable area
#[no_mangle]
pub unsafe extern "C" fn memzone_map_exec_area(len: size_t) -> *mut u_char {
    mmap_or_null(null_mut(), len, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED | libc::MAP_ANONYMOUS, -1, 0).cast::<_>()
}

// Map a memory zone from a file
#[no_mangle]
pub unsafe extern "C" fn memzone_map_file(fd: c_int, len: size_t) -> *mut u_char {
    mmap_or_null(null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0).cast::<_>()
}

// Map a memory zone from a ro file
#[no_mangle]
pub unsafe extern "C" fn memzone_map_file_ro(fd: c_int, len: size_t) -> *mut u_char {
    mmap_or_null(null_mut(), len, libc::PROT_READ, libc::MAP_PRIVATE, fd, 0).cast::<_>()
}

// Map a memory zone from a file, with copy-on-write (COW)
#[no_mangle]
pub unsafe extern "C" fn memzone_map_cow_file(fd: c_int, len: size_t) -> *mut u_char {
    mmap_or_null(null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_PRIVATE, fd, 0).cast::<_>()
}

// Create a file to serve as a memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_create_file(filename: *mut c_char, len: size_t, ptr: *mut *mut u_char) -> c_int {
    let mut fd: c_int = libc::open(filename, libc::O_CREAT | libc::O_RDWR, libc::S_IRWXU as c_uint);
    if fd == -1 {
        libc::perror(c"memzone_create_file: open".as_ptr());
        return -1;
    }

    if libc::ftruncate(fd, len as off_t) == -1 {
        libc::perror(c"memzone_create_file: ftruncate".as_ptr());
        libc::close(fd);
        return -1;
    }

    *ptr = memzone_map_file(fd, len);

    if (*ptr).is_null() {
        libc::close(fd);
        fd = -1;
    }

    fd
}

// Open a file to serve as a COW memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_open_cow_file(filename: *mut c_char, len: size_t, ptr: *mut *mut u_char) -> c_int {
    let mut fd: c_int = libc::open(filename, libc::O_RDONLY, libc::S_IRWXU as c_uint);

    if fd == -1 {
        libc::perror(c"memzone_open_file: open".as_ptr());
        return -1;
    }

    *ptr = memzone_map_cow_file(fd, len);

    if (*ptr).is_null() {
        libc::close(fd);
        fd = -1;
    }

    fd
}

// Open a file and map it in memory
#[no_mangle]
pub unsafe extern "C" fn memzone_open_file(filename: *mut c_char, ptr: *mut *mut u_char, fsize: *mut off_t) -> c_int {
    let mut fprop: libc::stat = zeroed();

    let fd: c_int = libc::open(filename, libc::O_RDWR, libc::S_IRWXU as c_uint);
    if fd == -1 {
        return -1;
    }

    let err_fstat_mmap = || {
        libc::close(fd);
        -1
    };
    if libc::fstat(fd, addr_of_mut!(fprop)) == -1 {
        return err_fstat_mmap();
    }

    *fsize = fprop.st_size;
    *ptr = memzone_map_file(fd, *fsize as size_t);
    if (*ptr).is_null() {
        return err_fstat_mmap();
    }

    fd
}

#[no_mangle]
pub unsafe extern "C" fn memzone_open_file_ro(filename: *mut c_char, ptr: *mut *mut u_char, fsize: *mut off_t) -> c_int {
    let mut fprop: libc::stat = zeroed();

    let fd: c_int = libc::open(filename, libc::O_RDONLY, libc::S_IRWXU as c_uint);
    if fd == -1 {
        return -1;
    }

    let err_fstat_mmap = || {
        libc::close(fd);
        -1
    };
    if libc::fstat(fd, addr_of_mut!(fprop)) == -1 {
        return err_fstat_mmap();
    }

    *fsize = fprop.st_size;
    *ptr = memzone_map_file_ro(fd, *fsize as size_t);
    if (*ptr).is_null() {
        return err_fstat_mmap();
    }

    fd
}

// Compute NVRAM checksum
#[no_mangle]
pub unsafe extern "C" fn nvram_cksum(mut ptr: *mut m_uint16_t, mut count: size_t) -> m_uint16_t {
    let mut sum: m_uint32_t = 0;

    while count > 1 {
        sum += libc::ntohs(*ptr) as m_uint32_t;
        ptr = ptr.add(1);
        count -= size_of::<m_uint16_t>();
    }

    if count > 0 {
        sum += ((libc::ntohs(*ptr) as m_uint32_t) & 0xFF) << 8;
    }

    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    (!sum) as m_uint16_t
}

// Byte-swap a memory block
#[no_mangle]
pub unsafe extern "C" fn mem_bswap32(ptr: *mut c_void, len: size_t) {
    let mut p: *mut m_uint32_t = ptr.cast::<_>(); // not aligned
    let count: size_t = len >> 2;

    for _ in 0..count {
        write_unaligned(p, swap32(read_unaligned(p)));
        p = p.add(1);
    }
}

// Reverse a byte
#[no_mangle]
pub unsafe extern "C" fn m_reverse_u8(val: m_uint8_t) -> m_uint8_t {
    let mut res: m_uint8_t = 0;

    for i in 0..8 {
        if (val & (1 << i)) != 0 {
            res |= 1 << (7 - i);
        }
    }

    res
}

// Generate a pseudo random block of data
#[no_mangle]
pub unsafe extern "C" fn m_randomize_block(buf: *mut m_uint8_t, len: size_t) {
    for i in 0..len {
        *buf.add(i) = (libc::rand() & 0xFF) as m_uint8_t;
    }
}

// Free an FD pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_free(pool: *mut fd_pool_t) {
    let mut p: *mut fd_pool_t;
    let mut next: *mut fd_pool_t;

    p = pool;
    while !p.is_null() {
        next = (*p).next;

        for i in 0..FD_POOL_MAX {
            if (*p).fd[i] != -1 {
                libc::shutdown((*p).fd[i], 2);
                libc::close((*p).fd[i]);
            }
        }

        if pool != p {
            libc::free(p.cast::<_>());
        }
        p = next;
    }
}

// Initialize an empty pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_init(pool: *mut fd_pool_t) {
    for i in 0..FD_POOL_MAX {
        (*pool).fd[i] = -1;
    }

    (*pool).next = null_mut();
}

// Get a free slot for a FD in a pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_get_free_slot(pool: *mut fd_pool_t, slot: *mut *mut c_int) -> c_int {
    let mut p: *mut fd_pool_t;

    p = pool;
    while !p.is_null() {
        for i in 0..FD_POOL_MAX {
            if (*p).fd[i] == -1 {
                *slot = addr_of_mut!((*p).fd[i]);
                return 0;
            }
        }
        p = (*p).next;
    }

    // No free slot, allocate a new pool
    p = libc::malloc(size_of::<fd_pool_t>()).cast::<_>();
    if p.is_null() {
        return -1;
    }

    fd_pool_init(p);
    *slot = addr_of_mut!((*p).fd[0]);

    (*p).next = (*pool).next;
    (*pool).next = p;
    0
}

// Fill a FD set and get the maximum FD in order to use with select
#[no_mangle]
pub unsafe extern "C" fn fd_pool_set_fds(pool: *mut fd_pool_t, fds: *mut libc::fd_set) -> c_int {
    let mut p: *mut fd_pool_t;
    let mut max_fd: c_int = -1;

    p = pool;
    while !p.is_null() {
        for i in 0..FD_POOL_MAX {
            if (*p).fd[i] != -1 {
                libc::FD_SET((*p).fd[i], fds);

                if (*p).fd[i] > max_fd {
                    max_fd = (*p).fd[i];
                }
            }
        }
        p = (*p).next;
    }

    max_fd
}

// Send a buffer to all FDs of a pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_send(pool: *mut fd_pool_t, buffer: *mut c_void, len: size_t, flags: c_int) -> c_int {
    let mut p: *mut fd_pool_t;
    let mut res: ssize_t;
    let mut err: c_int;

    p = pool;
    err = 0;
    while !p.is_null() {
        for i in 0..FD_POOL_MAX {
            if (*p).fd[i] == -1 {
                continue;
            }

            res = libc::send((*p).fd[i], buffer, len, flags);

            if res as size_t != len {
                libc::shutdown((*p).fd[i], 2);
                libc::close((*p).fd[i]);
                (*p).fd[i] = -1;
                err += 1;
            }
        }
        p = (*p).next;
    }

    err
}

// Call a function for each FD having incoming data
#[no_mangle]
pub unsafe extern "C" fn fd_pool_check_input(pool: *mut fd_pool_t, fds: *mut libc::fd_set, cbk: Option<extern "C" fn(fd_slot: *mut c_int, opt: *mut c_void)>, opt: *mut c_void) -> c_int {
    let mut p: *mut fd_pool_t;
    let mut count: c_int;

    p = pool;
    count = 0;
    while !p.is_null() {
        for i in 0..FD_POOL_MAX {
            if ((*p).fd[i] != -1) && libc::FD_ISSET((*p).fd[i], fds) {
                cbk.expect("cbk")(addr_of_mut!((*p).fd[i]), opt);
                count += 1;
            }
        }
        p = (*p).next;
    }

    count
}

// Equivalent to fprintf, but for a posix fd
macro_rules! fd_printf {
    ($fd:expr, $flags:expr, $fmt:expr$(, $arg:expr)*) => {{
        let fd: c_int = $fd;
        let flags: c_int = $flags;
        let fmt: &str = $fmt;
        let args: &[&dyn sprintf::Printf] = &[$(&crate::_extra::Printf($arg)),*];
        let mut buffer = std::mem::transmute::<Vec<u8>,Vec<c_char>>(sprintf::vsprintf(fmt, args).expect("fd_printf").into_bytes());
        buffer.push(0);
        libc::send(fd, buffer.as_ptr().cast::<_>(), libc::strlen(buffer.as_ptr()), flags) as c_int
    }};
}
pub(crate) use fd_printf;
