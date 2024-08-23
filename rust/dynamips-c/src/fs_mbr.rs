//! Master Boot Record
//!
//! Based on http://thestarman.pcministry.com/asm/mbr/PartTables.htm
//!
//! Copyright (c) 2014 Flávio J. Saraiva <flaviojs2005@gmail.com>

use crate::_private::*;
use crate::dynamips_common::*;

pub const MBR_CYLINDER_MIN: m_uint16_t = 0;
pub const MBR_CYLINDER_MAX: m_uint16_t = 1023;
pub const MBR_HEAD_MIN: m_uint8_t = 0;
pub const MBR_HEAD_MAX: m_uint8_t = 254;
pub const MBR_SECTOR_MIN: m_uint8_t = 1;
pub const MBR_SECTOR_MAX: m_uint8_t = 63;

pub const MBR_PARTITION_BOOTABLE: m_uint8_t = 0x80;

pub const MBR_PARTITION_TYPE_FAT16: m_uint8_t = 0x04;

pub const MBR_SIGNATURE_0: m_uint8_t = 0x55;
pub const MBR_SIGNATURE_1: m_uint8_t = 0xAA;

const MBR_OFFSET: libc::off_t = 512 - size_of::<mbr_data>() as libc::off_t;

/// A partition of the MBR
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct mbr_partition {
    pub bootable: m_uint8_t,
    pub first_chs: [m_uint8_t; 3],
    pub r#type: m_uint8_t,
    pub last_chs: [m_uint8_t; 3],
    pub lba: m_uint32_t,
    pub nr_sectors: m_uint32_t,
}

/// The MBR data
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct mbr_data {
    pub partition: [mbr_partition; 4],
    pub signature: [m_uint8_t; 2],
}

/// Decode a CHS reference
#[no_mangle]
pub unsafe extern "C" fn mbr_get_chs(chs: *mut m_uint8_t, cyl: *mut m_uint16_t, head: *mut m_uint8_t, sect: *mut m_uint8_t) {
    if !head.is_null() {
        *head = *chs.add(0);
    }
    if !sect.is_null() {
        *sect = *chs.add(1) & 0x3F;
    }
    if !cyl.is_null() {
        *cyl = (((*chs.add(1) & 0xC0) as m_uint16_t) << 2) | (*chs.add(2) as m_uint16_t);
    }
}

/// Encode a CHS reference
#[no_mangle]
pub unsafe extern "C" fn mbr_set_chs(chs: *mut m_uint8_t, cyl: m_uint16_t, head: m_uint8_t, sect: m_uint8_t) {
    if cyl > MBR_CYLINDER_MAX {
        // c=1023, h=254, s=63
        *chs.add(0) = 0xFE;
        *chs.add(1) = 0xFF;
        *chs.add(2) = 0xFF;
    } else {
        *chs.add(0) = head;
        *chs.add(1) = ((cyl >> 2) & 0xC0) as m_uint8_t | (sect & 0x3F);
        *chs.add(2) = (cyl & 0xFF) as m_uint8_t;
    }
}

/// Write MBR data
#[no_mangle]
pub unsafe extern "C" fn mbr_write_fd(fd: c_int, mbr: *mut mbr_data) -> c_int {
    if mbr.is_null() {
        libc::fprintf(c_stderr(), cstr!("mbr_write_fd: null"));
        return -1;
    }

    if libc::lseek(fd, MBR_OFFSET, libc::SEEK_SET) != MBR_OFFSET {
        libc::perror(cstr!("mbr_write_fd: lseek"));
        return -1;
    }

    if libc::write(fd, mbr.cast::<_>(), size_of::<mbr_data>()) != size_of::<mbr_data>() as ssize_t {
        libc::perror(cstr!("mbr_write_fd: write"));
        return -1;
    }

    0
}

/// Read MBR data
#[no_mangle]
pub unsafe extern "C" fn mbr_read_fd(fd: c_int, mbr: *mut mbr_data) -> c_int {
    if mbr.is_null() {
        libc::fprintf(c_stderr(), cstr!("mbr_read_fd: null"));
        return -1;
    }

    if libc::lseek(fd, MBR_OFFSET, libc::SEEK_SET) != MBR_OFFSET as libc::off_t {
        libc::perror(cstr!("mbr_read_fd: lseek"));
        return -1;
    }

    if libc::read(fd, mbr.cast::<_>(), size_of::<mbr_data>()) != size_of::<mbr_data>() as ssize_t {
        libc::perror(cstr!("mbr_read_fd: read"));
        return -1;
    }

    0
}
