//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! NMC93C46/NMC93C56 Serial EEPROM.

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dynamips_common::*;
use crate::utils::*;

/// EEPROM types // TODO enum
pub const EEPROM_TYPE_NMC93C46: u_int = 0;
pub const EEPROM_TYPE_NMC93C56: u_int = 1;

/// EEPROM data bit order // TODO enum
pub const EEPROM_DORD_NORMAL: c_int = 0;
pub const EEPROM_DORD_REVERSED: c_int = 1;

/// EEPROM debugging // TODO enum
pub const EEPROM_DEBUG_DISABLED: c_int = 0;
pub const EEPROM_DEBUG_ENABLED: c_int = 1;

/// EEPROM DOUT default status // TODO enum
pub const EEPROM_DOUT_HIGH: u_int = 0;
pub const EEPROM_DOUT_KEEP: u_int = 1;

/// 8 groups with 4 differents bits (clock,select,data_in,data_out)
pub const NMC93CX6_MAX_EEPROM_PER_GROUP: usize = 16;

/// NMC93C46 EEPROM command bit length
pub const NMC93C46_CMD_BITLEN: u_int = 9;

/// NMC93C56 EEPROM command bit length
pub const NMC93C56_CMD_BITLEN: u_int = 11;

/// NMC93C46 EEPROM data bit length
pub const NMC93CX6_CMD_DATALEN: u_int = 16;

/// NMC93C46 EEPROM commands:     SB (1) OP(2) Address(6/9)
#[allow(clippy::identity_op)]
pub const NMC93CX6_CMD_CONTROL: u_int = 0x1 | 0x0;
#[allow(clippy::identity_op)]
pub const NMC93CX6_CMD_WRDS: u_int = 0x1 | 0x0 | 0x00;
#[allow(clippy::identity_op)]
pub const NMC93CX6_CMD_ERASE_ALL: u_int = 0x1 | 0x0 | 0x08;
#[allow(clippy::identity_op)]
pub const NMC93CX6_CMD_WRITE_ALL: u_int = 0x1 | 0x0 | 0x10;
#[allow(clippy::identity_op)]
pub const NMC93CX6_CMD_WREN: u_int = 0x1 | 0x0 | 0x18;
pub const NMC93CX6_CMD_READ: u_int = 0x1 | 0x2;
pub const NMC93CX6_CMD_WRITE: u_int = 0x1 | 0x4;
pub const NMC93CX6_CMD_ERASE: u_int = 0x1 | 0x6;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nmc93cX6_eeprom_def {
    pub clock_bit: u_int,
    pub select_bit: u_int,
    pub din_bit: u_int,
    pub dout_bit: u_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nmc93cX6_eeprom_state {
    pub cmd_len: u_int,
    pub cmd_val: u_int,
    pub state: u_int,
    pub dataout_pos: u_int,
    pub dataout_val: u_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nmc93cX6_group {
    pub eeprom_type: u_int,
    pub nr_eeprom: u_int,
    pub eeprom_reg: u_int,
    pub reverse_data: u_int,
    pub dout_status: u_int,
    pub debug: c_int,
    pub description: *mut c_char,
    pub def: [*const nmc93cX6_eeprom_def; NMC93CX6_MAX_EEPROM_PER_GROUP],
    pub state: [nmc93cX6_eeprom_state; NMC93CX6_MAX_EEPROM_PER_GROUP],
    pub eeprom: [*mut cisco_eeprom; NMC93CX6_MAX_EEPROM_PER_GROUP],
}

const DEBUG_EEPROM: c_int = 0;

/// Internal states // TODO enum
pub const EEPROM_STATE_INACTIVE: u_int = 0;
pub const EEPROM_STATE_WAIT_CMD: u_int = 1;
pub const EEPROM_STATE_DATAOUT: u_int = 2;

/// Get command length for the specified group
unsafe fn nmc94cX6_get_cmd_len(g: *mut nmc93cX6_group) -> u_int {
    match (*g).eeprom_type {
        EEPROM_TYPE_NMC93C46 => NMC93C46_CMD_BITLEN,
        EEPROM_TYPE_NMC93C56 => NMC93C56_CMD_BITLEN,
        _ => 0,
    }
}

/// Extract EEPROM data address
unsafe fn nmc94cX6_get_addr(g: *mut nmc93cX6_group, cmd: u_int) -> u_int {
    match (*g).eeprom_type {
        EEPROM_TYPE_NMC93C46 => (cmd >> 3) & 0x3f,
        EEPROM_TYPE_NMC93C56 => m_reverse_u8(((cmd >> 3) & 0xff) as m_uint8_t) as u_int,
        _ => 0,
    }
}

/// Check chip select
unsafe fn nmc93cX6_check_cs(g: *mut nmc93cX6_group, old: u_int, new: u_int) {
    for i in 0..(*g).nr_eeprom {
        if (*g).dout_status == EEPROM_DOUT_HIGH {
            (*g).state[i as usize].dataout_val = 1;
        }

        if (*g).debug != 0 {
            libc::printf(cstr!("EEPROM %s(%d): check_cs:  check_bit(old,new,select_bit) [%8.8x, %8.8x, %d (mask = %8.8x)] = %d\n"), (*g).description, i, old, new, (*(*g).def[i as usize]).select_bit, 1 << (*(*g).def[i as usize]).select_bit, check_bit(old, new, (*(*g).def[i as usize]).select_bit));
        }

        let res: c_int = check_bit(old, new, (*(*g).def[i as usize]).select_bit);
        if res != 0 {
            (*g).state[i as usize].cmd_len = 0; // no bit for command sent now
            (*g).state[i as usize].cmd_val = 0;
            //(*g).state[i as usize].dataout_val = 1;

            if res == 2 {
                (*g).state[i as usize].state = EEPROM_STATE_WAIT_CMD;
            } else {
                (*g).state[i as usize].state = EEPROM_STATE_INACTIVE;
            }
        }
    }
}

/// Check clock set for a specific group
unsafe fn nmc93cX6_check_clk_group(g: *mut nmc93cX6_group, group_id: c_int, old: u_int, new: u_int) {
    let eeprom: *mut cisco_eeprom;
    let cmd: u_int;
    let op: u_int;
    let addr: u_int;
    let mut pos: u_int;
    let cmd_len: u_int;

    let clk_bit: u_int = (*(*g).def[group_id as usize]).clock_bit;
    let din_bit: u_int = (*(*g).def[group_id as usize]).din_bit;

    if (*g).debug != 0 {
        libc::printf(cstr!("EEPROM %s(%d): check_clk: check_bit(old,new,select_bit) [%8.8x, %8.8x, %d (mask = %8.8x)] = %d\n"), (*g).description, group_id, old, new, clk_bit, 1 << clk_bit, check_bit(old, new, clk_bit));
    }

    // CLK bit set ?
    if check_bit(old, new, clk_bit) != 2 {
        return;
    }

    match (*g).state[group_id as usize].state {
        EEPROM_STATE_WAIT_CMD => 'block: {
            // The first bit must be set to "1"
            if ((*g).state[group_id as usize].cmd_len == 0) && (new & (1 << din_bit)) == 0 {
                break 'block;
            }

            // Read DATAIN bit
            if (new & (1 << din_bit)) != 0 {
                (*g).state[group_id as usize].cmd_val |= 1 << (*g).state[group_id as usize].cmd_len;
            }

            (*g).state[group_id as usize].cmd_len += 1;

            cmd_len = nmc94cX6_get_cmd_len(g);

            // Command is complete ?
            if (*g).state[group_id as usize].cmd_len == cmd_len {
                if DEBUG_EEPROM != 0 {
                    libc::printf(cstr!("nmc93cX6: %s(%d): command = %x\n"), (*g).description, group_id, (*g).state[group_id as usize].cmd_val);
                }
                (*g).state[group_id as usize].cmd_len = 0;

                // we have the command! extract the opcode
                cmd = (*g).state[group_id as usize].cmd_val;
                op = cmd & 0x7;

                match op {
                    NMC93CX6_CMD_READ => {
                        (*g).state[group_id as usize].state = EEPROM_STATE_DATAOUT;
                        (*g).state[group_id as usize].dataout_pos = 0;
                    }
                    _ => {
                        if DEBUG_EEPROM != 0 {
                            libc::printf(cstr!("nmc93cX6: unhandled opcode %d\n"), op);
                        }
                    }
                }
            }
        }

        EEPROM_STATE_DATAOUT => {
            // user want to read data. we read 16-bits.
            // extract address (6/9 bits) from command.

            cmd = (*g).state[group_id as usize].cmd_val;
            addr = nmc94cX6_get_addr(g, cmd);

            if DEBUG_EEPROM != 0 {
                #[allow(clippy::collapsible_if)]
                if (*g).state[group_id as usize].dataout_pos == 0 {
                    libc::printf(cstr!("nmc93cX6: %s(%d): read addr=%x (%d), val=%4.4x [eeprom=%p]\n"), (*g).description, group_id, addr, addr, (*g).state[group_id as usize].cmd_val, (*g).eeprom[group_id as usize]);
                }
            }

            pos = (*g).state[group_id as usize].dataout_pos;
            (*g).state[group_id as usize].dataout_pos += 1;

            if (*g).reverse_data != 0 {
                pos = 15 - pos;
            }

            eeprom = (*g).eeprom[group_id as usize];

            if !eeprom.is_null() && !(*eeprom).data.is_null() && ((addr as size_t) < (*eeprom).len) {
                (*g).state[group_id as usize].dataout_val = (*(*eeprom).data.add(addr as usize) & (1 << pos)) as u_int;
            } else {
                // access out of bounds
                (*g).state[group_id as usize].dataout_val = 1 << pos;
            }

            if (*g).state[group_id as usize].dataout_pos == NMC93CX6_CMD_DATALEN {
                (*g).state[group_id as usize].state = EEPROM_STATE_INACTIVE;
                (*g).state[group_id as usize].dataout_pos = 0;
            }
        }

        _ => {
            if DEBUG_EEPROM != 0 {
                libc::printf(cstr!("nmc93cX6: unhandled state %d\n"), (*g).state[group_id as usize].state);
            }
        }
    }
}

/// Check clock set for all group
#[no_mangle]
pub unsafe extern "C" fn nmc93cX6_check_clk(g: *mut nmc93cX6_group, old: u_int, new: u_int) {
    for i in 0..(*g).nr_eeprom as c_int {
        nmc93cX6_check_clk_group(g, i, old, new);
    }
}

/// Handle write
#[no_mangle]
pub unsafe extern "C" fn nmc93cX6_write(g: *mut nmc93cX6_group, data: u_int) {
    let new: u_int = data;
    let old: u_int = (*g).eeprom_reg;

    nmc93cX6_check_cs(g, old, new);
    nmc93cX6_check_clk(g, old, new);
    (*g).eeprom_reg = new;
}

/// Returns the TRUE if the EEPROM is active
#[no_mangle]
pub unsafe extern "C" fn nmc93cX6_is_active(g: *mut nmc93cX6_group, group_id: u_int) -> u_int {
    (*g).eeprom_reg & (1 << (*(*g).def[group_id as usize]).select_bit)
}

/// Returns the DOUT bit value
#[no_mangle]
pub unsafe extern "C" fn nmc93cX6_get_dout(g: *mut nmc93cX6_group, group_id: u_int) -> u_int {
    if (*g).state[group_id as usize].dataout_val != 0 {
        1 << (*(*g).def[group_id as usize]).dout_bit
    } else {
        0
    }
}

/// Handle read
#[no_mangle]
pub unsafe extern "C" fn nmc93cX6_read(g: *mut nmc93cX6_group) -> u_int {
    let mut res: u_int = (*g).eeprom_reg;

    for i in 0..(*g).nr_eeprom as c_int {
        if ((*g).eeprom_reg & (1 << (*(*g).def[i as usize]).select_bit)) == 0 {
            continue;
        }

        if (*g).state[i as usize].dataout_val != 0 {
            res |= 1 << (*(*g).def[i as usize]).dout_bit;
        } else {
            res &= !(1 << (*(*g).def[i as usize]).dout_bit);
        }
    }

    res
}
