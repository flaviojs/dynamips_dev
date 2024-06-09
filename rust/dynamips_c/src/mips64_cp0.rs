//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! MIPS Coprocessor 0 (System Coprocessor) implementation.
//! We don't use the JIT here, since there is no high performance needed.

use crate::_private::*;
use crate::mips64::*;

/// MIPS cp0 registers names
#[no_mangle]
#[rustfmt::skip]
pub static mut mips64_cp0_reg_names: [*mut c_char; MIPS64_CP0_REG_NR] = [
    cstr!("index"), 
    cstr!("random"), 
    cstr!("entry_lo0"), 
    cstr!("entry_lo1"), 
    cstr!("context"), 
    cstr!("pagemask"),
    cstr!("wired"),
    cstr!("info"),
    cstr!("badvaddr"), 
    cstr!("count"), 
    cstr!("entry_hi"), 
    cstr!("compare"), 
    cstr!("status"), 
    cstr!("cause"),
    cstr!("epc"), 
    cstr!("prid"), 
    cstr!("config"), 
    cstr!("ll_addr"), 
    cstr!("watch_lo"), 
    cstr!("watch_hi"), 
    cstr!("xcontext"),
    cstr!("cp0_r21"),
    cstr!("cp0_r22"),
    cstr!("cp0_r23"),
    cstr!("cp0_r24"),
    cstr!("cp0_r25"),
    cstr!("ecc"), 
    cstr!("cache_err"), 
    cstr!("tag_lo"), 
    cstr!("tag_hi"), 
    cstr!("err_epc"),
    cstr!("cp0_r31"),
];

/// Get cp0 register index given its name
#[allow(clippy::needless_range_loop)]
#[no_mangle]
pub unsafe extern "C" fn mips64_cp0_get_reg_index(name: *mut c_char) -> c_int {
    for i in 0..MIPS64_CP0_REG_NR {
        if libc::strcmp(mips64_cp0_reg_names[i], name) == 0 {
            return i as c_int;
        }
    }

    -1
}
