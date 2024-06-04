//! Utility functions.

use crate::dynamips_common::*;
use crate::prelude::*;

pub type insn_exec_page_t = insn_exec_page;
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

/// MIPS instruction
pub type mips_insn_t = m_uint32_t;

/// cbindgen:no-export
#[repr(C)]
pub struct insn_exec_page {
    _todo: u8,
}

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
