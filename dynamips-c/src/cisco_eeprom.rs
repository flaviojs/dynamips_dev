//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot.  All rights reserved.
//!
//! Cisco EEPROM manipulation functions.

use crate::dynamips_common::*;
use libc::size_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_uint;
use std::ptr::addr_of;
use std::ptr::addr_of_mut;
use std::ptr::null;
use std::ptr::null_mut;

/*
CISCO EEPROM format version 1 (size=0x20?)
0x00: 01(version)
0x01: XX(product id)
// TODO format might depend on the type or class of hardware
0x02: XX (.) XX(Hardware revision)
0x04: XX XX XX XX(Serial number)
0x08: XX (-) XX XX (-) XX(Part number)
0x0C: XX(Test history)
0x0D: XX (-) XX (-) XX(RMA number)
0x10: XX(Board Revision)
0x11: ...FF(padding?)
0x17: XX(Connector type)(0=PCI,1=Wan Module,other=PCI)
0x18: ...FF(padding?)
// 0x20+ is optional? ignored if FF FF...
0x26: XX XX XX XX(Version Identifier)(4 chars)
0x2A: XX XX XX XX XX XX XX XX XX XX XX XX XX XX XX XX XX XX(FRU Part Number)(18 chars)
0x3C: ...FF(padding?)
*/

/*
CISCO EEPROM format version 4 (size=0x80?)
0x00: 04(version) FF(padding?)
0x02: {
   // {00000000b}.* adds 0x100 to id?
   // {LLDDDDDDb} has length=2^LLb(1,2,4) and id=DDDDDDb
   // {11DDDDDDb TTLLLLLLb} has id=DDDDDDb, length=LLLLLLb and type=TTb(00b=hex,01b=number,10b=string,11b=hex or reserved?)
   : 01 XX(Number of Slots)
   : 02 XX(Fab Version)
   : 03 XX(RMA Test History)
   : 04 XX(RMA History)
   : 05 XX(Connector Type)
   : 06 XX(EHSA Preferred Master)
   : 07 XX(Vendor ID)
   : 09 XX(Processor type)
   : 0B XX(Power Supply Type: 0=AC, !0=DC)
   : 0C XX(ignored?)
   : 40 XX XX(product id)
   : 41 XX (.) XX (Hardware Revision)
   : 42 XX XX(Board Revision)
   : 43 XXXX(MAC Address block size)
   : 44 XX XX(Capabilities)
   : 45 XX XX(Self test result)
   : 4A XX XX(Radio Country Code)
   : 80 XXXX XXXX(Deviation Number)
   : 81 XX (-) XX (-) XX (-) XX(RMA Number)
   : 82 XX (-) XXXX (-) XX(Part Number)
   : 83 XXXXXXXX(Hardware date code)
   : 84 XX XX XX XX(Manufacturing Engineer)
   : 85 XX (-) XXXX (-) XX(Fab Part Number)
   : C0 46 XX XX (-) XX XX XX (-) XX(Part Number)(number)
   : C1 8B XX XX XX XX XX XX XX XX XX XX XX(PCB Serial Number)(string)
   : C2 8B XX XX XX XX XX XX XX XX XX XX XX(Chassis Serial Number)(string)
   : C3 06 XX XX XX XX XX XX(Chassis MAC Address)
   : C4 08 XX XX XX XX XX XX XX XX(Manufacturing Test Data)
   : C5 08 XX XX XX XX XX XX XX XX(Field Diagnostics Data)
   : C6 8A XX XX XX XX XX XX XX XX XX XX(CLEI Code)(string)
   : C7 ?? XX?(ignored?)
   : C8 09 XX[min dBmV] XX[max dBmV] XX[num_values=3] XXXX[value_0] XXXX[value_1] XXXX[value_2](Calibration Data)
   : C9 ?? XX?(Platform features)(hex)
   : CB 88 XX XX XX XX XX XX XX XX(Product (FRU) Number)(string)
   : CF 06 XXXX (.) XXXX (.) XXXX(Base MAC Address)
}.*
0x??: ...FF(padding?)
*/

// Cisco EEPROM
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cisco_eeprom {
    pub name: *mut c_char,
    pub data: *mut m_uint16_t,
    pub len: size_t,
}
// allow cisco_eeprom in the static arrays
unsafe impl Sync for cisco_eeprom {}

// ======================================================================
// NM-1E: 1 Ethernet Port Network Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_1e_data: [m_uint16_t; 16] = [
    0x0143, 0x0100, 0x0075, 0xCD81, 0x500D, 0xA201, 0x0000, 0x0000,
    0x5800, 0x0000, 0x9803, 0x2000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-4E: 4 Ethernet Port Network Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_4e_data: [m_uint16_t; 16] = [
    0x0142, 0x0100, 0x0075, 0xCD81, 0x500D, 0xA201, 0x0000, 0x0000,
    0x5800, 0x0000, 0x9803, 0x2000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-1FE-TX: 1 FastEthernet Port Network Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_1fe_tx_data: [m_uint16_t; 16] = [
    0x0144, 0x0100, 0x0075, 0xCD81, 0x500D, 0xA201, 0x0000, 0x0000,
    0x5800, 0x0000, 0x9803, 0x2000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-16ESW: 16 FastEthernet Port Switch Network Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_16esw_data: [m_uint16_t; 64] = [
    0x04FF, 0x4002, 0xA941, 0x0100, 0xC046, 0x0320, 0x003B, 0x3401,
    0x4245, 0x3080, 0x0000, 0x0000, 0x0203, 0xC18B, 0x3030, 0x3030,
    0x3030, 0x3030, 0x3030, 0x3003, 0x0081, 0x0000, 0x0000, 0x0400,
    0xCF06, 0x0013, 0x1A1D, 0x0BD1, 0x4300, 0x11FF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NMD-36ESW: 36 FastEthernet Port Switch Network Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nmd_36esw_data: [m_uint16_t; 64] = [
    0x04FF, 0x4002, 0xB141, 0x0100, 0xC046, 0x0320, 0x003B, 0x3401,
    0x4245, 0x3080, 0x0000, 0x0000, 0x0203, 0xC18B, 0x3030, 0x3030,
    0x3030, 0x3030, 0x3030, 0x3003, 0x0081, 0x0000, 0x0000, 0x0400,
    0xCF06, 0x0013, 0x1A1D, 0x0BD1, 0x4300, 0x26FF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-4T: 4 Serial Network Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_4t_data:[m_uint16_t; 16] = [
    0x0154, 0x0101, 0x009D, 0x2D64, 0x5009, 0x0A02, 0x0000, 0x0000,
    0x5800, 0x0000, 0x9811, 0x0300, 0x0005, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-2E2W: 2 Ethernet ports with 2 WIC slots Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_2e2w_data: [m_uint16_t; 16] = [
    0x011E, 0x0102, 0x009A, 0xEBB1, 0x5004, 0x9305, 0x0000, 0x0000,
    0x5000, 0x0000, 0x9808, 0x1217, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-2W: 2 WIC slots Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_2w_data: [m_uint16_t; 64] = [
    0x04FF, 0x4000, 0xD641, 0x0100, 0xC046, 0x0320, 0x0012, 0xBF01,
    0x4247, 0x3080, 0x0000, 0x0000, 0x0205, 0xC18B, 0x4A41, 0x4430,
    0x3730, 0x3330, 0x375A, 0x3203, 0x0081, 0x0000, 0x0000, 0x0400,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-1A-OC3MM: 1 ATM OC3 port Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_1a_oc3mm_data: [m_uint16_t; 16] = [
    0x019A, 0x0100, 0x015B, 0x41D9, 0x500E, 0x7402, 0x0000, 0x0000,
    0x7800, 0x0000, 0x0011, 0x2117, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-NAM: Network Analysis Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_nam_data: [m_uint16_t; 64] = [
    0x04FF, 0x4004, 0x6A41, 0x0100, 0xC046, 0x0320, 0x004F, 0x9E01,
    0x4241, 0x3080, 0x0000, 0x0000, 0x0202, 0xC18B, 0x4A41, 0x4230,
    0x3630, 0x3630, 0x3543, 0x3403, 0x0081, 0x0000, 0x0000, 0x0400,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM-CIDS: Network Analysis Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_nm_cids_data: [m_uint16_t; 64] = [
    0x04FF, 0x4004, 0x2541, 0x0100, 0xC046, 0x0320, 0x004F, 0x9E01,
    0x4241, 0x3080, 0x0000, 0x0000, 0x0202, 0xC18B, 0x4A41, 0x4230,
    0x3630, 0x3630, 0x3543, 0x3403, 0x0081, 0x0000, 0x0000, 0x0400,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// NM EEPROMs
// ======================================================================

static eeprom_nm_array: [cisco_eeprom; 12] = [
    cisco_eeprom { name: c"NM-1E".as_ptr().cast_mut(), data: eeprom_nm_1e_data.as_ptr().cast_mut(), len: eeprom_nm_1e_data.len() },
    cisco_eeprom { name: c"NM-4E".as_ptr().cast_mut(), data: eeprom_nm_4e_data.as_ptr().cast_mut(), len: eeprom_nm_4e_data.len() },
    cisco_eeprom { name: c"NM-1FE-TX".as_ptr().cast_mut(), data: eeprom_nm_1fe_tx_data.as_ptr().cast_mut(), len: eeprom_nm_1fe_tx_data.len() },
    cisco_eeprom { name: c"NM-16ESW".as_ptr().cast_mut(), data: eeprom_nm_16esw_data.as_ptr().cast_mut(), len: eeprom_nm_16esw_data.len() },
    cisco_eeprom { name: c"NMD-36ESW".as_ptr().cast_mut(), data: eeprom_nmd_36esw_data.as_ptr().cast_mut(), len: eeprom_nmd_36esw_data.len() },
    cisco_eeprom { name: c"NM-4T".as_ptr().cast_mut(), data: eeprom_nm_4t_data.as_ptr().cast_mut(), len: eeprom_nm_4t_data.len() },
    cisco_eeprom { name: c"NM-2E2W".as_ptr().cast_mut(), data: eeprom_nm_2e2w_data.as_ptr().cast_mut(), len: eeprom_nm_2e2w_data.len() },
    cisco_eeprom { name: c"NM-2W".as_ptr().cast_mut(), data: eeprom_nm_2w_data.as_ptr().cast_mut(), len: eeprom_nm_2w_data.len() },
    cisco_eeprom { name: c"NM-1A-OC3MM".as_ptr().cast_mut(), data: eeprom_nm_1a_oc3mm_data.as_ptr().cast_mut(), len: eeprom_nm_1a_oc3mm_data.len() },
    cisco_eeprom { name: c"NM-NAM".as_ptr().cast_mut(), data: eeprom_nm_nam_data.as_ptr().cast_mut(), len: eeprom_nm_nam_data.len() },
    cisco_eeprom { name: c"NM-CIDS".as_ptr().cast_mut(), data: eeprom_nm_cids_data.as_ptr().cast_mut(), len: eeprom_nm_cids_data.len() },
    cisco_eeprom { name: null_mut(), data: null_mut(), len: 0 },
];

// Find a NM EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_find_nm(name: *mut c_char) -> *const cisco_eeprom {
    cisco_eeprom_find(eeprom_nm_array.as_ptr(), name)
}

// ======================================================================
// PA-FE-TX: 1 FastEthernet Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_fe_tx_data: [m_uint16_t; 16] = [
    0x0111, 0x0102, 0xffff, 0xffff, 0x4906, 0x9804, 0x0000, 0x0000,
    0x6000, 0x0000, 0x9812, 0x1700, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-2FE-TX: 2 FastEthernet Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_2fe_tx_data: [m_uint16_t; 64] = [
    0x04FF, 0x4002, 0x2441, 0x0100, 0xC18B, 0x5858, 0x5830, 0x3030,
    0x3030, 0x3030, 0x3082, 0x4915, 0x2C04, 0x4241, 0x3003, 0x0081,
    0x0000, 0x0000, 0x0400, 0x8000, 0x0000, 0x00CB, 0x9450, 0x412D,
    0x3246, 0x452D, 0x4658, 0x2020, 0x2020, 0x2020, 0x2020, 0x2020,
    0x20C0, 0x4603, 0x2000, 0x20A0, 0x04FF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-GE: 1 GigabitEthernet Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_ge_data: [m_uint16_t; 16] = [
    0x0198, 0x0100, 0x0000, 0x0000, 0x000C, 0x4803, 0x0000, 0x0000,
    0x5000, 0x0000, 0x9906, 0x0300, 0x0001, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-4E: 4 Ethernet Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_4e_data: [m_uint16_t; 16] = [
    0x0102, 0x010E, 0xFFFF, 0xFFFF, 0x4906, 0x1404, 0x0000, 0x0000,
    0x5000, 0x0000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-8E: 8 Ethernet Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_8e_data: [m_uint16_t; 16] = [
    0x0101, 0x010E, 0xFFFF, 0xFFFF, 0x4906, 0x1404, 0x0000, 0x0000,
    0x5000, 0x0000, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-4T+: 4 Serial Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_4t_data: [m_uint16_t; 16] = [
    0x010C, 0x010F, 0xffff, 0xffff, 0x4906, 0x2E07, 0x0000, 0x0000,
    0x5000, 0x0000, 0x0010, 0x2400, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-8T: 8 Serial Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_8t_data: [m_uint16_t; 16] = [
    0x010E, 0x010F, 0xffff, 0xffff, 0x4906, 0x2E07, 0x0000, 0x0000,
    0x5000, 0x0000, 0x0010, 0x2400, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-A1: 1 ATM Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_a1_data: [m_uint16_t; 16] = [
    0x0117, 0x010F, 0xffff, 0xffff, 0x4906, 0x2E07, 0x0000, 0x0000,
    0x5000, 0x0000, 0x0010, 0x2400, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-A3: 1 ATM Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_a3_data: [m_uint16_t; 16] = [
    0x0159, 0x0200, 0xFFFF, 0xFFFF, 0x4909, 0x7E04, 0x0000, 0x0000,
    0x5000, 0x0000, 0x0007, 0x1100, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-POS-OC3: 1 POS Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_pos_oc3_data: [m_uint16_t; 16] = [
    0x0196, 0x0202, 0xffff, 0xffff, 0x490C, 0x7806, 0x0000, 0x0000,
    0x5000, 0x0000, 0x0208, 0x1900, 0x0000, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-4B: 4 BRI Port Adapter EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_4b_data: [m_uint16_t; 16] = [
    0x013D, 0x0202, 0xffff, 0xffff, 0x490C, 0x7806, 0x0000, 0x0000,
    0x5000, 0x0000, 0x0208, 0x1900, 0x0000, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA-MC-8TE1
// ======================================================================
#[rustfmt::skip]
static eeprom_pa_mc8te1_data: [m_uint16_t; 64] = [
    0x04FF, 0x4003, 0x4E41, 0x0200, 0xC18B, 0x4A41, 0x4530, 0x3834,
    0x3159, 0x3251, 0x3082, 0x491D, 0x7D02, 0x4241, 0x3003, 0x0081,
    0x0000, 0x0000, 0x0400, 0x8000, 0x0127, 0x9BCB, 0x9450, 0x412D,
    0x4D43, 0x2D38, 0x5445, 0x312B, 0x2020, 0x2020, 0x2020, 0x2020,
    0x20C0, 0x4603, 0x2000, 0x4BBB, 0x02FF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// C7200-JC-PA
// ======================================================================
#[rustfmt::skip]
static eeprom_c7200_jc_pa_data: [m_uint16_t; 64] = [
    0x04FF, 0x4005, 0x1141, 0x0101, 0x8744, 0x0A3B, 0x0382, 0x4928,
    0xB003, 0x4241, 0x30C1, 0x8B58, 0x5858, 0x5858, 0x5858, 0x5858,
    0x5858, 0x0400, 0x0203, 0x851C, 0x1DDA, 0x03CB, 0x8B43, 0x3732,
    0x3030, 0x2D4A, 0x432D, 0x5041, 0x8800, 0x0145, 0xC589, 0x5630,
    0x3120, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// PA EEPROMs
// ======================================================================

static eeprom_pa_array: [cisco_eeprom; 14] = [
    cisco_eeprom { name: c"PA-FE-TX".as_ptr().cast_mut(), data: eeprom_pa_fe_tx_data.as_ptr().cast_mut(), len: eeprom_pa_fe_tx_data.len() },
    cisco_eeprom { name: c"PA-2FE-TX".as_ptr().cast_mut(), data: eeprom_pa_2fe_tx_data.as_ptr().cast_mut(), len: eeprom_pa_2fe_tx_data.len() },
    cisco_eeprom { name: c"PA-GE".as_ptr().cast_mut(), data: eeprom_pa_ge_data.as_ptr().cast_mut(), len: eeprom_pa_ge_data.len() },
    cisco_eeprom { name: c"PA-4E".as_ptr().cast_mut(), data: eeprom_pa_4e_data.as_ptr().cast_mut(), len: eeprom_pa_4e_data.len() },
    cisco_eeprom { name: c"PA-8E".as_ptr().cast_mut(), data: eeprom_pa_8e_data.as_ptr().cast_mut(), len: eeprom_pa_8e_data.len() },
    cisco_eeprom { name: c"PA-4T+".as_ptr().cast_mut(), data: eeprom_pa_4t_data.as_ptr().cast_mut(), len: eeprom_pa_4t_data.len() },
    cisco_eeprom { name: c"PA-8T".as_ptr().cast_mut(), data: eeprom_pa_8t_data.as_ptr().cast_mut(), len: eeprom_pa_8t_data.len() },
    cisco_eeprom { name: c"PA-A1".as_ptr().cast_mut(), data: eeprom_pa_a1_data.as_ptr().cast_mut(), len: eeprom_pa_a1_data.len() },
    cisco_eeprom { name: c"PA-A3".as_ptr().cast_mut(), data: eeprom_pa_a3_data.as_ptr().cast_mut(), len: eeprom_pa_a3_data.len() },
    cisco_eeprom { name: c"PA-POS-OC3".as_ptr().cast_mut(), data: eeprom_pa_pos_oc3_data.as_ptr().cast_mut(), len: eeprom_pa_pos_oc3_data.len() },
    cisco_eeprom { name: c"PA-4B".as_ptr().cast_mut(), data: eeprom_pa_4b_data.as_ptr().cast_mut(), len: eeprom_pa_4b_data.len() },
    cisco_eeprom { name: c"PA-MC-8TE1".as_ptr().cast_mut(), data: eeprom_pa_mc8te1_data.as_ptr().cast_mut(), len: eeprom_pa_mc8te1_data.len() },
    cisco_eeprom { name: c"C7200-JC-PA".as_ptr().cast_mut(), data: eeprom_c7200_jc_pa_data.as_ptr().cast_mut(), len: eeprom_c7200_jc_pa_data.len() },
    cisco_eeprom { name: null_mut(), data: null_mut(), len: 0 },
];

// Find a PA EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_find_pa(name: *mut c_char) -> *const cisco_eeprom {
    cisco_eeprom_find(eeprom_pa_array.as_ptr(), name)
}

// ======================================================================
// WIC-1T: 1 Serial port Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_wic_1t_data: [m_uint16_t; 16] = [
    0x0102, 0x0100, 0x0000, 0x0000, 0x5005, 0xEA01, 0x0000, 0x0000,
    0xB000, 0x0000, 0x0303, 0x0401, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// WIC-2T: 2 Serial ports Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_wic_2t_data: [m_uint16_t; 16] = [
    0x0112, 0x0100, 0x0000, 0x0000, 0x5005, 0xEA01, 0x0000, 0x0000,
    0xB000, 0x0000, 0x0303, 0x0401, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// WIC-1B-S/T: 1 BRI port Module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_wic_1b_st_data: [m_uint16_t; 16] = [
    0x0107, 0x0100, 0x0000, 0x0000, 0x5005, 0xEA01, 0x0000, 0x0000,
    0xB000, 0x0000, 0x0303, 0x0401, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// WIC-4ESW: 4 Ethernet port switch module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_wic_4esw_data: [m_uint16_t; 64] = [
    0x04FF, 0x4000, 0x6441, 0x0100, 0x8249, 0x22FE, 0x0142, 0x4430,
    0x8000, 0x0000, 0x0002, 0x01C1, 0x8B46, 0x4F43, 0x3039, 0x3435,
    0x344C, 0x5345, 0x0300, 0x8100, 0x0000, 0x0004, 0x00C0, 0x4603,
    0x2000, 0x60F1, 0x0105, 0x01CF, 0x0600, 0x1646, 0x37F4, 0x6843,
    0x0014, 0xCB88, 0x5749, 0x432D, 0x3445, 0x5357, 0xC68A, 0x4950,
    0x4D45, 0x4430, 0x3042, 0x5241, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// WIC-1ENET: 1 Ethernet port module EEPROM
// ======================================================================
#[rustfmt::skip]
static eeprom_wic_1enet_data: [m_uint16_t; 56] = [
    0x04FF, 0x4000, 0x3941, 0x0101, 0xC18B, 0x464F, 0x4330, 0x3830,
    0x3832, 0x4330, 0x3682, 0x4923, 0x0901, 0x4242, 0x3002, 0x04CB,
    0x8957, 0x4943, 0x2D31, 0x454E, 0x4554, 0x0700, 0x0300, 0x8100,
    0x0000, 0x0005, 0x0104, 0x00CF, 0x0644, 0x5566, 0x7788, 0xAA43,
    0x0001, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
];

// ======================================================================
// WIC EEPROMs
// ======================================================================

static eeprom_wic_array: [cisco_eeprom; 6] = [
    cisco_eeprom { name: c"WIC-1T".as_ptr().cast_mut(), data: eeprom_wic_1t_data.as_ptr().cast_mut(), len: eeprom_wic_1t_data.len() },
    cisco_eeprom { name: c"WIC-2T".as_ptr().cast_mut(), data: eeprom_wic_2t_data.as_ptr().cast_mut(), len: eeprom_wic_2t_data.len() },
    cisco_eeprom { name: c"WIC-1B".as_ptr().cast_mut(), data: eeprom_wic_1b_st_data.as_ptr().cast_mut(), len: eeprom_wic_1b_st_data.len() },
    cisco_eeprom { name: c"WIC-4ESW".as_ptr().cast_mut(), data: eeprom_wic_4esw_data.as_ptr().cast_mut(), len: eeprom_wic_4esw_data.len() },
    cisco_eeprom { name: c"WIC-1ENET".as_ptr().cast_mut(), data: eeprom_wic_1enet_data.as_ptr().cast_mut(), len: eeprom_wic_1enet_data.len() },
    cisco_eeprom { name: null_mut(), data: null_mut(), len: 0 },
];

// Find a WIC EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_find_wic(name: *mut c_char) -> *const cisco_eeprom {
    cisco_eeprom_find(eeprom_wic_array.as_ptr(), name)
}

// ======================================================================
// C6k EEPROMs
// ======================================================================

// Chassis: 6509
#[rustfmt::skip]
static eeprom_c6k_chassis_6509_data: [m_uint16_t; 128] = [
    0xABAB, 0x0190, 0x0F0D, 0x0100, 0x0002, 0x6001, 0x9002, 0x4369,
    0x7363, 0x6F20, 0x5379, 0x7374, 0x656D, 0x7300, 0x0000, 0x0000,
    0x0000, 0x5753, 0x2D43, 0x3635, 0x3039, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x5343, 0x4130, 0x3333, 0x3730, 0x314A,
    0x5500, 0x0000, 0x0000, 0x0000, 0x0000, 0x3733, 0x2D33, 0x3433,
    0x382D, 0x3033, 0x0000, 0x0000, 0x0000, 0x4230, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0002, 0x0000, 0x0000, 0x0000, 0x0009, 0x0005, 0x0001,
    0x0002, 0x0001, 0x0016, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x6001, 0x0124, 0x01AD, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0016, 0x00D0, 0x000F, 0x2000, 0x0400,
    0x0009, 0x0005, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

// Supervisor: SUP1A-2GE
#[rustfmt::skip]
static eeprom_c6k_sup1a_2ge_data: [m_uint16_t; 128] = [
    0xABAB, 0x0190, 0x138F, 0x0100, 0x0002, 0x6003, 0x00DB, 0x4369,
    0x7363, 0x6F20, 0x5379, 0x7374, 0x656D, 0x7300, 0x0000, 0x0000,
    0x0000, 0x5753, 0x2D58, 0x364B, 0x2D53, 0x5550, 0x3141, 0x2D32,
    0x4745, 0x0000, 0x0000, 0x5341, 0x4430, 0x3333, 0x3431, 0x3639,
    0x3800, 0x0000, 0x0000, 0x0000, 0x0000, 0x3733, 0x2D34, 0x3336,
    0x382D, 0x3031, 0x0000, 0x0000, 0x0000, 0x4130, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0001, 0x0000, 0x0000, 0x0000, 0x0009, 0x0005, 0x0001,
    0x0003, 0x0001, 0x0001, 0x0002, 0x00DB, 0xFF56, 0x0000, 0x0000,
    0x6003, 0x0162, 0x0B56, 0x0000, 0x0000, 0x0000, 0x0005, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0014, 0x00D0, 0xBCEE, 0xB920, 0x0002,
    0x0100, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x1F02, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0200, 0x4132, 0x8181, 0x8181, 0x4B3C, 0x8080, 0x8080, 0x8080,
    0x8080, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

// EARL: PFC1 (aka EARL5)
#[rustfmt::skip]
static eeprom_c6k_earl_pfc1_data: [m_uint16_t; 128] = [
    0xABAB, 0x0190, 0x117D, 0x0100, 0x0002, 0x6004, 0x0066, 0x4369,
    0x7363, 0x6F20, 0x5379, 0x7374, 0x656D, 0x7300, 0x0000, 0x0000,
    0x0000, 0x5753, 0x2D46, 0x364B, 0x2D50, 0x4643, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x5341, 0x4430, 0x3334, 0x3333, 0x3637,
    0x3800, 0x0000, 0x0000, 0x0000, 0x0000, 0x3733, 0x2D34, 0x3037,
    0x352D, 0x3033, 0x0000, 0x0000, 0x0000, 0x4130, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0001, 0x0000, 0x0000, 0x0000, 0x0009, 0x0005, 0x0001,
    0x0003, 0x0001, 0x0001, 0x0010, 0x0066, 0xFFB0, 0x0000, 0x0000,
    0x6004, 0x0148, 0x07B7, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x000E, 0x0001, 0x0001, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x4B3C, 0x4132, 0x8080, 0x8080,
    0x8080, 0x8080, 0x8080, 0x8080, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

// Power Supply: 1000W
#[rustfmt::skip]
static eeprom_c6k_power_1000w_data: [m_uint16_t; 128] = [
    0xABAB, 0x0190, 0x121C, 0x0100, 0x0002, 0xAB01, 0x0003, 0x4369,
    0x7363, 0x6F20, 0x5379, 0x7374, 0x656D, 0x732C, 0x2049, 0x6E63,
    0x2E00, 0x5753, 0x2D43, 0x4143, 0x2D31, 0x3030, 0x3057, 0x0000,
    0x0000, 0x0000, 0x0000, 0x534F, 0x4E30, 0x3430, 0x3930, 0x3036,
    0x3600, 0x0000, 0x0000, 0x0000, 0x0000, 0x3334, 0x2D30, 0x3932,
    0x332D, 0x3031, 0x0000, 0x0000, 0x0000, 0x4230, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0001, 0x0000, 0x0000, 0x0000, 0x0009, 0x000C, 0x0003,
    0x0001, 0x0006, 0x0003, 0x0000, 0x0000, 0x07EE, 0x0000, 0x0000,
    0xAB01, 0x0114, 0x02C0, 0x0000, 0x0000, 0x0000, 0x0000, 0x07EE,
    0x07EE, 0x0015, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

// VTT: Voltage Termination module
#[rustfmt::skip]
static eeprom_c6k_vtt_data: [m_uint16_t; 128] = [
    0xABAB, 0x0190, 0x0FC4, 0x0100, 0x0002, 0xAB02, 0x0001, 0x4369,
    0x7363, 0x6F20, 0x5379, 0x7374, 0x656D, 0x7300, 0x0000, 0x0000,
    0x0000, 0x5753, 0x2D43, 0x3630, 0x3030, 0x2D56, 0x5454, 0x0000,
    0x0000, 0x0000, 0x0000, 0x534D, 0x5430, 0x3333, 0x3531, 0x3330,
    0x3400, 0x0000, 0x0000, 0x0000, 0x0000, 0x3733, 0x2D33, 0x3230,
    0x382D, 0x3034, 0x0000, 0x0000, 0x0000, 0x4130, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0001, 0x0001, 0x0000, 0x0000, 0x0009, 0x0005, 0x0001,
    0x0002, 0x0012, 0x0001, 0x0002, 0x0003, 0x0000, 0x0000, 0x0000,
    0xAB02, 0x0118, 0x00C9, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0003, 0x6455, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

// Linecard: WS-X6248
#[rustfmt::skip]
static eeprom_c6k_lc_wsx6248_data: [m_uint16_t; 128] = [
    0xABAB, 0x0190, 0x1339, 0x0100, 0x0002, 0x6003, 0x00CB, 0x4369,
    0x7363, 0x6F20, 0x5379, 0x7374, 0x656D, 0x7300, 0x0000, 0x0000,
    0x0000, 0x5753, 0x2D58, 0x3632, 0x3438, 0x2D52, 0x4A2D, 0x3435,
    0x0000, 0x0000, 0x0000, 0x5341, 0x4430, 0x3333, 0x3436, 0x3834,
    0x3200, 0x0000, 0x0000, 0x0000, 0x0000, 0x3733, 0x2D33, 0x3234,
    0x342D, 0x3038, 0x0000, 0x0000, 0x0000, 0x4330, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0001, 0x0001, 0x0000, 0x0000, 0x0009, 0x0005, 0x0001,
    0x0003, 0x0001, 0x0001, 0x0002, 0x00CB, 0xFEF3, 0x0000, 0x0000,
    0x6003, 0x0162, 0x0B02, 0x0000, 0x0000, 0x0000, 0x0005, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0003, 0x0030, 0xB6CC, 0x3CC0, 0x0030,
    0x0106, 0x0003, 0x0001, 0x0002, 0x0002, 0x0001, 0x0004, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x1230, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0200, 0x4B3C, 0x4132, 0x8181, 0x8181, 0x8080, 0x8080, 0x8080,
    0x8080, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

static eeprom_c6k_array: [cisco_eeprom; 7] = [
    cisco_eeprom { name: c"C6K-CHASSIS-6509".as_ptr().cast_mut(), data: eeprom_c6k_chassis_6509_data.as_ptr().cast_mut(), len: eeprom_c6k_chassis_6509_data.len() },
    cisco_eeprom { name: c"C6K-SUP-SUP1A-2GE".as_ptr().cast_mut(), data: eeprom_c6k_sup1a_2ge_data.as_ptr().cast_mut(), len: eeprom_c6k_sup1a_2ge_data.len() },
    cisco_eeprom { name: c"C6K-EARL-PFC1".as_ptr().cast_mut(), data: eeprom_c6k_earl_pfc1_data.as_ptr().cast_mut(), len: eeprom_c6k_earl_pfc1_data.len() },
    cisco_eeprom { name: c"C6K-POWER-1000W".as_ptr().cast_mut(), data: eeprom_c6k_power_1000w_data.as_ptr().cast_mut(), len: eeprom_c6k_power_1000w_data.len() },
    cisco_eeprom { name: c"C6K-VTT".as_ptr().cast_mut(), data: eeprom_c6k_vtt_data.as_ptr().cast_mut(), len: eeprom_c6k_vtt_data.len() },
    cisco_eeprom { name: c"C6K-LC-WS-X6248".as_ptr().cast_mut(), data: eeprom_c6k_lc_wsx6248_data.as_ptr().cast_mut(), len: eeprom_c6k_lc_wsx6248_data.len() },
    cisco_eeprom { name: null_mut(), data: null_mut(), len: 0 },
];

// Find a C6k EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_find_c6k(name: *mut c_char) -> *const cisco_eeprom {
    cisco_eeprom_find(eeprom_c6k_array.as_ptr(), name)
}

// ======================================================================
// Utility functions
// ======================================================================

// Find an EEPROM in the specified EEPROM array
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_find(eeproms: *const cisco_eeprom, name: *mut c_char) -> *const cisco_eeprom {
    for i in 0.. {
        if (*eeproms.add(i)).name.is_null() {
            break;
        }
        if 0 == libc::strcmp((*eeproms.add(i)).name, name) {
            return addr_of!(*eeproms.add(i));
        }
    }

    null()
}

// Copy an EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_copy(dst: *mut cisco_eeprom, src: *const cisco_eeprom) -> c_int {
    if dst.is_null() || src.is_null() {
        return -1;
    }

    cisco_eeprom_free(dst);

    let data: *mut m_uint16_t = libc::malloc((*src).len << 1).cast::<_>();
    if data.is_null() {
        return -1;
    }

    libc::memcpy(data.cast::<_>(), (*src).data.cast::<_>(), (*src).len << 1);
    (*dst).name = (*src).name;
    (*dst).data = data;
    (*dst).len = (*src).len;
    0
}

// Free resources used by an EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_free(eeprom: *mut cisco_eeprom) {
    if !eeprom.is_null() && !(*eeprom).data.is_null() {
        libc::free((*eeprom).data.cast::<_>());
        (*eeprom).data = null_mut();
        (*eeprom).len = 0;
    }
}

// Return TRUE if the specified EEPROM contains usable data
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_valid(eeprom: *mut cisco_eeprom) -> c_int {
    if !eeprom.is_null() && !(*eeprom).data.is_null() {
        TRUE
    } else {
        FALSE
    }
}

// Get a byte from an EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_get_byte(eeprom: *mut cisco_eeprom, offset: size_t, val: *mut m_uint8_t) -> c_int {
    let mut tmp: m_uint16_t;

    if offset >= ((*eeprom).len << 1) {
        *val = 0xFF;
        return -1;
    }

    tmp = *(*eeprom).data.add(offset >> 1);

    if 0 == (offset & 1) {
        tmp >>= 8;
    }

    *val = (tmp & 0xFF) as m_uint8_t;
    0
}

// Set a byte to an EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_set_byte(eeprom: *mut cisco_eeprom, offset: size_t, val: m_uint8_t) -> c_int {
    let mut tmp: m_uint16_t;

    if offset >= ((*eeprom).len << 1) {
        return -1;
    }

    tmp = *(*eeprom).data.add(offset >> 1);

    if (offset & 1) != 0 {
        tmp = (tmp & 0xFF00) | (val as m_uint16_t);
    } else {
        tmp = (tmp & 0x00FF) | (val as m_uint16_t) << 8;
    }

    *(*eeprom).data.add(offset >> 1) = tmp;
    0
}

// Get an EEPROM region
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_get_region(eeprom: *mut cisco_eeprom, offset: size_t, data: *mut m_uint8_t, data_len: size_t) -> c_int {
    for i in 0..data_len {
        if cisco_eeprom_get_byte(eeprom, offset + i, data.add(i)) == -1 {
            return -1;
        }
    }

    0
}

// Set an EEPROM region
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_set_region(eeprom: *mut cisco_eeprom, offset: size_t, data: *mut m_uint8_t, data_len: size_t) -> c_int {
    for i in 0..data_len {
        if cisco_eeprom_set_byte(eeprom, offset + i, *data.add(i)) == -1 {
            return -1;
        }
    }

    0
}

// Get a field of a Cisco EEPROM v4
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_v4_get_field(eeprom: *mut cisco_eeprom, type_: *mut m_uint8_t, len: *mut m_uint8_t, mut offset: *mut size_t) -> c_int {
    let mut tmp: m_uint8_t;

    // Read field type
    if cisco_eeprom_get_byte(eeprom, *offset, type_) == -1 {
        return -1;
    }
    offset = offset.add(1);

    // No more field
    if *type_ == 0xFF {
        return 0;
    }

    // Get field length
    tmp = (*type_ >> 6) & 0x03;

    if tmp == 0x03 {
        // Variable len
        if cisco_eeprom_get_byte(eeprom, *offset, addr_of_mut!(tmp)) == -1 {
            return -1;
        }
        #[allow(unused_assignments)]
        {
            offset = offset.add(1);
        }

        *len = tmp & 0x0F;
    } else {
        // Fixed len
        *len = 1 << tmp;
    }

    1
}

// Dump a Cisco EEPROM unformatted
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_dump(eeprom: *mut cisco_eeprom) {
    libc::printf(c"Dumping EEPROM contents:\n".as_ptr());
    let mut i: size_t = 0;
    let mut tmp: m_uint8_t = 0;
    loop {
        if cisco_eeprom_get_byte(eeprom, i, addr_of_mut!(tmp)) == -1 {
            break;
        }
        libc::printf(c" 0x%2.2x".as_ptr(), tmp as c_uint);
        i += 1;
        if i % 16 == 0 {
            libc::printf(c"\n".as_ptr());
        }
    }
    libc::printf(c"\n".as_ptr());
}

// Dump a Cisco EEPROM with format version 4
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_v4_dump(eeprom: *mut cisco_eeprom) {
    let mut type_: m_uint8_t = 0;
    let mut len: m_uint8_t = 0;
    let mut tmp: m_uint8_t = 0;
    let mut offset: size_t = 2;

    libc::printf(c"Dumping EEPROM contents:\n".as_ptr());

    loop {
        // Read field
        if cisco_eeprom_v4_get_field(eeprom, addr_of_mut!(type_), addr_of_mut!(len), addr_of_mut!(offset)) < 1 {
            break;
        }

        libc::printf(c"  Field 0x%2.2x: ".as_ptr(), type_ as c_uint);

        for i in 0..len as size_t {
            if cisco_eeprom_get_byte(eeprom, offset + i, addr_of_mut!(tmp)) == -1 {
                break;
            }

            libc::printf(c"%2.2x ".as_ptr(), tmp as c_uint);
        }

        libc::printf(c"\n".as_ptr());

        offset += len as size_t;
        if offset < ((*eeprom).len << 1) {
            continue;
        }
        break;
    }
}

// Returns the offset of the specified field
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_v4_find_field(eeprom: *mut cisco_eeprom, field_type: m_uint8_t, field_offset: *mut size_t) -> c_int {
    let mut type_: m_uint8_t = 0;
    let mut len: m_uint8_t = 0;
    let mut offset: size_t = 2;

    loop {
        // Read field
        if cisco_eeprom_v4_get_field(eeprom, addr_of_mut!(type_), addr_of_mut!(len), addr_of_mut!(offset)) < 1 {
            break;
        }

        if type_ == field_type {
            *field_offset = offset;
            return 0;
        }

        offset += len as size_t;
        if offset < ((*eeprom).len << 1) {
            continue;
        }
        break;
    }

    -1
}
