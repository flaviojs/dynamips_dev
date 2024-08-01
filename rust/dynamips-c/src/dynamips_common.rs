//! Common includes, types, defines and platform specific stuff.
//!
//! This header should be included before other headers.
//! This header should not contain code.
//!
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Copyright (c) 2014 Flávio J. Saraiva <flaviojs2005@gmail.com>

use crate::_private::*;

// True/False definitions
pub const FALSE: c_int = 0;
pub const TRUE: c_int = 1;

// Endianness
pub const ARCH_BIG_ENDIAN: c_int = 0x4321;
pub const ARCH_LITTLE_ENDIAN: c_int = 0x1234;

#[cfg(target_endian = "big")]
pub const ARCH_BYTE_ORDER: c_int = ARCH_BIG_ENDIAN;
#[cfg(target_endian = "little")]
pub const ARCH_BYTE_ORDER: c_int = ARCH_LITTLE_ENDIAN;

pub use likely_stable::likely;
pub use likely_stable::unlikely;

// Common types
pub type m_uint8_t = c_uchar;
pub type m_int8_t = c_schar;

pub type m_uint16_t = c_ushort;
pub type m_int16_t = c_short;

pub type m_uint32_t = c_uint;
pub type m_int32_t = c_int;

pub type m_uint64_t = c_ulonglong;
pub type m_int64_t = c_longlong;

pub type m_iptr_t = c_ulong;
pub type m_tmcnt_t = m_uint64_t;

// Max and min macro
#[inline(always)]
pub fn m_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}
#[inline(always)]
pub fn m_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

/// A simple macro for adjusting pointers
/// # SAFETY
/// the caller ensures the resulting pointer is valid (points to readable memory, is aligned, is initialized)
#[macro_export]
macro_rules! PTR_ADJUST {
    ($type_:ty, $ptr:expr, $size:expr) => {
        $ptr.byte_offset($size.try_into().unwrap()) as $type_
    };
}
pub use PTR_ADJUST;

/// Size of a field in a structure
#[macro_export]
macro_rules! SIZEOF {
    ($st:ty, $field:ident) => {{
        let p: *mut $st = ::std::ptr::null_mut();
        // SAFETY this operation is safe because the data is not read and the pointer/reference is not exposed
        ::std::mem::size_of_val(unsafe { &(*p).$field }) as ::std::ffi::c_long
    }};
}
pub use SIZEOF;

/// Compute offset of a field in a structure
#[macro_export]
macro_rules! OFFSET {
    ($st:ty, $($tt:tt)*) => {
        ::std::mem::offset_of!($st, $($tt)*) as ::std::ffi::c_long
    };
}
pub use OFFSET;
