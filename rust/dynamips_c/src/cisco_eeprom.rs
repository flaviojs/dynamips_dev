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

/* Get a byte from an EEPROM */
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_get_byte(eeprom: *mut cisco_eeprom, offset: size_t, val: *mut m_uint8_t) -> c_int {
    if offset >= ((*eeprom).len << 1) {
        *val = 0xFF;
        return -1;
    }

    let mut tmp: u16 = *(*eeprom).data.add(offset >> 1);

    if (offset & 1) == 0 {
        tmp >>= 8;
    }

    *val = (tmp & 0xff) as u8;
    0
}

/// Set a byte to an EEPROM
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_set_byte(eeprom: *mut cisco_eeprom, offset: size_t, val: m_uint8_t) -> c_int {
    if offset >= ((*eeprom).len << 1) {
        return -1;
    }

    let mut tmp: u16 = *(*eeprom).data.add(offset >> 1);

    if (offset & 1) != 0 {
        tmp = (tmp & 0xFF00) | u16::from(val);
    } else {
        tmp = (tmp & 0x00FF) | (u16::from(val) << 8);
    }

    *(*eeprom).data.add(offset >> 1) = tmp;
    0
}

/// Get an EEPROM region
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_get_region(eeprom: *mut cisco_eeprom, offset: size_t, data: *mut m_uint8_t, data_len: size_t) -> c_int {
    for i in 0..data_len {
        if cisco_eeprom_get_byte(eeprom, offset + i, data.add(i)) == -1 {
            return -1;
        }
    }

    0
}

/// Set an EEPROM region
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_set_region(eeprom: *mut cisco_eeprom, offset: size_t, data: *mut m_uint8_t, data_len: size_t) -> c_int {
    for i in 0..data_len {
        if cisco_eeprom_set_byte(eeprom, offset + i, *data.add(i)) == -1 {
            return -1;
        }
    }

    0
}

/// Get a field of a Cisco EEPROM v4
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_v4_get_field(eeprom: *mut cisco_eeprom, type_: *mut m_uint8_t, len: *mut m_uint8_t, offset: *mut size_t) -> c_int {
    // Read field type
    let off = *offset;
    *offset += 1;
    if cisco_eeprom_get_byte(eeprom, off, type_) == -1 {
        return -1;
    }

    // No more field
    if *type_ == 0xFF {
        return 0;
    }

    // Get field length */
    let mut tmp: u8 = (*type_ >> 6) & 0x03;

    if tmp == 0x03 {
        // Variable len
        let off = *offset;
        *offset += 1;
        if cisco_eeprom_get_byte(eeprom, off, addr_of_mut!(tmp)) == -1 {
            return -1;
        }

        *len = tmp & 0x0F;
    } else {
        // Fixed len
        *len = 1 << tmp;
    }

    1
}

/// Dump a Cisco EEPROM unformatted
#[no_mangle]
pub unsafe extern "C" fn cisco_eeprom_dump(eeprom: *mut cisco_eeprom) {
    libc::printf(cstr!("Dumping EEPROM contents:\n"));
    let mut i: size_t = 0;
    loop {
        let mut tmp: u8 = 0;
        if cisco_eeprom_get_byte(eeprom, i, addr_of_mut!(tmp)) == -1 {
            break;
        }
        libc::printf(cstr!(" 0x%2.2x"), tmp as c_uint);
        i += 1;
        if i % 16 == 0 {
            libc::printf(cstr!("\n"));
        }
    }
    libc::printf(cstr!("\n"));
}

#[no_mangle]
pub extern "C" fn _export(_: *mut cisco_eeprom) {}
