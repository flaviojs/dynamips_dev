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

/// DS1620 commands
const DS1620_READ_TEMP: u8 = 0xAA;
const DS1620_READ_COUNTER: u8 = 0xA0;
const DS1620_READ_SLOPE: u8 = 0xA9;
const DS1620_WRITE_TH: u8 = 0x01;
const DS1620_WRITE_TL: u8 = 0x02;
const DS1620_READ_TH: u8 = 0xA1;
const DS1620_READ_TL: u8 = 0xA2;
const DS1620_START_CONVT: u8 = 0xEE;
const DS1620_STOP_CONVT: u8 = 0x22;
const DS1620_WRITE_CONFIG: u8 = 0x0C;
const DS1620_READ_CONFIG: u8 = 0xAC;

/// DS1620 config register
const DS1620_CONFIG_STATUS_DONE: u8 = 0x80;
const DS1620_CONFIG_STATUS_THF: u8 = 0x40;
const DS1620_CONFIG_STATUS_TLF: u8 = 0x20;
const DS1620_CONFIG_STATUS_CPU: u8 = 0x02;
const DS1620_CONFIG_STATUS_1SHOT: u8 = 0x01;

/// Size of various operations in bits (command, config and temp data)
const DS1620_CMD_SIZE: c_uint = 8;
const DS1620_CONFIG_SIZE: c_uint = 8;
const DS1620_TEMP_SIZE: c_uint = 9;

/// Internal states // TODO enum
const DS1620_STATE_CMD_IN: c_uint = 0;
const DS1620_STATE_DATA_IN: c_uint = 1;
const DS1620_STATE_DATA_OUT: c_uint = 2;

/// Set CLK bit
#[no_mangle]
pub unsafe extern "C" fn ds1620_set_clk_bit(d: *mut ds1620_data, clk_bit: u_int) {
    (*d).clk_bit = clk_bit;
}

/// Update status register (TH/TL values)
unsafe fn ds1620_update_status(d: *mut ds1620_data) {
    if (*d).temp >= (*d).reg_th.into() {
        (*d).reg_config |= DS1620_CONFIG_STATUS_THF;
    }

    if (*d).temp <= (*d).reg_tl.into() {
        (*d).reg_config |= DS1620_CONFIG_STATUS_TLF;
    }
}

/// Set temperature
#[no_mangle]
pub unsafe extern "C" fn ds1620_set_temp(d: *mut ds1620_data, temp: c_int) {
    (*d).temp = temp << 1;
    ds1620_update_status(d);
}

#[no_mangle]
pub extern "C" fn _export(_: *mut ds1620_data) {}
