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
pub mod amd64_codegen;
pub mod atm;
pub mod atm_bridge;
pub mod atm_vsar;
pub mod base64;
pub mod cisco_card;
pub mod cisco_eeprom;
pub mod cpu;
pub mod crc;
pub mod dev_am79c971;
pub mod dev_ap1011;
pub mod dev_bootflash;
pub mod dev_bswap;
pub mod dev_c1700;
pub mod dev_c1700_eth;
pub mod dev_c1700_iofpga;
pub mod dev_c1700_wic;
pub mod dev_c2600;
pub mod dev_c2600_eth;
pub mod dev_c2600_iofpga;
pub mod dev_c2600_pci;
pub mod dev_c2600_pcmod;
pub mod dev_c2600_wic;
pub mod dev_c2691;
pub mod dev_c2691_eth;
pub mod dev_c2691_iofpga;
pub mod dev_c2691_pcmod;
pub mod dev_c2691_serial;
pub mod dev_c2691_wic;
pub mod dev_c3600;
pub mod dev_c3600_bay;
pub mod dev_c3600_eth;
pub mod dev_c3600_iofpga;
pub mod dev_c3600_serial;
pub mod dev_c3725;
pub mod dev_c3725_eth;
pub mod dev_c3725_iofpga;
pub mod dev_c3725_pcmod;
pub mod dev_c3725_serial;
pub mod dev_c3725_wic;
pub mod dev_c3745;
pub mod dev_c3745_eth;
pub mod dev_c3745_iofpga;
pub mod dev_c3745_pcmod;
pub mod dev_c3745_serial;
pub mod dev_c3745_wic;
pub mod dev_c6msfc1;
pub mod dev_c6msfc1_iofpga;
pub mod dev_c6msfc1_mpfpga;
pub mod dev_c6sup1;
pub mod dev_c6sup1_iofpga;
pub mod dev_c6sup1_mpfpga;
pub mod dev_c7200;
pub mod dev_c7200_bri;
pub mod dev_c7200_eth;
pub mod dev_c7200_iofpga;
pub mod dev_c7200_jcpa;
pub mod dev_c7200_mpfpga;
pub mod dev_c7200_pos;
pub mod dev_c7200_serial;
pub mod dev_c7200_sram;
pub mod dev_clpd6729;
pub mod dev_dec21140;
pub mod dev_dec21x50;
pub mod dev_ds1620;
pub mod dev_flash;
pub mod dev_gt;
pub mod dev_i8254x;
pub mod dev_i8255x;
pub mod dev_lxt970a;
pub mod dev_mpc860;
pub mod dev_mueslix;
pub mod dev_mv64460;
pub mod dev_nm_16esw;
pub mod dev_ns16552;
pub mod dev_nvram;
pub mod dev_pa_a1;
pub mod dev_pa_mc8te1;
pub mod dev_pcmcia_disk;
pub mod dev_pericom;
pub mod dev_plx;
pub mod dev_plx6520cb;
pub mod dev_ram;
pub mod dev_remote;
pub mod dev_rom;
pub mod dev_sb1;
pub mod dev_sb1_io;
pub mod dev_sb1_pci;
pub mod dev_ti2050b;
pub mod dev_vtty;
pub mod dev_wic_serial;
pub mod dev_zero;
pub mod device;
pub mod dynamips;
pub mod dynamips_common;
pub mod eth_switch;
pub mod frame_relay;
pub mod fs_fat;
pub mod fs_mbr;
pub mod fs_nvram;
#[cfg(feature = "ENABLE_GEN_ETH")]
pub mod gen_eth;
pub mod get_cpu_time;
pub mod hash;
pub mod hv_atm_bridge;
pub mod hv_atmsw;
pub mod hv_c1700;
pub mod hv_c2600;
pub mod hv_c2691;
pub mod hv_c3600;
pub mod hv_c3725;
pub mod hv_c3745;
pub mod hv_c7200;
pub mod hv_ethsw;
pub mod hv_frsw;
pub mod hv_nio;
pub mod hv_nio_bridge;
pub mod hv_store;
pub mod hv_vm;
pub mod hv_vm_debug;
pub mod hypervisor;
pub mod insn_lookup;
pub mod jit_op;
#[cfg(feature = "ENABLE_LINUX_ETH")]
pub mod linux_eth;
pub mod memory;
pub mod mempool;
pub mod mips64;
#[cfg(feature = "USE_MIPS64_AMD64_TRANS")]
pub mod mips64_amd64_trans;
pub mod mips64_cp0;
pub mod mips64_exec;
pub mod mips64_jit;
pub mod mips64_mem;
#[cfg(feature = "USE_MIPS64_NOJIT_TRANS")]
pub mod mips64_nojit_trans;
#[cfg(feature = "USE_MIPS64_PPC32_TRANS")]
pub mod mips64_ppc32_trans;
#[cfg(feature = "USE_MIPS64_X86_TRANS")]
pub mod mips64_x86_trans;
pub mod mips_mts;
pub mod net;
pub mod net_io;
pub mod net_io_bridge;
pub mod net_io_filter;
pub mod nmc93cx6;
pub mod nvram_export;
pub mod parser;
pub mod pci_dev;
pub mod pci_io;
pub mod plugin;
pub mod ppc32;
#[cfg(feature = "USE_PPC32_AMD64_TRANS")]
pub mod ppc32_amd64_trans;
pub mod ppc32_exec;
pub mod ppc32_jit;
pub mod ppc32_mem;
#[cfg(feature = "USE_PPC32_NOJIT_TRANS")]
pub mod ppc32_nojit_trans;
pub mod ppc32_vmtest;
#[cfg(feature = "USE_PPC32_X86_TRANS")]
pub mod ppc32_x86_trans;
pub mod ppc_codegen;
pub mod profiler;
pub mod ptask;
pub mod rbtree;
pub mod registry;
pub mod rommon_var;
pub mod sbox;
#[cfg(feature = "USE_UNSTABLE")]
pub mod tcb;
pub mod timer;
pub mod udp_recv;
pub mod udp_send;
pub mod utils;
pub mod vm;
pub mod x86_codegen;
