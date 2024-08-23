//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot.  All rights reserved.
//!
//! PCMCIA ATA Flash emulation.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::fs_fat::*;
use crate::fs_mbr::*;
use crate::memory::*;
use crate::utils::*;
use crate::vm::*;

const DEBUG_ACCESS: c_int = 0;
const DEBUG_ATA: c_int = 0;
const DEBUG_READ: c_int = 0;
const DEBUG_WRITE: c_int = 0;

/// Default disk parameters: 4 heads, 32 sectors per track
const DISK_NR_HEADS: u_int = 4;
const DISK_SECTS_PER_TRACK: u_int = 32;

/// Size (in bytes) of a sector
const SECTOR_SIZE: usize = 512;

/// ATA commands
const ATA_CMD_NOP: m_uint8_t = 0x00;
const ATA_CMD_READ_SECTOR: m_uint8_t = 0x20;
const ATA_CMD_WRITE_SECTOR: m_uint8_t = 0x30;
const ATA_CMD_IDENT_DEVICE: m_uint8_t = 0xEC;

/// ATA status
const ATA_STATUS_BUSY: m_uint8_t = 0x80; // Controller busy
const ATA_STATUS_RDY: m_uint8_t = 0x40; // Device ready
const ATA_STATUS_DWF: m_uint8_t = 0x20; // Write fault
const ATA_STATUS_DSC: m_uint8_t = 0x10; // Device ready
const ATA_STATUS_DRQ: m_uint8_t = 0x08; // Data Request
const ATA_STATUS_CORR: m_uint8_t = 0x04; // Correctable error
const ATA_STATUS_IDX: m_uint8_t = 0x02; // Always 0
const ATA_STATUS_ERR: m_uint8_t = 0x01; // Error

/// ATA Drive/Head register
const ATA_DH_LBA: m_uint8_t = 0x40; // LBA Mode

/// Card Information Structure
#[rustfmt::skip]
static mut cis_table: [m_uint8_t; 189] = [
    // CISTPL_DEVICE
    0x01, 0x03,
        0xd9, // DSPEED_250NS | WPS | DTYPE_FUNCSPEC
        0x01, // 2 KBytes(Units)/64 KBytes(Max Size)
        0xff, // 0xFF
    // CISTPL_DEVICE_OC
    0x1c, 0x04,
        0x03, // MWAIT | 3.3 volt VCC operation
        0xd9, // DSPEED_250NS | WPS | DTYPE_FUNCSPEC
        0x01, // 2 KBytes(Units)/64 KBytes(Max Size)
        0xff, // 0xFF
    // CISTPL_JEDEC_C
    0x18, 0x02,
        0xdf, // PCMCIA
        0x01, // 0x01
    // CISTPL_MANFID
    0x20, 0x04,
        0x34, 0x12, // 0x1234 ???
        0x00, 0x02, // 0x0200
    // CISTPL_VERS_1
    0x15, 0x2b,
        0x04, 0x01, // PCMCIA 2.0/2.1 / JEIDA 4.1/4.2
        0x44, 0x79, 0x6e, 0x61, 0x6d, 0x69, 0x70, 0x73,
        0x20, 0x41, 0x54, 0x41, 0x20, 0x46, 0x6c, 0x61,
        0x73, 0x68, 0x20, 0x43, 0x61, 0x72, 0x64, 0x20,
        0x20, 0x00, // "Dynamips ATA Flash Card  "
        0x44, 0x59, 0x4e, 0x41, 0x30, 0x20, 0x20, 0x00, // "DYNA0  "
        0x44, 0x59, 0x4e, 0x41, 0x30, 0x00, // "DYNA0"
        0xff, // 0xFF
    // CISTPL_FUNCID
    0x21, 0x02,
        0x04, // Fixed Disk
        0x01, // Power-On Self Test
    // CISTPL_FUNCE
    0x22, 0x02,
        0x01, // Disk Device Interface tuple
        0x01, // PC Card-ATA Interface
    // CISTPL_FUNCE:
    0x22, 0x03,
        0x02, // Basic PC Card ATA Interface tuple
        0x04, // S(Silicon Device)
        0x5f, // P0(Sleep) | P1(Standy) | P2(Idle) | P3(Auto) | N(3F7/377 Register Inhibit Available) | I(IOIS16# on Twin Card)
    // CISTPL_CONFIG
    0x1a, 0x05,
        0x01, // TPCC_RASZ=2, TPCC_RMSZ=1, TPCC_RFSZ=0
        0x03, // TPCC_LAST=3
        0x00, 0x02, // TPCC_RADR(Configuration Register Base Address)=0x0200
        0x0f, // TPCC_RMSK(Configuration Register Presence Mask Field)=0x0F
    // CISTPL_CFTABLE_ENTRY
    0x1b, 0x0b,
        0xc0, // Index=0 | Default | Interface
        0x40, // Memory | READY Active
        0xa1, // VCC power-description-structure only | Single 2-byte length specified | Misc
        0x27, // Nom V | Min V | Max V | Peak I
        0x55, // Nom V=5V
        0x4d, // Min V=4.5V
        0x5d, // Max V=5V
        0x75, // Peak I=80mA
        0x08, 0x00, // Card Address=0 | Host Address=0x0008 * 256 bytes
        0x21, // Max Twin Cards=1 | Power Down
    // CISTPL_CFTABLE_ENTRY
    0x1b, 0x06,
        0x00, // Index=0
        0x01, // VCC power-description-structure only
        0x21, // Nom V | Peak I
        0xb5, 0x1e, // Nom V=3.30V
        0x4d, // Peak I=45mA
    // CISTPL_CFTABLE_ENTRY
    0x1b, 0x0d,
        0xc1, // Index=1 | Default | Interface
        0x41, // I/O and Memory | READY Active
        0x99, // VCC power-description-structure only | IO Space | IRQ | Misc
        0x27, // Nom V | Min V | Max V | Peak I
        0x55, // Nom V=5V
        0x4d, // Min V=4.5V
        0x5d, // Max V=5V
        0x75, // Peak I=80mA
        0x64, // IOAddrLines=4 | All registers are accessible by both 8-bit or 16-bit accesses
        0xf0, 0xff, 0xff, // Mask | Level | Pulse | Share | IRQ0..IRQ15
        0x21, // Max Twin Cards=1 | Power Down
    // CISTPL_CFTABLE_ENTRY
    0x1b, 0x06,
        0x01, // Index=1
        0x01, // VCC power-description-structure only
        0x21, // Nom V | Peak I
        0xb5, 0x1e, // Nom V=3.30V
        0x4d, // Peak I=45mA
    // CISTPL_CFTABLE_ENTRY
    0x1b, 0x12,
        0xc2, // Index=2 | Default | Interface
        0x41, // I/O and Memory | READY Active
        0x99, // VCC power-description-structure only | IO Space | IRQ | Misc
        0x27, // Nom V | Min V | Max V | Peak I
        0x55, // Nom V=5V
        0x4d, // Min V=4.5V
        0x5d, // Max V=5V
        0x75, // Peak I=80mA
        0xea, // IOAddrLines=10 | All registers are accessible by both 8-bit or 16-bit accesses | Range
        0x61, // Number of I/O Address Ranges=2 | Size of Address=2 | Size of Length=1
        0xf0, 0x01, 0x07, // Address=0x1F0, Length=8
        0xf6, 0x03, 0x01, // Address=0x3F6, Length=2
        0xee, // IRQ14 | Level | Pulse | Share
        0x21, // Max Twin Cards=1 | Power Down
    // CISTPL_CFTABLE_ENTRY
    0x1b, 0x06,
        0x02, // Index=2
        0x01, // VCC power-description-structure only
        0x21, // Nom V | Peak I
        0xb5, 0x1e, // Nom V=3.30V
        0x4d, // Peak I=45mA
    // CISTPL_CFTABLE_ENTRY
    0x1b, 0x12,
        0xc3, // Index=3 | Default | Interface
        0x41, // I/O and Memory | READY Active
        0x99, // VCC power-description-structure only | IO Space | IRQ | Misc
        0x27, // Nom V | Min V | Max V | Peak I
        0x55, // Nom V=5V
        0x4d, // Min V=4.5V
        0x5d, // Max V=5V
        0x75, // Peak I=80mA
        0xea, // IOAddrLines=10 | All registers are accessible by both 8-bit or 16-bit accesses | Range
        0x61, // Number of I/O Address Ranges=2 | Size of Address=2 | Size of Length=1
        0x70, 0x01, 0x07, // Address=0x170, Length=8
        0x76, 0x03, 0x01, // Address=0x376, Length=2
        0xee, // IRQ14 | Level | Pulse | Share
        0x21, // Max Twin Cards=1 | Power Down
    // CISTPL_CFTABLE_ENTRY
    0x1b, 0x06,
        0x03, // Index=3
        0x01, // VCC power-description-structure only
        0x21, // Nom V | Peak I
        0xb5, 0x1e, // Nom V=3.30V
        0x4d, // Peak I=45mA
    // CISTPL_NO_LINK
    0x14, 0x00,
    // CISTPL_END
    0xff,
];

/// PCMCIA private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pcmcia_disk_data {
    pub vm: *mut vm_instance_t,
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub filename: *mut c_char,
    pub fd: c_int,

    /// Disk parameters (C/H/S)
    pub nr_heads: u_int,
    pub nr_cylinders: u_int,
    pub sects_per_track: u_int,

    /// Current ATA command and CHS info
    pub ata_cmd: m_uint8_t,
    pub ata_cmd_in_progress: m_uint8_t,
    pub ata_status: m_uint8_t,
    pub cyl_low: m_uint8_t,
    pub cyl_high: m_uint8_t,
    pub head: m_uint8_t,
    pub sect_no: m_uint8_t,
    pub sect_count: m_uint8_t,

    /// Current sector
    pub sect_pos: m_uint32_t,

    /// Remaining sectors to read or write
    pub sect_remaining: u_int,

    /// Callback function when data buffer is validated
    pub ata_cmd_callback: Option<unsafe extern "C" fn(_: *mut pcmcia_disk_data)>,

    /// Data buffer
    pub data_offset: m_uint32_t,
    pub data_pos: u_int,
    pub data_buffer: [m_uint8_t; SECTOR_SIZE],
}

/// Convert a CHS reference to an LBA reference
#[inline]
unsafe fn chs_to_lba(d: *mut pcmcia_disk_data, cyl: u_int, head: u_int, sect: u_int) -> m_uint32_t {
    (((cyl * (*d).nr_heads) + head) * (*d).sects_per_track) + sect - 1
}

/// Convert a LBA reference to a CHS reference
#[inline]
unsafe fn lba_to_chs(d: *mut pcmcia_disk_data, lba: m_uint32_t, cyl: *mut u_int, head: *mut u_int, sect: *mut u_int) {
    *cyl = lba / ((*d).sects_per_track * (*d).nr_heads);
    *head = (lba / (*d).sects_per_track) % (*d).nr_heads;
    *sect = (lba % (*d).sects_per_track) + 1;
}

/// Format disk with a single FAT16 partition
unsafe fn disk_format(d: *mut pcmcia_disk_data) -> c_int {
    let mut mbr: mbr_data = zeroed::<_>();
    let mut cyl: u_int = 0;
    let mut head: u_int = 1;
    let mut sect: u_int = 1;

    // Master Boot Record
    libc::memset(addr_of_mut!(mbr).cast::<_>(), 0, size_of::<mbr_data>());
    mbr.signature[0] = MBR_SIGNATURE_0;
    mbr.signature[1] = MBR_SIGNATURE_1;
    let part: *mut mbr_partition = addr_of_mut!(mbr.partition[0]);
    (*part).bootable = 0;
    (*part).r#type = MBR_PARTITION_TYPE_FAT16;
    (*part).lba = chs_to_lba(d, 0, 1, 1);
    (*part).nr_sectors = (*d).nr_heads * (*d).nr_cylinders * (*d).sects_per_track - (*part).lba;
    lba_to_chs(d, (*part).lba + (*part).nr_sectors - 1, addr_of_mut!(cyl), addr_of_mut!(head), addr_of_mut!(sect));
    mbr_set_chs((*part).first_chs.as_c_mut(), 0, 1, 1);
    mbr_set_chs((*part).last_chs.as_c_mut(), cyl as m_uint16_t, head as m_uint8_t, sect as m_uint8_t);

    if mbr_write_fd((*d).fd, addr_of_mut!(mbr)) < 0 {
        return -1;
    }

    // FAT16 partition
    if fs_fat_format16((*d).fd, (*part).lba, (*part).nr_sectors, (*d).sects_per_track as m_uint16_t, (*d).nr_heads as m_uint16_t, (*d).vm_obj.name) != 0 {
        return -1;
    }

    0
}

/// Create the virtual disk
unsafe fn disk_create(d: *mut pcmcia_disk_data) -> c_int {
    (*d).fd = libc::open((*d).filename, libc::O_CREAT | libc::O_EXCL | libc::O_RDWR, 0o600);
    if (*d).fd < 0 {
        // already exists?
        (*d).fd = libc::open((*d).filename, libc::O_CREAT | libc::O_RDWR, 0o600);
        if (*d).fd < 0 {
            libc::perror(cstr!("disk_create: open"));
            return -1;
        }
    } else {
        // new disk
        if disk_format(d) != 0 {
            return -1;
        }
    }

    let disk_len: libc::off_t = (*d).nr_heads as libc::off_t * (*d).nr_cylinders as libc::off_t * (*d).sects_per_track as libc::off_t * SECTOR_SIZE as libc::off_t;
    libc::ftruncate((*d).fd, disk_len);
    0
}

/// Read a sector from disk file
unsafe fn disk_read_sector(d: *mut pcmcia_disk_data, sect: m_uint32_t, buffer: *mut m_uint8_t) -> c_int {
    let disk_offset: libc::off_t = sect as libc::off_t * SECTOR_SIZE as libc::off_t;

    if DEBUG_READ != 0 {
        vm_log!((*d).vm, (*d).dev.name, cstr!("reading sector 0x%8.8x\n"), sect);
    }

    if libc::lseek((*d).fd, disk_offset, libc::SEEK_SET) == -1 {
        libc::perror(cstr!("read_sector: lseek"));
        return -1;
    }

    if libc::read((*d).fd, buffer.cast::<_>(), SECTOR_SIZE) != SECTOR_SIZE as ssize_t {
        libc::perror(cstr!("read_sector: read"));
        return -1;
    }

    0
}

/// Write a sector to disk file
unsafe fn disk_write_sector(d: *mut pcmcia_disk_data, sect: m_uint32_t, buffer: *mut m_uint8_t) -> c_int {
    let disk_offset: libc::off_t = sect as libc::off_t * SECTOR_SIZE as libc::off_t;

    if DEBUG_WRITE != 0 {
        vm_log!((*d).vm, (*d).dev.name, cstr!("writing sector 0x%8.8x\n"), sect);
    }

    if libc::lseek((*d).fd, disk_offset, libc::SEEK_SET) == -1 {
        libc::perror(cstr!("write_sector: lseek"));
        return -1;
    }

    if libc::write((*d).fd, buffer.cast::<_>(), SECTOR_SIZE) != SECTOR_SIZE as ssize_t {
        libc::perror(cstr!("write_sector: write"));
        return -1;
    }

    0
}

/// Identify PCMCIA device (ATA command 0xEC)
unsafe fn ata_identify_device(d: *mut pcmcia_disk_data) {
    let p: *mut m_uint8_t = (*d).data_buffer.as_c_mut();

    let sect_count: m_uint32_t = (*d).nr_heads * (*d).nr_cylinders * (*d).sects_per_track;

    // Clear all fields (for safety)
    libc::memset(p.cast::<_>(), 0x00, SECTOR_SIZE);

    // Word 0: General Configuration
    *p.add(0) = 0x8a; // Not MFM encoded | Hard sectored | Removable cartridge drive
    *p.add(1) = 0x84; // Disk transfer rate !<= 10Mbs | Non-rotating disk drive

    // Word 1: Default number of cylinders
    *p.add(2) = ((*d).nr_cylinders & 0xFF) as m_uint8_t;
    *p.add(3) = (((*d).nr_cylinders >> 8) & 0xFF) as m_uint8_t;

    // Word 3: Default number of heads
    *p.add(6) = (*d).nr_heads as m_uint8_t;

    // Word 6: Default number of sectors per track
    *p.add(12) = (*d).sects_per_track as m_uint8_t;

    // Word 7: Number of sectors per card (MSW)
    *p.add(14) = ((sect_count >> 16) & 0xFF) as m_uint8_t;
    *p.add(15) = (sect_count >> 24) as m_uint8_t;

    // Word 8: Number of sectors per card (LSW)
    *p.add(16) = (sect_count & 0xFF) as m_uint8_t;
    *p.add(17) = ((sect_count >> 8) & 0xFF) as m_uint8_t;

    // Word 22: ECC count
    *p.add(44) = 0x04;

    // Word 53: Translation parameters valid
    *p.add(106) = 0x3;

    // Word 54: Current number of cylinders
    *p.add(108) = ((*d).nr_cylinders & 0xFF) as m_uint8_t;
    *p.add(109) = (((*d).nr_cylinders >> 8) & 0xFF) as m_uint8_t;

    // Word 55: Current number of heads
    *p.add(110) = (*d).nr_heads as m_uint8_t;

    // Word 56: Current number of sectors per track
    *p.add(112) = (*d).sects_per_track as m_uint8_t;

    // Word 57/58: Current of sectors per card (LSW/MSW)
    *p.add(114) = (sect_count & 0xFF) as m_uint8_t;
    *p.add(115) = ((sect_count >> 8) & 0xFF) as m_uint8_t;

    *p.add(116) = ((sect_count >> 16) & 0xFF) as m_uint8_t;
    *p.add(117) = (sect_count >> 24) as m_uint8_t;

    if false {
        // Word 60/61: Total sectors addressable in LBA mode (MSW/LSW)
        *p.add(120) = ((sect_count >> 16) & 0xFF) as m_uint8_t;
        *p.add(121) = (sect_count >> 24) as m_uint8_t;
        *p.add(122) = (sect_count & 0xFF) as m_uint8_t;
        *p.add(123) = ((sect_count >> 8) & 0xFF) as m_uint8_t;
    }
}

/// Set sector position
unsafe fn ata_set_sect_pos(d: *mut pcmcia_disk_data) {
    let cyl: u_int;

    if ((*d).head & ATA_DH_LBA) != 0 {
        (*d).sect_pos = (((*d).head & 0x0F) as u_int) << 24;
        (*d).sect_pos |= ((*d).cyl_high as u_int) << 16;
        (*d).sect_pos |= ((*d).cyl_low as u_int) << 8;
        (*d).sect_pos |= (*d).sect_no as u_int;

        if DEBUG_ATA != 0 {
            vm_log!((*d).vm, (*d).dev.name, cstr!("ata_set_sect_pos: LBA sect=0x%x\n"), (*d).sect_pos);
        }
    } else {
        cyl = (((*d).cyl_high as u_int) << 8) + (*d).cyl_low as u_int;
        (*d).sect_pos = chs_to_lba(d, cyl, ((*d).head & 0x0F) as u_int, (*d).sect_no as u_int);

        if DEBUG_ATA != 0 {
            vm_log!((*d).vm, (*d).dev.name, cstr!("ata_set_sect_pos: cyl=0x%x,head=0x%x,sect=0x%x => sect_pos=0x%x\n"), cyl, (*d).head & 0x0F, (*d).sect_no, (*d).sect_pos);
        }
    }
}

/// ATA device identifier callback
unsafe extern "C" fn ata_cmd_ident_device_callback(d: *mut pcmcia_disk_data) {
    (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC;
}

/// ATA read sector callback
unsafe extern "C" fn ata_cmd_read_callback(d: *mut pcmcia_disk_data) {
    (*d).sect_remaining -= 1;

    if (*d).sect_remaining == 0 {
        (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC;
        return;
    }

    // Read the next sector
    (*d).sect_pos += 1;
    disk_read_sector(d, (*d).sect_pos, (*d).data_buffer.as_c_mut());
    (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC | ATA_STATUS_DRQ;
}

/// ATA write sector callback
unsafe extern "C" fn ata_cmd_write_callback(d: *mut pcmcia_disk_data) {
    // Write the sector
    disk_write_sector(d, (*d).sect_pos, (*d).data_buffer.as_c_mut());
    (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC | ATA_STATUS_DRQ;
    (*d).sect_pos += 1;

    (*d).sect_remaining -= 1;

    if (*d).sect_remaining == 0 {
        (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC;
    }
}

/// Handle an ATA command
unsafe fn ata_handle_cmd(d: *mut pcmcia_disk_data) {
    if DEBUG_ATA != 0 {
        vm_log!((*d).vm, (*d).dev.name, cstr!("ATA command 0x%2.2x\n"), (*d).ata_cmd as u_int);
    }

    (*d).data_pos = 0;

    match (*d).ata_cmd {
        ATA_CMD_IDENT_DEVICE => {
            ata_identify_device(d);
            (*d).ata_cmd_callback = Some(ata_cmd_ident_device_callback);
            (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC | ATA_STATUS_DRQ;
        }

        ATA_CMD_READ_SECTOR => {
            (*d).sect_remaining = (*d).sect_count as u_int;

            if (*d).sect_remaining == 0 {
                (*d).sect_remaining = 256;
            }

            ata_set_sect_pos(d);
            disk_read_sector(d, (*d).sect_pos, (*d).data_buffer.as_c_mut());
            (*d).ata_cmd_callback = Some(ata_cmd_read_callback);
            (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC | ATA_STATUS_DRQ;
        }

        ATA_CMD_WRITE_SECTOR => {
            (*d).sect_remaining = (*d).sect_count as u_int;

            if (*d).sect_remaining == 0 {
                (*d).sect_remaining = 256;
            }

            ata_set_sect_pos(d);
            (*d).ata_cmd_callback = Some(ata_cmd_write_callback);
            (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC | ATA_STATUS_DRQ;
        }

        _ => {
            vm_log!((*d).vm, (*d).dev.name, cstr!("unhandled ATA command 0x%2.2x\n"), (*d).ata_cmd as u_int);
        }
    }
}

/// dev_pcmcia_disk_access_0()
unsafe extern "C" fn dev_pcmcia_disk_access_0(cpu: *mut cpu_gen_t, dev: *mut vdevice, mut offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut pcmcia_disk_data = (*dev).priv_data.cast::<_>();

    // Compute the good internal offset
    offset = (offset >> 1) ^ 1;

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*d).dev.name, cstr!("reading offset 0x%5.5x at pc=0x%llx (size=%u)\n"), offset, cpu_get_pc(cpu), op_size);
        } else {
            cpu_log!(cpu, (*d).dev.name, cstr!("writing offset 0x%5.5x, data=0x%llx at pc=0x%llx (size=%u)\n"), offset, *data, cpu_get_pc(cpu), op_size);
        }
    }

    // Card Information Structure
    if (offset as usize) < cis_table.len() {
        if op_type == MTS_READ {
            *data = cis_table[offset as usize] as m_uint64_t;
        }

        return null_mut();
    }

    match offset {
        0x102 => {
            // Pin Replacement Register
            if op_type == MTS_READ {
                *data = 0x22;
            }
        }

        0x80001 => {
            // Sector Count + Sector no
            if op_type == MTS_READ {
                *data = ((((*d).sect_no as c_int) << 8) + (*d).sect_count as c_int) as m_uint64_t;
            } else {
                (*d).sect_no = (*data >> 8) as m_uint8_t;
                (*d).sect_count = (*data & 0xFF) as m_uint8_t;
            }
        }

        0x80002 => {
            // Cylinder Low + Cylinder High
            if op_type == MTS_READ {
                *data = ((((*d).cyl_high as c_int) << 8) + (*d).cyl_low as c_int) as m_uint64_t;
            } else {
                (*d).cyl_high = (*data >> 8) as m_uint8_t;
                (*d).cyl_low = (*data & 0xFF) as m_uint8_t;
            }
        }

        0x80003 => {
            // Select Card/Head + Status/Command register
            if op_type == MTS_READ {
                *data = ((((*d).ata_status as c_int) << 8) + (*d).head as c_int) as m_uint64_t;
            } else {
                (*d).ata_cmd = (*data >> 8) as m_uint8_t;
                (*d).head = *data as m_uint8_t;
                ata_handle_cmd(d);
            }
        }

        _ => {
            // Data buffer access ?
            if (offset >= (*d).data_offset) && (offset < (*d).data_offset + (SECTOR_SIZE as u_int / 2)) {
                if op_type == MTS_READ {
                    *data = (*d).data_buffer[((*d).data_pos << 1) as usize] as m_uint64_t;
                    *data += (((*d).data_buffer[(((*d).data_pos << 1) + 1) as usize] as c_int) << 8) as m_uint64_t;
                } else {
                    (*d).data_buffer[((*d).data_pos << 1) as usize] = (*data & 0xFF) as m_uint8_t;
                    (*d).data_buffer[(((*d).data_pos << 1) + 1) as usize] = (*data >> 8) as m_uint8_t;
                }

                (*d).data_pos += 1;

                // Buffer complete: call the callback function
                if (*d).data_pos == (SECTOR_SIZE as u_int / 2) {
                    (*d).data_pos = 0;

                    if (*d).ata_cmd_callback.is_some() {
                        (*d).ata_cmd_callback.unwrap()(d);
                    }
                }
            }
        }
    }

    null_mut()
}

/// dev_pcmcia_disk_access_1()
unsafe extern "C" fn dev_pcmcia_disk_access_1(cpu: *mut cpu_gen_t, dev: *mut vdevice, mut offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut pcmcia_disk_data = (*dev).priv_data.cast::<_>();

    // Compute the good internal offset
    offset = (offset >> 1) ^ 1;

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*d).dev.name, cstr!("reading offset 0x%5.5x at pc=0x%llx (size=%u)\n"), offset, cpu_get_pc(cpu), op_size);
        } else {
            cpu_log!(cpu, (*d).dev.name, cstr!("writing offset 0x%5.5x, data=0x%llx at pc=0x%llx (size=%u)\n"), offset, *data, cpu_get_pc(cpu), op_size);
        }
    }

    match offset {
        0x02 => {
            // Sector Count + Sector no
            if op_type == MTS_READ {
                *data = ((((*d).sect_no as c_int) << 8) + (*d).sect_count as c_int) as m_uint64_t;
            } else {
                (*d).sect_no = (*data >> 8) as m_uint8_t;
                (*d).sect_count = (*data & 0xFF) as m_uint8_t;
            }
        }

        0x04 => {
            // Cylinder Low + Cylinder High
            if op_type == MTS_READ {
                *data = ((((*d).cyl_high as c_int) << 8) + (*d).cyl_low as c_int) as m_uint64_t;
            } else {
                (*d).cyl_high = (*data >> 8) as m_uint8_t;
                (*d).cyl_low = (*data & 0xFF) as m_uint8_t;
            }
        }

        0x06 => {
            // Select Card/Head + Status/Command register
            if op_type == MTS_READ {
                *data = ((((*d).ata_status as c_int) << 8) + (*d).head as c_int) as m_uint64_t;
            } else {
                (*d).ata_cmd = (*data >> 8) as m_uint8_t;
                (*d).head = (*data & 0xFF) as m_uint8_t;
                ata_handle_cmd(d);
            }
        }

        0x08 => {
            // Data
            if op_type == MTS_READ {
                *data = ((*d).data_buffer[((*d).data_pos << 1) as usize]) as m_uint64_t;
                *data += (((*d).data_buffer[(((*d).data_pos << 1) + 1) as usize] as c_int) << 8) as m_uint64_t;
            } else {
                (*d).data_buffer[((*d).data_pos << 1) as usize] = (*data & 0xFF) as m_uint8_t;
                (*d).data_buffer[(((*d).data_pos << 1) + 1) as usize] = (*data >> 8) as m_uint8_t;
            }

            (*d).data_pos += 1;

            // Buffer complete: call the callback function
            if (*d).data_pos == (SECTOR_SIZE as u_int / 2) {
                (*d).data_pos = 0;

                if (*d).ata_cmd_callback.is_some() {
                    (*d).ata_cmd_callback.unwrap()(d);
                }
            }
        }

        0x0E => { // Status/Drive Control + Drive Address
        }

        _ => {}
    }

    null_mut()
}

/// Shutdown a PCMCIA disk device
unsafe extern "C" fn dev_pcmcia_disk_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut pcmcia_disk_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // Close disk file
        if (*d).fd != -1 {
            libc::close((*d).fd);
        }

        // Free filename
        libc::free((*d).filename.cast::<_>());

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Initialize a PCMCIA disk
#[no_mangle]
pub unsafe extern "C" fn dev_pcmcia_disk_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t, disk_size: u_int, mode: c_int) -> *mut vm_obj_t {
    // allocate the private data structure
    let d: *mut pcmcia_disk_data = libc::malloc(size_of::<pcmcia_disk_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("PCMCIA: unable to create disk device '%s'.\n"), name);
        return null_mut();
    }

    libc::memset(d.cast::<_>(), 0, size_of::<pcmcia_disk_data>());
    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm = vm;
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_pcmcia_disk_shutdown);
    (*d).fd = -1;

    (*d).filename = vm_build_filename(vm, name);
    if (*d).filename.is_null() {
        libc::fprintf(c_stderr(), cstr!("PCMCIA: unable to create filename.\n"));
        libc::free(d.cast::<_>());
        return null_mut();
    }

    // Data buffer offset in mapped memory
    (*d).data_offset = 0x80200;
    (*d).ata_status = ATA_STATUS_RDY | ATA_STATUS_DSC;

    // Compute the number of cylinders given a disk size in Mb
    let tot_sect: m_uint32_t = (((disk_size as m_uint64_t) * 1048576) / SECTOR_SIZE as m_uint64_t) as m_uint32_t;

    (*d).nr_heads = DISK_NR_HEADS;
    (*d).sects_per_track = DISK_SECTS_PER_TRACK;
    (*d).nr_cylinders = tot_sect / ((*d).nr_heads * (*d).sects_per_track);

    vm_log!(vm, name, cstr!("C/H/S settings = %u/%u/%u\n"), (*d).nr_cylinders, (*d).nr_heads, (*d).sects_per_track);

    // Create the disk file
    if disk_create(d) == -1 {
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return null_mut();
    }

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.flags = VDEVICE_FLAG_CACHING;

    if mode == 0 {
        (*d).dev.handler = Some(dev_pcmcia_disk_access_0);
    } else {
        (*d).dev.handler = Some(dev_pcmcia_disk_access_1);
    }

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    addr_of_mut!((*d).vm_obj)
}

/// Get the device associated with a PCMCIA disk object
#[no_mangle]
pub unsafe extern "C" fn dev_pcmcia_disk_get_device(obj: *mut vm_obj_t) -> *mut vdevice {
    if obj.is_null() {
        return null_mut();
    }

    let d: *mut pcmcia_disk_data = (*obj).data.cast::<_>();
    if d.is_null() {
        return null_mut();
    }

    addr_of_mut!((*d).dev)
}
