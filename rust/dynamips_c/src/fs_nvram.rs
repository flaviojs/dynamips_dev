//! Cisco NVRAM filesystem.
//!
//! Format was inferred by analysing the NVRAM data after changing/erasing stuff.
//! All data is big endian.
//!
//! Based on the platforms c1700/c2600/c2692/c3600/c3725/c3745/c7200/c6msfc1.

use crate::dynamips_common::*;
use crate::prelude::*;
use std::cmp::min;

//=========================================================
// Filesystem

/// Size of a sector.
pub const FS_NVRAM_SECTOR_SIZE: size_t = 0x400;

/// Sector contains the start of the file.
pub const FS_NVRAM_FLAG_FILE_START: u16 = 0x01;

/// Sector contains the end of the file.
pub const FS_NVRAM_FLAG_FILE_END: u16 = 0x02;

/// File does not have read or write permission.
pub const FS_NVRAM_FLAG_FILE_NO_RW: u16 = 0x00; // TODO what is the correct value?

pub const FS_NVRAM_MAGIC_FILESYSTEM: u16 = 0xF0A5;
pub const FS_NVRAM_MAGIC_STARTUP_CONFIG: u16 = 0xABCD;
pub const FS_NVRAM_MAGIC_PRIVATE_CONFIG: u16 = 0xFEDC;
pub const FS_NVRAM_MAGIC_FILE_SECTOR: u16 = 0xDCBA;

/// Data is not compressed.
pub const FS_NVRAM_FORMAT_RAW: u16 = 1;

/// Data is compressed in .Z file format.
pub const FS_NVRAM_FORMAT_LZC: u16 = 2;

/// Magic not found - custom errno code.
pub const FS_NVRAM_ERR_NO_MAGIC: c_int = -(FS_NVRAM_MAGIC_FILESYSTEM as c_int);

/// Backup data doesn't match.
pub const FS_NVRAM_ERR_BACKUP_MISSMATCH: c_int = FS_NVRAM_ERR_NO_MAGIC - 1;

/// Invalid address found in filesystem.
pub const FS_NVRAM_ERR_INVALID_ADDRESS: c_int = FS_NVRAM_ERR_NO_MAGIC - 2;

/// Size of blocks in a NVRAM filesystem with backup (total size is 0x4C000 in c3745)
pub const FS_NVRAM_NORMAL_FILESYSTEM_BLOCK1: size_t = 0x20000;
pub const FS_NVRAM_BACKUP_FILESYSTEM_BLOCK1: size_t = 0x1C000;

//=========================================================
// Optional flags for open

/// Create NVRAM filesystem if no magic.
pub const FS_NVRAM_FLAG_OPEN_CREATE: c_uint = 0x0001;

/// Don't scale byte offsets. (default, ignored)
pub const FS_NVRAM_FLAG_NO_SCALE: c_uint = 0x0010;

/// Scale byte offsets by 4.
pub const FS_NVRAM_FLAG_SCALE_4: c_uint = 0x0020;

/// Align the private-config header to 4 bytes with a padding of 7/6/5/0 bytes. (default, ignored)
pub const FS_NVRAM_FLAG_ALIGN_4_PAD_8: c_uint = 0x0040;

/// Align the private-config header to 4 bytes with a padding of 3/2/1/0 bytes.
pub const FS_NVRAM_FLAG_ALIGN_4_PAD_4: c_uint = 0x0080;

/// Has a backup filesystem.
/// Data is not continuous:
///   up to 0x20000 bytes of the normal filesystem;
///   up to 0x1C000 bytes of the backup filesystem;
///   rest of normal filesystem;
///   rest of backup filesystem.
pub const FS_NVRAM_FLAG_WITH_BACKUP: c_uint = 0x0100;

/// Use addresses relative to the the end of the filesystem magic. (default, ignored)
/// Add 8 to get the raw offset.
pub const FS_NVRAM_FLAG_ADDR_RELATIVE: c_uint = 0x0200;

/// Use absolute addresses.
/// The base address of the filesystem is the addr argument.
pub const FS_NVRAM_FLAG_ADDR_ABSOLUTE: c_uint = 0x0400;

/// Value of unk1 is set to 0x0C04. (default, ignored)
pub const FS_NVRAM_FLAGS_UNK1_0C04: c_uint = 0x0800;

/// Value of unk1 is set to 0x0C03.
pub const FS_NVRAM_FLAGS_UNK1_0C03: c_uint = 0x1000;

/// Value of unk1 is set to 0x0C01.
pub const FS_NVRAM_FLAGS_UNK1_0C01: c_uint = 0x2000;

pub const FS_NVRAM_FORMAT_MASK: c_uint = 0x3FF0;

/// Default filesystem format. (default, ignored)
pub const FS_NVRAM_FORMAT_DEFAULT: c_uint = FS_NVRAM_FLAG_NO_SCALE | FS_NVRAM_FLAG_ALIGN_4_PAD_8 | FS_NVRAM_FLAG_ADDR_RELATIVE | FS_NVRAM_FLAGS_UNK1_0C04;

/// Filesystem format for the c2600 platform.
pub const FS_NVRAM_FORMAT_SCALE_4: c_uint = FS_NVRAM_FLAG_SCALE_4 | FS_NVRAM_FLAG_ALIGN_4_PAD_8 | FS_NVRAM_FLAG_ADDR_RELATIVE | FS_NVRAM_FLAGS_UNK1_0C03;

/// Filesystem format for the c3725/c3745 platforms.
pub const FS_NVRAM_FORMAT_WITH_BACKUP: c_uint = FS_NVRAM_FLAG_NO_SCALE | FS_NVRAM_FLAG_ALIGN_4_PAD_4 | FS_NVRAM_FLAG_ADDR_RELATIVE | FS_NVRAM_FLAGS_UNK1_0C04 | FS_NVRAM_FLAG_WITH_BACKUP;

/// Filesystem format for the c7000 platform.
pub const FS_NVRAM_FORMAT_ABSOLUTE: c_uint = FS_NVRAM_FLAG_NO_SCALE | FS_NVRAM_FLAG_ALIGN_4_PAD_4 | FS_NVRAM_FLAG_ADDR_ABSOLUTE | FS_NVRAM_FLAGS_UNK1_0C04;

/// Filesystem format for the c6msfc1 platform.
pub const FS_NVRAM_FORMAT_ABSOLUTE_C6: c_uint = FS_NVRAM_FLAG_NO_SCALE | FS_NVRAM_FLAG_ALIGN_4_PAD_4 | FS_NVRAM_FLAG_ADDR_ABSOLUTE | FS_NVRAM_FLAGS_UNK1_0C01;

//=========================================================
// Flags for verify

/// Verify backup data.
pub const FS_NVRAM_VERIFY_BACKUP: c_uint = 0x01;

/// Verify config data.
pub const FS_NVRAM_VERIFY_CONFIG: c_uint = 0x02;

// TODO Verify file data.
//pub const FS_NVRAM_VERIFY_FILES: c_uint = 0x04;

/// Verify everything.
pub const FS_NVRAM_VERIFY_ALL: c_uint = 0x07;

//=========================================================

/// Header of the NVRAM filesystem.
/// When empty, only this magic and the checksum are filled.
/// @see nvram_header_startup_config
/// @see nvram_header_private_config
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct fs_nvram_header {
    /// Padding.
    pub padding: [u8; 6],
    /// Magic value 0xF0A5.
    pub magic: u16,
}

/// Header of special file startup-config.
/// @see nvram_header
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct fs_nvram_header_startup_config {
    /// Magic value 0xABCD.
    pub magic: u16,
    /// Format of the data.
    /// 0x0001 - raw data;
    /// 0x0002 - .Z compressed (12 bits);
    pub format: u16,
    /// Checksum of filesystem data. (all data after the filesystem magic)
    pub checksum: u16,
    /// 0x0C04 - maybe maximum amount of free space that will be reserved?
    pub unk1: u16,
    /// Address of the data.
    pub start: u32,
    /// Address right after the data.
    pub end: u32,
    /// Length of block.
    pub len: u32,
    /// 0x00000000
    pub unk2: u32,
    /// 0x00000000 if raw data, 0x00000001 if compressed
    pub unk3: u32,
    /// 0x0000 if raw data, 0x0001 if compressed
    pub unk4: u16,
    /// 0x0000
    pub unk5: u16,
    /// Length of uncompressed data, 0 if raw data.
    pub uncompressed_len: u32,
}

/// Header of special file private-config.
/// @see nvram_header
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct fs_nvram_header_private_config {
    /// Magic value 0xFEDC.
    pub magic: u16,
    /// Format of the file.
    /// 0x0001 - raw data;
    pub format: u16,
    /// Address of the data.
    pub start: u32,
    /// Address right after the data.
    pub end: u32,
    /// Length of block.
    pub len: u32,
}

pub type fs_nvram_t = fs_nvram;

/// NVRAM filesystem.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fs_nvram {
    pub base: *mut c_uchar,
    pub len: size_t,
    pub addr: m_uint32_t,
    /// start address of the filesystem (for absolute addresses)
    pub flags: c_uint,
    /// filesystem flags
    pub shift: u_int,
    /// scale byte offsets
    pub padding: u_int,
    /// base padding value
    pub backup: size_t,
    /// start offset of the backup filesystem
    pub read_byte: Option<unsafe extern "C" fn(fs: *mut fs_nvram_t, offset: u_int) -> m_uint8_t>,
    pub write_byte: Option<unsafe extern "C" fn(fs: *mut fs_nvram_t, offset: u_int, val: m_uint8_t)>,
}

//=========================================================
// Auxiliary

/// Convert a 16 bit value from big endian to native.
unsafe fn be_to_native16(val: *mut u16) {
    *val = u16::from_be(*val);
}

/// Convert a 32 bit value from big endian to native.
unsafe fn be_to_native32(val: *mut u32) {
    *val = u32::from_be(*val);
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
unsafe fn native_to_be16(val: *mut u16) {
    *val = u16::to_be(*val);
}

/// Convert a 32 bit value from native to big endian.
unsafe fn native_to_be32(val: *mut u32) {
    *val = u32::to_be(*val);
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
unsafe fn uncompress_LZC(in_data: *const u8, in_len: size_t, out_data: *mut u8, out_len: size_t) -> c_int {
    const LZC_MAGIC_1: u8 = 0x1F;
    const LZC_MAGIC_2: u8 = 0x9D;
    const LZC_NUM_BITS_MASK: u8 = 0x1F;
    const LZC_BLOCK_MODE_MASK: u8 = 0x80;
    const LZC_NUM_BITS_MIN: size_t = 9;
    const LZC_NUM_BITS_MAX: size_t = 16;

    if in_len < 3 || (in_data.is_null() && in_len > 0) || (out_data.is_null() && out_len > 0) {
        return libc::EINVAL; // invalid argument
    }

    if *in_data.add(0) != LZC_MAGIC_1 || *in_data.add(1) != LZC_MAGIC_2 {
        return libc::ENOTSUP; // no magic
    }

    let maxbits: size_t = (*in_data.add(2) & LZC_NUM_BITS_MASK) as size_t;
    if !(LZC_NUM_BITS_MIN..=LZC_NUM_BITS_MAX).contains(&maxbits) {
        return libc::ENOTSUP; // maxbits not supported
    }

    let numItems: size_t = 1 << maxbits;
    let blockMode: bool = (*in_data.add(2) & LZC_BLOCK_MODE_MASK) != 0;

    let parents: *mut u16 = libc::malloc(numItems * size_of::<u16>()).cast::<_>();
    if parents.is_null() {
        return libc::ENOMEM; // out of memory
    }
    let suffixes: *mut u8 = libc::malloc(numItems * size_of::<u8>()).cast::<_>();
    if suffixes.is_null() {
        libc::free(parents.cast::<_>());
        return libc::ENOMEM; // out of memory
    }
    let stack: *mut u8 = libc::malloc(numItems * size_of::<u8>()).cast::<_>();
    if stack.is_null() {
        libc::free(parents.cast::<_>());
        libc::free(suffixes.cast::<_>());
        return libc::ENOMEM; // out of memory
    }

    let mut in_pos: size_t = 3;
    let mut out_pos: size_t = 0;
    let mut numBits: size_t = LZC_NUM_BITS_MIN;
    let mut head: size_t = if blockMode { 257 } else { 256 };

    let mut needPrev: bool = false;

    let mut bitPos: size_t = 0;
    let mut numBufBits: size_t = 0;

    let buf = [0_u8; LZC_NUM_BITS_MAX + 4];

    *parents.add(256) = 0;
    *suffixes.add(256) = 0;

    loop {
        if numBufBits == bitPos {
            let len: size_t = min(in_len - in_pos, numBits);
            libc::memcpy(buf.as_ptr().cast_mut().cast::<_>(), in_data.add(in_pos).cast::<_>(), len);
            numBufBits = len << 3;
            bitPos = 0;
            in_pos += len;
        }
        let bytePos: size_t = bitPos >> 3;
        let mut symbol: size_t = (buf[bytePos] as size_t) | ((buf[bytePos + 1] as size_t) << 8) | ((buf[bytePos + 2] as size_t) << 16);
        symbol >>= bitPos & 7;
        symbol &= (1 << numBits) - 1;
        bitPos += numBits;
        if bitPos > numBufBits {
            break;
        }
        if symbol >= head {
            libc::free(parents.cast::<_>());
            libc::free(suffixes.cast::<_>());
            libc::free(stack.cast::<_>());
            return libc::EIO; // invalid data
        }
        if blockMode && symbol == 256 {
            numBufBits = 0;
            bitPos = 0;
            numBits = LZC_NUM_BITS_MIN;
            head = 257;
            needPrev = false;
            continue;
        }
        let mut cur: size_t = symbol;
        let mut i: size_t = 0;
        while cur >= 256 {
            *stack.add(i) = *suffixes.add(cur);
            i += 1;
            cur = *parents.add(cur) as size_t;
        }
        *stack.add(i) = cur as u8;
        i += 1;
        if needPrev {
            *suffixes.add(head - 1) = cur as u8;
            if symbol == head - 1 {
                *stack.add(0) = cur as u8;
            }
        }
        loop {
            if out_pos < out_len {
                i -= 1;
                *out_data.add(out_pos) = *stack.add(i);
                out_pos += 1;
            } else {
                i = 0;
            }
            if i == 0 {
                break;
            }
        }
        if head < numItems {
            needPrev = true;
            *parents.add(head) = symbol as u16;
            head += 1;
            if head > (1 << numBits) && numBits < maxbits {
                numBufBits = 0;
                bitPos = 0;
                numBits += 1;
            }
        } else {
            needPrev = false;
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
unsafe fn fs_nvram_address_of(fs: *mut fs_nvram_t, offset: size_t) -> size_t {
    if ((*fs).flags & FS_NVRAM_FLAG_ADDR_ABSOLUTE) != 0 {
        (*fs).addr as size_t + offset
    } else {
        offset - 8
    }
}

/// Retuns filesystem offset of the specified address
unsafe fn fs_nvram_offset_of(fs: *mut fs_nvram_t, address: size_t) -> size_t {
    if ((*fs).flags & FS_NVRAM_FLAG_ADDR_ABSOLUTE) != 0 {
        address - (*fs).addr as size_t
    } else {
        address + 8
    }
}

/// Retuns padding at the specified offset
unsafe fn fs_nvram_padding_at(fs: *mut fs_nvram_t, offset: size_t) -> size_t {
    let mut padding: size_t = 0;

    if offset % 4 != 0 {
        padding = (*fs).padding as size_t - offset % 4;
    }

    padding
}

/// Read a 16-bit value from NVRAM.
unsafe fn fs_nvram_read16(fs: *mut fs_nvram_t, offset: size_t) -> u16 {
    let mut val: u16;
    val = ((*fs).read_byte.unwrap()(fs, offset as c_uint) as u16) << 8;
    val |= (*fs).read_byte.unwrap()(fs, (offset + 1) as c_uint) as u16;
    val
}

/// Write a 16-bit value to NVRAM.
unsafe fn fs_nvram_write16(fs: *mut fs_nvram_t, offset: size_t, val: u16) {
    (*fs).write_byte.unwrap()(fs, offset as c_uint, (val >> 8) as u8);
    (*fs).write_byte.unwrap()(fs, (offset + 1) as c_uint, (val & 0xFF) as u8);
}

/// Read a 32-bit value from NVRAM.
unsafe fn fs_nvram_read32(fs: *mut fs_nvram_t, offset: size_t) -> u32 {
    let mut val: u32;
    val = ((*fs).read_byte.unwrap()(fs, offset as c_uint) as u32) << 24;
    val |= ((*fs).read_byte.unwrap()(fs, (offset + 1) as c_uint) as u32) << 16;
    val |= ((*fs).read_byte.unwrap()(fs, (offset + 2) as c_uint) as u32) << 8;
    val |= (*fs).read_byte.unwrap()(fs, (offset + 3) as c_uint) as u32;
    val
}

/// Read a buffer from NVRAM.
unsafe fn fs_nvram_memcpy_from(fs: *mut fs_nvram_t, offset: size_t, mut data: *mut u8, len: size_t) {
    for i in 0..len {
        *data = (*fs).read_byte.unwrap()(fs, (offset + i) as c_uint);
        data = data.add(1);
    }
}

/// Write a buffer to NVRAM.
unsafe fn fs_nvram_memcpy_to(fs: *mut fs_nvram_t, offset: size_t, mut data: *const u8, len: size_t) {
    for i in 0..len {
        (*fs).write_byte.unwrap()(fs, (offset + i) as c_uint, *data);
        data = data.add(1);
    }
}

/// Clear section of NVRAM.
unsafe fn fs_nvram_clear(fs: *mut fs_nvram_t, offset: size_t, len: size_t) {
    for i in 0..len {
        (*fs).write_byte.unwrap()(fs, (offset + i) as c_uint, 0);
    }
}

/// Update the filesystem checksum.
unsafe fn fs_nvram_update_checksum(fs: *mut fs_nvram_t) {
    let mut sum: u32 = 0;

    fs_nvram_write16(fs, size_of::<fs_nvram_header>() + offset_of!(fs_nvram_header_startup_config, checksum), 0x0000);

    let mut offset: size_t = size_of::<fs_nvram_header>();
    let mut count: size_t = (*fs).len - offset;
    while count > 1 {
        sum += fs_nvram_read16(fs, offset) as u32;
        offset += 2;
        count -= size_of::<u16>();
    }

    if count > 0 {
        sum += (((*fs).read_byte.unwrap()(fs, offset as c_uint)) as u32) << 8;
    }

    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    sum = !sum;

    fs_nvram_write16(fs, size_of::<fs_nvram_header>() + offset_of!(fs_nvram_header_startup_config, checksum), sum as u16);
}

/// Read data from NVRAM.
unsafe fn fs_nvram_read_data(fs: *mut fs_nvram_t, offset: size_t, len: size_t) -> *mut u8 {
    let data: *mut u8 = libc::malloc(len + 1).cast::<_>();
    if data.is_null() {
        return null_mut(); // out of memory
    }

    fs_nvram_memcpy_from(fs, offset, data, len);
    *data.add(len) = 0;

    data
}

/// Returns the normal offset of the NVRAM filesystem with backup.
unsafe fn fs_nvram_offset1_with_backup(fs: *mut fs_nvram_t, offset: size_t) -> size_t {
    if offset < FS_NVRAM_NORMAL_FILESYSTEM_BLOCK1 {
        offset << (*fs).shift
    } else {
        (FS_NVRAM_BACKUP_FILESYSTEM_BLOCK1 + offset) << (*fs).shift
    }
}

/// Returns the backup offset of the NVRAM filesystem with backup.
unsafe fn fs_nvram_offset2_with_backup(fs: *mut fs_nvram_t, offset: size_t) -> size_t {
    if offset < FS_NVRAM_BACKUP_FILESYSTEM_BLOCK1 {
        ((*fs).backup + offset) << (*fs).shift
    } else {
        ((*fs).len + offset as size_t) << (*fs).shift
    }
}

//=========================================================
// Public

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
    let mut off = size_of::<fs_nvram_header>();
    let mut startup_head: fs_nvram_header_startup_config = zeroed::<_>();
    fs_nvram_memcpy_from(fs, off, addr_of_mut!(startup_head).cast::<_>(), size_of::<fs_nvram_header_startup_config>());
    be_to_native_header_startup(addr_of_mut!(startup_head));
    if FS_NVRAM_MAGIC_STARTUP_CONFIG != startup_head.magic {
        return 0; // done, no startup-config and no private-config
    }

    unsafe fn reset(startup_config: *mut *mut u8, startup_len: *mut size_t, private_config: *mut *mut u8, private_len: *mut size_t) {
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

    off = fs_nvram_offset_of(fs, (startup_head.start + startup_head.len) as size_t);
    off += fs_nvram_padding_at(fs, off);

    if off + size_of::<fs_nvram_header_private_config>() > (*fs).len {
        reset(startup_config, startup_len, private_config, private_len);
        return libc::ENOMEM; // out of memory
    }

    let mut private_head: fs_nvram_header_private_config = zeroed::<_>();
    fs_nvram_memcpy_from(fs, off, addr_of_mut!(private_head).cast::<_>(), size_of::<fs_nvram_header_private_config>());
    be_to_native_header_private(addr_of_mut!(private_head));

    // read startup-config
    if FS_NVRAM_FORMAT_RAW == startup_head.format {
        if !startup_config.is_null() {
            off = fs_nvram_offset_of(fs, startup_head.start as size_t);
            *startup_config = fs_nvram_read_data(fs, off, startup_head.len as size_t);
            if (*startup_config).is_null() {
                reset(startup_config, startup_len, private_config, private_len);
                return libc::ENOMEM; // out of memory
            }
        }

        if !startup_len.is_null() {
            *startup_len = startup_head.len as size_t;
        }
    } else if FS_NVRAM_FORMAT_LZC == startup_head.format {
        if !startup_config.is_null() {
            off = fs_nvram_offset_of(fs, startup_head.start as size_t);
            *startup_config = libc::malloc((startup_head.uncompressed_len + 1) as size_t).cast::<_>();
            if (*startup_config).is_null() {
                reset(startup_config, startup_len, private_config, private_len);
                return libc::ENOMEM; // out of memory
            }

            let buf: *mut u8 = fs_nvram_read_data(fs, off, startup_head.len as size_t);
            if buf.is_null() {
                reset(startup_config, startup_len, private_config, private_len);
                return libc::ENOMEM; // out of memory
            }

            let err = uncompress_LZC(buf, startup_head.len as size_t, *startup_config, startup_head.uncompressed_len as size_t);
            if err != 0 {
                libc::free(buf.cast::<_>());
                reset(startup_config, startup_len, private_config, private_len);
                return err;
            }

            *(*startup_config).add(startup_head.uncompressed_len as size_t) = 0;
            libc::free(buf.cast::<_>());
        }

        if !startup_len.is_null() {
            *startup_len = startup_head.uncompressed_len as size_t;
        }
    } else {
        reset(startup_config, startup_len, private_config, private_len);
        return libc::ENOTSUP; // unsupported format
    }

    // read private-config
    if fs_nvram_offset_of(fs, (private_head.start + private_head.len) as size_t) > (*fs).len || FS_NVRAM_MAGIC_PRIVATE_CONFIG != private_head.magic {
        return 0; // done, no private-config
    }

    if FS_NVRAM_FORMAT_RAW == private_head.format {
        if !private_config.is_null() {
            off = fs_nvram_offset_of(fs, private_head.start as size_t);
            *private_config = fs_nvram_read_data(fs, off, private_head.len as size_t);
            if (*private_config).is_null() {
                reset(startup_config, startup_len, private_config, private_len);
                return libc::ENOMEM; // out of memory
            }
        }

        if !private_len.is_null() {
            *private_len = private_head.len as size_t;
        }
    } else {
        reset(startup_config, startup_len, private_config, private_len);
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
    let padding: size_t = fs_nvram_padding_at(fs, len);
    len += padding + size_of::<fs_nvram_header_private_config>() + private_len;
    if (*fs).len < len {
        return libc::ENOSPC; // not enough space
    }

    // old length
    len = size_of::<fs_nvram_header>();
    if FS_NVRAM_MAGIC_STARTUP_CONFIG == fs_nvram_read16(fs, len + offset_of!(fs_nvram_header_startup_config, magic)) {
        len += fs_nvram_read32(fs, len + offset_of!(fs_nvram_header_startup_config, len)) as size_t;
        if len % 4 != 0 {
            len += 8 - len % 4;
        }

        if FS_NVRAM_MAGIC_PRIVATE_CONFIG == fs_nvram_read16(fs, len + offset_of!(fs_nvram_header_private_config, magic)) {
            len += fs_nvram_read32(fs, len + offset_of!(fs_nvram_header_private_config, len)) as size_t;
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
    startup_head.start = fs_nvram_address_of(fs, size_of::<fs_nvram_header>() + size_of::<fs_nvram_header_startup_config>()) as u32;
    startup_head.end = startup_head.start + startup_len as u32;
    startup_head.len = startup_len as u32;

    let mut private_head: fs_nvram_header_private_config = zeroed::<_>();
    libc::memset(addr_of_mut!(private_head).cast::<_>(), 0, size_of::<fs_nvram_header_private_config>());
    private_head.magic = FS_NVRAM_MAGIC_PRIVATE_CONFIG;
    private_head.format = FS_NVRAM_FORMAT_RAW;
    private_head.start = startup_head.end + (padding + size_of::<fs_nvram_header_private_config>()) as u32;
    private_head.end = private_head.start + private_len as u32;
    private_head.len = private_len as u32;

    native_to_be_header_startup(addr_of_mut!(startup_head));
    native_to_be_header_private(addr_of_mut!(private_head));

    // write data
    let mut off: size_t = size_of::<fs_nvram_header>();

    fs_nvram_memcpy_to(fs, off, addr_of_mut!(startup_head).cast::<_>(), size_of::<fs_nvram_header_startup_config>());
    off += size_of::<fs_nvram_header_startup_config>();
    fs_nvram_memcpy_to(fs, off, startup_config, startup_len);
    off += startup_len;

    fs_nvram_clear(fs, off, padding);
    off += padding;

    fs_nvram_memcpy_to(fs, off, addr_of_mut!(private_head).cast::<_>(), size_of::<fs_nvram_header_private_config>());
    off += size_of::<fs_nvram_header_private_config>();
    fs_nvram_memcpy_to(fs, off, private_config, private_len);
    off += private_len;

    if off < len {
        fs_nvram_clear(fs, off, len - off);
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
            let b1: u8 = *(*fs).base.add(fs_nvram_offset1_with_backup(fs, offset));
            let b2: u8 = *(*fs).base.add(fs_nvram_offset2_with_backup(fs, offset));
            if b1 != b2 {
                return FS_NVRAM_ERR_BACKUP_MISSMATCH; // data is corrupted? length is wrong?
            }
        }
    }

    if (what & FS_NVRAM_VERIFY_CONFIG) != 0 {
        let mut startup_head: fs_nvram_header_startup_config = zeroed::<_>();
        let mut private_head: fs_nvram_header_private_config = zeroed::<_>();

        let mut offset = size_of::<fs_nvram_header>();
        fs_nvram_memcpy_from(fs, offset, addr_of_mut!(startup_head).cast::<_>(), size_of::<fs_nvram_header_startup_config>());
        be_to_native_header_startup(addr_of_mut!(startup_head));
        if FS_NVRAM_MAGIC_STARTUP_CONFIG == startup_head.magic {
            if startup_head.end != startup_head.start + startup_head.len || startup_head.len as size_t > (*fs).len {
                return FS_NVRAM_ERR_INVALID_ADDRESS; // data is corrupted?
            }
            if startup_head.start < (*fs).addr || startup_head.end as size_t > (*fs).addr as size_t + (*fs).len {
                return FS_NVRAM_ERR_INVALID_ADDRESS; // fs.addr has the wrong value?
            }

            offset = fs_nvram_offset_of(fs, startup_head.end as size_t);
            offset += fs_nvram_padding_at(fs, offset);
            if (*fs).len < offset + size_of::<fs_nvram_header_private_config>() {
                return FS_NVRAM_ERR_INVALID_ADDRESS; // data is corrupted?
            }

            fs_nvram_memcpy_from(fs, offset, addr_of_mut!(private_head).cast::<_>(), size_of::<fs_nvram_header_private_config>());
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
