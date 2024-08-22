//! Cisco Simulation Platform.
//! Copyright (c) 2006 Christophe Fillot.  All rights reserved.
//!
//! 23-Oct-2006: only basic code at this time.
//!
//! Considering the access pattern, this might be emulating SST39VF1681/SST39VF1682.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::utils::*;
use crate::vm::*;

const DEBUG_ACCESS: c_int = 0;
const DEBUG_WRITE: c_int = 0;

/// Flash private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flash_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub state: m_uint32_t,
    pub sector_size: u_int,
    pub filename: *mut c_char,
}

unsafe fn BPTR(d: *mut flash_data, offset: u_int) -> *mut c_char {
    ((*d).dev.host_addr as *mut c_char).add(offset as usize)
}

/// dev_bootflash_access()
unsafe extern "C" fn dev_flash_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut flash_data = (*dev).priv_data.cast::<_>();

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*dev).name, cstr!("read  access to offset=0x%x, pc=0x%llx (state=%u)\n"), offset, cpu_get_pc(cpu), (*d).state);
        } else {
            cpu_log!(cpu, (*dev).name, cstr!("write access to vaddr=0x%x, pc=0x%llx, val=0x%llx (state=%d)\n"), offset, cpu_get_pc(cpu), *data, (*d).state);
        }
    }

    if op_type == MTS_READ {
        match (*d).state {
            0 => {
                return BPTR(d, offset).cast::<_>();
            }
            _ => {
                cpu_log!(cpu, (*dev).name, cstr!("read: unhandled state %d\n"), (*d).state);
            }
        }

        return null_mut();
    }

    // Write mode
    if DEBUG_WRITE != 0 {
        cpu_log!(cpu, (*dev).name, cstr!("write to offset 0x%x, data=0x%llx\n"), offset, *data);
    }

    match (*d).state {
        // Initial Cycle
        0 => {
            match offset {
                0xAAA => {
                    if *data == 0xAA {
                        (*d).state = 1;
                    }
                }
                _ => {
                    match *data {
                        0xB0 => {
                            // Erase/Program Suspend
                            (*d).state = 0;
                        }
                        0x30 => {
                            // Erase/Program Resume
                            (*d).state = 0;
                        }
                        0xF0 => {
                            // Product ID Exit
                        }
                        _ => {}
                    }
                }
            }
        }

        // Cycle 1 was: 0xAAA, 0xAA
        1 => {
            if (offset != 0x555) && (*data != 0x55) {
                (*d).state = 0;
            } else {
                (*d).state = 2;
            }
        }

        // Cycle 1 was: 0xAAA, 0xAA, Cycle 2 was: 0x555, 0x55
        2 => {
            (*d).state = 0;

            if offset == 0xAAA {
                match *data {
                    0x80 => {
                        (*d).state = 3;
                    }
                    0xA0 => {
                        // Byte/Word program
                        (*d).state = 4;
                    }
                    0xF0 => {
                        // Product ID Exit
                    }
                    0xC0 => {
                        // Program Protection Register / Lock Protection Register
                        (*d).state = 5;
                    }
                    0x90 => {
                        // Product ID Entry / Status of Block B protection
                        (*d).state = 6;
                    }
                    0xD0 => {
                        // Set configuration register
                        (*d).state = 7;
                    }
                    _ => {}
                }
            }
        }

        // Cycle 1 was 0xAAA, 0xAA
        // Cycle 2 was 0x555, 0x55
        // Cycle 3 was 0xAAA, 0x80
        3 => {
            if (offset != 0xAAA) && (*data != 0xAA) {
                (*d).state = 0;
            } else {
                (*d).state = 8;
            }
        }

        // Cycle 1 was 0xAAA, 0xAA
        // Cycle 2 was 0x555, 0x55
        // Cycle 3 was 0xAAA, 0x80
        // Cycle 4 was 0xAAA, 0xAA
        8 => {
            if (offset != 0x555) && (*data != 0x55) {
                (*d).state = 0;
            } else {
                (*d).state = 9;
            }
        }

        // Cycle 1 was 0xAAA, 0xAA
        // Cycle 2 was 0x555, 0x55
        // Cycle 3 was 0xAAA, 0x80
        // Cycle 4 was 0xAAA, 0xAA
        // Cycle 5 was 0x555, 0x55
        9 => {
            (*d).state = 0;

            match *data {
                0x10 => {
                    // Chip Erase
                    libc::memset(BPTR(d, offset).cast::<_>(), 0, (*d).dev.phys_len as size_t);
                }

                0x30 => {
                    // Sector Erase
                    libc::memset(BPTR(d, offset).cast::<_>(), 0, (*d).sector_size as size_t);
                }

                0xA0 => {
                    // Enter Single Pulse Program Mode
                }

                0x60 => {
                    // Sector Lockdown
                }

                _ => {}
            }
        }

        // Byte/Word Program
        4 => {
            (*d).state = 0;
            *(BPTR(d, offset) as *mut m_uint8_t) = *data as m_uint8_t;
        }

        _ => {
            cpu_log!(cpu, (*dev).name, cstr!("write: unhandled state %d\n"), (*d).state);
        }
    }

    null_mut()
}

/// Copy data directly to a flash device
#[no_mangle]
pub unsafe extern "C" fn dev_flash_copy_data(obj: *mut vm_obj_t, offset: m_uint32_t, ptr: *mut u_char, len: ssize_t) -> c_int {
    let d: *mut flash_data = (*obj).data.cast::<_>();

    if d.is_null() || (*d).dev.host_addr == 0 {
        return -1;
    }

    let p: *mut u_char = ((*d).dev.host_addr as *mut u_char).add(offset as usize);
    libc::memcpy(p.cast::<_>(), ptr.cast::<_>(), len as size_t);
    0
}

/// Shutdown a flash device
unsafe extern "C" fn dev_flash_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut flash_data = d.cast::<_>();
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

/// Create a Flash device
#[no_mangle]
pub unsafe extern "C" fn dev_flash_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t) -> *mut vm_obj_t {
    let mut ptr: *mut u_char = null_mut();

    // Allocate the private data structure
    let d: *mut flash_data = libc::malloc(size_of::<flash_data>()).cast::<flash_data>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("Flash: unable to create device.\n"));
        return null_mut();
    }

    libc::memset(d.cast::<_>(), 0, size_of::<flash_data>());
    (*d).sector_size = 0x4000;

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_flash_shutdown);

    (*d).filename = vm_build_filename(vm, name);
    if (*d).filename.is_null() {
        libc::fprintf(c_stderr(), cstr!("Flash: unable to create filename.\n"));
        libc::free(d.cast::<_>());
        return null_mut();
    }

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.priv_data = d.cast::<_>();
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_flash_access);
    (*d).dev.fd = memzone_create_file((*d).filename, (*d).dev.phys_len as size_t, addr_of_mut!(ptr));
    (*d).dev.host_addr = ptr as m_iptr_t;
    (*d).dev.flags = VDEVICE_FLAG_NO_MTS_MMAP;

    if (*d).dev.fd == -1 {
        libc::fprintf(c_stderr(), cstr!("Flash: unable to map file '%s'\n"), (*d).filename);
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return null_mut();
    }

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    addr_of_mut!((*d).vm_obj)
}
