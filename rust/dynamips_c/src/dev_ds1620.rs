//! Dallas DS1620 Temperature sensors.

use crate::dynamips_common::*;
use crate::prelude::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ds1620_data {
    pub state: u_int,
    pub clk_bit: u_int,
    pub temp: c_int,

    /// command input
    pub cmd: m_uint8_t,
    pub cmd_pos: u_int,

    /// data input/output
    pub data: m_uint16_t,
    pub data_pos: u_int,
    pub data_len: u_int,

    /// registers
    pub reg_config: m_uint8_t,
    pub reg_th: m_uint16_t,
    pub reg_tl: m_uint16_t,
}

#[no_mangle]
pub extern "C" fn _export(_: *mut ds1620_data) {}
