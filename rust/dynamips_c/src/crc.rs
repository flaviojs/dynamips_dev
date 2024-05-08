//! CRC functions.

use crate::dynamips_common::*;
use crate::prelude::*;

const CRC12_POLY: u16 = 0x0f01;
const CRC16_POLY: u16 = 0xa001;
const CRC32_POLY: u32 = 0xedb88320;

/// CRC tables
static mut crc12_array: [u16; 256] = [0; 256];
static mut crc16_array: [u16; 256] = [0; 256];
static mut crc32_array: [u32; 256] = [0; 256];

/// Initialize CRC-12 algorithm
unsafe fn crc12_init() {
    for (i, crc12) in crc12_array.iter_mut().enumerate() {
        let mut crc: u16 = 0;
        let mut c: u16 = i as u16;

        for _ in 0..8 {
            if ((crc ^ c) & 0x0001) != 0 {
                crc = (crc >> 1) ^ CRC12_POLY;
            } else {
                crc >>= 1;
            }

            c >>= 1;
        }

        *crc12 = crc;
    }
}

/// Initialize CRC-16 algorithm
unsafe fn crc16_init() {
    for (i, crc16) in crc16_array.iter_mut().enumerate() {
        let mut crc: u16 = 0;
        let mut c: u16 = i as u16;

        for _ in 0..8 {
            if ((crc ^ c) & 0x0001) != 0 {
                crc = (crc >> 1) ^ CRC16_POLY;
            } else {
                crc >>= 1;
            }

            c >>= 1;
        }

        *crc16 = crc;
    }
}

/// Initialize CRC-32 algorithm
unsafe fn crc32_init() {
    for (i, crc32) in crc32_array.iter_mut().enumerate() {
        let mut c: u32 = i as u32;
        for _ in 0..8 {
            if (c & 1) != 0 {
                c = CRC32_POLY ^ (c >> 1);
            } else {
                c >>= 1;
            }
        }
        *crc32 = c;
    }
}

/// Initialize CRC algorithms
#[no_mangle]
pub unsafe extern "C" fn crc_init() {
    crc12_init();
    crc16_init();
    crc32_init();
}

/// Compute a CRC-12 hash on a 32-bit integer
#[no_mangle]
pub unsafe extern "C" fn crc12_hash_u32(mut val: m_uint32_t) -> m_uint32_t {
    let mut crc: u32 = 0;

    for _ in 0..4 {
        crc = (crc >> 8) ^ crc12_array[((crc ^ val) & 0xff) as usize] as u32;
        val >>= 8;
    }

    crc
}

/// Compute a CRC-16 hash on a 32-bit integer
#[no_mangle]
pub unsafe extern "C" fn crc16_hash_u32(mut val: m_uint32_t) -> m_uint32_t {
    let mut crc: u32 = 0;

    for _ in 0..4 {
        crc = (crc >> 8) ^ crc16_array[((crc ^ val) & 0xff) as usize] as u32;
        val >>= 8;
    }

    crc
}

/// Compute a CRC-32 on the specified block
#[no_mangle]
pub unsafe extern "C" fn crc32_compute(crc_accum: m_uint32_t, ptr: *mut u8, len: c_int) -> m_uint32_t {
    let mut c: u32 = crc_accum;

    for n in 0..len {
        c = crc32_array[((c ^ *ptr.offset(n as isize) as u32) & 0xff) as usize] ^ (c >> 8);
    }

    !c
}
