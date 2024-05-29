//! Generic Cisco 3725 routines and definitions (EEPROM,...).

use crate::prelude::*;

/// Offset of simulated NVRAM in ROM flash
pub const C3725_NVRAM_OFFSET: size_t = 0xE0000;
pub const C3725_NVRAM_SIZE: size_t = 0x1C000; // with backup
