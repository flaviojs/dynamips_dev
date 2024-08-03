//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot.  All rights reserved.
//!
//! Utility functions.

use crate::_private::*;
use crate::dynamips_common::*;

pub type fd_pool_t = fd_pool;
pub type insn_exec_page_t = insn_exec_page;
pub type m_list_t = m_list;
pub type mts_map_t = mts_map;
pub type mts32_entry_t = mts32_entry;
pub type mts64_entry_t = mts64_entry;

/// Host CPU Types
pub const CPU_x86: c_int = 0;
pub const CPU_amd64: c_int = 1;
pub const CPU_nojit: c_int = 2;

#[cfg(all(feature = "USE_MIPS64_X86_TRANS", feature = "USE_PPC32_X86_TRANS"))]
pub const JIT_CPU: c_int = CPU_x86;
#[cfg(all(feature = "USE_MIPS64_AMD64_TRANS", feature = "USE_PPC32_AMD64_TRANS"))]
pub const JIT_CPU: c_int = CPU_amd64;
#[cfg(all(feature = "USE_MIPS64_NOJIT_TRANS", feature = "USE_PPC32_NOJIT_TRANS"))]
pub const JIT_CPU: c_int = CPU_nojit;

/// Number of host registers available for JIT
#[cfg(all(feature = "USE_MIPS64_X86_TRANS", feature = "USE_PPC32_X86_TRANS"))]
pub const JIT_HOST_NREG: usize = 8;
#[cfg(all(feature = "USE_MIPS64_AMD64_TRANS", feature = "USE_PPC32_AMD64_TRANS"))]
pub const JIT_HOST_NREG: usize = 16;
#[cfg(all(feature = "USE_MIPS64_NOJIT_TRANS", feature = "USE_PPC32_NOJIT_TRANS"))]
pub const JIT_HOST_NREG: usize = 0;

// Host to VM (big-endian) conversion functions
#[no_mangle]
pub unsafe extern "C" fn htovm16(x: u16) -> u16 {
    if cfg!(target_endian = "little") {
        htons(x)
    } else {
        x
    }
}
#[no_mangle]
pub unsafe extern "C" fn htovm32(x: u32) -> u32 {
    if cfg!(target_endian = "little") {
        htonl(x)
    } else {
        x
    }
}
#[no_mangle]
pub unsafe extern "C" fn htovm64(x: u64) -> u64 {
    if cfg!(target_endian = "little") {
        swap64(x)
    } else {
        x
    }
}

#[no_mangle]
pub unsafe extern "C" fn vmtoh16(x: u16) -> u16 {
    if cfg!(target_endian = "little") {
        ntohs(x)
    } else {
        x
    }
}
#[no_mangle]
pub unsafe extern "C" fn vmtoh32(x: u32) -> u32 {
    if cfg!(target_endian = "little") {
        ntohl(x)
    } else {
        x
    }
}
#[no_mangle]
pub unsafe extern "C" fn vmtoh64(x: u64) -> u64 {
    if cfg!(target_endian = "little") {
        swap64(x)
    } else {
        x
    }
}

/// FD pool
pub const FD_POOL_MAX: usize = 16;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fd_pool {
    pub fd: [c_int; FD_POOL_MAX],
    pub next: *mut fd_pool,
}

/// Translated block function pointer
pub type insn_tblock_fptr = Option<unsafe extern "C" fn()>;

/// Host executable page
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct insn_exec_page {
    pub ptr: *mut u_char,
    pub next: *mut insn_exec_page_t,
}

/// MIPS instruction
pub type mips_insn_t = m_uint32_t;

/// PowerPC instruction
pub type ppc_insn_t = m_uint32_t;

/// Macros for double linked list
#[cfg(feature = "USE_UNSTABLE")]
#[macro_export]
macro_rules! M_LIST_ADD {
    ($item:expr, $head:expr, $prefix:ident) => {
        paste! {
            (*$item).[<$prefix _next>] = $head;
            (*$item).[<$prefix _pprev>] = addr_of_mut!($head);

            if !$head.is_null() {
                (*$head).[<$prefix _pprev>] = addr_of_mut!((*$item).[<$prefix _next>]);
            }

            $head = $item;
        }
    };
}
#[cfg(feature = "USE_UNSTABLE")]
pub use M_LIST_ADD;

#[cfg(feature = "USE_UNSTABLE")]
#[macro_export]
macro_rules! M_LIST_REMOVE {
    ($item:expr, $prefix:ident) => {
        paste! {
            if !(*$item).[<$prefix _pprev>].is_null() {
                if !(*$item).[<$prefix _next>].is_null() {
                    (*(*$item).[<$prefix _next>]).[<$prefix _pprev>] = (*$item).[<$prefix _pprev>];
                }

                *(*$item).[<$prefix _pprev>] = (*$item).[<$prefix _next>];

                (*$item).[<$prefix _pprev>] = null_mut();
                (*$item).[<$prefix _next>]  = null_mut();
            }
        }
    };
}
#[cfg(feature = "USE_UNSTABLE")]
pub use M_LIST_REMOVE;

/// List item
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct m_list {
    pub data: *mut c_void,
    pub next: *mut m_list_t,
}

/// MTS mapping info
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

/// Invalid VTLB entry
pub const MTS_INV_ENTRY_MASK: m_uint32_t = 0x00000001;

/// MTS entry flags
pub const MTS_FLAG_DEV: m_uint32_t = 0x000000001; // Virtual device used
pub const MTS_FLAG_COW: m_uint32_t = 0x000000002; // Copy-On-Write
pub const MTS_FLAG_EXEC: m_uint32_t = 0x000000004; // Exec page
pub const MTS_FLAG_RO: m_uint32_t = 0x000000008; // Read-only page

pub const MTS_FLAG_WRCATCH: m_uint32_t = MTS_FLAG_RO | MTS_FLAG_COW; // Catch writes

/// Virtual TLB entry (32-bit MMU)
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct mts32_entry {
    pub gvpa: m_uint32_t,  // Guest Virtual Page Address
    pub gppa: m_uint32_t,  // Guest Physical Page Address
    pub hpa: m_iptr_t,     // Host Page Address
    pub flags: m_uint32_t, // Flags
}

/// Virtual TLB entry (64-bit MMU)
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct mts64_entry {
    pub gvpa: m_uint64_t,  // Guest Virtual Page Address
    pub gppa: m_uint64_t,  // Guest Physical Page Address
    pub hpa: m_iptr_t,     // Host Page Address
    pub flags: m_uint32_t, // Flags
}

/// Host register allocation
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

/// Check status of a bit
#[inline]
#[no_mangle]
pub unsafe extern "C" fn check_bit(old: u_int, new: u_int, bit: u_int) -> c_int {
    let mask: c_int = 1 << bit;

    if (old & mask as u_int) != 0 && (new & mask as u_int) == 0 {
        return 1; // bit unset
    }

    if (old & mask as u_int) == 0 && (new & mask as u_int) != 0 {
        return 2; // bit set
    }

    // no change
    0
}

/// Sign-extension
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn sign_extend(x: m_int64_t, mut len: c_int) -> m_int64_t {
    len = 64 - len;
    (x << len) >> len
}

/// Sign-extension (32-bit)
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn sign_extend_32(x: m_int32_t, mut len: c_int) -> m_int32_t {
    len = 32 - len;
    (x << len) >> len
}

/// Extract bits from a 32-bit values
#[inline]
#[no_mangle]
pub unsafe extern "C" fn bits(val: m_uint32_t, start: c_int, end: c_int) -> c_int {
    ((val >> start) & ((1 << (end - start + 1)) - 1) as m_uint32_t) as c_int
}

/// Normalize a size
#[inline]
#[no_mangle]
pub unsafe extern "C" fn normalize_size(val: u_int, nb: u_int, shift: c_int) -> u_int {
    ((val + nb - 1) & !(nb - 1)) >> shift
}

/// Convert a 16-bit number between little and big endian
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn swap16(value: m_uint16_t) -> m_uint16_t {
    (value >> 8) | ((value & 0xFF) << 8)
}

/// Convert a 32-bit number between little and big endian
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn swap32(value: m_uint32_t) -> m_uint32_t {
    let mut result: m_uint32_t;

    result = value >> 24;
    result |= ((value >> 16) & 0xff) << 8;
    result |= ((value >> 8) & 0xff) << 16;
    result |= (value & 0xff) << 24;
    result
}

/// Convert a 64-bit number between little and big endian
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn swap64(value: m_uint64_t) -> m_uint64_t {
    let mut result: m_uint64_t;

    result = (swap32((value & 0xffffffff) as m_uint32_t) as m_uint64_t) << 32;
    result |= swap32((value >> 32) as m_uint32_t) as m_uint64_t;
    result
}

/// Get current time in number of msec since epoch
#[no_mangle]
pub unsafe extern "C" fn m_gettime() -> m_tmcnt_t {
    let mut tvp: libc::timeval = zeroed::<_>();

    libc::gettimeofday(addr_of_mut!(tvp), null_mut());
    (tvp.tv_sec as m_tmcnt_t) * 1000 + (tvp.tv_usec as m_tmcnt_t) / 1000
}

/// Get current time in number of usec since epoch
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_gettime_usec() -> m_tmcnt_t {
    let mut tvp: libc::timeval = zeroed::<_>();

    libc::gettimeofday(addr_of_mut!(tvp), null_mut());
    (tvp.tv_sec as m_tmcnt_t) * 1000000 + (tvp.tv_usec as m_tmcnt_t)
}

/// Get current time in number of ms (localtime)
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_gettime_adj() -> m_tmcnt_t {
    let mut tvp: libc::timeval = zeroed::<_>();
    let mut tmx: libc::tm = zeroed::<_>();
    let gmt_adjust: libc::time_t;
    let mut ct: libc::time_t;

    libc::gettimeofday(addr_of_mut!(tvp), null_mut());
    ct = tvp.tv_sec;
    libc::localtime_r(addr_of_mut!(ct), addr_of_mut!(tmx));

    #[cfg(not(has_libc_tm_tm_gmtoff))]
    {
        // #if defined(__CYGWIN__) || defined(SUNOS)
        gmt_adjust = -(if tmx.tm_isdst != 0 { c_timezone() - 3600 } else { c_timezone() });
    }
    #[cfg(has_libc_tm_tm_gmtoff)]
    {
        gmt_adjust = tmx.tm_gmtoff;
    }

    tvp.tv_sec += gmt_adjust;
    (tvp.tv_sec as m_tmcnt_t) * 1000 + (tvp.tv_usec as m_tmcnt_t) / 1000
}

/// Get a byte-swapped 16-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_ntoh16(ptr: *mut m_uint8_t) -> m_uint16_t {
    let val: m_uint16_t = ((*ptr.add(0) as m_uint16_t) << 8) | *ptr.add(1) as m_uint16_t;
    val
}

/// Get a byte-swapped 32-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_ntoh32(ptr: *mut m_uint8_t) -> m_uint32_t {
    let val: m_uint32_t = ((*ptr.add(0) as m_uint32_t) << 24) | ((*ptr.add(1) as m_uint32_t) << 16) | ((*ptr.add(2) as m_uint32_t) << 8) | *ptr.add(3) as m_uint32_t;
    val
}

/// Set a byte-swapped 16-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_hton16(ptr: *mut m_uint8_t, val: m_uint16_t) {
    *ptr.add(0) = (val >> 8) as m_uint8_t;
    *ptr.add(1) = val as m_uint8_t;
}

/// Set a byte-swapped 32-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_hton32(ptr: *mut m_uint8_t, val: m_uint32_t) {
    *ptr.add(0) = (val >> 24) as m_uint8_t;
    *ptr.add(1) = (val >> 16) as m_uint8_t;
    *ptr.add(2) = (val >> 8) as m_uint8_t;
    *ptr.add(3) = val as m_uint8_t;
}

/// Global logfile
#[no_mangle]
pub static mut log_file: *mut libc::FILE = null_mut();

/// Add an element to a list
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

/// Dynamic sprintf
#[macro_export]
macro_rules! dyn_sprintf {
    ($fmt:expr$(, $arg:expr)*) => {
        {
            let fmt: *const c_char = $fmt;
            let args: &[&dyn sprintf::Printf] = &[$(&CustomPrintf($arg)),*];
            match sprintf::vsprintf(CStr::from_ptr(fmt).to_str().unwrap(), args) {
                Ok(s) => {
                    let p = libc::malloc(s.len() + 1);
                    if !p.is_null() {
                        libc::memcpy(p, s.as_str().as_ptr().cast::<_>(), s.len());
                        *p.cast::<u8>().add(s.len()) = 0;
                        p.cast::<c_char>()
                    } else {
                        libc::perror(cstr!("dyn_sprintf: malloc"));
                        null_mut()
                    }
                }
                Err(err) => {
                    let msg = CString::new(format!("dyn_sprintf: {}", err)).unwrap();
                    libc::fputs(msg.as_c_str().as_ptr(), c_stderr());
                    null_mut()
                }
            }
        }
    }
}
pub use dyn_sprintf;

/// Split a string
#[no_mangle]
pub unsafe extern "C" fn m_strsplit(mut str_: *mut c_char, delim: c_char, array: *mut *mut c_char, max_count: c_int) -> c_int {
    let mut pos: c_int = 0;
    let mut len: size_t;
    let mut ptr: *mut c_char;

    for i in 0..max_count {
        *array.offset(i as isize) = null_mut();
    }

    loop {
        if pos == max_count {
            for i in 0..max_count {
                libc::free((*array.offset(i as isize)).cast::<_>());
            }
            return -1;
        }

        ptr = libc::strchr(str_, delim as c_int);
        if ptr.is_null() {
            ptr = str_.wrapping_add(libc::strlen(str_));
        }

        len = ptr.offset_from(str_) as size_t;

        *array.offset(pos as isize) = libc::malloc(len + 1).cast::<_>();
        if (*array.offset(pos as isize)).is_null() {
            for i in 0..max_count {
                libc::free((*array.offset(i as isize)).cast::<_>());
            }
            return -1;
        }

        libc::memcpy((*array.offset(pos as isize)).cast::<_>(), str_.cast::<_>(), len);
        *(*array.offset(pos as isize)).add(len) = 0;

        str_ = ptr.wrapping_add(1);
        pos += 1;
        if *ptr == 0 {
            break;
        }
    }

    pos
}

/// Tokenize a string
#[no_mangle]
pub unsafe extern "C" fn m_strtok(mut str_: *mut c_char, delim: c_char, array: *mut *mut c_char, max_count: c_int) -> c_int {
    for i in 0..max_count {
        *array.offset(i as isize) = null_mut();
    }

    let mut pos: c_int = 0;
    loop {
        if pos == max_count {
            for i in 0..max_count {
                libc::free(*array.offset(i as isize).cast::<_>());
            }
            return -1;
        }

        let mut ptr: *mut c_char = libc::strchr(str_, delim as c_int);
        if ptr.is_null() {
            ptr = str_.add(libc::strlen(str_));
        }

        let len: size_t = ptr.offset_from(str_) as size_t;

        *array.offset(pos as isize) = libc::malloc(len + 1).cast::<_>();
        if (*array.offset(pos as isize)).is_null() {
            for i in 0..max_count {
                libc::free(*array.offset(i as isize).cast::<_>());
            }
            return -1;
        }

        libc::memcpy(*array.offset(pos as isize).cast::<_>(), str_.cast::<_>(), len);
        *((*array.offset(pos as isize)).add(len)) = 0;

        while *ptr == delim {
            ptr = ptr.add(1);
        }

        str_ = ptr;
        pos += 1;
        if *ptr == 0 {
            break;
        }
    }

    pos
}

/// Quote a string
#[no_mangle]
pub unsafe extern "C" fn m_strquote(buffer: *mut c_char, buf_len: size_t, str_: *mut c_char) -> *mut c_char {
    let p: *mut c_char = libc::strpbrk(str_, cstr!(" \t\"'"));

    if p.is_null() {
        return str_;
    }

    libc::snprintf(buffer, buf_len, cstr!("\"%s\""), str_);
    buffer
}

/// Decode from hex.
///
/// hex to raw bytes, returning count of bytes.
/// maxlen limits output buffer size.
#[no_mangle]
pub unsafe extern "C" fn hex_decode(mut out: *mut u_char, mut in_: *const u_char, maxlen: c_int) -> c_int {
    const BAD: c_char = -1 as c_char;
    #[rustfmt::skip]
    static hexval: [c_char; 112] = [
        BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
          0,  1,  2,  3,   4,  5,  6,  7,   8,  9,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD, 10, 11, 12,  13, 14, 15,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        BAD, 10, 11, 12,  13, 14, 15,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD
    ];
    let mut len: c_int = 0;
    let mut empty: bool = true;

    while len < maxlen {
        if *in_ as usize >= hexval.len() || hexval[*in_ as usize] == BAD {
            break;
        }

        if empty {
            *out = (hexval[*in_ as usize] as u_char) << 4;
        } else {
            *out |= hexval[*in_ as usize] as u_char;
            out = out.add(1);
            len += 1;
        }
        in_ = in_.add(1);
        empty = !empty;
    }

    len
}

/// Ugly function that dumps a structure in hexa and ascii.
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

        libc::fprintf(f_output, cstr!("%4.4x: "), i);

        for tmp in 0..x {
            libc::fprintf(f_output, cstr!("%2.2x "), *pkt.add((i + tmp) as usize) as u_int);
        }
        for _ in x..16 {
            libc::fprintf(f_output, cstr!("   "));
        }

        for tmp in 0..x {
            let c: u_char = *pkt.add((i + tmp) as usize);

            if c.is_ascii_alphanumeric() {
                libc::fprintf(f_output, cstr!("%c"), c as u_int);
            } else {
                libc::fputs(cstr!("."), f_output);
            }
        }

        i += x;
        libc::fprintf(f_output, cstr!("\n"));
    }

    libc::fprintf(f_output, cstr!("\n"));
    libc::fflush(f_output);
}

/// Logging function
pub unsafe fn m_flog(fd: *mut libc::FILE, module: *mut c_char, fmt: *mut c_char, args: &[&dyn sprintf::Printf]) {
    let mut spec: libc::timespec = zeroed::<_>();
    let mut tmn: libc::tm = zeroed::<_>();

    if !fd.is_null() {
        libc::clock_gettime(libc::CLOCK_REALTIME, addr_of_mut!(spec));
        libc::gmtime_r(addr_of!(spec.tv_sec), addr_of_mut!(tmn));

        // NOTE never use strftime for timestamps, it is crashy
        libc::fprintf(fd, cstr!("%d-%02d-%02dT%02d:%02d:%02d.%03dZ %s: "), tmn.tm_year + 1900, tmn.tm_mon + 1, tmn.tm_mday, tmn.tm_hour, tmn.tm_min, tmn.tm_sec, (spec.tv_nsec / 1000000) as c_int, module);
        if let Ok(s) = sprintf::vsprintf(CStr::from_ptr(fmt).to_str().unwrap(), args) {
            let s = CString::new(s).unwrap();
            libc::fputs(s.as_c(), fd);
        }
        libc::fflush(fd);
    }
}

/// Logging function
#[macro_export]
macro_rules! m_log {
    ($module:expr, $fmt:expr$(, $arg:expr)*) => {
        let module: *mut c_char = $module;
        let fmt: *mut c_char = $fmt;
        let args: &[&dyn sprintf::Printf] = &[$(&CustomPrintf($arg)),*];

        m_flog(log_file, module, fmt, args)
    };
}
pub use m_log;

/// Write an array of string to a logfile
#[no_mangle]
pub unsafe extern "C" fn m_flog_str_array(fd: *mut libc::FILE, count: c_int, str_: *mut *mut c_char) {
    for i in 0..count {
        libc::fprintf(fd, cstr!("%s "), *str_.offset(i as isize));
    }

    libc::fprintf(fd, cstr!("\n"));
    libc::fflush(fd);
}

/// Returns a line from specified file (remove trailing '\n')
#[no_mangle]
pub unsafe extern "C" fn m_fgets(buffer: *mut c_char, size: c_int, fd: *mut libc::FILE) -> *mut c_char {
    *buffer = 0;
    libc::fgets(buffer, size, fd);

    let len = libc::strlen(buffer);
    if len == 0 {
        return null_mut();
    }

    // remove trailing '\n'
    if *buffer.add(len - 1) == b'\n' as c_char {
        *buffer.add(len - 1) = 0;
    }

    buffer
}

/// Read a file and returns it in a buffer
#[no_mangle]
pub unsafe extern "C" fn m_read_file(filename: *const c_char, buffer: *mut *mut u_char, length: *mut size_t) -> c_int {
    // open
    let fd: *mut libc::FILE = libc::fopen(filename, cstr!("rb"));
    if fd.is_null() {
        return -1;
    }

    // len
    libc::fseek(fd, 0, libc::SEEK_END);
    let len: c_long = libc::ftell(fd);
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

/// Allocate aligned memory
#[no_mangle]
pub unsafe extern "C" fn m_memalign(boundary: size_t, size: size_t) -> *mut c_void {
    let mut p: *mut c_void;

    #[cfg(has_libc_posix_memalign)]
    {
        p = null_mut();
        if libc::posix_memalign(addr_of_mut!(p), boundary, size) != 0 {
            return null_mut();
        }
    }
    #[cfg(not(has_libc_posix_memalign))]
    {
        #[cfg(has_libc_memalign)]
        {
            p = libc::memalign(boundary, size);
            if p.is_null() {
                return null_mut();
            }
        }
        #[cfg(not(has_libc_memalign))]
        {
            p = libc::malloc(size);
            if p.is_null() {
                return null_mut();
            }
        }
    }

    assert_eq!(((p as m_iptr_t) & (boundary as m_iptr_t - 1)), 0);
    p
}

/// Block specified signal for calling thread
#[no_mangle]
pub unsafe extern "C" fn m_signal_block(sig: c_int) -> c_int {
    let mut sig_mask: libc::sigset_t = zeroed::<_>();
    libc::sigemptyset(addr_of_mut!(sig_mask));
    libc::sigaddset(addr_of_mut!(sig_mask), sig);
    libc::pthread_sigmask(libc::SIG_BLOCK, addr_of_mut!(sig_mask), null_mut())
}

/// Unblock specified signal for calling thread
#[no_mangle]
pub unsafe extern "C" fn m_signal_unblock(sig: c_int) -> c_int {
    let mut sig_mask: libc::sigset_t = zeroed::<_>();
    libc::sigemptyset(addr_of_mut!(sig_mask));
    libc::sigaddset(addr_of_mut!(sig_mask), sig);
    libc::pthread_sigmask(libc::SIG_UNBLOCK, addr_of_mut!(sig_mask), null_mut())
}

/// Set non-blocking mode on a file descriptor
#[no_mangle]
pub unsafe extern "C" fn m_fd_set_non_block(fd: c_int) -> c_int {
    let flags: c_int = libc::fcntl(fd, libc::F_GETFL, 0);
    if flags < 0 {
        return -1;
    }

    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK)
}

/// Sync a memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_sync(addr: *mut c_void, len: size_t) -> c_int {
    libc::msync(addr, len, libc::MS_SYNC)
}

/// Sync all mappings of a memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_sync_all(addr: *mut c_void, len: size_t) -> c_int {
    libc::msync(addr, len, libc::MS_SYNC | libc::MS_INVALIDATE)
}

/// Unmap a memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_unmap(addr: *mut c_void, len: size_t) -> c_int {
    libc::munmap(addr, len)
}

/// Return a memory zone or NULL on error
unsafe fn mmap_or_null(addr: *mut c_void, length: size_t, prot: c_int, flags: c_int, fd: c_int, offset: libc::off_t) -> *mut c_void {
    let ptr: *mut c_void = libc::mmap(addr, length, prot, flags, fd, offset);
    if ptr == libc::MAP_FAILED {
        return null_mut();
    }

    ptr
}

/// Map a memory zone as an executable area
#[no_mangle]
pub unsafe extern "C" fn memzone_map_exec_area(len: size_t) -> *mut u_char {
    mmap_or_null(null_mut(), len, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED | libc::MAP_ANONYMOUS, -1, 0 as libc::off_t).cast::<_>()
}

/// Map a memory zone from a file
#[no_mangle]
pub unsafe extern "C" fn memzone_map_file(fd: c_int, len: size_t) -> *mut u_char {
    mmap_or_null(null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, fd, 0 as libc::off_t).cast::<_>()
}

/// Map a memory zone from a ro file
#[no_mangle]
pub unsafe extern "C" fn memzone_map_file_ro(fd: c_int, len: size_t) -> *mut u_char {
    mmap_or_null(null_mut(), len, libc::PROT_READ, libc::MAP_PRIVATE, fd, 0 as libc::off_t).cast::<_>()
}

/// Map a memory zone from a file, with copy-on-write (COW)
#[no_mangle]
pub unsafe extern "C" fn memzone_map_cow_file(fd: c_int, len: size_t) -> *mut u_char {
    mmap_or_null(null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_PRIVATE, fd, 0 as libc::off_t).cast::<_>()
}

/// Create a file to serve as a memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_create_file(filename: *mut c_char, len: size_t, ptr: *mut *mut u_char) -> c_int {
    let mut fd: c_int = libc::open(filename, libc::O_CREAT | libc::O_RDWR, libc::S_IRWXU as c_int);
    if fd == -1 {
        libc::perror(cstr!("memzone_create_file: open"));
        return -1;
    }

    if libc::ftruncate(fd, len as libc::off_t) == -1 {
        libc::perror(cstr!("memzone_create_file: ftruncate"));
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

/// Open a file to serve as a COW memory zone
#[no_mangle]
pub unsafe extern "C" fn memzone_open_cow_file(filename: *mut c_char, len: size_t, ptr: *mut *mut u_char) -> c_int {
    let mut fd: c_int = libc::open(filename, libc::O_RDONLY, libc::S_IRWXU as c_int);
    if fd == -1 {
        libc::perror(cstr!("memzone_open_file: open"));
        return -1;
    }

    *ptr = memzone_map_cow_file(fd, len);

    if (*ptr).is_null() {
        libc::close(fd);
        fd = -1;
    }

    fd
}

/// Open a file and map it in memory
#[no_mangle]
pub unsafe extern "C" fn memzone_open_file(filename: *mut c_char, ptr: *mut *mut u_char, fsize: *mut libc::off_t) -> c_int {
    let mut fprop: libc::stat = zeroed::<_>();

    let fd: c_int = libc::open(filename, libc::O_RDWR, libc::S_IRWXU as c_int);
    if fd == -1 {
        return -1;
    }

    if libc::fstat(fd, addr_of_mut!(fprop)) == -1 {
        libc::close(fd);
        return -1;
    }

    *fsize = fprop.st_size;
    *ptr = memzone_map_file(fd, *fsize as size_t);
    if (*ptr).is_null() {
        libc::close(fd);
        return -1;
    }

    fd
}

#[no_mangle]
pub unsafe extern "C" fn memzone_open_file_ro(filename: *mut c_char, ptr: *mut *mut u_char, fsize: *mut libc::off_t) -> c_int {
    let mut fprop: libc::stat = zeroed::<_>();

    let fd: c_int = libc::open(filename, libc::O_RDONLY, libc::S_IRWXU as c_int);
    if fd == -1 {
        return -1;
    }

    if libc::fstat(fd, addr_of_mut!(fprop)) == -1 {
        libc::close(fd);
        return -1;
    }

    *fsize = fprop.st_size;
    *ptr = memzone_map_file_ro(fd, *fsize as size_t);
    if (*ptr).is_null() {
        libc::close(fd);
        return -1;
    }

    fd
}

/// Compute NVRAM checksum
#[no_mangle]
pub unsafe extern "C" fn nvram_cksum(mut ptr: *mut m_uint16_t, mut count: size_t) -> m_uint16_t {
    let mut sum: m_uint32_t = 0;

    while count > 1 {
        sum += ntohs(*ptr) as m_uint32_t;
        ptr = ptr.add(1);
        count -= size_of::<m_uint16_t>();
    }

    if count > 0 {
        sum += ((ntohs(*ptr) & 0xFF) << 8) as m_uint32_t;
    }

    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !(sum as m_uint16_t)
}

/// Byte-swap a memory block
#[no_mangle]
pub unsafe extern "C" fn mem_bswap32(ptr: *mut c_void, len: size_t) {
    let mut p: *mut m_uint32_t = ptr.cast::<_>(); // not aligned
    let count: size_t = len >> 2;

    for _ in 0..count {
        *p = swap32(*p);
        p = p.add(1);
    }
}

/// Reverse a byte
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

/// Generate a pseudo random block of data
#[no_mangle]
pub unsafe extern "C" fn m_randomize_block(buf: *mut m_uint8_t, len: size_t) {
    for i in 0..len {
        *buf.add(i) = (libc::rand() & 0xFF) as m_uint8_t;
    }
}

/// Free an FD pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_free(pool: *mut fd_pool_t) {
    let mut p: *mut fd_pool_t = pool;
    let mut next: *mut fd_pool_t;
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

/// Initialize an empty pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_init(pool: *mut fd_pool_t) {
    for i in 0..FD_POOL_MAX {
        (*pool).fd[i] = -1;
    }

    (*pool).next = null_mut();
}

/// Get a free slot for a FD in a pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_get_free_slot(pool: *mut fd_pool_t, slot: *mut *mut c_int) -> c_int {
    let mut p: *mut fd_pool_t = pool;
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

/// Fill a FD set and get the maximum FD in order to use with select
#[no_mangle]
pub unsafe extern "C" fn fd_pool_set_fds(pool: *mut fd_pool_t, fds: *mut libc::fd_set) -> c_int {
    let mut p: *mut fd_pool_t = pool;
    let mut max_fd: c_int = -1;
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

/// Send a buffer to all FDs of a pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_send(pool: *mut fd_pool_t, buffer: *mut c_void, len: size_t, flags: c_int) -> c_int {
    let mut p: *mut fd_pool_t = pool;
    let mut next: *mut fd_pool_t;
    let mut err: c_int = 0;
    while !p.is_null() {
        next = (*p).next;
        for i in 0..FD_POOL_MAX {
            if (*p).fd[i] == -1 {
                continue;
            }

            let res: ssize_t = libc::send((*p).fd[i], buffer, len, flags);

            if res != len as ssize_t {
                libc::shutdown((*p).fd[i], 2);
                libc::close((*p).fd[i]);
                (*p).fd[i] = -1;
                err += 1;
            }
        }
        p = next;
    }

    err
}

/// Call a function for each FD having incoming data
#[no_mangle]
pub unsafe extern "C" fn fd_pool_check_input(pool: *mut fd_pool_t, fds: *mut libc::fd_set, cbk: Option<unsafe extern "C" fn(fd_slot: *mut c_int, opt: *mut c_void)>, opt: *mut c_void) -> c_int {
    let mut p: *mut fd_pool_t = pool;
    let mut count: c_int = 0;
    while !p.is_null() {
        for i in 0..FD_POOL_MAX {
            if (*p).fd[i] != -1 && libc::FD_ISSET((*p).fd[i], fds) {
                cbk.unwrap()(addr_of_mut!((*p).fd[i]), opt);
                count += 1;
            }
        }
        p = (*p).next;
    }

    count
}

/// Equivalent to fprintf, but for a posix fd
#[macro_export]
macro_rules! fd_printf {
    ($fd:expr, $flags:expr, $fmt:expr$(, $arg:expr)*) => {
        {
            let fd: c_int = $fd;
            let flags: c_int = $flags;
            let fmt: *mut c_char = $fmt;
            let args: &[&dyn sprintf::Printf] = &[$(&CustomPrintf($arg)),*];
            match sprintf::vsprintf(CStr::from_ptr(fmt).to_str().unwrap(), args) {
                Ok(s) => {
                    libc::send(fd, s.as_c_void(), s.len(), flags)
                }
                Err(_) => {
                    libc::EINVAL as ssize_t
                }
            }
        }
    };
}
pub use fd_printf;
