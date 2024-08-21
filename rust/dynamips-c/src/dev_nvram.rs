//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot.  All rights reserved.
//!
//! Dallas DS1216 chip emulation:
//!   - NVRAM
//!   - Calendar
//!
//! Manuals:
//!    http://pdfserv.maxim-ic.com/en/ds/DS1216-DS1216H.pdf
//!
//! Calendar stuff written by Mtve.

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::fs_nvram::*;
use crate::memory::*;
use crate::utils::*;
use crate::vm::*;

const DEBUG_ACCESS: c_int = 0;

/// SmartWatch pattern (p.5 of documentation)
const PATTERN: u64 = 0x5ca33ac55ca33ac5_u64;

/// NVRAM private data
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nvram_data {
    pub vm_obj: vm_obj_t,
    pub dev: vdevice,
    pub filename: *mut c_char,
    pub cal_state: u_int,
    pub cal_read: m_uint64_t,
    pub cal_write: m_uint64_t,
}

/// Convert an 8-bit number to a BCD form
unsafe fn u8_to_bcd(val: m_uint8_t) -> m_uint8_t {
    ((val / 10) << 4) + (val % 10)
}

/// Get the current time (p.8)
unsafe fn get_current_time(_cpu: *mut cpu_gen_t) -> m_uint64_t {
    let mut res: m_uint64_t;
    let mut tmx: libc::tm = zeroed::<_>();
    let mut spec: libc::timespec = zeroed::<_>();

    libc::clock_gettime(libc::CLOCK_REALTIME, addr_of_mut!(spec));
    libc::gmtime_r(addr_of_mut!(spec.tv_sec), addr_of_mut!(tmx));

    res = (u8_to_bcd(tmx.tm_sec as m_uint8_t) as m_uint64_t) << 8;
    res += (u8_to_bcd(tmx.tm_min as m_uint8_t) as m_uint64_t) << 16;
    res += (u8_to_bcd(tmx.tm_hour as m_uint8_t) as m_uint64_t) << 24;
    res += (u8_to_bcd(tmx.tm_wday as m_uint8_t) as m_uint64_t) << 32;
    res += (u8_to_bcd(tmx.tm_mday as m_uint8_t) as m_uint64_t) << 40;
    res += (u8_to_bcd((tmx.tm_mon + 1) as m_uint8_t) as m_uint64_t) << 48;
    res += (u8_to_bcd(tmx.tm_year as m_uint8_t) as m_uint64_t) << 56;

    res
}

/// dev_nvram_access()
unsafe extern "C" fn dev_nvram_access(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let d: *mut nvram_data = (*dev).priv_data.cast::<_>();

    if DEBUG_ACCESS != 0 {
        if op_type == MTS_READ {
            cpu_log!(cpu, (*dev).name, cstr!("read  access to offset=0x%x, pc=0x%llx\n"), offset, cpu_get_pc(cpu));
        } else {
            cpu_log!(cpu, (*dev).name, cstr!("write access to vaddr=0x%x, pc=0x%llx, val=0x%llx\n"), offset, cpu_get_pc(cpu), *data);
        }
    }

    #[allow(clippy::single_match)]
    match offset {
        0x03 => {
            if op_type == MTS_READ {
                *data = (*d).cal_read & 1;
                (*d).cal_read >>= 1;
            } else {
                (*d).cal_write >>= 1;
                (*d).cal_write |= *data << 63;

                if (*d).cal_write == PATTERN {
                    (*d).cal_state = 1;
                    vm_log!((*cpu).vm, cstr!("Calendar"), cstr!("reset\n"));
                    (*d).cal_read = get_current_time(cpu);
                } else if (*d).cal_state > 0 {
                    if (*d).cal_state == 64 {
                        // untested
                        vm_log!((*cpu).vm, cstr!("Calendar"), cstr!("set 0x%016llx\n"), (*d).cal_write);
                        (*d).cal_state = 0;
                    } else {
                        (*d).cal_state += 1;
                    }
                }
            }
            return null_mut();
        }
        _ => {}
    }

    ((*dev).host_addr + offset as m_iptr_t) as *mut c_void
}

/// Set appropriately the config register if the NVRAM is empty
unsafe fn set_config_register(dev: *mut vdevice, conf_reg: *mut u_int) {
    let mut ptr: *mut m_uint32_t;

    ptr = (*dev).host_addr as *mut m_uint32_t;
    for _ in 0..((*dev).phys_len / 4) {
        if *ptr != 0 {
            return;
        }
        ptr = ptr.add(1);
    }

    // nvram is empty: tells IOS to ignore its contents.
    // http://www.cisco.com/en/US/products/hw/routers/ps274/products_installation_guide_chapter09186a008007de4c.html
    *conf_reg |= 0x0040;
    libc::printf(cstr!("NVRAM is empty, setting config register to 0x%x\n"), *conf_reg);
}

/// Shutdown the NVRAM device
unsafe extern "C" fn dev_nvram_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut nvram_data = d.cast::<_>();
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

/// Create the NVRAM device
#[no_mangle]
pub unsafe extern "C" fn dev_nvram_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t, conf_reg: *mut u_int) -> c_int {
    let mut ptr: *mut u_char = null_mut();

    // allocate the private data structure
    let d: *mut nvram_data = libc::malloc(size_of::<nvram_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("NVRAM: out of memory\n"));
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<nvram_data>());

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_nvram_shutdown);

    (*d).filename = vm_build_filename(vm, name);
    if !(*d).filename.is_null() {
        libc::fprintf(c_stderr(), cstr!("NVRAM: unable to create filename.\n"));
        libc::free(d.cast::<_>());
        return -1;
    }

    dev_init(addr_of_mut!((*d).dev));
    (*d).dev.name = name;
    (*d).dev.phys_addr = paddr;
    (*d).dev.phys_len = len;
    (*d).dev.handler = Some(dev_nvram_access);
    (*d).dev.fd = memzone_create_file((*d).filename, (*d).dev.phys_len as size_t, addr_of_mut!(ptr));
    (*d).dev.host_addr = ptr as m_iptr_t;
    (*d).dev.flags = VDEVICE_FLAG_NO_MTS_MMAP | VDEVICE_FLAG_SYNC;
    (*d).dev.priv_data = d.cast::<_>();

    if (*d).dev.fd == -1 {
        libc::fprintf(c_stderr(), cstr!("NVRAM: unable to map file '%s'\n"), (*d).filename);
        libc::free(d.cast::<_>());
        return -1;
    }

    // Modify the configuration register if NVRAM is empty
    set_config_register(addr_of_mut!((*d).dev), conf_reg);

    // Map this device to the VM
    vm_bind_device(vm, addr_of_mut!((*d).dev));
    vm_object_add(vm, addr_of_mut!((*d).vm_obj));
    0
}

/// Compute NVRAM checksum
unsafe fn nvram_cksum_old(vm: *mut vm_instance_t, mut addr: m_uint64_t, mut count: size_t) -> m_uint16_t {
    let mut sum: m_uint32_t = 0;

    while count > 1 {
        sum += physmem_copy_u16_from_vm(vm, addr) as m_uint32_t;
        addr += size_of::<m_uint16_t>() as m_uint64_t;
        count -= size_of::<m_uint16_t>();
    }

    if count > 0 {
        sum += ((physmem_copy_u16_from_vm(vm, addr) & 0xFF) as m_uint32_t) << 8;
    }

    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !sum as m_uint16_t
}

/// Generic function for implementations of vm->platform->nvram_extract_config.
/// If nvram_size is 0, it is set based on the file size.
/// @param dev_name     Device name
/// @param nvram_offset Where the filesystem starts
/// @param nvram_size   Size of the filesystem
#[no_mangle]
pub unsafe extern "C" fn generic_nvram_extract_config(
    vm: *mut vm_instance_t,
    dev_name: *mut c_char,
    nvram_offset: size_t,
    mut nvram_size: size_t,
    addr: m_uint32_t,
    format: u_int,
    startup_config: *mut *mut u_char,
    startup_len: *mut size_t,
    private_config: *mut *mut u_char,
    private_len: *mut size_t,
) -> c_int {
    // XXX add const to dev_name
    let mut base_ptr: *mut u_char = null_mut();
    let mut file_size: libc::off_t = 0;
    let mut ret: c_int = 0;

    let nvram_dev: *mut vdevice = dev_get_by_name(vm, dev_name);
    if !nvram_dev.is_null() {
        dev_sync(nvram_dev);
    }

    let fd: c_int = vm_mmap_open_file(vm, dev_name, addr_of_mut!(base_ptr), addr_of_mut!(file_size));
    if fd == -1 {
        return -1;
    }

    if nvram_size == 0 && (file_size as size_t) >= nvram_offset + FS_NVRAM_SECTOR_SIZE {
        nvram_size = file_size as size_t - nvram_offset;
    }

    if (file_size as size_t) < nvram_offset + nvram_size {
        vm_error!(vm, cstr!("generic_nvram_extract_config: NVRAM filesystem doesn't fit inside the %s file!\n"), dev_name);

        vm_mmap_close_file(fd, base_ptr, file_size as size_t);

        return ret;
    }

    // normal + backup
    let fs: *mut fs_nvram_t = fs_nvram_open(base_ptr.add(nvram_offset), nvram_size, addr, format & FS_NVRAM_FORMAT_MASK);
    if fs.is_null() {
        ret = c_errno();
        vm_error!(vm, cstr!("generic_nvram_extract_config: %s\n"), libc::strerror(ret));
        ret = -1;

        vm_mmap_close_file(fd, base_ptr, file_size as size_t);

        return ret;
    }

    ret = fs_nvram_read_config(fs, startup_config, startup_len, private_config, private_len);
    if ret != 0 {
        vm_error!(vm, cstr!("generic_nvram_extract_config: %s\n"), libc::strerror(ret));
        ret = -1;

        fs_nvram_close(fs);
    }

    vm_mmap_close_file(fd, base_ptr, file_size as size_t);

    ret
}

/// Generic function for implementations of vm->platform->nvram_push_config.
/// If nvram_size is 0, it is set based on the file size.
/// Preserves startup-config if startup_config is NULL.
/// Preserves private-config if private_config is NULL.
/// @param dev_name     Device name
/// @param file_size    File size
/// @param nvram_offset Where the filesystem starts
/// @param nvram_size   Size of the filesystem
#[no_mangle]
pub unsafe extern "C" fn generic_nvram_push_config(
    vm: *mut vm_instance_t,
    dev_name: *mut c_char,
    file_size: size_t,
    nvram_offset: size_t,
    mut nvram_size: size_t,
    addr: m_uint32_t,
    format: u_int,
    mut startup_config: *mut u_char,
    mut startup_len: size_t,
    mut private_config: *mut u_char,
    mut private_len: size_t,
) -> c_int {
    // XXX add const to dev_name
    let mut base_ptr: *mut u_char = null_mut();
    let mut ret: c_int;
    let mut prev_startup_config: *mut u_char = null_mut();
    let mut prev_private_config: *mut u_char = null_mut();
    let mut prev_startup_len: size_t = 0;
    let mut prev_private_len: size_t = 0;

    if nvram_size == 0 && file_size >= nvram_offset + FS_NVRAM_SECTOR_SIZE {
        nvram_size = file_size - nvram_offset;
    }

    if file_size < nvram_offset + nvram_size {
        vm_error!(vm, cstr!("generic_nvram_push_config: NVRAM filesystem doesn't fit inside the %s file!\n"), dev_name);
        return -1;
    }

    let fd: c_int = vm_mmap_create_file(vm, dev_name, file_size, addr_of_mut!(base_ptr));
    if fd == -1 {
        return -1;
    }

    let fs: *mut fs_nvram_t = fs_nvram_open(base_ptr.add(nvram_offset), nvram_size, addr, (format & FS_NVRAM_FORMAT_MASK) | FS_NVRAM_FLAG_OPEN_CREATE);
    if fs.is_null() {
        ret = c_errno();
        vm_error!(vm, cstr!("generic_nvram_push_config: %s\n"), libc::strerror(ret));
        ret = -1;

        vm_mmap_close_file(fd, base_ptr, file_size);

        return ret;
    }

    ret = fs_nvram_read_config(fs, addr_of_mut!(prev_startup_config), addr_of_mut!(prev_startup_len), addr_of_mut!(prev_private_config), addr_of_mut!(prev_private_len));
    if ret != 0 {
        vm_error!(vm, cstr!("generic_nvram_push_config: %s\n"), libc::strerror(ret));
        ret = -1;

        fs_nvram_close(fs);

        if !prev_startup_config.is_null() {
            libc::free(prev_startup_config.cast::<_>());
        }

        if !prev_private_config.is_null() {
            libc::free(prev_private_config.cast::<_>());
        }

        vm_mmap_close_file(fd, base_ptr, file_size);

        return ret;
    }

    if startup_config.is_null() {
        startup_config = prev_startup_config;
        startup_len = prev_startup_len;
    }

    if private_config.is_null() {
        private_config = prev_private_config;
        private_len = prev_private_len;
    }

    ret = fs_nvram_write_config(fs, startup_config, startup_len, private_config, private_len);
    if ret != 0 {
        vm_error!(vm, cstr!("generic_nvram_push_config: %s\n"), libc::strerror(ret));
        ret = -1;
    }

    fs_nvram_close(fs);

    if !prev_startup_config.is_null() {
        libc::free(prev_startup_config.cast::<_>());
    }

    if !prev_private_config.is_null() {
        libc::free(prev_private_config.cast::<_>());
    }

    vm_mmap_close_file(fd, base_ptr, file_size);

    ret
}
