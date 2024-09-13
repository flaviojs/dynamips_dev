//! Cisco router simulation platform.
//! Copyright (C) 2006 Christophe Fillot.  All rights reserved.
//!
//! Remote control module.

use crate::_private::*;
use crate::cpu::*;
use crate::dev_vtty::*;
use crate::device::*;
use crate::dynamips::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::mips64::*;
use crate::rommon_var::*;
use crate::utils::*;
use crate::vm::*;

const DEBUG_ACCESS: c_int = 0;

const ROMMON_SET_VAR: c_int = 0x01;
const ROMMON_GET_VAR: c_int = 0x02;
const ROMMON_CLEAR_VAR_STAT: c_int = 0x03;

/// Remote control private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct remote_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,

    /// Console buffer
    pub con_buffer: [c_char; 512],
    pub con_buf_pos: u_int,

    /// ROMMON variables buffer
    pub var_buffer: [c_char; 512],
    pub var_buf_pos: u_int,
    pub var_status: u_int,

    /// Position for cookie reading
    pub cookie_pos: u_int,
}

/// dev_remote_control_access()
unsafe extern "C" fn dev_remote_control_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let vm: *mut vm_instance_t = (*cpu).vm;
    let d: *mut remote_data = (*dev).priv_data.cast::<_>();
    let mut storage_dev: *mut vdevice;
    let len: size_t;

    if op_type == MTS_READ {
        *data = 0;
    }

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, cstr!("REMOTE"), cstr!("reading reg 0x%x at pc=0x%llx\n"), offset, cpu_get_pc(cpu));
        } else {
            cpu_log!(cpu, cstr!("REMOTE"), cstr!("writing reg 0x%x at pc=0x%llx, data=0x%llx\n"), offset, cpu_get_pc(cpu), *data);
        }
    }

    match offset {
        // ROM Identification tag
        0x000 => {
            if op_type == MTS_READ {
                *data = ROM_ID as m_uint64_t;
            }
        }

        // CPU ID
        0x004 => {
            if op_type == MTS_READ {
                *data = (*cpu).id as m_uint64_t;
            }
        }

        // Display CPU registers
        0x008 => {
            if op_type == MTS_WRITE {
                (*cpu).reg_dump.unwrap()(cpu);
            }
        }

        // Display CPU memory info
        0x00c => {
            if op_type == MTS_WRITE {
                (*cpu).mmu_dump.unwrap()(cpu);
            }
        }

        // Reserved/Unused
        0x010 => {}

        // RAM size
        0x014 => {
            if op_type == MTS_READ {
                *data = ((*vm).ram_size - (*vm).ram_res_size) as m_uint64_t;
            }
        }

        // ROM size
        0x018 => {
            if op_type == MTS_READ {
                *data = (*vm).rom_size as m_uint64_t;
            }
        }

        // NVRAM size
        0x01c => {
            if op_type == MTS_READ {
                *data = (*vm).nvram_size as m_uint64_t;
            }
        }

        // IOMEM size
        0x020 => {
            if op_type == MTS_READ {
                *data = (*vm).iomem_size as m_uint64_t;
            }
        }

        // Config Register
        0x024 => {
            if op_type == MTS_READ {
                *data = (*vm).conf_reg as m_uint64_t;
            }
        }

        // ELF entry point
        0x028 => {
            if op_type == MTS_READ {
                *data = (*vm).ios_entry_point as m_uint64_t;
            }
        }

        // ELF machine id
        0x02c => {
            if op_type == MTS_READ {
                *data = (*vm).elf_machine_id as m_uint64_t;
            }
        }

        // Restart IOS Image
        0x030 => {
            // not implemented
        }

        // Stop the virtual machine
        0x034 => {
            (*vm).status = VM_STATUS_SHUTDOWN;
        }

        // Debugging/Log message: /!\ physical address
        0x038 => {
            if op_type == MTS_WRITE {
                len = physmem_strlen(vm, *data);
                if len < (*d).con_buffer.len() {
                    physmem_copy_from_vm(vm, (*d).con_buffer.as_c_void_mut(), *data, len + 1);
                    vm_log!(vm, cstr!("ROM"), (*d).con_buffer.as_c_mut());
                }
            }
        }

        // Console Buffering
        0x03c => {
            if op_type == MTS_WRITE {
                if (*d).con_buf_pos < ((*d).con_buffer.len() as u_int - 1) {
                    (*d).con_buffer[(*d).con_buf_pos as usize] = (*data & 0xFF) as c_char;
                    (*d).con_buf_pos += 1;
                    (*d).con_buffer[(*d).con_buf_pos as usize] = 0;

                    if (*d).con_buffer[(*d).con_buf_pos as usize - 1] == b'\n' as c_char {
                        vm_log!(vm, cstr!("ROM"), cstr!("%s"), (*d).con_buffer.as_c());
                        (*d).con_buf_pos = 0;
                    }
                } else {
                    (*d).con_buf_pos = 0;
                }
            }
        }

        // Console output
        0x040 => {
            if op_type == MTS_WRITE {
                vtty_put_char((*vm).vtty_con, *data as c_char);
            }
        }

        // NVRAM address
        0x044 => {
            if op_type == MTS_READ {
                storage_dev = dev_get_by_name(vm, cstr!("nvram"));
                if !storage_dev.is_null() {
                    *data = (*storage_dev).phys_addr;
                }

                storage_dev = dev_get_by_name(vm, cstr!("ssa"));
                if !storage_dev.is_null() {
                    *data = (*storage_dev).phys_addr;
                }

                if (*cpu).r#type == CPU_TYPE_MIPS64 {
                    *data += MIPS_KSEG1_BASE as m_uint64_t;
                }
            }
        }

        // IO memory size for Smart-Init (C3600, others ?)
        0x048 => {
            if op_type == MTS_READ {
                *data = (*vm).nm_iomem_size as m_uint64_t;
            }
        }

        // Cookie position selector
        0x04c => {
            if op_type == MTS_READ {
                *data = (*d).cookie_pos as m_uint64_t;
            } else {
                (*d).cookie_pos = *data as u_int;
            }
        }

        // Cookie data
        0x050 => {
            if (op_type == MTS_READ) && ((*d).cookie_pos < 64) {
                *data = (*vm).chassis_cookie[(*d).cookie_pos as usize] as m_uint64_t;
            }
        }

        // ROMMON variable
        0x054 => {
            if op_type == MTS_WRITE {
                if (*d).var_buf_pos < ((*d).var_buffer.len() as u_int - 1) {
                    (*d).var_buffer[(*d).var_buf_pos as usize] = (*data & 0xFF) as c_char;
                    (*d).var_buf_pos += 1;
                    (*d).var_buffer[(*d).var_buf_pos as usize] = 0;
                } else {
                    (*d).var_buf_pos = 0;
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if (*d).var_buf_pos < ((*d).var_buffer.len() as u_int - 1) {
                    *data = (*d).var_buffer[(*d).var_buf_pos as usize] as m_uint64_t;
                    (*d).var_buf_pos += 1;
                } else {
                    (*d).var_buf_pos = 0;
                    *data = 0;
                }
            }
        }

        // ROMMON variable command
        0x058 => {
            if op_type == MTS_WRITE {
                match (*data & 0xFF) as c_int {
                    ROMMON_SET_VAR => {
                        (*d).var_status = rommon_var_add_str(addr_of_mut!((*vm).rommon_vars), (*d).var_buffer.as_c_mut()) as u_int;
                        (*d).var_buf_pos = 0;
                    }
                    ROMMON_GET_VAR => {
                        (*d).var_status = rommon_var_get(addr_of_mut!((*vm).rommon_vars), (*d).var_buffer.as_c_mut(), (*d).var_buffer.as_c_mut(), (*d).var_buffer.len()) as u_int;
                        (*d).var_buf_pos = 0;
                    }
                    ROMMON_CLEAR_VAR_STAT => {
                        (*d).var_buf_pos = 0;
                    }
                    _ => {
                        (*d).var_status = -1 as c_int as u_int;
                    }
                }
            } else {
                *data = (*d).var_status as m_uint64_t;
            }
        }

        _ => {}
    }

    null_mut()
}

/// Shutdown a remote control device
unsafe extern "C" fn dev_remote_control_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut remote_data = d.cast::<_>();
    if !d.is_null() {
        dev_remove(vm, addr_of_mut!((*d).dev));
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// remote control device
#[no_mangle]
pub unsafe extern "C" fn dev_remote_control_init(vm: *mut vm_instance_t, paddr: m_uint64_t, len: m_uint32_t) -> c_int {
    let d: *mut remote_data = libc::malloc(size_of::<remote_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("Remote Control: unable to create device.\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<remote_data>());

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = cstr!("remote_ctrl");
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_remote_control_shutdown);

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = cstr!("remote_ctrl");
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_remote_control_access);
    (*d).dev.priv_data = d.cast::<_>();

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}
