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
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::needless_range_loop)]
#![allow(deprecated)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]
#![allow(unused_macros)]

pub mod _extra;
#[cfg(test)]
pub mod _tests;

pub mod base64;
pub mod cisco_eeprom;
pub mod crc;
pub mod dynamips_common;
pub mod net;
pub mod utils;
