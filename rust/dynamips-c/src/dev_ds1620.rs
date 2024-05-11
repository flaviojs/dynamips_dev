//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Dallas DS1620 Temperature sensors.

use crate::_private::*;
use crate::dynamips_common::*;

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

/// Set CLK bit
#[no_mangle]
pub unsafe extern "C" fn ds1620_set_clk_bit(d: *mut ds1620_data, clk_bit: u_int) {
    (*d).clk_bit = clk_bit;
}

/// DS1620 commands
const DS1620_READ_TEMP: m_uint8_t = 0xAA;
const DS1620_READ_COUNTER: m_uint8_t = 0xA0;
const DS1620_READ_SLOPE: m_uint8_t = 0xA9;
const DS1620_WRITE_TH: m_uint8_t = 0x01;
const DS1620_WRITE_TL: m_uint8_t = 0x02;
const DS1620_READ_TH: m_uint8_t = 0xA1;
const DS1620_READ_TL: m_uint8_t = 0xA2;
const DS1620_START_CONVT: m_uint8_t = 0xEE;
const DS1620_STOP_CONVT: m_uint8_t = 0x22;
const DS1620_WRITE_CONFIG: m_uint8_t = 0x0C;
const DS1620_READ_CONFIG: m_uint8_t = 0xAC;

/// DS1620 config register
const DS1620_CONFIG_STATUS_DONE: m_uint8_t = 0x80;
const DS1620_CONFIG_STATUS_THF: m_uint8_t = 0x40;
const DS1620_CONFIG_STATUS_TLF: m_uint8_t = 0x20;
const DS1620_CONFIG_STATUS_CPU: m_uint8_t = 0x02;
const DS1620_CONFIG_STATUS_1SHOT: m_uint8_t = 0x01;

/// Size of various operations in bits (command, config and temp data)
const DS1620_CMD_SIZE: u_int = 8;
const DS1620_CONFIG_SIZE: u_int = 8;
const DS1620_TEMP_SIZE: u_int = 9;

/// Internal states // TODO enum
const DS1620_STATE_CMD_IN: u_int = 0;
const DS1620_STATE_DATA_IN: u_int = 1;
const DS1620_STATE_DATA_OUT: u_int = 2;

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

/// Set reset bit
#[no_mangle]
pub unsafe extern "C" fn ds1620_set_rst_bit(d: *mut ds1620_data, rst_bit: u_int) {
    if rst_bit == 0 {
        (*d).state = DS1620_STATE_CMD_IN;
        (*d).cmd_pos = 0;
        (*d).cmd = 0;
        (*d).data = 0;
        (*d).data_pos = 0;
        (*d).data_len = 0;
    }
}

/// Set state after command
unsafe fn ds1620_cmd_set_state(d: *mut ds1620_data) {
    (*d).data = 0;
    (*d).data_pos = 0;

    match (*d).cmd {
        DS1620_READ_TEMP => {
            (*d).state = DS1620_STATE_DATA_OUT;
            (*d).data_len = DS1620_TEMP_SIZE;
            (*d).data = (*d).temp as u16;
        }

        DS1620_READ_COUNTER | DS1620_READ_SLOPE => {
            (*d).state = DS1620_STATE_DATA_OUT;
            (*d).data_len = DS1620_TEMP_SIZE;
            (*d).data = 0;
        }

        DS1620_WRITE_TH | DS1620_WRITE_TL => {
            (*d).state = DS1620_STATE_DATA_IN;
            (*d).data_len = DS1620_TEMP_SIZE;
        }

        DS1620_READ_TH => {
            (*d).state = DS1620_STATE_DATA_OUT;
            (*d).data_len = DS1620_TEMP_SIZE;
            (*d).data = (*d).reg_th;
        }

        DS1620_READ_TL => {
            (*d).state = DS1620_STATE_DATA_OUT;
            (*d).data_len = DS1620_TEMP_SIZE;
            (*d).data = (*d).reg_tl;
        }

        DS1620_START_CONVT | DS1620_STOP_CONVT => {
            (*d).state = DS1620_STATE_CMD_IN;
        }

        DS1620_WRITE_CONFIG => {
            (*d).state = DS1620_STATE_DATA_IN;
            (*d).data_len = DS1620_CONFIG_SIZE;
        }

        DS1620_READ_CONFIG => {
            (*d).state = DS1620_STATE_DATA_OUT;
            (*d).data_len = DS1620_CONFIG_SIZE;
            (*d).data = (*d).reg_config as u16;
        }

        _ => {}
    }
}

/// Execute command
unsafe fn ds1620_exec_cmd(d: *mut ds1620_data) {
    match (*d).cmd {
        DS1620_WRITE_TH => {
            (*d).reg_th = (*d).data;
        }
        DS1620_WRITE_TL => {
            (*d).reg_tl = (*d).data;
        }
        DS1620_WRITE_CONFIG => {
            (*d).reg_config = (*d).data as u8;
        }
        _ => {}
    }

    // return in command input state
    (*d).state = DS1620_STATE_CMD_IN;
}

/// Write data bit
#[no_mangle]
pub unsafe extern "C" fn ds1620_write_data_bit(d: *mut ds1620_data, data_bit: u_int) {
    // CLK must be low
    if (*d).clk_bit != 0 {
        return;
    }

    match (*d).state {
        DS1620_STATE_CMD_IN => {
            if data_bit != 0 {
                (*d).cmd |= 1 << (*d).cmd_pos;
            }

            (*d).cmd_pos += 1;
            if (*d).cmd_pos == DS1620_CMD_SIZE {
                ds1620_cmd_set_state(d);
            }
        }

        DS1620_STATE_DATA_OUT => {
            // ignore input since it shouldn't happen
        }

        DS1620_STATE_DATA_IN => {
            if data_bit != 0 {
                (*d).data |= 1 << (*d).data_pos;
            }

            (*d).data_pos += 1;
            if (*d).data_pos == (*d).data_len {
                ds1620_exec_cmd(d);
            }
        }

        _ => {}
    }
}

/// Read data bit
#[no_mangle]
pub unsafe extern "C" fn ds1620_read_data_bit(d: *mut ds1620_data) -> u_int {
    if (*d).state != DS1620_STATE_DATA_OUT {
        return 1;
    }

    let val: c_uint = (((*d).data >> (*d).data_pos) & 0x1) as c_uint;

    (*d).data_pos += 1;
    if (*d).data_pos == (*d).data_len {
        // return in command input state
        (*d).state = DS1620_STATE_CMD_IN;
    }

    val
}

/// Initialize a DS1620
#[no_mangle]
pub unsafe extern "C" fn ds1620_init(d: *mut ds1620_data, temp: c_int) {
    libc::memset(d.cast::<_>(), 0, size_of::<ds1620_data>());

    // reset state
    ds1620_set_rst_bit(d, 0);

    // set initial temperature
    ds1620_set_temp(d, temp);

    // chip in CPU mode (3-wire communications)
    (*d).reg_config = DS1620_CONFIG_STATUS_CPU;
}
