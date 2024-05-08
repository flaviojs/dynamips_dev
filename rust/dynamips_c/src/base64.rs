//! base64.c -- base-64 conversion routines.
//!
//! For license terms, see the file COPYING in this directory.
//!
//! This base 64 encoding is defined in RFC2045 section 6.8,
//! "Base64 Content-Transfer-Encoding", but lines must not be broken in the
//! scheme used here.

use crate::_private::*;

static base64digits: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const BAD: u8 = -1_i8 as u8;

#[rustfmt::skip]
static base64val: [u8; 128] = [
    BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
    BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
    BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD, 62, BAD,BAD,BAD, 63,
     52, 53, 54, 55,  56, 57, 58, 59,  60, 61,BAD,BAD, BAD,BAD,BAD,BAD,
    BAD,  0,  1,  2,   3,  4,  5,  6,   7,  8,  9, 10,  11, 12, 13, 14,
     15, 16, 17, 18,  19, 20, 21, 22,  23, 24, 25,BAD, BAD,BAD,BAD,BAD,
    BAD, 26, 27, 28,  29, 30, 31, 32,  33, 34, 35, 36,  37, 38, 39, 40,
     41, 42, 43, 44,  45, 46, 47, 48,  49, 50, 51,BAD, BAD,BAD,BAD,BAD
];

fn DECODE64(c: u8) -> u8 {
    if c.is_ascii() {
        base64val[c as usize]
    } else {
        BAD
    }
}

/// Encode into base64
#[no_mangle]
pub unsafe extern "C" fn base64_encode(mut out: *mut c_uchar, mut in_: *const c_uchar, mut inlen: c_int) {
    // raw bytes in quasi-big-endian order to base 64 string (NUL-terminated)
    while inlen >= 3 {
        *out = base64digits[(*in_.add(0) >> 2) as usize];
        out = out.add(1);
        *out = base64digits[(((*in_.add(0) << 4) & 0x30) | (*in_.add(1) >> 4)) as usize];
        out = out.add(1);
        *out = base64digits[(((*in_.add(1) << 2) & 0x3c) | (*in_.add(2) >> 6)) as usize];
        out = out.add(1);
        *out = base64digits[(*in_.add(2) & 0x3f) as usize];
        out = out.add(1);
        in_ = in_.add(3);
        inlen -= 3;
    }

    if inlen > 0 {
        *out = base64digits[(*in_.add(0) >> 2) as usize];
        out = out.add(1);
        let mut fragment: u8 = (*in_.add(0) << 4) & 0x30;
        if inlen > 1 {
            fragment |= *in_.add(1) >> 4;
        }
        *out = base64digits[fragment as usize];
        out = out.add(1);
        *out = if inlen < 2 { b'=' } else { base64digits[((*in_.add(1) << 2) & 0x3c) as usize] };
        out = out.add(1);
        *out = b'=';
        out = out.add(1);
    }
    *out = b'\0';
}

/// Decode from base64.
///
/// base 64 to raw bytes in quasi-big-endian order, returning count of bytes
/// maxlen limits output buffer size, set to zero to ignore.
#[no_mangle]
pub unsafe extern "C" fn base64_decode(mut out: *mut c_uchar, mut in_: *const c_uchar, maxlen: c_int) -> c_int {
    let mut len: c_int = 0;

    if *in_.add(0) == b'+' && *in_.add(1) == b' ' {
        in_ = in_.add(2);
    }
    if *in_ == b'\r' {
        return 0;
    }

    loop {
        let digit1: u8 = *in_.add(0);
        if DECODE64(digit1) == BAD {
            return -1;
        }
        let digit2: u8 = *in_.add(1);
        if DECODE64(digit2) == BAD {
            return -1;
        }
        let digit3: u8 = *in_.add(2);
        if digit3 != b'=' && DECODE64(digit3) == BAD {
            return -1;
        }
        let digit4: u8 = *in_.add(3);
        if digit4 != b'=' && DECODE64(digit4) == BAD {
            return -1;
        }
        in_ = in_.add(4);
        len += 1;

        if maxlen != 0 && len > maxlen {
            return -1;
        }

        *out = (DECODE64(digit1) << 2) | (DECODE64(digit2) >> 4);
        out = out.add(1);
        if digit3 != b'=' {
            len += 1;
            if maxlen != 0 && len > maxlen {
                return -1;
            }

            *out = ((DECODE64(digit2) << 4) & 0xf0) | (DECODE64(digit3) >> 2);
            out = out.add(1);
            if digit4 != b'=' {
                len += 1;
                if maxlen != 0 && len > maxlen {
                    return -1;
                }
                *out = ((DECODE64(digit3) << 6) & 0xc0) | DECODE64(digit4);
                out = out.add(1);
            }
        }
        if !(*in_ != 0 && *in_ != b'\r' && digit4 != b'=') {
            break;
        }
    }

    len
}
