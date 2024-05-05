//! Common includes, types, defines and platform specific stuff.

use crate::prelude::*;

// True/False definitions
pub const TRUE: c_int = 1;

// Common types
pub type m_uint64_t = c_ulonglong;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_common_type() {
        assert_eq!(size_of::<m_uint64_t>(), 64 / 8);
    }
}
