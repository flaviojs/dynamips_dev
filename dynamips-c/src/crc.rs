//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//!
//! CRC functions.

use crate::dynamips_common::*;
use std::ffi::c_int;

// Compute a CRC-12 hash on a 32-bit integer
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn crc12_hash_u32(mut val: m_uint32_t) -> m_uint32_t {
    let mut crc: m_uint32_t = 0;

    for _ in 0..4 {
        crc = (crc >> 8) ^ crc12_array[((crc ^ val) & 0xff) as usize] as m_uint32_t;
        val >>= 8;
    }

    crc
}

// Compute a CRC-16 hash on a 32-bit integer
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn crc16_hash_u32(mut val: m_uint32_t) -> m_uint32_t {
    let mut crc: m_uint32_t = 0;

    for _ in 0..4 {
        crc = (crc >> 8) ^ crc16_array[((crc ^ val) & 0xff) as usize] as m_uint32_t;
        val >>= 8;
    }

    crc
}

// Compute a CRC-32 on the specified block
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn crc32_compute(crc_accum: m_uint32_t, ptr: *mut m_uint8_t, len: c_int) -> m_uint32_t {
    let mut c: m_uint32_t = crc_accum;

    for n in 0..len {
        c = crc32_array[((c ^ *ptr.offset(n as isize) as m_uint32_t) & 0xff) as usize] ^ (c >> 8);
    }

    !c
}

const CRC12_POLY: m_uint16_t = 0x0f01;
const CRC16_POLY: m_uint16_t = 0xa001;
const CRC32_POLY: m_uint32_t = 0xedb88320;

// CRC tables
pub static mut crc12_array: [m_uint16_t; 256] = [0; 256];
pub static mut crc16_array: [m_uint16_t; 256] = [0; 256];
pub static mut crc32_array: [m_uint32_t; 256] = [0; 256];

// Initialize CRC-12 algorithm
unsafe fn crc12_init() {
    for i in 0..256 {
        let mut crc: m_uint16_t = 0;
        let mut c: m_uint16_t = i as m_uint16_t;

        for _ in 0..8 {
            if ((crc ^ c) & 0x0001) != 0 {
                crc = (crc >> 1) ^ CRC12_POLY;
            } else {
                crc >>= 1;
            }
            c >>= 1;
        }

        crc12_array[i] = crc;
    }
}

// Initialize CRC-16 algorithm
unsafe fn crc16_init() {
    for i in 0..256 {
        let mut crc: m_uint16_t = 0;
        let mut c: m_uint16_t = i as m_uint16_t;

        for _ in 0..8 {
            if ((crc ^ c) & 0x0001) != 0 {
                crc = (crc >> 1) ^ CRC16_POLY;
            } else {
                crc >>= 1;
            }
            c >>= 1;
        }

        crc16_array[i] = crc;
    }
}

/* Initialize CRC-32 algorithm */
unsafe fn crc32_init() {
    for n in 0..256 {
        let mut c: m_uint32_t = n as m_uint32_t;
        for _ in 0..8 {
            if (c & 1) != 0 {
                c = CRC32_POLY ^ (c >> 1);
            } else {
                c >>= 1;
            }
        }
        crc32_array[n] = c;
    }
}

/* Initialize CRC algorithms */
#[no_mangle]
pub unsafe extern "C" fn crc_init() {
    crc12_init();
    crc16_init();
    crc32_init();
}
