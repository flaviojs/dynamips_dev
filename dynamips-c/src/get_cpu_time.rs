//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Author:  David Robert Nadeau
//! Site:    http://NadeauSoftware.com/
//! License: Creative Commons Attribution 3.0 Unported License
//!         http://creativecommons.org/licenses/by/3.0/deed.en_US

use std::mem::zeroed;
use std::ptr::addr_of_mut;

/// Returns the amount of CPU time used by the current process,
/// in seconds, or -1.0 if an error occurred.
#[no_mangle]
pub unsafe extern "C" fn get_cpu_time() -> f64 {
    #[cfg(target_os = "windows")]
    {
        // Windows --------------------------------------------------
        use windows_sys::Win32::Foundation::FILETIME;
        use windows_sys::Win32::Foundation::SYSTEMTIME;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        use windows_sys::Win32::System::Threading::GetProcessTimes;
        use windows_sys::Win32::System::Time::FileTimeToSystemTime;

        let mut createTime: FILETIME = zeroed();
        let mut exitTime: FILETIME = zeroed();
        let mut kernelTime: FILETIME = zeroed();
        let mut userTime: FILETIME = zeroed();
        if GetProcessTimes(GetCurrentProcess(), addr_of_mut!(createTime), addr_of_mut!(exitTime), addr_of_mut!(kernelTime), addr_of_mut!(userTime)) != -1 {
            let mut userSystemTime: SYSTEMTIME = zeroed();
            if FileTimeToSystemTime(addr_of_mut!(userTime), addr_of_mut!(userSystemTime)) != -1 {
                return (userSystemTime.wHour as f64) * 3600.0 + (userSystemTime.wMinute as f64) * 60.0 + (userSystemTime.wSecond as f64) + (userSystemTime.wMilliseconds as f64) / 1000.0;
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // AIX, BSD, Cygwin, HP-UX, Linux, OSX, and Solaris ---------
        #[cfg(has_libc_clock_gettime)] // _POSIX_TIMERS > 0
        {
            // Prefer high-res POSIX timers, when available.
            {
                const minus_one: libc::clockid_t = !0;
                #[allow(unused_mut)]
                let mut id: libc::clockid_t = minus_one;
                let mut ts: libc::timespec = zeroed();

                #[cfg(has_libc_clock_getcpuclockid)] // _POSIX_CPUTIME > 0
                {
                    libc::clock_getcpuclockid(0, addr_of_mut!(id));
                }
                #[cfg(has_libc_clock_process_cputime_id)]
                {
                    // Use known clock id for AIX, Linux, or Solaris.
                    if id == minus_one {
                        id = libc::CLOCK_PROCESS_CPUTIME_ID;
                    }
                }
                #[cfg(has_libc_clock_virtual)]
                {
                    // Use known clock id for BSD or HP-UX.
                    if id == minus_one {
                        id = libc::CLOCK_VIRTUAL;
                    }
                }
                if id != minus_one && libc::clock_gettime(id, addr_of_mut!(ts)) != -1 {
                    return (ts.tv_sec as f64) + (ts.tv_nsec as f64) / 1000000000.0;
                }
            }
        }

        #[cfg(has_libc_rusage_self)]
        {
            let mut rusage: libc::rusage = zeroed();
            if libc::getrusage(libc::RUSAGE_SELF, addr_of_mut!(rusage)) != -1 {
                return (rusage.ru_utime.tv_sec as f64) + (rusage.ru_utime.tv_usec as f64) / 1000000.0;
            }
        }

        #[cfg(has_libc__sc_clk_tck)]
        {
            let minus_one: libc::clock_t = !0;
            let ticks: f64 = libc::sysconf(libc::_SC_CLK_TCK) as f64;
            let mut tms: libc::tms = zeroed();
            if libc::times(addr_of_mut!(tms)) != minus_one {
                return (tms.tms_utime as f64) / ticks;
            }
        }

        #[cfg(has_libc_clocks_per_sec)]
        {
            let cl: libc::clock_t = libc::clock();
            if cl != -1 as libc::clock_t {
                return (cl as f64) / (CLOCKS_PER_SEC as f64);
            }
        }
    }

    -1.0 // Failed.
}
