//! This crate contains a C-to-rust conversion of dynamips.
//!
//! The focus of this crate is a simple conversion of dynamips.
//! Safe rust code will be developed later and placed in other crates.
//!
//! # Converting C to rust
//!
//! #### Code
//! Try to keep the converted code close to the original C code.
//! Avoid changing logic, prefer FIXME annotations instead of fixes.
//!
//! #### Macros
//! They do unsafe text replacement before compiling the code.
//! Convert to `const` or `type` or `fn` or `macro!`.
//! ```rust
//! const SOME_CONST: std::ffi::c_int = 1;
//!
//! type SOME_TYPE = std::ffi::c_int;
//!
//! unsafe fn SOME_FN(p: *mut u8) -> std::ffi::c_int { *p as std::ffi::c_int }
//!
//! #[macro_export]
//! macro_rules! SOME_MACRO {
//!     ($arg:expr, $($tt:tt)*) => {
//!         // do stuff
//!     };
//! }
//! use SOME_MACRO;
//! ```
//!
//! # Raw numbers
//! Before being assigned to a variable, a number has a type determined by the prefix, suffix, and value.
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
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod _export;
/// cbindgen:ignore
pub mod _private;
pub mod atm;
pub mod atm_bridge;
pub mod atm_vsar;
pub mod base64;
pub mod cisco_card;
pub mod cisco_eeprom;
pub mod cpu;
pub mod crc;
pub mod dev_am79c971;
pub mod dev_c1700;
pub mod dev_c2600;
pub mod dev_c2691;
pub mod dev_c3600;
pub mod dev_c3725;
pub mod dev_c3745;
pub mod dev_c6msfc1;
pub mod dev_c7200;
pub mod dev_ds1620;
pub mod dev_gt;
pub mod dev_rom;
pub mod dev_vtty;
pub mod device;
pub mod dynamips;
pub mod dynamips_common;
pub mod fs_fat;
pub mod fs_nvram;
#[cfg(feature = "ENABLE_GEN_ETH")]
pub mod gen_eth;
pub mod hash;
pub mod insn_lookup;
pub mod jit_op;
#[cfg(feature = "ENABLE_LINUX_ETH")]
pub mod linux_eth;
pub mod memory;
pub mod mempool;
pub mod mips64;
pub mod mips64_cp0;
pub mod mips64_exec;
pub mod mips64_jit;
pub mod mips64_mem;
pub mod net;
pub mod net_io;
pub mod net_io_bridge;
pub mod net_io_filter;
pub mod parser;
pub mod pci_dev;
pub mod pci_io;
pub mod ppc32;
pub mod ppc32_exec;
pub mod ppc32_jit;
pub mod ppc32_mem;
pub mod ptask;
pub mod rbtree;
pub mod registry;
pub mod rommon_var;
pub mod sbox;
pub mod tcb;
pub mod timer;
pub mod utils;
pub mod vm;
