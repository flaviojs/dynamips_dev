//! Cisco NVRAM filesystem.
//!
//! Format was inferred by analysing the NVRAM data after changing/erasing stuff.
//! All data is big endian.
//!
//! Based on the platforms c1700/c2600/c2692/c3600/c3725/c3745/c7200/c6msfc1.
//!
//! Copyright (c) 2013 Flávio J. Saraiva <flaviojs2005@gmail.com>

use crate::_private::*;
use crate::dynamips_common::*;
use std::cmp::min;

pub type fs_nvram_t = fs_nvram;

//=========================================================
// Filesystem

/// Size of a sector.
pub const FS_NVRAM_SECTOR_SIZE: size_t = 0x400;

/// Sector contains the start of the file.
pub const FS_NVRAM_FLAG_FILE_START: m_uint16_t = 0x01;

/// Sector contains the end of the file.
pub const FS_NVRAM_FLAG_FILE_END: m_uint16_t = 0x02;

/// File does not have read or write permission.
pub const FS_NVRAM_FLAG_FILE_NO_RW: m_uint16_t = 0x00; // TODO what is the correct value?

pub const FS_NVRAM_MAGIC_FILESYSTEM: m_uint16_t = 0xF0A5;
pub const FS_NVRAM_MAGIC_STARTUP_CONFIG: m_uint16_t = 0xABCD;
pub const FS_NVRAM_MAGIC_PRIVATE_CONFIG: m_uint16_t = 0xFEDC;
pub const FS_NVRAM_MAGIC_FILE_SECTOR: m_uint16_t = 0xDCBA;

/// Data is not compressed.
pub const FS_NVRAM_FORMAT_RAW: m_uint16_t = 1;

/// Data is compressed in .Z file format.
pub const FS_NVRAM_FORMAT_LZC: m_uint16_t = 2;

/// Magic not found - custom errno code.
pub const FS_NVRAM_ERR_NO_MAGIC: c_int = -(FS_NVRAM_MAGIC_FILESYSTEM as c_int);

/// Backup data doesn't match.
pub const FS_NVRAM_ERR_BACKUP_MISSMATCH: c_int = FS_NVRAM_ERR_NO_MAGIC - 1;

/// Invalid address found in filesystem.
pub const FS_NVRAM_ERR_INVALID_ADDRESS: c_int = FS_NVRAM_ERR_NO_MAGIC - 2;

/// Size of blocks in a NVRAM filesystem with backup (total size is 0x4C000 in c3745)
pub const FS_NVRAM_NORMAL_FILESYSTEM_BLOCK1: u_int = 0x20000;
pub const FS_NVRAM_BACKUP_FILESYSTEM_BLOCK1: u_int = 0x1C000;

//=========================================================
// Optional flags for open

/// Create NVRAM filesystem if no magic.
pub const FS_NVRAM_FLAG_OPEN_CREATE: u_int = 0x0001;

/// Don't scale byte offsets. (default, ignored)
pub const FS_NVRAM_FLAG_NO_SCALE: u_int = 0x0010;

/// Scale byte offsets by 4.
pub const FS_NVRAM_FLAG_SCALE_4: u_int = 0x0020;

/// Align the private-config header to 4 bytes with a padding of 7/6/5/0 bytes. (default, ignored)
pub const FS_NVRAM_FLAG_ALIGN_4_PAD_8: u_int = 0x0040;

/// Align the private-config header to 4 bytes with a padding of 3/2/1/0 bytes.
pub const FS_NVRAM_FLAG_ALIGN_4_PAD_4: u_int = 0x0080;

/// Has a backup filesystem.
/// Data is not continuous:
///   up to 0x20000 bytes of the normal filesystem;
///   up to 0x1C000 bytes of the backup filesystem;
///   rest of normal filesystem;
///   rest of backup filesystem.
pub const FS_NVRAM_FLAG_WITH_BACKUP: u_int = 0x0100;

/// Use addresses relative to the the end of the filesystem magic. (default, ignored)
/// Add 8 to get the raw offset.
pub const FS_NVRAM_FLAG_ADDR_RELATIVE: u_int = 0x0200;

/// Use absolute addresses.
/// The base address of the filesystem is the addr argument.
pub const FS_NVRAM_FLAG_ADDR_ABSOLUTE: u_int = 0x0400;

/// Value of unk1 is set to 0x0C04. (default, ignored)
pub const FS_NVRAM_FLAGS_UNK1_0C04: u_int = 0x0800;

/// Value of unk1 is set to 0x0C03.
pub const FS_NVRAM_FLAGS_UNK1_0C03: u_int = 0x1000;

/// Value of unk1 is set to 0x0C01.
pub const FS_NVRAM_FLAGS_UNK1_0C01: u_int = 0x2000;

pub const FS_NVRAM_FORMAT_MASK: u_int = 0x3FF0;

/// Default filesystem format. (default, ignored)
pub const FS_NVRAM_FORMAT_DEFAULT: u_int = FS_NVRAM_FLAG_NO_SCALE | FS_NVRAM_FLAG_ALIGN_4_PAD_8 | FS_NVRAM_FLAG_ADDR_RELATIVE | FS_NVRAM_FLAGS_UNK1_0C04;

/// Filesystem format for the c2600 platform.
pub const FS_NVRAM_FORMAT_SCALE_4: u_int = FS_NVRAM_FLAG_SCALE_4 | FS_NVRAM_FLAG_ALIGN_4_PAD_8 | FS_NVRAM_FLAG_ADDR_RELATIVE | FS_NVRAM_FLAGS_UNK1_0C03;

/// Filesystem format for the c3725/c3745 platforms.
pub const FS_NVRAM_FORMAT_WITH_BACKUP: u_int = FS_NVRAM_FLAG_NO_SCALE | FS_NVRAM_FLAG_ALIGN_4_PAD_4 | FS_NVRAM_FLAG_ADDR_RELATIVE | FS_NVRAM_FLAGS_UNK1_0C04 | FS_NVRAM_FLAG_WITH_BACKUP;

/// Filesystem format for the c7000 platform.
pub const FS_NVRAM_FORMAT_ABSOLUTE: u_int = FS_NVRAM_FLAG_NO_SCALE | FS_NVRAM_FLAG_ALIGN_4_PAD_4 | FS_NVRAM_FLAG_ADDR_ABSOLUTE | FS_NVRAM_FLAGS_UNK1_0C04;

/// Filesystem format for the c6msfc1 platform.
pub const FS_NVRAM_FORMAT_ABSOLUTE_C6: u_int = FS_NVRAM_FLAG_NO_SCALE | FS_NVRAM_FLAG_ALIGN_4_PAD_4 | FS_NVRAM_FLAG_ADDR_ABSOLUTE | FS_NVRAM_FLAGS_UNK1_0C01;

//=========================================================
// Flags for verify

/// Verify backup data.
pub const FS_NVRAM_VERIFY_BACKUP: u_int = 0x01;

/// Verify config data.
pub const FS_NVRAM_VERIFY_CONFIG: u_int = 0x02;

// TODO Verify file data.
//pub const FS_NVRAM_VERIFY_FILES: u_int = 0x04;

/// Verify everything.
pub const FS_NVRAM_VERIFY_ALL: u_int = 0x07;

//=========================================================

/// Header of the NVRAM filesystem.
/// When empty, only this magic and the checksum are filled.
/// @see nvram_header_startup_config
/// @see nvram_header_private_config
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct fs_nvram_header {
    /// Padding.
    pub padding: [u_char; 6],

    /// Magic value 0xF0A5.
    pub magic: m_uint16_t,
}

/// Header of special file startup-config.
/// @see nvram_header
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct fs_nvram_header_startup_config {
    /// Magic value 0xABCD.
    pub magic: m_uint16_t,

    /// Format of the data.
    /// 0x0001 - raw data;
    /// 0x0002 - .Z compressed (12 bits);
    pub format: m_uint16_t,

    /// Checksum of filesystem data. (all data after the filesystem magic)
    pub checksum: m_uint16_t,

    /// 0x0C04 - maybe maximum amount of free space that will be reserved?
    pub unk1: m_uint16_t,

    /// Address of the data.
    pub start: m_uint32_t,

    /// Address right after the data.
    pub end: m_uint32_t,

    /// Length of block.
    pub len: m_uint32_t,

    /// 0x00000000
    pub unk2: m_uint32_t,

    /// 0x00000000 if raw data, 0x00000001 if compressed
    pub unk3: m_uint32_t,

    /// 0x0000 if raw data, 0x0001 if compressed
    pub unk4: m_uint16_t,

    /// 0x0000
    pub unk5: m_uint16_t,

    /// Length of uncompressed data, 0 if raw data.
    pub uncompressed_len: m_uint32_t,
}

/// Header of special file private-config.
/// @see nvram_header
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct fs_nvram_header_private_config {
    /// Magic value 0xFEDC.
    pub magic: m_uint16_t,

    /// Format of the file.
    /// 0x0001 - raw data;
    pub format: m_uint16_t,

    /// Address of the data.
    pub start: m_uint32_t,

    /// Address right after the data.
    pub end: m_uint32_t,

    /// Length of block.
    pub len: m_uint32_t,
}

/// Sector containing file data.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct fs_nvram_file_sector {
    /// Magic value 0xDCBA
    pub magic: m_uint16_t,

    /// Next sector with data, 0 by default
    pub next_sector: m_uint16_t,

    /// Flags.
    /// @see FS_NVRAM_FLAG_FILE_START
    /// @see FS_NVRAM_FLAG_FILE_END
    /// @see FS_NVRAM_FLAG_FILE_NO_RW
    pub flags: m_uint16_t,

    /// Amount of data in this sector.
    pub length: m_uint16_t,

    /// File name, always NUL-terminated.
    pub filename: [c_char; 24],

    /// File data.
    pub data: [u_char; 992],
}

const DEBUG_BACKUP: c_int = 0;

/// NVRAM filesystem.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fs_nvram {
    pub base: *mut u_char,
    pub len: size_t,
    pub addr: m_uint32_t, // start address of the filesystem (for absolute addresses)
    pub flags: u_int,     // filesystem flags
    pub shift: u_int,     // scale byte offsets
    pub padding: u_int,   // base padding value
    pub backup: size_t,   // start offset of the backup filesystem
    pub read_byte: Option<unsafe extern "C" fn(fs: *mut fs_nvram_t, offset: u_int) -> m_uint8_t>,
    pub write_byte: Option<unsafe extern "C" fn(fs: *mut fs_nvram_t, offset: u_int, val: m_uint8_t)>,
}

//=========================================================
// Auxiliary

/// Convert a 16 bit value from big endian to native.
#[inline]
unsafe fn be_to_native16(val: *mut m_uint16_t) {
    union U {
        val: m_uint16_t,
        b: [m_uint8_t; 2],
    }
    let u = U { val: *val };
    *val = ((u.b[0] as m_uint16_t) << 8) | (u.b[1] as m_uint16_t);
}

/// Convert a 32 bit value from big endian to native.
#[inline]
unsafe fn be_to_native32(val: *mut m_uint32_t) {
    union U {
        val: m_uint32_t,
        b: [m_uint8_t; 4],
    }
    let u = U { val: *val };
    *val = ((u.b[0] as m_uint32_t) << 24) | ((u.b[1] as m_uint32_t) << 16) | ((u.b[2] as m_uint32_t) << 8) | (u.b[3] as m_uint32_t);
}

/// Convert startup-config header values from big endian to native.
unsafe fn be_to_native_header_startup(head: *mut fs_nvram_header_startup_config) {
    be_to_native16(addr_of_mut!((*head).magic));
    be_to_native16(addr_of_mut!((*head).format));
    be_to_native16(addr_of_mut!((*head).checksum));
    be_to_native16(addr_of_mut!((*head).unk1));
    be_to_native32(addr_of_mut!((*head).start));
    be_to_native32(addr_of_mut!((*head).end));
    be_to_native32(addr_of_mut!((*head).len));
    be_to_native32(addr_of_mut!((*head).unk2));
    be_to_native32(addr_of_mut!((*head).unk3));
    be_to_native16(addr_of_mut!((*head).unk4));
    be_to_native16(addr_of_mut!((*head).unk5));
    be_to_native32(addr_of_mut!((*head).uncompressed_len));
}

/// Convert private-config header values from big endian to native.
unsafe fn be_to_native_header_private(head: *mut fs_nvram_header_private_config) {
    be_to_native16(addr_of_mut!((*head).magic));
    be_to_native16(addr_of_mut!((*head).format));
    be_to_native32(addr_of_mut!((*head).start));
    be_to_native32(addr_of_mut!((*head).end));
    be_to_native32(addr_of_mut!((*head).len));
}

/// Convert a 16 bit value from native to big endian.
#[inline]
unsafe fn native_to_be16(val: *mut m_uint16_t) {
    union U {
        val: m_uint16_t,
        b: [m_uint8_t; 2],
    }
    let u = U { b: [(*val >> 8) as m_uint8_t, (*val & 0xFF) as m_uint8_t] };
    *val = u.val;
}

/// Convert a 32 bit value from native to big endian.
#[inline]
unsafe fn native_to_be32(val: *mut m_uint32_t) {
    union U {
        val: m_uint32_t,
        b: [m_uint8_t; 4],
    }
    let u = U { b: [(*val >> 24) as m_uint8_t, (*val >> 16) as m_uint8_t, (*val >> 8) as m_uint8_t, (*val & 0xFF) as m_uint8_t] };
    *val = u.val;
}

/// Convert startup-config header values from native to big endian.
unsafe fn native_to_be_header_startup(head: *mut fs_nvram_header_startup_config) {
    native_to_be16(addr_of_mut!((*head).magic));
    native_to_be16(addr_of_mut!((*head).format));
    native_to_be16(addr_of_mut!((*head).checksum));
    native_to_be16(addr_of_mut!((*head).unk1));
    native_to_be32(addr_of_mut!((*head).start));
    native_to_be32(addr_of_mut!((*head).end));
    native_to_be32(addr_of_mut!((*head).len));
    native_to_be32(addr_of_mut!((*head).unk2));
    native_to_be32(addr_of_mut!((*head).unk3));
    native_to_be16(addr_of_mut!((*head).unk4));
    native_to_be16(addr_of_mut!((*head).unk5));
    native_to_be32(addr_of_mut!((*head).uncompressed_len));
}

/// Convert private-config header values from native to big endian.
unsafe fn native_to_be_header_private(head: *mut fs_nvram_header_private_config) {
    native_to_be16(addr_of_mut!((*head).magic));
    native_to_be16(addr_of_mut!((*head).format));
    native_to_be32(addr_of_mut!((*head).start));
    native_to_be32(addr_of_mut!((*head).end));
    native_to_be32(addr_of_mut!((*head).len));
}

/// Uncompress data in .Z file format.
/// Adapted from 7zip's ZDecoder.cpp, which is licensed under LGPL 2.1.
unsafe fn uncompress_LZC(in_data: *const u_char, in_len: u_int, out_data: *mut u_char, out_len: u_int) -> c_int {
    const LZC_MAGIC_1: u_char = 0x1F;
    const LZC_MAGIC_2: u_char = 0x9D;
    const LZC_NUM_BITS_MASK: u_char = 0x1F;
    const LZC_BLOCK_MODE_MASK: u_char = 0x80;
    const LZC_NUM_BITS_MIN: c_int = 9;
    const LZC_NUM_BITS_MAX: c_int = 16;

    if in_len < 3 || (in_data.is_null() && in_len > 0) || (out_data.is_null() && out_len > 0) {
        return libc::EINVAL; // invalid argument
    }

    if *in_data.add(0) != LZC_MAGIC_1 || *in_data.add(1) != LZC_MAGIC_2 {
        return libc::ENOTSUP; // no magic
    }

    let maxbits: c_int = (*in_data.add(2) & LZC_NUM_BITS_MASK) as c_int;
    #[allow(clippy::manual_range_contains)]
    if maxbits < LZC_NUM_BITS_MIN || maxbits > LZC_NUM_BITS_MAX {
        return libc::ENOTSUP; // maxbits not supported
    }

    let numItems: m_uint32_t = 1 << maxbits;
    let blockMode: m_uint8_t = ((*in_data.add(2) & LZC_BLOCK_MODE_MASK) != 0) as m_uint8_t;

    let parents: *mut m_uint16_t = libc::malloc(numItems as usize * size_of::<m_uint16_t>()).cast::<_>();
    if parents.is_null() {
        return libc::ENOMEM; // out of memory
    }
    let suffixes: *mut m_uint8_t = libc::malloc(numItems as usize * size_of::<m_uint8_t>()).cast::<_>();
    if suffixes.is_null() {
        libc::free(parents.cast::<_>());
        return libc::ENOMEM; // out of memory
    }
    let stack: *mut m_uint8_t = libc::malloc(numItems as usize * size_of::<m_uint8_t>()).cast::<_>();
    if stack.is_null() {
        libc::free(parents.cast::<_>());
        libc::free(suffixes.cast::<_>());
        return libc::ENOMEM; // out of memory
    }

    let mut in_pos: u_int = 3;
    let mut out_pos: u_int = 0;
    let mut numBits: c_int = LZC_NUM_BITS_MIN;
    let mut head: m_uint32_t = if blockMode != 0 { 257 } else { 256 };

    let mut needPrev: m_uint8_t = 0;

    let mut bitPos: u_int = 0;
    let mut numBufBits: u_int = 0;

    let buf: [u_char; LZC_NUM_BITS_MAX as usize + 4] = [0; LZC_NUM_BITS_MAX as usize + 4];

    *parents.add(256) = 0;
    *suffixes.add(256) = 0;

    loop {
        if numBufBits == bitPos {
            let len: u_int = min(in_len - in_pos, numBits as u_int);
            libc::memcpy(buf.as_ptr().cast_mut().cast::<_>(), in_data.add(in_pos as usize).cast::<_>(), len as size_t);
            numBufBits = len << 3;
            bitPos = 0;
            in_pos += len;
        }
        let bytePos: u_int = bitPos >> 3;
        let mut symbol: m_uint32_t = (buf[bytePos as usize] as m_uint32_t) | ((buf[bytePos as usize + 1] as m_uint32_t) << 8) | ((buf[bytePos as usize + 2] as m_uint32_t) << 16);
        symbol >>= bitPos & 7;
        symbol &= (1 << numBits) - 1;
        bitPos += numBits as u_int;
        if bitPos > numBufBits {
            break;
        }
        if symbol >= head {
            libc::free(parents.cast::<_>());
            libc::free(suffixes.cast::<_>());
            libc::free(stack.cast::<_>());
            return -1; //libc::EIO; // invalid data
        }
        if blockMode != 0 && symbol == 256 {
            numBufBits = 0;
            bitPos = 0;
            numBits = LZC_NUM_BITS_MIN;
            head = 257;
            needPrev = 0;
            continue;
        }
        let mut cur: m_uint32_t = symbol;
        let mut i: c_int = 0;
        while cur >= 256 {
            *stack.offset(i as isize) = *suffixes.add(cur as usize);
            i += 1;
            cur = *parents.add(cur as usize) as m_uint32_t;
        }
        *stack.offset(i as isize) = cur as u_char;
        i += 1;
        if needPrev != 0 {
            *suffixes.add(head as usize - 1) = cur as u_char;
            if symbol == head - 1 {
                *stack.add(0) = cur as u_char;
            }
        }
        loop {
            if out_pos < out_len {
                i -= 1;
                *out_data.add(out_pos as usize) = *stack.offset(i as isize);
                out_pos += 1;
            } else {
                i = 0;
            }
            if i > 0 {
                continue;
            }
            break;
        }
        if head < numItems {
            needPrev = 1;
            *parents.add(head as usize) = symbol as m_uint16_t;
            head += 1;
            if head > (1 << numBits) && numBits < maxbits {
                numBufBits = 0;
                bitPos = 0;
                numBits += 1;
            }
        } else {
            needPrev = 0;
        }
    }

    libc::free(parents.cast::<_>());
    libc::free(suffixes.cast::<_>());
    libc::free(stack.cast::<_>());
    0
}

//=========================================================
// Private

/// Retuns address of the specified filesystem offset
#[inline]
unsafe fn fs_nvram_address_of(fs: *mut fs_nvram_t, offset: m_uint32_t) -> m_uint32_t {
    if ((*fs).flags & FS_NVRAM_FLAG_ADDR_ABSOLUTE) != 0 {
        (*fs).addr + offset
    } else {
        offset - 8
    }
}

/// Retuns filesystem offset of the specified address
#[inline]
unsafe fn fs_nvram_offset_of(fs: *mut fs_nvram_t, address: m_uint32_t) -> m_uint32_t {
    if ((*fs).flags & FS_NVRAM_FLAG_ADDR_ABSOLUTE) != 0 {
        address - (*fs).addr
    } else {
        address + 8
    }
}

/// Retuns padding at the specified offset
#[inline]
unsafe fn fs_nvram_padding_at(fs: *mut fs_nvram_t, offset: m_uint32_t) -> m_uint32_t {
    let mut padding: u_int = 0;

    if offset % 4 != 0 {
        padding = (*fs).padding - offset % 4;
    }

    padding
}

/// Read a 16-bit value from NVRAM.
#[inline]
unsafe fn fs_nvram_read16(fs: *mut fs_nvram_t, offset: u_int) -> m_uint16_t {
    let mut val: m_uint16_t;
    val = ((*fs).read_byte.unwrap()(fs, offset) as m_uint16_t) << 8;
    val |= (*fs).read_byte.unwrap()(fs, offset + 1) as m_uint16_t;
    val
}

/// Write a 16-bit value to NVRAM.
unsafe fn fs_nvram_write16(fs: *mut fs_nvram_t, offset: u_int, val: m_uint16_t) {
    (*fs).write_byte.unwrap()(fs, offset, (val >> 8) as m_uint8_t);
    (*fs).write_byte.unwrap()(fs, offset + 1, (val & 0xFF) as m_uint8_t);
}

/// Read a 32-bit value from NVRAM.
unsafe fn fs_nvram_read32(fs: *mut fs_nvram_t, offset: u_int) -> m_uint32_t {
    let mut val: m_uint32_t;
    val = ((*fs).read_byte.unwrap()(fs, offset) as m_uint32_t) << 24;
    val |= ((*fs).read_byte.unwrap()(fs, offset + 1) as m_uint32_t) << 16;
    val |= ((*fs).read_byte.unwrap()(fs, offset + 2) as m_uint32_t) << 8;
    val |= (*fs).read_byte.unwrap()(fs, offset + 3) as m_uint32_t;
    val
}

/// Write a 32-bit value to NVRAM.
unsafe fn fs_nvram_write32(fs: *mut fs_nvram_t, offset: u_int, val: m_uint32_t) {
    (*fs).write_byte.unwrap()(fs, offset, (val >> 24) as m_uint8_t);
    (*fs).write_byte.unwrap()(fs, offset + 1, (val >> 16) as m_uint8_t);
    (*fs).write_byte.unwrap()(fs, offset + 2, (val >> 8) as m_uint8_t);
    (*fs).write_byte.unwrap()(fs, offset + 3, (val & 0xFF) as m_uint8_t);
}

/// Read a buffer from NVRAM.
unsafe fn fs_nvram_memcpy_from(fs: *mut fs_nvram_t, offset: u_int, mut data: *mut u_char, len: u_int) {
    for i in 0..len {
        *data = (*fs).read_byte.unwrap()(fs, offset + i);
        data = data.add(1);
    }
}

/// Write a buffer to NVRAM.
unsafe fn fs_nvram_memcpy_to(fs: *mut fs_nvram_t, offset: u_int, mut data: *const u_char, len: u_int) {
    for i in 0..len {
        (*fs).write_byte.unwrap()(fs, offset + i, *data);
        data = data.add(1);
    }
}

/// Clear section of NVRAM.
unsafe fn fs_nvram_clear(fs: *mut fs_nvram_t, offset: u_int, len: u_int) {
    for i in 0..len {
        (*fs).write_byte.unwrap()(fs, offset + i, 0);
    }
}

/// Update the filesystem checksum.
unsafe fn fs_nvram_update_checksum(fs: *mut fs_nvram_t) {
    let mut sum: m_uint32_t = 0;

    fs_nvram_write16(fs, (size_of::<fs_nvram_header>() + offset_of!(fs_nvram_header_startup_config, checksum)) as u_int, 0x0000);

    let mut offset: u_int = size_of::<fs_nvram_header>() as u_int;
    let mut count: u_int = ((*fs).len - offset as size_t) as u_int;
    while count > 1 {
        sum += fs_nvram_read16(fs, offset) as m_uint32_t;
        offset += 2;
        count -= size_of::<m_uint16_t>() as u_int;
    }

    if count > 0 {
        sum += (((*fs).read_byte.unwrap()(fs, offset as c_uint)) as m_uint32_t) << 8;
    }

    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    sum = !sum;

    fs_nvram_write16(fs, (size_of::<fs_nvram_header>() + offset_of!(fs_nvram_header_startup_config, checksum)) as u_int, sum as u16);
}

/// Read data from NVRAM.
#[inline]
unsafe fn fs_nvram_read_data(fs: *mut fs_nvram_t, offset: u_int, len: u_int) -> *mut u8 {
    let data: *mut u_char = libc::malloc(len as size_t + 1).cast::<_>();
    if data.is_null() {
        return null_mut(); // out of memory
    }

    fs_nvram_memcpy_from(fs, offset, data, len);
    *data.add(len as usize) = 0;

    data
}

/// Create a NVRAM filesystem.
unsafe fn fs_nvram_create(fs: *mut fs_nvram_t) {
    fs_nvram_clear(fs, 0, (*fs).len as u_int);
    fs_nvram_write16(fs, offset_of!(fs_nvram_header, magic) as u_int, FS_NVRAM_MAGIC_FILESYSTEM);
    fs_nvram_write16(fs, (size_of::<fs_nvram_header>() + offset_of!(fs_nvram_header_startup_config, checksum)) as u_int, 0xFFFF);
}

/// Read a byte from the NVRAM filesystem.
unsafe extern "C" fn fs_nvram_read_byte(fs: *mut fs_nvram_t, offset: u_int) -> m_uint8_t {
    let ptr: *mut m_uint8_t = (*fs).base.add((offset << (*fs).shift) as usize);
    *ptr
}

/// Write a byte to the NVRAM filesystem.
unsafe extern "C" fn fs_nvram_write_byte(fs: *mut fs_nvram_t, offset: u_int, val: m_uint8_t) {
    let ptr: *mut m_uint8_t = (*fs).base.add((offset << (*fs).shift) as usize);
    *ptr = val;
}

/// Returns the normal offset of the NVRAM filesystem with backup.
#[inline]
unsafe fn fs_nvram_offset1_with_backup(fs: *mut fs_nvram_t, offset: u_int) -> u_int {
    if offset < FS_NVRAM_NORMAL_FILESYSTEM_BLOCK1 {
        offset << (*fs).shift
    } else {
        (FS_NVRAM_BACKUP_FILESYSTEM_BLOCK1 + offset) << (*fs).shift
    }
}

/// Returns the backup offset of the NVRAM filesystem with backup.
#[inline]
unsafe fn fs_nvram_offset2_with_backup(fs: *mut fs_nvram_t, offset: u_int) -> u_int {
    if offset < FS_NVRAM_BACKUP_FILESYSTEM_BLOCK1 {
        (((*fs).backup + offset as size_t) << (*fs).shift) as u_int
    } else {
        (((*fs).len + offset as size_t) << (*fs).shift) as u_int
    }
}

/// Read a byte from the NVRAM filesystem with backup.
unsafe extern "C" fn fs_nvram_read_byte_with_backup(fs: *mut fs_nvram_t, offset: u_int) -> m_uint8_t {
    let ptr1: *mut m_uint8_t = (*fs).base.add(fs_nvram_offset1_with_backup(fs, offset) as usize);
    if DEBUG_BACKUP != 0 {
        let ptr2: *mut m_uint8_t = (*fs).base.add(fs_nvram_offset2_with_backup(fs, offset) as usize);
        if *ptr1 != *ptr2 {
            libc::fprintf(
                c_stderr(),
                cstr!("fs_nvram_read_byte_with_backup: data in backup filesystem doesn't match (offset=%u, offset1=%u, offset2=%u, normal=0x%02X, backup=0x%02X)\n"),
                offset,
                fs_nvram_offset1_with_backup(fs, offset),
                fs_nvram_offset2_with_backup(fs, offset),
                *ptr1 as u_int,
                *ptr2 as u_int,
            );
        }
    }

    *ptr1
}

/// Write a byte to the NVRAM filesystem with backup.
unsafe extern "C" fn fs_nvram_write_byte_with_backup(fs: *mut fs_nvram_t, offset: u_int, val: m_uint8_t) {
    let ptr1: *mut m_uint8_t = (*fs).base.add(fs_nvram_offset1_with_backup(fs, offset) as usize);
    let ptr2: *mut m_uint8_t = (*fs).base.add(fs_nvram_offset2_with_backup(fs, offset) as usize);

    *ptr1 = val;
    *ptr2 = val;
}

//=========================================================
// Public

/// Open NVRAM filesystem. Sets errno.
#[no_mangle]
pub unsafe extern "C" fn fs_nvram_open(base: *mut u_char, len: size_t, addr: m_uint32_t, flags: u_int) -> *mut fs_nvram_t {
    let mut len_div: size_t = 1;

    if (flags & FS_NVRAM_FLAG_SCALE_4) != 0 {
        len_div *= 4; // a quarter of the size
    }

    if (flags & FS_NVRAM_FLAG_WITH_BACKUP) != 0 {
        len_div *= 2; // half the size is for the backup
    }

    if base.is_null() || len < size_of::<fs_nvram_header>() * len_div || len % (FS_NVRAM_SECTOR_SIZE * len_div) != 0 {
        c_errno_set(libc::EINVAL);
        return null_mut(); // invalid argument
    }

    let fs: *mut fs_nvram = libc::malloc(size_of::<fs_nvram>()).cast::<_>();
    if fs.is_null() {
        c_errno_set(libc::ENOMEM);
        return null_mut(); // out of memory
    }

    (*fs).base = base;
    (*fs).len = len / len_div;
    (*fs).addr = addr;
    (*fs).flags = flags;
    (*fs).shift = if (flags & FS_NVRAM_FLAG_SCALE_4) != 0 { 2 } else { 0 };
    (*fs).padding = if (flags & FS_NVRAM_FLAG_ALIGN_4_PAD_4) != 0 { 4 } else { 8 };
    (*fs).backup = if (flags & FS_NVRAM_FLAG_WITH_BACKUP) != 0 { min((*fs).len, FS_NVRAM_NORMAL_FILESYSTEM_BLOCK1 as size_t) } else { 0 };
    (*fs).read_byte = if (flags & FS_NVRAM_FLAG_WITH_BACKUP) != 0 { Some(fs_nvram_read_byte_with_backup) } else { Some(fs_nvram_read_byte) };
    (*fs).write_byte = if (flags & FS_NVRAM_FLAG_WITH_BACKUP) != 0 { Some(fs_nvram_write_byte_with_backup) } else { Some(fs_nvram_write_byte) };

    if FS_NVRAM_MAGIC_FILESYSTEM != fs_nvram_read16(fs, offset_of!(fs_nvram_header, magic) as u_int) {
        if (flags & FS_NVRAM_FLAG_OPEN_CREATE) == 0 {
            fs_nvram_close(fs);
            c_errno_set(FS_NVRAM_ERR_NO_MAGIC);
            return null_mut(); // no magic
        }

        fs_nvram_create(fs);
    }

    c_errno_set(0);
    fs
}

/// Close NVRAM filesystem.
#[no_mangle]
pub unsafe extern "C" fn fs_nvram_close(fs: *mut fs_nvram_t) {
    if !fs.is_null() {
        libc::free(fs.cast::<_>());
    }
}

/// Read startup-config and/or private-config from NVRAM.
/// Returns 0 on success.
#[no_mangle]
pub unsafe extern "C" fn fs_nvram_read_config(fs: *mut fs_nvram_t, startup_config: *mut *mut u_char, startup_len: *mut size_t, private_config: *mut *mut u_char, private_len: *mut size_t) -> c_int {
    if fs.is_null() {
        return libc::EINVAL; // invalid argument
    }

    // initial values
    if !startup_config.is_null() {
        *startup_config = null_mut();
    }

    if !startup_len.is_null() {
        *startup_len = 0;
    }

    if !private_config.is_null() {
        *private_config = null_mut();
    }

    if !private_len.is_null() {
        *private_len = 0;
    }

    // read headers
    let mut off: size_t = size_of::<fs_nvram_header>();
    let mut startup_head: fs_nvram_header_startup_config = zeroed::<_>();
    fs_nvram_memcpy_from(fs, off as u_int, addr_of_mut!(startup_head).cast::<_>(), size_of::<fs_nvram_header_startup_config>() as u_int);
    be_to_native_header_startup(addr_of_mut!(startup_head));
    if FS_NVRAM_MAGIC_STARTUP_CONFIG != startup_head.magic {
        return 0; // done, no startup-config and no private-config
    }

    unsafe fn _err_cleanup(startup_config: *mut *mut u_char, startup_len: *mut size_t, private_config: *mut *mut u_char, private_len: *mut size_t) {
        if !startup_config.is_null() && !(*startup_config).is_null() {
            libc::free((*startup_config).cast::<_>());
            *startup_config = null_mut();
        }

        if !startup_len.is_null() {
            *startup_len = 0;
        }

        if !private_config.is_null() && !(*private_config).is_null() {
            libc::free((*private_config).cast::<_>());
            *private_config = null_mut();
        }

        if !private_len.is_null() {
            *private_len = 0;
        }
    }

    off = fs_nvram_offset_of(fs, startup_head.start + startup_head.len) as size_t;
    off += fs_nvram_padding_at(fs, off as m_uint32_t) as size_t;

    if off + size_of::<fs_nvram_header_private_config>() > (*fs).len {
        _err_cleanup(startup_config, startup_len, private_config, private_len);
        return libc::ENOMEM; // out of memory
    }

    let mut private_head: fs_nvram_header_private_config = zeroed::<_>();
    fs_nvram_memcpy_from(fs, off as u_int, addr_of_mut!(private_head).cast::<_>(), size_of::<fs_nvram_header_private_config>() as u_int);
    be_to_native_header_private(addr_of_mut!(private_head));

    // read startup-config
    if FS_NVRAM_FORMAT_RAW == startup_head.format {
        if !startup_config.is_null() {
            off = fs_nvram_offset_of(fs, startup_head.start) as size_t;
            *startup_config = fs_nvram_read_data(fs, off as u_int, startup_head.len);
            if (*startup_config).is_null() {
                _err_cleanup(startup_config, startup_len, private_config, private_len);
                return libc::ENOMEM; // out of memory
            }
        }

        if !startup_len.is_null() {
            *startup_len = startup_head.len as size_t;
        }
    } else if FS_NVRAM_FORMAT_LZC == startup_head.format {
        if !startup_config.is_null() {
            off = fs_nvram_offset_of(fs, startup_head.start) as size_t;
            *startup_config = libc::malloc((startup_head.uncompressed_len + 1) as size_t).cast::<_>();
            if (*startup_config).is_null() {
                _err_cleanup(startup_config, startup_len, private_config, private_len);
                return libc::ENOMEM; // out of memory
            }

            let buf: *mut u_char = fs_nvram_read_data(fs, off as u_int, startup_head.len);
            if buf.is_null() {
                _err_cleanup(startup_config, startup_len, private_config, private_len);
                return libc::ENOMEM; // out of memory
            }

            let err: c_int = uncompress_LZC(buf, startup_head.len, *startup_config, startup_head.uncompressed_len);
            if err != 0 {
                libc::free(buf.cast::<_>());
                _err_cleanup(startup_config, startup_len, private_config, private_len);
                return err;
            }

            *(*startup_config).add(startup_head.uncompressed_len as usize) = 0;
            libc::free(buf.cast::<_>());
        }

        if !startup_len.is_null() {
            *startup_len = startup_head.uncompressed_len as size_t;
        }
    } else {
        _err_cleanup(startup_config, startup_len, private_config, private_len);
        return libc::ENOTSUP; // unsupported format
    }

    // read private-config
    if fs_nvram_offset_of(fs, private_head.start + private_head.len) as size_t > (*fs).len || FS_NVRAM_MAGIC_PRIVATE_CONFIG != private_head.magic {
        return 0; // done, no private-config
    }

    if FS_NVRAM_FORMAT_RAW == private_head.format {
        if !private_config.is_null() {
            off = fs_nvram_offset_of(fs, private_head.start) as size_t;
            *private_config = fs_nvram_read_data(fs, off as u_int, private_head.len);
            if (*private_config).is_null() {
                _err_cleanup(startup_config, startup_len, private_config, private_len);
                return libc::ENOMEM; // out of memory
            }
        }

        if !private_len.is_null() {
            *private_len = private_head.len as size_t;
        }
    } else {
        _err_cleanup(startup_config, startup_len, private_config, private_len);
        return libc::ENOTSUP; // unsupported format
    }

    0 // done
}

/// Write startup-config and private-config to NVRAM.
/// Returns 0 on success.
#[no_mangle]
pub unsafe extern "C" fn fs_nvram_write_config(fs: *mut fs_nvram_t, startup_config: *const u_char, startup_len: size_t, private_config: *const u_char, private_len: size_t) -> c_int {
    if fs.is_null() || (startup_config.is_null() && startup_len > 0) || (private_config.is_null() && private_len > 0) {
        return libc::EINVAL; // invalid argument
    }

    // check space and padding
    // XXX ignores normal files in NVRAM
    let mut len: size_t = size_of::<fs_nvram_header>() + size_of::<fs_nvram_header_startup_config>() + startup_len;
    let padding: size_t = fs_nvram_padding_at(fs, len as m_uint32_t) as size_t;
    len += padding + size_of::<fs_nvram_header_private_config>() + private_len;
    if (*fs).len < len {
        return libc::ENOSPC; // not enough space
    }

    // old length
    len = size_of::<fs_nvram_header>();
    if FS_NVRAM_MAGIC_STARTUP_CONFIG == fs_nvram_read16(fs, (len + offset_of!(fs_nvram_header_startup_config, magic)) as u_int) {
        len += fs_nvram_read32(fs, (len + offset_of!(fs_nvram_header_startup_config, len)) as u_int) as size_t;
        if len % 4 != 0 {
            len += 8 - len % 4;
        }

        if FS_NVRAM_MAGIC_PRIVATE_CONFIG == fs_nvram_read16(fs, (len + offset_of!(fs_nvram_header_private_config, magic)) as u_int) {
            len += fs_nvram_read32(fs, (len + offset_of!(fs_nvram_header_private_config, len)) as u_int) as size_t;
        }
    }

    if len % FS_NVRAM_SECTOR_SIZE != 0 {
        len += FS_NVRAM_SECTOR_SIZE - len % FS_NVRAM_SECTOR_SIZE; // whole sector
    }

    if len > (*fs).len {
        len = (*fs).len; // should never happen
    }

    // prepare headers
    let mut startup_head: fs_nvram_header_startup_config = zeroed::<_>();
    libc::memset(addr_of_mut!(startup_head).cast::<_>(), 0, size_of::<fs_nvram_header_startup_config>());
    startup_head.magic = FS_NVRAM_MAGIC_STARTUP_CONFIG;
    startup_head.format = FS_NVRAM_FORMAT_RAW;
    startup_head.unk1 = if ((*fs).flags & FS_NVRAM_FLAGS_UNK1_0C01) != 0 {
        0x0C01
    } else if ((*fs).flags & FS_NVRAM_FLAGS_UNK1_0C03) != 0 {
        0x0C03
    } else {
        0x0C04
    };
    startup_head.start = fs_nvram_address_of(fs, (size_of::<fs_nvram_header>() + size_of::<fs_nvram_header_startup_config>()) as m_uint32_t);
    startup_head.end = startup_head.start + startup_len as m_uint32_t;
    startup_head.len = startup_len as m_uint32_t;

    let mut private_head: fs_nvram_header_private_config = zeroed::<_>();
    libc::memset(addr_of_mut!(private_head).cast::<_>(), 0, size_of::<fs_nvram_header_private_config>());
    private_head.magic = FS_NVRAM_MAGIC_PRIVATE_CONFIG;
    private_head.format = FS_NVRAM_FORMAT_RAW;
    private_head.start = startup_head.end + (padding + size_of::<fs_nvram_header_private_config>()) as m_uint32_t;
    private_head.end = private_head.start + private_len as m_uint32_t;
    private_head.len = private_len as m_uint32_t;

    native_to_be_header_startup(addr_of_mut!(startup_head));
    native_to_be_header_private(addr_of_mut!(private_head));

    // write data
    let mut off: size_t = size_of::<fs_nvram_header>();

    fs_nvram_memcpy_to(fs, off as u_int, addr_of_mut!(startup_head).cast::<_>(), size_of::<fs_nvram_header_startup_config>() as u_int);
    off += size_of::<fs_nvram_header_startup_config>();
    fs_nvram_memcpy_to(fs, off as u_int, startup_config, startup_len as u_int);
    off += startup_len;

    fs_nvram_clear(fs, off as u_int, padding as u_int);
    off += padding;

    fs_nvram_memcpy_to(fs, off as u_int, addr_of_mut!(private_head).cast::<_>(), size_of::<fs_nvram_header_private_config>() as u_int);
    off += size_of::<fs_nvram_header_private_config>();
    fs_nvram_memcpy_to(fs, off as u_int, private_config, private_len as u_int);
    off += private_len;

    if off < len {
        fs_nvram_clear(fs, off as u_int, (len - off) as u_int);
    }

    fs_nvram_update_checksum(fs);

    0
}

/// Returns the number of sectors in the NVRAM filesystem.
#[no_mangle]
pub unsafe extern "C" fn fs_nvram_num_sectors(fs: *mut fs_nvram_t) -> size_t {
    if fs.is_null() {
        return 0;
    }

    (*fs).len / FS_NVRAM_SECTOR_SIZE
}

/// Verify the contents of the filesystem.
/// Returns 0 on success.
#[no_mangle]
pub unsafe extern "C" fn fs_nvram_verify(fs: *mut fs_nvram_t, what: u_int) -> c_int {
    if fs.is_null() {
        return libc::EINVAL; // invalid argument
    }

    if (what & FS_NVRAM_VERIFY_BACKUP) != 0 && ((*fs).flags & FS_NVRAM_FLAG_WITH_BACKUP) != 0 {
        for offset in 0..(*fs).len {
            let b1: m_uint8_t = *(*fs).base.add(fs_nvram_offset1_with_backup(fs, offset as u_int) as usize);
            let b2: m_uint8_t = *(*fs).base.add(fs_nvram_offset2_with_backup(fs, offset as u_int) as usize);
            if b1 != b2 {
                return FS_NVRAM_ERR_BACKUP_MISSMATCH; // data is corrupted? length is wrong?
            }
        }
    }

    if (what & FS_NVRAM_VERIFY_CONFIG) != 0 {
        let mut startup_head: fs_nvram_header_startup_config = zeroed::<_>();
        let mut private_head: fs_nvram_header_private_config = zeroed::<_>();

        let mut offset: size_t = size_of::<fs_nvram_header>();
        fs_nvram_memcpy_from(fs, offset as u_int, addr_of_mut!(startup_head).cast::<_>(), size_of::<fs_nvram_header_startup_config>() as u_int);
        be_to_native_header_startup(addr_of_mut!(startup_head));
        if FS_NVRAM_MAGIC_STARTUP_CONFIG == startup_head.magic {
            if startup_head.end != startup_head.start + startup_head.len || startup_head.len as size_t > (*fs).len {
                return FS_NVRAM_ERR_INVALID_ADDRESS; // data is corrupted?
            }
            if startup_head.start < (*fs).addr || startup_head.end as size_t > (*fs).addr as size_t + (*fs).len {
                return FS_NVRAM_ERR_INVALID_ADDRESS; // fs.addr has the wrong value?
            }

            offset = fs_nvram_offset_of(fs, startup_head.end) as size_t;
            offset += fs_nvram_padding_at(fs, offset as m_uint32_t) as size_t;
            if (*fs).len < offset + size_of::<fs_nvram_header_private_config>() {
                return FS_NVRAM_ERR_INVALID_ADDRESS; // data is corrupted?
            }

            fs_nvram_memcpy_from(fs, offset as u_int, addr_of_mut!(private_head).cast::<_>(), size_of::<fs_nvram_header_private_config>() as u_int);
            be_to_native_header_private(addr_of_mut!(private_head));
            if FS_NVRAM_MAGIC_PRIVATE_CONFIG == private_head.magic {
                if private_head.end != private_head.start + private_head.len || private_head.len as size_t > (*fs).len {
                    return FS_NVRAM_ERR_INVALID_ADDRESS; // data is corrupted?
                }
                if private_head.start < (*fs).addr || private_head.end as size_t > (*fs).addr as size_t + (*fs).len {
                    return FS_NVRAM_ERR_INVALID_ADDRESS; // fs->addr has the wrong value?
                }
                if private_head.end != private_head.start + private_head.len {
                    return FS_NVRAM_ERR_INVALID_ADDRESS; // data is corrupted?
                }
            }
        }
    }

    0
}
