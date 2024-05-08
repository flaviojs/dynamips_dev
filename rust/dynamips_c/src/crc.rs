//! CRC functions.

use crate::dynamips_common::*;

const CRC12_POLY: u16 = 0x0f01;

/// CRC tables
static mut crc12_array: [u16; 256] = [0; 256];

/// Initialize CRC-12 algorithm
// TODO private
#[no_mangle]
pub unsafe extern "C" fn crc12_init() {
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
