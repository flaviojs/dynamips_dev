//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//! Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
//!
//! Generic Cisco 1700 routines and definitions (EEPROM,...).

use crate::_private::*;

/// Reserved space for ROM in NVRAM
pub const C1700_NVRAM_ROM_RES_SIZE: size_t = 2048;
