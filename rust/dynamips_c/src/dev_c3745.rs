//! Generic Cisco 3745 routines and definitions (EEPROM,...).

use crate::prelude::*;

// Offset of simulated NVRAM in ROM flash
pub const C3745_NVRAM_OFFSET: size_t = 0xB0000;
pub const C3745_NVRAM_SIZE: size_t = 0x4C000; // with backup
