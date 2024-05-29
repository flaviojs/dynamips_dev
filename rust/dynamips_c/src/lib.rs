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
pub mod base64;
pub mod cisco_eeprom;
pub mod cpu;
pub mod crc;
pub mod dev_c1700;
pub mod dev_c2600;
pub mod dev_c2691;
pub mod dev_c3600;
pub mod dev_c3725;
pub mod dev_c3745;
pub mod dev_c6msfc1;
pub mod dev_c7200;
pub mod dev_ds1620;
pub mod dynamips;
pub mod dynamips_common;
pub mod fs_fat;
pub mod fs_nvram;
pub mod hash;
pub mod insn_lookup;
pub mod jit_op;
pub mod mempool;
pub mod mips64;
pub mod mips64_exec;
pub mod mips64_jit;
pub mod net;
pub mod parser;
/// cbindgen:ignore
pub mod prelude;
pub mod ptask;
pub mod rbtree;
pub mod registry;
pub mod rommon_var;
pub mod sbox;
pub mod tcb;
pub mod timer;
pub mod utils;
pub mod vm;
