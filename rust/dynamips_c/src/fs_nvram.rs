//! Cisco NVRAM filesystem.
//!
//! Format was inferred by analysing the NVRAM data after changing/erasing stuff.
//! All data is big endian.
//!
//! Based on the platforms c1700/c2600/c2692/c3600/c3725/c3745/c7200/c6msfc1.

use crate::dynamips_common::*;
use crate::prelude::*;

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

//=========================================================
// Private

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

/// Read a buffer from NVRAM.
unsafe fn fs_nvram_memcpy_from(fs: *mut fs_nvram_t, offset: size_t, mut data: *mut u8, len: size_t) {
    for i in 0..len {
        *data = (*fs).read_byte.unwrap()(fs, (offset + i) as c_uint);
        data = data.add(1);
    }
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
