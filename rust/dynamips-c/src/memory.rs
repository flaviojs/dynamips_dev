//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::_private::*;
use crate::dynamips_common::*;
use crate::utils::*;
#[cfg(feature = "USE_UNSTABLE")]
use std::ops::Shr;

/// MTS operation
pub const MTS_READ: u_int = 0;
pub const MTS_WRITE: u_int = 1;

/// 0.5GB value
pub const MTS_SIZE_512M: u_int = 0x20000000;

/// MTS flag bits: D (device), ACC (memory access), C (chain)
pub const MTS_FLAG_BITS: c_int = 4;
pub const MTS_FLAG_MASK: u_long = 0x0000000f_u64 as u_long;

/// Masks for MTS entries
pub const MTS_CHAIN_MASK: u_int = 0x00000001;
pub const MTS_ACC_MASK: u_int = 0x00000006;
pub const MTS_DEV_MASK: u_int = 0x00000008;
pub const MTS_ADDR_MASK: u_long = !MTS_FLAG_MASK;

/// Device ID mask and shift, device offset mask
pub const MTS_DEVID_MASK: u_int = 0xfc000000;
pub const MTS_DEVID_SHIFT: c_int = 26;
pub const MTS_DEVOFF_MASK: u_int = 0x03ffffff;

/// Memory access flags
pub const MTS_ACC_AE: u_int = 0x00000002; // Address Error
pub const MTS_ACC_T: u_int = 0x00000004; // TLB Exception
pub const MTS_ACC_U: u_int = 0x00000006; // Unexistent

/// Macro for easy hash computing
#[cfg(feature = "USE_UNSTABLE")]
#[inline]
unsafe fn MTS_SHR<T: Shr<c_int, Output = T>>(v: T, sr: c_int) -> T {
    v >> sr
}

/// Hash table size for MTS64 (default: [shift:16,bits:12])
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS64_HASH_SHIFT: c_int = 12;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS64_HASH_BITS: c_int = 14;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS64_HASH_SIZE: m_uint32_t = 1 << MTS64_HASH_BITS;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS64_HASH_MASK: m_uint32_t = MTS64_HASH_SIZE - 1;

/// Hash table size for MTS64 (default: [shift:16,bits:12])
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_SHIFT1: c_int = 12;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_SHIFT2: c_int = 20;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_BITS: c_int = 8;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_SIZE: m_uint32_t = 1 << MTS64_HASH_BITS;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_MASK: m_uint32_t = MTS64_HASH_SIZE - 1;

/// MTS64 hash on virtual addresses
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn MTS64_HASH(vaddr: m_uint64_t) -> m_uint32_t {
    (vaddr >> MTS64_HASH_SHIFT) as m_uint32_t & MTS64_HASH_MASK
}

/// MTS64 hash on virtual addresses
#[cfg(feature = "USE_UNSTABLE")]
macro_rules! MTS64_SHR {
    ($v:expr, $i:expr) => {
        paste! {
            MTS_SHR($v, [<MTS64_HASH_SHIFT $i>])
        }
    };
}
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn MTS64_HASH(vaddr: m_uint64_t) -> m_uint32_t {
    (MTS64_SHR!(vaddr, 1) ^ MTS64_SHR!(vaddr, 2)) as m_uint32_t & MTS64_HASH_MASK
}

/// Hash table size for MTS32 (default: [shift:15,bits:15])
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS32_HASH_SHIFT: c_int = 12;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS32_HASH_BITS: c_int = 14;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS32_HASH_SIZE: m_uint32_t = 1 << MTS32_HASH_BITS;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS32_HASH_MASK: m_uint32_t = MTS32_HASH_SIZE - 1;

/// Hash table size for MTS32 (default: [shift:15,bits:15])
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_SHIFT1: c_int = 12;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_SHIFT2: c_int = 20;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_BITS: c_int = 8;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_SIZE: m_uint32_t = 1 << MTS32_HASH_BITS;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_MASK: m_uint32_t = MTS32_HASH_SIZE - 1;

/// MTS32 hash on virtual addresses
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn MTS32_HASH(vaddr: m_uint32_t) -> m_uint32_t {
    (vaddr >> MTS32_HASH_SHIFT) & MTS32_HASH_MASK
}

/// MTS32 hash on virtual addresses
#[cfg(feature = "USE_UNSTABLE")]
macro_rules! MTS32_SHR {
    ($v:expr, $i:expr) => {
        paste! {
            MTS_SHR($v, [<MTS32_HASH_SHIFT $i>])
        }
    };
}
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn MTS32_HASH(vaddr: m_uint32_t) -> m_uint32_t {
    (MTS32_SHR!(vaddr, 1) ^ MTS32_SHR!(vaddr, 2)) & MTS32_HASH_MASK
}

/// Number of entries per chunk
pub const MTS64_CHUNK_SIZE: usize = 256;
pub const MTS32_CHUNK_SIZE: usize = 256;

/// MTS64: chunk definition
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct mts64_chunk {
    pub entry: [mts64_entry_t; MTS64_CHUNK_SIZE],
    pub next: *mut mts64_chunk,
    pub count: u_int,
}

/// MTS32: chunk definition
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct mts32_chunk {
    pub entry: [mts32_entry_t; MTS32_CHUNK_SIZE],
    pub next: *mut mts32_chunk,
    pub count: u_int,
}
