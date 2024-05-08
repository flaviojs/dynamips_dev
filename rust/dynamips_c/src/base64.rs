//! This base 64 encoding is defined in RFC2045 section 6.8,
//! "Base64 Content-Transfer-Encoding", but lines must not be broken in the
//! scheme used here.

use crate::prelude::*;

static base64digits: [u8; 64] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

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
