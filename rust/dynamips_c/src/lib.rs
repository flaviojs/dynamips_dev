//! This crate contains a C-to-rust conversion of dynamips.
//!
//! The focus of this crate is a simple C-to-rust conversion of dynamips.
//! Safe rust code will be developed and placed in other crates as needed.
#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod _ext;
pub mod crc;
pub mod dynamips_common;
pub mod hash;
pub mod mempool;
pub mod net;
/// cbindgen:ignore
pub mod prelude;
pub mod ptask;
pub mod rbtree;
pub mod registry;
pub mod sbox;
pub mod timer;
pub mod utils;
