//! Extra stuff that does not come from the dynamips C code.

use std::ffi::c_uchar;
use std::ffi::c_uint;
use std::ffi::c_ulong;

// Non-standard types. The C header that contains them is unknown.
pub type u_char = c_uchar;
pub type u_int = c_uint;
pub type u_long = c_ulong;

/// Make sure cbindgen exports types by using them as arguments in this empty function.
#[rustfmt::skip]
#[no_mangle]
pub extern "C" fn _export(
    _: crate::dynamips_common::m_int16_t,
    _: crate::dynamips_common::m_int32_t,
    _: crate::dynamips_common::m_int64_t,
    _: crate::dynamips_common::m_int8_t,
    _: crate::dynamips_common::m_iptr_t,
    _: crate::dynamips_common::m_tmcnt_t,
    _: crate::dynamips_common::m_uint16_t,
    _: crate::dynamips_common::m_uint32_t,
    _: crate::dynamips_common::m_uint64_t,
    _: crate::dynamips_common::m_uint8_t,
    _: u_char,
    _: u_int,
    _: u_long,
) {
}
