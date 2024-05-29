//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 2691 routines and definitions (EEPROM,...).

use crate::_private::*;

/// Offset of simulated NVRAM in ROM flash
pub const C2691_NVRAM_OFFSET: size_t = 0xE0000;
pub const C2691_NVRAM_SIZE: size_t = 0x1C000; // with backup
