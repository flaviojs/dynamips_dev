//! Utility functions.

use crate::dynamips_common::*;
use crate::prelude::*;

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
