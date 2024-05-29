//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! Generic Cisco MSFC1 routines and definitions (EEPROM,...).

use crate::_private::*;
use crate::dynamips_common::*;

// MSFC1 common device addresses
pub const C6MSFC1_NVRAM_ADDR: m_uint64_t = 0x1e000000;

/// Reserved space for ROM in NVRAM
pub const C6MSFC1_NVRAM_ROM_RES_SIZE: size_t = 2048;
