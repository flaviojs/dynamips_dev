//! This crate contains a C-to-rust conversion of dynamips.
//!
//! The focus of this crate is a simple C-to-rust conversion of dynamips.
//! Safe rust code will be developed and placed in other crates as needed.
#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

pub mod _ext;
pub mod mempool;
pub mod net;
/// cbindgen:ignore
pub mod prelude;
pub mod utils;
