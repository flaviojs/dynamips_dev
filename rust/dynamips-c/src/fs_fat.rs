//! FAT filesystem.
//!
//! Based on http://www.win.tue.nl/~aeb/linux/fs/fat/fat-1.html
//!
//! Copyright (c) 2014 Flávio J. Saraiva <flaviojs2005@gmail.com>

use crate::_private::*;
use crate::dynamips_common::*;

const FS_FAT_SECTOR_SIZE: u32 = 512;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fat16_data {
    pub volume_name: *const c_char,
    pub volume_sectors: m_uint32_t,
    pub reserved_sectors: m_uint16_t,
    pub root_entry_count: m_uint16_t,
    pub fat_sectors: m_uint16_t,
    pub sects_per_track: m_uint16_t,
    pub heads: m_uint16_t,
    pub sects_per_cluster: m_uint8_t,
    pub nr_fats: m_uint8_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sec_per_clus_table {
    pub sectors: m_uint32_t,
    pub sects_per_cluster: m_uint8_t,
}
impl sec_per_clus_table {
    pub const fn new(sectors: m_uint32_t, sects_per_cluster: m_uint8_t) -> Self {
        Self { sectors, sects_per_cluster }
    }
}

static cluster_size_table16: [sec_per_clus_table; 8] = [
    sec_per_clus_table::new(32680, 2),     // 16MB - 1K
    sec_per_clus_table::new(262144, 4),    // 128MB - 2K
    sec_per_clus_table::new(524288, 8),    // 256MB - 4K
    sec_per_clus_table::new(1048576, 16),  // 512MB - 8K
    sec_per_clus_table::new(2097152, 32),  // 1GB - 16K
    sec_per_clus_table::new(4194304, 64),  // 2GB - 32K
    sec_per_clus_table::new(8388608, 128), // 2GB - 64K (not supported on some systems)
    sec_per_clus_table::new(0, 0),         // done
];

#[inline]
unsafe fn set_u32(p: *mut m_uint8_t, i: size_t, v: m_uint32_t) {
    *p.add(i) = (v & 0xFF) as m_uint8_t;
    *p.add(i + 1) = ((v >> 8) & 0xFF) as m_uint8_t;
    *p.add(i + 2) = ((v >> 16) & 0xFF) as m_uint8_t;
    *p.add(i + 3) = ((v >> 24) & 0xFF) as m_uint8_t;
}

#[inline]
unsafe fn set_u16(p: *mut m_uint8_t, i: size_t, v: m_uint16_t) {
    *p.add(i) = (v & 0xFF) as m_uint8_t;
    *p.add(i + 1) = ((v >> 8) & 0xFF) as m_uint8_t;
}

unsafe fn boot16(sector: *mut m_uint8_t, fat16: *mut fat16_data) {
    libc::memset(sector.cast::<_>(), 0x00, FS_FAT_SECTOR_SIZE as usize);

    // start of boot program
    *sector.add(0x0) = 0xEB; // jmp 0x3E
    *sector.add(0x1) = 0x3C;
    *sector.add(0x2) = 0x90; // nop

    // OEM string
    *sector.add(0x3) = b'D';
    *sector.add(0x4) = b'Y';
    *sector.add(0x5) = b'N';
    *sector.add(0x6) = b'A';
    *sector.add(0x7) = b'M';
    *sector.add(0x8) = b'I';
    *sector.add(0x9) = b'P';
    *sector.add(0xA) = b'S';

    // Bytes per sector
    set_u16(sector, 0xB, FS_FAT_SECTOR_SIZE as m_uint16_t);

    // Sectors per cluster
    *sector.add(0xD) = (*fat16).sects_per_cluster;

    // Reserved Sectors
    set_u16(sector, 0xE, (*fat16).reserved_sectors);

    // Number of FATS
    *sector.add(0x10) = (*fat16).nr_fats;

    // Max entries in root dir (FAT16 only)
    set_u16(sector, 0x11, (*fat16).root_entry_count);

    // [FAT16] Total sectors (use FAT32 count instead)
    set_u16(sector, 0x13, 0x0000);

    // Media type (Fixed Disk)
    *sector.add(0x15) = 0xF8;

    // FAT16 Bootstrap Details

    // Count of sectors used by the FAT table (FAT16 only)
    set_u16(sector, 0x16, (*fat16).fat_sectors);

    // Sectors per track
    set_u16(sector, 0x18, (*fat16).sects_per_track);

    // Heads
    set_u16(sector, 0x1A, (*fat16).heads);

    // Hidden sectors
    set_u16(sector, 0x1C, 0x0000);

    // Total sectors for this volume
    set_u32(sector, 0x20, (*fat16).volume_sectors);

    // Drive number (1st Hard Disk)
    *sector.add(0x24) = 0x80;

    // Reserved
    *sector.add(0x25) = 0x00;

    // Boot signature
    *sector.add(0x26) = 0x29;

    // Volume ID
    *sector.add(0x27) = (libc::rand() & 0xFF) as m_uint8_t;
    *sector.add(0x28) = (libc::rand() & 0xFF) as m_uint8_t;
    *sector.add(0x29) = (libc::rand() & 0xFF) as m_uint8_t;
    *sector.add(0x2A) = (libc::rand() & 0xFF) as m_uint8_t;

    // Volume name
    for i in 0..11 {
        if *(*fat16).volume_name.add(i) == 0 {
            for i in i..11 {
                *sector.add(i + 0x2B) = b' ';
            }
            break;
        }
        *sector.add(i + 0x2B) = *(*fat16).volume_name.add(i) as m_uint8_t;
    }

    // File sys type
    *sector.add(0x36) = b'F';
    *sector.add(0x37) = b'A';
    *sector.add(0x38) = b'T';
    *sector.add(0x39) = b'1';
    *sector.add(0x3A) = b'6';
    *sector.add(0x3B) = b' ';
    *sector.add(0x3C) = b' ';
    *sector.add(0x3D) = b' ';

    // boot program (empty)

    // Signature
    *sector.add(0x1FE) = 0x55;
    *sector.add(0x1FF) = 0xAA;
}

unsafe fn fat16_first(sector: *mut m_uint8_t, _fat16: *mut fat16_data) {
    libc::memset(sector.cast::<_>(), 0x00, FS_FAT_SECTOR_SIZE as size_t);

    // Initialise default allocate / reserved clusters
    set_u16(sector, 0x0, 0xFFF8);
    set_u16(sector, 0x2, 0xFFFF);
}

unsafe fn fat16_empty(sector: *mut m_uint8_t, _fat16: *mut fat16_data) {
    libc::memset(sector.cast::<_>(), 0x00, FS_FAT_SECTOR_SIZE as size_t);
}

unsafe fn write_sector(fd: c_int, lba: m_uint32_t, sector: *mut m_uint8_t) -> c_int {
    c_errno_set(0);
    let offset: libc::off_t = lba as libc::off_t * FS_FAT_SECTOR_SIZE as libc::off_t;
    if libc::lseek(fd, offset, libc::SEEK_SET) != offset {
        libc::perror(cstr!("write_sector(fs_fat): lseek"));
        return -1;
    }

    if libc::write(fd, sector.cast::<_>(), FS_FAT_SECTOR_SIZE as size_t) != FS_FAT_SECTOR_SIZE as ssize_t {
        libc::perror(cstr!("write_sector(fs_fat): write"));
        return -1;
    }

    0
}

/// Format partition as FAT16.
#[no_mangle]
pub unsafe extern "C" fn fs_fat_format16(fd: c_int, begin_lba: m_uint32_t, nr_sectors: m_uint32_t, sects_per_track: m_uint16_t, heads: m_uint16_t, mut volume_name: *const c_char) -> c_int {
    let sector: [m_uint8_t; FS_FAT_SECTOR_SIZE as usize] = [0; FS_FAT_SECTOR_SIZE as usize];
    let sector = sector.as_ptr().cast_mut();
    let name: [c_char; 12] = [0; 12];

    if volume_name.is_null() {
        libc::snprintf(name.as_ptr().cast_mut(), 12, cstr!("DISK%dMB"), (nr_sectors / (1048576 / FS_FAT_SECTOR_SIZE)) as c_int);
        volume_name = name.as_ptr();
    }

    // prepare FAT16
    let mut data: fat16_data = zeroed::<_>();
    let fat16: *mut fat16_data = addr_of_mut!(data);
    libc::memset(fat16.cast::<_>(), 0x00, size_of::<fat16_data>());
    (*fat16).volume_name = volume_name;
    (*fat16).volume_sectors = nr_sectors;
    (*fat16).sects_per_track = sects_per_track;
    (*fat16).heads = heads;
    #[allow(clippy::needless_range_loop)]
    for i in 0.. {
        if cluster_size_table16[i].sectors == 0 {
            return -1;
        }
        if nr_sectors <= cluster_size_table16[i].sectors {
            (*fat16).sects_per_cluster = cluster_size_table16[i].sects_per_cluster;
            break;
        }
    }
    let total_clusters: m_uint32_t = ((*fat16).volume_sectors / (*fat16).sects_per_cluster as m_uint32_t) + 1;
    (*fat16).fat_sectors = ((total_clusters / (FS_FAT_SECTOR_SIZE / 2)) + 1) as m_uint16_t;
    (*fat16).reserved_sectors = 1;
    (*fat16).nr_fats = 2;
    (*fat16).root_entry_count = 512;

    // Boot sector
    boot16(sector, fat16);
    if write_sector(fd, begin_lba, sector) < 0 {
        return -1;
    }

    // FAT sectors
    for ifat in 0..(*fat16).nr_fats as m_uint32_t {
        let fat_lba: m_uint32_t = begin_lba + (*fat16).reserved_sectors as m_uint32_t + ifat * (*fat16).fat_sectors as m_uint32_t;
        fat16_first(sector, fat16);
        if write_sector(fd, fat_lba, sector) < 0 {
            return -1;
        }

        fat16_empty(sector, fat16);
        for isec in 1..(*fat16).fat_sectors as m_uint32_t {
            if write_sector(fd, isec + fat_lba, sector) < 0 {
                return -1;
            }
        }
    }

    // Root directory
    let rootdir_lba: u32 = begin_lba + (*fat16).reserved_sectors as m_uint32_t + ((*fat16).nr_fats as m_uint32_t * (*fat16).fat_sectors as m_uint32_t);
    let rootdir_sectors: u32 = (((*fat16).root_entry_count as m_uint32_t * 32) + (FS_FAT_SECTOR_SIZE - 1)) / FS_FAT_SECTOR_SIZE;
    fat16_empty(sector, fat16);
    for isec in 0..rootdir_sectors {
        if write_sector(fd, rootdir_lba + isec, sector) < 0 {
            return -1;
        }
    }

    0
}
