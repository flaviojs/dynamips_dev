//! Conversion of the original dynamips C code to rust
//!
//! # Converting C to rust
//!
//! #### Code
//! Try to keep the converted code close to the original C code.
//! Avoid changing logic, prefer FIXME annotations instead of fixes.
//!
//! #### Macros
//! In C they do unsafe text replacement before compiling the code.
//! Convert to `const` or `type` or `fn` or `macro!`.
//! ```rust
//! pub const SOME_CONST: std::ffi::c_int = 1;
//!
//! pub type SOME_TYPE = std::ffi::c_int;
//!
//! pub unsafe fn SOME_FN(p: *mut u8) -> std::ffi::c_int { *p as std::ffi::c_int }
//!
//! #[macro_export]
//! macro_rules! SOME_MACRO {
//!     ($arg:expr) => {
//!         // do stuff
//!     };
//! }
//! pub use SOME_MACRO;
//! ```
//!
//! #### Numbers
//! In C, before being assigned to a variable, a number has a type determined by the prefix, suffix, and value.
//!
//! Convert unassigned numbers to:
//!  * if number has suffix => the matching type
//!  * else if number is decimal => the first type that can represent the value
//!    * c_int or c_long or c_longlong
//!  * else if number is hexadecimal or octal => the first type that can represent the value
//!    * c_int or c_uint or c_long or c_ulong or c_longlong or c_ulonglong
//!
//! Implicit conversions are error prone but should be replicated:
//!  * if type is smaller than c_int => convert to c_int
//!  * if signed op unsigned => convert to unsigned
//!    * `(signed)-1 < (unsigned)1` is actually `(unsigned)(signed)-1 < (unsigned)1`
//!
//! References:
//!  * [`cppreference:language/integer_literal`](https://en.cppreference.com/w/cpp/language/integer_literal)
//!  * [`stackoverflow:a/11310578`](https://stackoverflow.com/a/11310578)
//!  * [`stackoverflow:a/17312930`](https://stackoverflow.com/a/17312930)
//!  * [`idryman:2012/11/21/integer-promotion`](http://www.idryman.org/blog/2012/11/21/integer-promotion/)
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unused_macros)]

pub mod _prelude {
    //! Symbols that don't come from the dynamips C code.

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
}
pub mod dynamips_common;
