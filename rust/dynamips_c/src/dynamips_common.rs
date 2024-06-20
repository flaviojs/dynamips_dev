//! Common includes, types, defines and platform specific stuff.

use crate::prelude::*;

// True/False definitions
pub const FALSE: c_int = 0;
pub const TRUE: c_int = 1;

// Common types
pub type m_uint8_t = c_uchar;

pub type m_int16_t = c_short;
pub type m_uint16_t = c_ushort;

pub type m_int32_t = c_int;
pub type m_uint32_t = c_uint;

pub type m_int64_t = c_longlong;
pub type m_uint64_t = c_ulonglong;

pub type m_tmcnt_t = m_uint64_t;

/// A simple macro for adjusting pointers
pub unsafe fn PTR_ADJUST<T, U>(ptr: *mut U, size: usize) -> *mut T {
    ptr.byte_add(size).cast::<_>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_common_type() {
        assert_eq!(size_of::<m_uint8_t>(), 8 / 8);
        assert_eq!(size_of::<m_uint16_t>(), 16 / 8);
        assert_eq!(size_of::<m_uint32_t>(), 32 / 8);
        assert_eq!(size_of::<m_int64_t>(), 64 / 8);
        assert_eq!(size_of::<m_uint64_t>(), 64 / 8);
    }
}
