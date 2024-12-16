//! Common includes, types, defines and platform specific stuff.
//!
//! This header should be included before other headers.
//! This header should not contain code.
//!
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Copyright (c) 2014 Flávio J. Saraiva <flaviojs2005@gmail.com>

use std::ffi::c_int;
use std::ffi::c_longlong;
use std::ffi::c_schar;
use std::ffi::c_short;
use std::ffi::c_uchar;
use std::ffi::c_uint;
use std::ffi::c_ulong;
use std::ffi::c_ulonglong;
use std::ffi::c_ushort;

// True/False definitions
pub const FALSE: c_int = 0;

pub const TRUE: c_int = 1;

// Endianness
pub const ARCH_BIG_ENDIAN: c_int = 0x4321;
pub const ARCH_LITTLE_ENDIAN: c_int = 0x1234;

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
macro_rules! m_max {
    ($a:expr, $b:expr) => {
        if $a > $b {
            $a
        } else {
            $b
        }
    };
}
pub(crate) use m_max;
macro_rules! m_min {
    ($a:expr, $b:expr) => {
        if $a < $b {
            $a
        } else {
            $b
        }
    };
}
pub(crate) use m_min;

// A simple macro for adjusting pointers
macro_rules! PTR_ADJUST {
    ($type:ty, $ptr:expr, $size:expr) => {
        $ptr.cast::<std::ffi::c_char>().offset($size) as $type
    };
}
pub(crate) use PTR_ADJUST;

// Size of a field in a structure
macro_rules! SIZEOF {
    ($st:ty, $($field:ident).+) => {{
        let uninit = std::mem::MaybeUninit::<$st>::uninit();
        let ptr = uninit.as_ptr();
        // SAFETY it is getting the field pointer as shown in the MaybeUninit<T> docs, so not UB
        let ptr_field = unsafe { std::ptr::addr_of!((*ptr).$($field).+) };
        const fn _size_of<T>(_: *const T) -> usize {
            std::mem::size_of::<T>()
        }
        _size_of(ptr_field)
    }};
    ($st:ty, $($field:ident).+[$index:expr]) => {{
        let uninit = std::mem::MaybeUninit::<$st>::uninit();
        let ptr = uninit.as_ptr();
        // SAFETY it is getting the field pointer as shown in the MaybeUninit<T> docs, so not UB
        let ptr_field = unsafe { std::ptr::addr_of!((*ptr).$($field).+) };
        const fn _size_of<T, const N: usize>(_: *const [T; N]) -> usize {
            std::mem::size_of::<T>()
        }
        _size_of(ptr_field)
    }};
}
pub(crate) use SIZEOF;

// Compute offset of a field in a structure
macro_rules! OFFSET {
    ($st:ty, $($field:ident).+) => {{
        std::mem::offset_of!($st, $($field).+) as std::ffi::c_long
    }};
    ($st:ty, $($field:ident).+[$index:expr]) => {{
        (std::mem::offset_of!($st, $($field).+) + $index * SIZEOF!($st, $($field).+[$index])) as std::ffi::c_long
    }};
}
pub(crate) use OFFSET;

#[cfg(test)]
mod tests {

    #[test]
    fn test_m_min() {
        use crate::dynamips_common::*;

        assert_eq!(m_min!(0, 1), 0);
        assert_eq!(m_min!(1, 1), 1);
        assert_eq!(m_min!(2, 1), 1);
    }

    #[test]
    fn test_m_max() {
        use crate::dynamips_common::*;

        assert_eq!(m_max!(0, 1), 1);
        assert_eq!(m_max!(1, 1), 1);
        assert_eq!(m_max!(2, 1), 2);
    }

    #[test]
    fn test_ptr_adjust() {
        use crate::dynamips_common::*;

        let mut buf: [u8; 0x10] = [0; 0x10];
        let ptr: *mut u8 = buf.as_mut_ptr();
        unsafe {
            assert_eq!(PTR_ADJUST!(*mut u8, ptr, 0x00), ptr);

            *PTR_ADJUST!(*mut m_uint32_t, ptr, 0x00) = 0x12345678_u32.to_be();
            *PTR_ADJUST!(*mut m_uint16_t, ptr, 0x0c) = 0x9012_u16.to_be();
        }
        assert_eq!(buf, [0x12, 0x34, 0x56, 0x78, 0, 0, 0, 0, 0, 0, 0, 0, 0x90, 0x12, 0, 0]);
    }

    #[test]
    fn test_sizeof_and_offset() {
        use crate::dynamips_common::*;

        #[repr(C)]
        struct S {
            field_u64: u64,
            field_u32: u32,
            field_u16: u16,
            field_u8: u8,
            arr: [u8; 5],
            inner: InnerS,
        }
        #[repr(C)]
        struct InnerS {
            field: u16,
            arr: [u8; 2],
        }

        assert_eq!(SIZEOF!(S, field_u64), 8);
        assert_eq!(SIZEOF!(S, field_u32), 4);
        assert_eq!(SIZEOF!(S, field_u16), 2);
        assert_eq!(SIZEOF!(S, field_u8), 1);
        assert_eq!(SIZEOF!(S, arr[0]), 1);
        assert_eq!(SIZEOF!(S, arr[1]), 1);
        assert_eq!(SIZEOF!(S, inner.field), 2);
        assert_eq!(SIZEOF!(S, inner.arr[0]), 1);
        assert_eq!(SIZEOF!(S, inner.arr[1]), 1);

        assert_eq!(OFFSET!(S, field_u64), 0);
        assert_eq!(OFFSET!(S, field_u32), 8);
        assert_eq!(OFFSET!(S, field_u16), 12);
        assert_eq!(OFFSET!(S, field_u8), 14);
        assert_eq!(OFFSET!(S, arr[0]), 15);
        assert_eq!(OFFSET!(S, arr[1]), 16);
        assert_eq!(OFFSET!(S, inner.field), 20);
        assert_eq!(OFFSET!(S, inner.arr[0]), 22);
        assert_eq!(OFFSET!(S, inner.arr[1]), 23);
    }
}
