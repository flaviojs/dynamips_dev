//! Cisco router simulation platform.
//! Copyright (c) 2006 Christophe Fillot.  All rights reserved.
//!
//! Intel Flash SIMM emulation.
//!
//! Intelligent ID Codes:
//!   28F008SA: 0x89A2 (1 Mb)
//!   28F016SA: 0x89A0 (2 Mb)
//!
//! Manuals:
//!    http://www.ortodoxism.ro/datasheets/Intel/mXvsysv.pdf
//!
//! TODO: A lot of commands are lacking. Doesn't work with NPE-G2.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::utils::*;
use crate::vm::*;

const DEBUG_ACCESS: c_int = 0;
const DEBUG_WRITE: c_int = 0;

/// Flash command states // TODO enum
const FLASH_CMD_READ_ARRAY: u_int = 0;
const FLASH_CMD_READ_ID: u_int = 1;
const FLASH_CMD_READ_QUERY: u_int = 2;
const FLASH_CMD_READ_STATUS: u_int = 3;
const FLASH_CMD_WRITE_BUF_CNT: u_int = 4;
const FLASH_CMD_WRITE_BUF_DATA: u_int = 5;
const FLASH_CMD_WRITE_BUF_CONFIRM: u_int = 6;
const FLASH_CMD_WB_PROG: u_int = 7;
const FLASH_CMD_WB_PROG_DONE: u_int = 8;
const FLASH_CMD_BLK_ERASE: u_int = 9;
const FLASH_CMD_BLK_ERASE_DONE: u_int = 10;
const FLASH_CMD_CONFIG: u_int = 11;

/// Flash access mode (byte or word) // TODO enum
const FLASH_MODE_BYTE: u_int = 1;
const FLASH_MODE_WORD: u_int = 2;

const MAX_FLASH: usize = 4;
const FLASH_BUF_SIZE: usize = 32;

/// Flash model
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flash_model {
    pub name: *mut c_char,
    pub total_size: u_int,
    pub mode: u_int,
    pub nr_flash_bits: u_int,
    pub blk_size: u_int,
    pub id_manufacturer: u_int,
    pub id_device: u_int,
}
impl flash_model {
    pub const fn new(name: *mut c_char, total_size: u_int, mode: u_int, nr_flash_bits: u_int, blk_size: u_int, id_manufacturer: u_int, id_device: u_int) -> Self {
        Self { name, total_size, mode, nr_flash_bits, blk_size, id_manufacturer, id_device }
    }
    pub const fn null() -> Self {
        Self::new(null_mut(), 0, 0, 0, 0, 0, 0)
    }
}

/// Flash internal data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flash_data {
    pub mode: u_int,
    pub offset_shift: u_int,
    pub state: u_int,
    pub blk_size: u_int,
    pub id_manufacturer: m_uint8_t,
    pub id_device: m_uint8_t,
    pub status_reg: m_uint8_t,
    pub flash_set: *mut flashset_data,
    pub flash_pos: u_int,
    pub wb_offset: u_int,
    pub wb_count: u_int,
    pub wb_remain: u_int,
    pub wbuf: [u_int; FLASH_BUF_SIZE],
}

/// Flashset private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flashset_data {
    pub vm: *mut vm_instance_t,
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub filename: *mut c_char,

    pub mode: u_int,
    pub nr_flash_bits: u_int,
    pub nr_flash_count: u_int,
    pub flash: [flash_data; MAX_FLASH],
}

/// Log a Flash message
macro_rules! FLASH_LOG {
    ($d:expr, $msg:expr$(, $arg:expr)*) => {
        let d: *mut flash_data = $d;
        let msg: *mut c_char = $msg;
        vm_log!((*(*d).flash_set).vm, (*(*d).flash_set).dev.name, msg$(, $arg)*);
    };
}

unsafe fn BPTR(d: *mut flashset_data, offset: u_int) -> *mut u_char {
    ((*d).dev.host_addr as *mut u_char).add(offset as usize)
}

/// Some Flash models
static mut flash_models: [flash_model; 8] = [
    // C1700 4 Mb bootflash: 1x28F320 in word mode
    flash_model::new(cstr!("c1700-bootflash-4mb"), 4 * 1048576, FLASH_MODE_WORD, 0, 0x10000, 0x89, 0x14),
    // C1700 8 Mb bootflash: 1x28F640 in word mode
    flash_model::new(cstr!("c1700-bootflash-8mb"), 8 * 1048576, FLASH_MODE_WORD, 0, 0x10000, 0x89, 0x15),
    // C3600 8 Mb bootflash: 4x28F016SA in byte mode
    flash_model::new(cstr!("c3600-bootflash-8mb"), 8 * 1048576, FLASH_MODE_BYTE, 2, 0x10000, 0x89, 0xA0),
    // C7200 4 Mb bootflash: 4x28F008SA in byte mode
    flash_model::new(cstr!("c7200-bootflash-4mb"), 4 * 1048576, FLASH_MODE_BYTE, 2, 0x10000, 0x89, 0xA2),
    // C7200 8 Mb bootflash: 4x28F016SA in byte mode
    flash_model::new(cstr!("c7200-bootflash-8mb"), 8 * 1048576, FLASH_MODE_BYTE, 2, 0x10000, 0x89, 0xA0),
    // C7200 64 Mb bootflash: 4x128 Mb Intel flash in byte mode
    // (for NPE-G2 but doesn't work now).
    flash_model::new(cstr!("c7200-bootflash-64mb"), 64 * 1048576, FLASH_MODE_BYTE, 2, 0x10000, 0x89, 0x18),
    // C2600 8 Mb bootflash: 4x28F016SA in byte mode
    flash_model::new(cstr!("c2600-bootflash-8mb"), 8 * 1048576, FLASH_MODE_BYTE, 2, 0x10000, 0x89, 0xA0),
    flash_model::null(),
];

/// Flash model lookup
unsafe fn flash_model_find(name: *mut c_char) -> *mut flash_model {
    let mut fm: *mut flash_model = flash_models.as_c_mut();
    while !(*fm).name.is_null() {
        if libc::strcmp((*fm).name, name) == 0 {
            return fm;
        }
        fm = fm.add(1);
    }

    null_mut()
}

/// Initialize a flashset
unsafe fn flashset_init(d: *mut flashset_data, mode: u_int, nr_flash_bits: u_int, blk_size: u_int, id_manufacturer: m_uint8_t, id_device: m_uint8_t) -> c_int {
    let mut flash: *mut flash_data;

    (*d).mode = mode;
    (*d).nr_flash_bits = nr_flash_bits;
    (*d).nr_flash_count = 1 << (*d).nr_flash_bits;

    let offset_shift: u_int = match mode {
        FLASH_MODE_BYTE => 0,
        FLASH_MODE_WORD => 1,
        _ => {
            return -1;
        }
    };

    for i in 0..(*d).nr_flash_count {
        flash = addr_of_mut!((*d).flash[i as usize]);

        (*flash).mode = mode;
        (*flash).offset_shift = offset_shift;
        (*flash).state = FLASH_CMD_READ_ARRAY;

        (*flash).id_manufacturer = id_manufacturer;
        (*flash).id_device = id_device;

        (*flash).flash_set = d;
        (*flash).flash_pos = i;

        (*flash).blk_size = blk_size;
    }

    0
}

/// Read a byte from a Flash
unsafe fn flash_read(d: *mut flash_data, offset: u_int, data: *mut u_int) -> c_int {
    let real_offset: u_int = (offset << ((*(*d).flash_set).nr_flash_bits)) + (*d).flash_pos;

    if (*d).mode == FLASH_MODE_BYTE {
        *data = *BPTR((*d).flash_set, real_offset) as u_int;
    } else {
        *data = (*BPTR((*d).flash_set, real_offset << 1) as u_int) << 8;
        *data |= *BPTR((*d).flash_set, (real_offset << 1) + 1) as u_int;
    }
    0
}

/// Write a byte to a Flash
unsafe fn flash_write(d: *mut flash_data, offset: u_int, data: u_int) -> c_int {
    let real_offset: u_int = (offset << ((*(*d).flash_set).nr_flash_bits)) + (*d).flash_pos;

    if (*d).mode == FLASH_MODE_BYTE {
        *BPTR((*d).flash_set, real_offset) = data as u_char;
    } else {
        *BPTR((*d).flash_set, real_offset << 1) = (data >> 8) as u_char;
        *BPTR((*d).flash_set, (real_offset << 1) + 1) = (data & 0xFF) as u_char;
    }
    0
}

/// Set machine state given a command
unsafe fn flash_cmd(d: *mut flash_data, offset: u_int, mut cmd: u_int) {
    cmd &= 0xFF;

    match cmd {
        0x40 | 0x10 => {
            (*d).state = FLASH_CMD_WB_PROG;
        }
        0xe8 => {
            (*d).state = FLASH_CMD_WRITE_BUF_CNT;
            (*d).wb_offset = offset;
            (*d).wb_count = 0;
            (*d).wb_remain = 0;
        }
        0x70 => {
            (*d).state = FLASH_CMD_READ_STATUS;
        }
        0x50 => {
            (*d).status_reg = 0;
            (*d).state = FLASH_CMD_READ_ARRAY;
        }
        0x90 => {
            (*d).state = FLASH_CMD_READ_ID;
        }
        0x20 => {
            (*d).state = FLASH_CMD_BLK_ERASE;
        }
        0xff => {
            (*d).state = FLASH_CMD_READ_ARRAY;
        }
        _ => {
            FLASH_LOG!(d, cstr!("flash_cmd(%u): command 0x%2.2x not implemented\n"), (*d).flash_pos, cmd as u_int);
        }
    }
}

/// Generic Flash access
unsafe fn flash_access(d: *mut flash_data, mut offset: m_uint32_t, op_type: u_int, data: *mut u_int) {
    if op_type == MTS_READ {
        *data = 0x00;
    }

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            FLASH_LOG!(d, cstr!("flash_access(%u): read  access to offset 0x%8.8x (state=%u)\n"), (*d).flash_pos, offset, (*d).state);
        } else {
            FLASH_LOG!(d, cstr!("flash_access(%u): write access to offset 0x%8.8x, data=0x%4.4x (state=%u)\n"), (*d).flash_pos, offset, *data, (*d).state);
        }
    }

    offset >>= (*d).offset_shift;

    // State machine for Flash commands
    match (*d).state {
        FLASH_CMD_READ_ARRAY => {
            if op_type == MTS_READ {
                flash_read(d, offset, data);
                return;
            }

            // Command Write
            flash_cmd(d, offset, *data);
        }

        // Write byte/word
        FLASH_CMD_WB_PROG => {
            if op_type == MTS_WRITE {
                flash_write(d, offset, *data);
                (*d).state = FLASH_CMD_WB_PROG_DONE;
            }
        }

        // Write byte/word (done)
        FLASH_CMD_WB_PROG_DONE => {
            if op_type == MTS_WRITE {
                flash_cmd(d, offset, *data);
            } else {
                *data = 0x80;
            }
        }

        // Write buffer (count)
        FLASH_CMD_WRITE_BUF_CNT => {
            if op_type == MTS_WRITE {
                (*d).wb_count = (*data & 0x1F) + 1;
                (*d).wb_remain = (*d).wb_count;
                (*d).state = FLASH_CMD_WRITE_BUF_DATA;
            } else {
                *data = 0x80;
            }
        }

        // Write buffer (data)
        FLASH_CMD_WRITE_BUF_DATA => {
            if op_type == MTS_WRITE {
                if (offset >= (*d).wb_offset) && (offset < ((*d).wb_offset + (*d).wb_count)) {
                    (*d).wbuf[(offset - (*d).wb_offset) as usize] = *data;
                    (*d).wb_remain -= 1;

                    if (*d).wb_remain == 0 {
                        (*d).state = FLASH_CMD_WRITE_BUF_CONFIRM;
                    }
                }
            } else {
                *data = 0x80;
            }
        }

        // Write buffer (confirm)
        FLASH_CMD_WRITE_BUF_CONFIRM => {
            if op_type == MTS_WRITE {
                if (*data & 0xFF) == 0xD0 {
                    for i in 0..(*d).wb_count {
                        flash_write(d, (*d).wb_offset + i, (*d).wbuf[i as usize]);
                    }
                } else {
                    // XXX Error
                }

                (*d).state = FLASH_CMD_READ_ARRAY;
            } else {
                *data = 0x80;
            }
        }

        // Read status register
        FLASH_CMD_READ_STATUS => {
            if op_type == MTS_READ {
                *data = 0x80; //(*d).status_reg;
            }

            (*d).state = FLASH_CMD_READ_ARRAY;
        }

        // Read identifier codes
        FLASH_CMD_READ_ID => {
            if op_type == MTS_READ {
                match offset {
                    0x00 => {
                        *data = (*d).id_manufacturer as u_int;
                    }
                    0x01 => {
                        *data = (*d).id_device as u_int;
                    }
                    _ => {
                        *data = 0x00;
                    }
                }
            } else {
                flash_cmd(d, offset, *data);
            }
        }

        // Block Erase
        FLASH_CMD_BLK_ERASE => {
            if op_type == MTS_WRITE {
                if DEBUG_WRITE != 0 {
                    FLASH_LOG!(d, cstr!("flash_access(%u): erasing block at offset 0x%8.8x\n"), offset);
                }
                if (*data & 0xFF) == 0xD0 {
                    for i in 0..(*d).blk_size {
                        flash_write(d, offset + i, 0xFFFF);
                    }

                    (*d).state = FLASH_CMD_BLK_ERASE_DONE;
                }
            } else {
                *data = 0x80;
            }
        }

        // Block Erase Done
        FLASH_CMD_BLK_ERASE_DONE => {
            if op_type == MTS_WRITE {
                flash_cmd(d, offset, *data);
            } else {
                *data = 0x80;
            }
        }

        _ => {}
    }
}

/// dev_bootflash_access()
#[no_mangle]
pub unsafe extern "C" fn dev_bootflash_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut flashset_data = (*dev).priv_data.cast::<_>();
    let mut flash_data: [u_int; 8] = [0; 8];
    let mut fi: u_int;
    let mut d_off: u_int;

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*dev).name, cstr!("read  access to offset = 0x%x, pc = 0x%llx\n"), offset, cpu_get_pc(cpu));
        } else {
            cpu_log!(cpu, (*dev).name, cstr!("write access to vaddr = 0x%x, pc = 0x%llx, val = 0x%llx\n"), offset, cpu_get_pc(cpu), *data);
        }
    }

    if op_type == MTS_READ {
        *data = 0;

        for i in (0..op_size).step_by((*d).mode as usize) {
            fi = (offset + i) & ((*d).nr_flash_count - 1);

            flash_access(addr_of_mut!((*d).flash[fi as usize]), (offset + i) >> (*d).nr_flash_bits, op_type, addr_of_mut!(flash_data[i as usize]));

            d_off = (op_size - i - (*d).mode) << 3;
            *data |= (flash_data[i as usize] as m_uint64_t) << d_off;
        }
    } else {
        for i in (0..op_size).step_by((*d).mode as usize) {
            fi = (offset + i) & ((*d).nr_flash_count - 1);

            d_off = (op_size - i - (*d).mode) << 3;
            flash_data[i as usize] = (*data >> d_off) as u_int;

            flash_access(addr_of_mut!((*d).flash[fi as usize]), (offset + i) >> (*d).nr_flash_bits, op_type, addr_of_mut!(flash_data[i as usize]));
        }
    }

    null_mut()
}

/// Shutdown a bootflash device
unsafe extern "C" fn dev_bootflash_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut flashset_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the device
        dev_remove(vm, addr_of_mut!((*d).dev));

        // We don't remove the file, since it used as permanent storage
        if !(*d).filename.is_null() {
            libc::free((*d).filename.cast::<_>());
        }

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Create a 8 Mb bootflash
#[no_mangle]
pub unsafe extern "C" fn dev_bootflash_init(vm: *mut vm_instance_t, name: *mut c_char, model: *mut c_char, paddr: m_uint64_t) -> c_int {
    let mut ptr: *mut u_char = null_mut();

    // Find the flash model
    let fm: *mut flash_model = flash_model_find(model);
    if fm.is_null() {
        vm_error!(vm, cstr!("bootflash: unable to find model '%s'\n"), model);
        return -1;
    }

    // Allocate the private data structure
    let d: *mut flashset_data = libc::malloc(size_of::<flashset_data>()).cast::<_>();
    if d.is_null() {
        vm_error!(vm, cstr!("bootflash: unable to create device.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<flashset_data>());
    (*d).vm = vm;

    // Initialize flash based on model properties
    flashset_init(d, (*fm).mode, (*fm).nr_flash_bits, (*fm).blk_size, (*fm).id_manufacturer as m_uint8_t, (*fm).id_device as m_uint8_t);

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_bootflash_shutdown);

    (*d).filename = vm_build_filename(vm, name);
    if (*d).filename.is_null() {
        vm_error!(vm, cstr!("bootflash: unable to create filename.\n"));
        libc::free(d.cast::<_>());
        return -1;
    }

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = (*fm).total_size;
    (*d).dev.handler = Some(dev_bootflash_access);
    (*d).dev.fd = memzone_create_file((*d).filename, (*d).dev.phys_len as size_t, addr_of_mut!(ptr));
    (*d).dev.host_addr = ptr as m_iptr_t;
    (*d).dev.flags = VDEVICE_FLAG_NO_MTS_MMAP;

    if (*d).dev.fd == -1 {
        vm_error!(vm, cstr!("bootflash: unable to map file '%s'\n"), (*d).filename);
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
