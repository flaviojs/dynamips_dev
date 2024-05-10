//! Cisco EEPROM manipulation functions.

use crate::dynamips_common::*;
use crate::prelude::*;

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

/// Cisco EEPROM
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cisco_eeprom {
    pub name: *mut c_char,
    pub data: *mut m_uint16_t,
    pub len: size_t,
}

// ======================================================================
// Utility functions
// ======================================================================

/// Find an EEPROM in the specified EEPROM array
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_find(eeproms: *const cisco_eeprom, name: *mut c_char) -> *const cisco_eeprom {
    let mut i = 0;

    while !(*eeproms.add(i)).name.is_null() {
        if libc::strcmp((*eeproms.add(i)).name, name) == 0 {
            return eeproms.add(i);
        }
        i += 1;
    }

    null_mut()
}

/// Copy an EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_copy(dst: *mut cisco_eeprom, src: *const cisco_eeprom) -> c_int {
    if dst.is_null() || src.is_null() {
        return -1;
    }

    cisco_eeprom_free(dst);

    let data: *mut u16 = libc::malloc((*src).len << 1).cast::<_>();
    if data.is_null() {
        return -1;
    }

    libc::memcpy(data.cast::<_>(), (*src).data.cast::<_>(), (*src).len << 1);
    (*dst).name = (*src).name;
    (*dst).data = data;
    (*dst).len = (*src).len;
    0
}

/// Free resources used by an EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_free(eeprom: *mut cisco_eeprom) {
    if !eeprom.is_null() && !(*eeprom).data.is_null() {
        libc::free((*eeprom).data.cast::<_>());
        (*eeprom).data = null_mut();
        (*eeprom).len = 0;
    }
}

/// Return TRUE if the specified EEPROM contains usable data
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_valid(eeprom: *mut cisco_eeprom) -> c_int {
    if !eeprom.is_null() && !(*eeprom).data.is_null() {
        TRUE
    } else {
        FALSE
    }
}

#[no_mangle]
pub extern "C" fn _export(_: *mut cisco_eeprom) {}
