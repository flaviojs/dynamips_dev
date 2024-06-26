//! Utility functions.

#[no_mangle]
pub unsafe extern "C" fn _export_utils(_: ppc_insn_t) {}

use crate::dynamips_common::*;
use crate::prelude::*;

pub type fd_pool_t = fd_pool;
pub type insn_exec_page_t = insn_exec_page;
pub type mts_map_t = mts_map;
pub type mts32_entry_t = mts32_entry;
pub type mts64_entry_t = mts64_entry;

/// Host CPU Types
pub const CPU_x86: c_int = 0;
pub const CPU_amd64: c_int = 1;
pub const CPU_nojit: c_int = 2;

/// cbindgen:no-export
pub const JIT_CPU: c_int = {
    #[cfg(target_arch = "x86")]
    {
        CPU_x86
    }
    #[cfg(target_arch = "x86_64")]
    {
        CPU_amd64
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        CPU_nojit
    }
};

/// Number of host registers available for JIT
/// cbindgen:no-export
pub const JIT_HOST_NREG: usize = {
    if JIT_CPU == CPU_x86 {
        8
    } else if JIT_CPU == CPU_amd64 {
        16
    } else {
        0
    }
};

// Host to VM (big-endian) conversion functions
#[no_mangle]
pub extern "C" fn htovm16(x: u16) -> u16 {
    x.to_be()
}
#[no_mangle]
pub extern "C" fn htovm32(x: u32) -> u32 {
    x.to_be()
}
#[no_mangle]
pub extern "C" fn htovm64(x: u64) -> u64 {
    x.to_be()
}

#[no_mangle]
pub extern "C" fn vmtoh16(x: u16) -> u16 {
    u16::from_be(x)
}
#[no_mangle]
pub extern "C" fn vmtoh32(x: u32) -> u32 {
    u32::from_be(x)
}
#[no_mangle]
pub extern "C" fn vmtoh64(x: u64) -> u64 {
    u64::from_be(x)
}

pub const FD_POOL_MAX: usize = 16;

/// FD pool
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fd_pool {
    fd: [c_int; FD_POOL_MAX],
    next: *mut fd_pool_t,
}

/// MIPS instruction
pub type mips_insn_t = m_uint32_t;

/// PowerPC instruction
pub type ppc_insn_t = m_uint32_t;

/// cbindgen:no-export
#[repr(C)]
pub struct insn_exec_page {
    _todo: u8,
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

/// cbindgen:no-export
#[repr(C)]
pub struct mts32_entry {
    _todo: u8,
}

/// cbindgen:no-export
#[repr(C)]
pub struct mts64_entry {
    _todo: u8,
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

/// Dynamic sprintf
#[macro_export]
macro_rules! dyn_sprintf {
    ($fmt:expr$(, $arg:expr)*) => {
        {
            let fmt: *const c_char = $fmt;
            let args: &[&dyn sprintf::Printf] = &[$(&Printf($arg)),*];
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

/// Equivalent to fprintf, but for a posix fd
#[macro_export]
macro_rules! fd_printf {
    ($fd:expr, $flags:expr, $fmt:expr$(, $arg:expr)*) => {
        {
            let fd: c_int = $fd;
            let flags: c_int = $flags;
            let fmt: *mut c_char = $fmt;
            let args: &[&dyn sprintf::Printf] = &[$(&Printf($arg)),*];
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

/// Normalize a size
#[no_mangle]
pub unsafe extern "C" fn normalize_size(val: u_int, nb: u_int, shift: c_int) -> u_int {
    ((val + nb - 1) & !(nb - 1)) >> shift
}

/// Get current time in number of msec since epoch
#[no_mangle]
pub unsafe extern "C" fn m_gettime() -> m_tmcnt_t {
    let mut tvp: libc::timeval = zeroed::<_>();

    libc::gettimeofday(addr_of_mut!(tvp), null_mut());
    (tvp.tv_sec as m_tmcnt_t) * 1000 + (tvp.tv_usec as m_tmcnt_t) / 1000
}

/// Get current time in number of usec since epoch
#[no_mangle]
pub unsafe extern "C" fn m_gettime_usec() -> m_tmcnt_t {
    let mut tvp: libc::timeval = zeroed::<_>();

    libc::gettimeofday(addr_of_mut!(tvp), null_mut());
    (tvp.tv_sec as m_tmcnt_t) * 1000000 + (tvp.tv_usec as m_tmcnt_t)
}

/// Get current time in number of ms (localtime)
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

/// Block specified signal for calling thread
#[no_mangle]
pub unsafe extern "C" fn m_signal_block(sig: c_int) -> c_int {
    let mut sig_mask: libc::sigset_t = zeroed::<_>();
    libc::sigemptyset(addr_of_mut!(sig_mask));
    libc::sigaddset(addr_of_mut!(sig_mask), sig);
    libc::pthread_sigmask(libc::SIG_BLOCK, addr_of_mut!(sig_mask), null_mut())
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

/// Sign-extension
#[inline(always)]
pub unsafe fn sign_extend(x: m_int64_t, mut len: c_int) -> m_int64_t {
    len = 64 - len;
    (x << len) >> len
}

/// Extract bits from a 32-bit values
#[inline]
pub unsafe fn bits(val: m_uint32_t, start: c_int, end: c_int) -> c_int {
    ((val >> start) & ((1 << (end - start + 1)) - 1)) as c_int
}

/// Free an FD pool
#[no_mangle]
pub unsafe extern "C" fn fd_pool_free(pool: *mut fd_pool_t) {
    let mut next_p: *mut fd_pool_t = pool;
    while !next_p.is_null() {
        let p: *mut fd_pool_t = next_p;
        next_p = (*p).next;

        for i in 0..FD_POOL_MAX {
            if (*p).fd[i] != -1 {
                libc::shutdown((*p).fd[i], 2);
                libc::close((*p).fd[i]);
            }
        }

        if pool != p {
            libc::free(p.cast::<_>());
        }
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
    let mut err: c_int = 0;
    let mut next_p: *mut fd_pool_t = pool;
    while !next_p.is_null() {
        let p: *mut fd_pool_t = next_p;
        next_p = (*p).next;
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
    }

    err
}

/// Call a function for each FD having incoming data
#[no_mangle]
pub unsafe extern "C" fn fd_pool_check_input(pool: *mut fd_pool_t, fds: *mut libc::fd_set, cbk: Option<unsafe extern "C" fn(fd_slot: *mut c_int, opt: *mut c_void)>, opt: *mut c_void) -> c_int {
    let mut count: c_int = 0;
    let mut p: *mut fd_pool_t = pool;
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

/// Generate a pseudo random block of data
#[no_mangle]
pub unsafe extern "C" fn m_randomize_block(buf: *mut m_uint8_t, len: size_t) {
    for i in 0..len {
        *buf.add(i) = (libc::rand() & 0xFF) as m_uint8_t;
    }
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

/// Get a byte-swapped 32-bit value on a non-aligned area
#[no_mangle]
#[inline]
pub unsafe extern "C" fn m_ntoh32(ptr: *mut m_uint8_t) -> m_uint32_t {
    let val: m_uint32_t = ((*ptr.add(0) as m_uint32_t) << 24) | ((*ptr.add(1) as m_uint32_t) << 16) | ((*ptr.add(2) as m_uint32_t) << 8) | *ptr.add(3) as m_uint32_t;
    val
}

/// Set a byte-swapped 32-bit value on a non-aligned area
#[no_mangle]
#[inline]
pub unsafe extern "C" fn m_hton32(ptr: *mut m_uint8_t, val: m_uint32_t) {
    *ptr.add(0) = (val >> 24) as m_uint8_t;
    *ptr.add(1) = (val >> 16) as m_uint8_t;
    *ptr.add(2) = (val >> 8) as m_uint8_t;
    *ptr.add(3) = val as m_uint8_t;
}

/// Get a byte-swapped 16-bit value on a non-aligned area
#[inline]
#[no_mangle]
pub unsafe extern "C" fn m_ntoh16(ptr: *mut m_uint8_t) -> m_uint16_t {
    let val: m_uint16_t = ((*ptr.add(0) as m_uint16_t) << 8) | *ptr.add(1) as m_uint16_t;
    val
}
