//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::_private::*;
use crate::cpu::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::utils::*;
use crate::vm::*;

/// Device Flags
pub const VDEVICE_FLAG_NO_MTS_MMAP: c_int = 0x01; // Prevent MMAPed access by MTS
pub const VDEVICE_FLAG_CACHING: c_int = 0x02; // Device does support caching
pub const VDEVICE_FLAG_REMAP: c_int = 0x04; // Physical address remapping
pub const VDEVICE_FLAG_SYNC: c_int = 0x08; // Forced sync
pub const VDEVICE_FLAG_SPARSE: c_int = 0x10; // Sparse device
pub const VDEVICE_FLAG_GHOST: c_int = 0x20; // Ghost device

pub const VDEVICE_PTE_DIRTY: m_iptr_t = 0x01;

pub type dev_handler_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, dev: *mut vdevice, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void>;

/// Virtual Device
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vdevice {
    pub name: *mut c_char,
    pub id: u_int,
    pub phys_addr: m_uint64_t,
    pub phys_len: m_uint32_t,
    pub host_addr: m_iptr_t,
    pub priv_data: *mut c_void,
    pub flags: c_int,
    pub fd: c_int,
    pub handler: dev_handler_t,
    pub sparse_map: *mut m_iptr_t,
    pub next: *mut vdevice,
    pub pprev: *mut *mut vdevice,
}

use crate::dynamips::*;
use std::arch::*;

/// device access function
#[inline(always)]
#[no_mangle]
pub unsafe extern "C" fn dev_access_fast(cpu: *mut cpu_gen_t, dev_id: u_int, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        // MAC64HACK
        pub unsafe extern "C" fn __dev_access_fast(cpu: *mut cpu_gen_t, dev_id: u_int, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
            let dev: *mut vdevice = (*(*cpu).vm).dev_array[dev_id as usize];

            if unlikely(dev.is_null()) {
                cpu_log!(cpu, cstr!("dev_access_fast"), cstr!("null handler (dev_id=%u,offset=0x%x)\n"), dev_id, offset);
                return null_mut();
            }

            if DEBUG_DEV_PERF_CNT != 0 {
                (*cpu).dev_access_counter += 1;
            }

            (*dev).handler.unwrap()(cpu, dev, offset, op_size, op_type, data)
        }
        asm!("sub rsp, 8");
        let ret: *mut c_void = __dev_access_fast(cpu, dev_id, offset, op_size, op_type, data);
        asm!("add rsp, 8");
        ret
    } else {
        let dev: *mut vdevice = (*(*cpu).vm).dev_array[dev_id as usize];

        if unlikely(dev.is_null()) {
            cpu_log!(cpu, cstr!("dev_access_fast"), cstr!("null handler (dev_id=%u,offset=0x%x)\n"), dev_id, offset);
            return null_mut();
        }

        if DEBUG_DEV_PERF_CNT != 0 {
            (*cpu).dev_access_counter += 1;
        }

        (*dev).handler.unwrap()(cpu, dev, offset, op_size, op_type, data)
    }
}

const DEBUG_DEV_ACCESS: c_int = 0;

/// Get device by ID
#[no_mangle]
pub unsafe extern "C" fn dev_get_by_id(vm: *mut vm_instance_t, dev_id: u_int) -> *mut vdevice {
    if vm.is_null() || ((dev_id as usize) >= VM_DEVICE_MAX) {
        return null_mut();
    }

    (*vm).dev_array[dev_id as usize]
}

/// Get device by name
#[no_mangle]
pub unsafe extern "C" fn dev_get_by_name(vm: *mut vm_instance_t, name: *mut c_char) -> *mut vdevice {
    let mut dev: *mut vdevice;

    if vm.is_null() {
        return null_mut();
    }

    dev = (*vm).dev_list;
    while !dev.is_null() {
        if libc::strcmp((*dev).name, name) == 0 {
            return dev;
        }
        dev = (*dev).next;
    }

    null_mut()
}

/// Device lookup by physical address
#[no_mangle]
pub unsafe extern "C" fn dev_lookup(vm: *mut vm_instance_t, phys_addr: m_uint64_t, cached: c_int) -> *mut vdevice {
    let mut dev: *mut vdevice;

    if vm.is_null() {
        return null_mut();
    }

    dev = (*vm).dev_list;
    while !dev.is_null() {
        if cached != 0 && ((*dev).flags & VDEVICE_FLAG_CACHING) == 0 {
            dev = (*dev).next;
            continue;
        }

        if (phys_addr >= (*dev).phys_addr) && ((phys_addr - (*dev).phys_addr) < (*dev).phys_len as m_uint64_t) {
            return dev;
        }
        dev = (*dev).next;
    }

    null_mut()
}

/// Find the next device after the specified address
#[no_mangle]
pub unsafe extern "C" fn dev_lookup_next(vm: *mut vm_instance_t, phys_addr: m_uint64_t, dev_start: *mut vdevice, cached: c_int) -> *mut vdevice {
    let mut dev: *mut vdevice;

    if vm.is_null() {
        return null_mut();
    }

    dev = if !dev_start.is_null() { dev_start } else { (*vm).dev_list };
    while !dev.is_null() {
        if cached != 0 && ((*dev).flags & VDEVICE_FLAG_CACHING) == 0 {
            dev = (*dev).next;
            continue;
        }

        if (*dev).phys_addr > phys_addr {
            return dev;
        }
        dev = (*dev).next;
    }

    null_mut()
}

/// Initialize a device
#[no_mangle]
pub unsafe extern "C" fn dev_init(dev: *mut vdevice) {
    libc::memset(dev.cast::<_>(), 0, size_of::<vdevice>());
    (*dev).fd = -1;
}

/// Allocate a device
#[no_mangle]
pub unsafe extern "C" fn dev_create(name: *mut c_char) -> *mut vdevice {
    let dev: *mut vdevice = libc::malloc(size_of::<vdevice>()).cast::<_>();
    if dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("dev_create: insufficient memory to create device '%s'.\n"), name);
        return null_mut();
    }

    dev_init(dev);
    (*dev).name = name;
    dev
}

/// Remove a device
#[no_mangle]
pub unsafe extern "C" fn dev_remove(vm: *mut vm_instance_t, dev: *mut vdevice) {
    if dev.is_null() {
        return;
    }

    vm_unbind_device(vm, dev);

    vm_log!(vm, cstr!("DEVICE"), cstr!("Removal of device %s, fd=%d, host_addr=0x%llx, flags=%d\n"), (*dev).name, (*dev).fd, (*dev).host_addr as m_uint64_t, (*dev).flags);

    if ((*dev).flags & VDEVICE_FLAG_REMAP) != 0 {
        dev_init(dev);
        return;
    }

    if ((*dev).flags & VDEVICE_FLAG_SPARSE) != 0 {
        dev_sparse_shutdown(dev);

        if ((*dev).flags & VDEVICE_FLAG_GHOST) != 0 {
            vm_ghost_image_release((*dev).fd);
            dev_init(dev);
            return;
        }
    }

    if (*dev).fd != -1 {
        // Unmap memory mapped file
        if (*dev).host_addr != 0 {
            if ((*dev).flags & VDEVICE_FLAG_SYNC) != 0 {
                memzone_sync_all((*dev).host_addr as *mut c_void, (*dev).phys_len as size_t);
            }

            vm_log!(vm, cstr!("MMAP"), cstr!("unmapping of device '%s', fd=%d, host_addr=0x%llx, len=0x%x\n"), (*dev).name, (*dev).fd, (*dev).host_addr as m_uint64_t, (*dev).phys_len);
            memzone_unmap((*dev).host_addr as *mut c_void, (*dev).phys_len as size_t);
        }

        if ((*dev).flags & VDEVICE_FLAG_SYNC) != 0 {
            libc::fsync((*dev).fd);
        }

        libc::close((*dev).fd);
    } else {
        // Use of malloc'ed host memory: free it
        if (*dev).host_addr != 0 {
            libc::free((*dev).host_addr as *mut c_void);
        }
    }

    // reinitialize the device to a clean state
    dev_init(dev);
}

/// Show properties of a device
#[no_mangle]
pub unsafe extern "C" fn dev_show(dev: *mut vdevice) {
    if dev.is_null() {
        return;
    }

    libc::printf(cstr!("   %-18s: 0x%12.12llx (0x%8.8x)\n"), (*dev).name, (*dev).phys_addr, (*dev).phys_len);
}

/// Show the device list
#[no_mangle]
pub unsafe extern "C" fn dev_show_list(vm: *mut vm_instance_t) {
    let mut dev: *mut vdevice;

    libc::printf(cstr!("\nVM \"%s\" (%u) Device list:\n"), (*vm).name, (*vm).instance_id);

    dev = (*vm).dev_list;
    while !dev.is_null() {
        dev_show(dev);
        dev = (*dev).next;
    }

    libc::printf(cstr!("\n"));
}

/// device access function
#[no_mangle]
pub unsafe extern "C" fn dev_access(cpu: *mut cpu_gen_t, dev_id: u_int, offset: m_uint32_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let dev: *mut vdevice = (*(*cpu).vm).dev_array[dev_id as usize];

    if DEBUG_DEV_ACCESS != 0 {
        cpu_log!(cpu, cstr!("DEV_ACCESS"), cstr!("%s: dev_id=%u, offset=0x%8.8x, op_size=%u, op_type=%u, data=%p\n"), (*dev).name, dev_id, offset, op_size, op_type, data);
    }

    (*dev).handler.unwrap()(cpu, dev, offset, op_size, op_type, data)
}

/// Synchronize memory for a memory-mapped (mmap) device
#[no_mangle]
pub unsafe extern "C" fn dev_sync(dev: *mut vdevice) -> c_int {
    if dev.is_null() || (*dev).host_addr == 0 {
        return -1;
    }

    memzone_sync((*dev).host_addr as *mut c_void, (*dev).phys_len as size_t)
}

/// Remap a device at specified physical address
#[no_mangle]
pub unsafe extern "C" fn dev_remap(name: *mut c_char, orig: *mut vdevice, paddr: m_uint64_t, len: m_uint32_t) -> *mut vdevice {
    let dev: *mut vdevice = dev_create(name);
    if dev.is_null() {
        return null_mut();
    }

    (*dev).phys_addr = paddr;
    (*dev).phys_len = len;
    (*dev).flags = (*orig).flags | VDEVICE_FLAG_REMAP;
    (*dev).fd = (*orig).fd;
    (*dev).host_addr = (*orig).host_addr;
    (*dev).handler = (*orig).handler;
    (*dev).sparse_map = (*orig).sparse_map;
    dev
}

/// Create a RAM device
#[no_mangle]
pub unsafe extern "C" fn dev_create_ram(vm: *mut vm_instance_t, name: *mut c_char, sparse: c_int, filename: *mut c_char, paddr: m_uint64_t, len: m_uint32_t) -> *mut vdevice {
    let mut ram_ptr: *mut u_char = null_mut();

    let dev: *mut vdevice = dev_create(name);
    if dev.is_null() {
        return null_mut();
    }

    (*dev).phys_addr = paddr;
    (*dev).phys_len = len;
    (*dev).flags = VDEVICE_FLAG_CACHING;

    if sparse == 0 {
        if !filename.is_null() {
            (*dev).fd = memzone_create_file(filename, (*dev).phys_len as size_t, addr_of_mut!(ram_ptr));

            if (*dev).fd == -1 {
                libc::perror(cstr!("dev_create_ram: mmap"));
                libc::free(dev.cast::<_>());
                return null_mut();
            }

            (*dev).host_addr = ram_ptr as m_iptr_t;
        } else {
            (*dev).host_addr = m_memalign(4096, (*dev).phys_len as size_t) as m_iptr_t;
        }

        if (*dev).host_addr == 0 {
            libc::free(dev.cast::<_>());
            return null_mut();
        }
    } else {
        dev_sparse_init(dev);
    }

    vm_bind_device(vm, dev);
    dev
}

/// Create a ghosted RAM device
#[no_mangle]
pub unsafe extern "C" fn dev_create_ghost_ram(vm: *mut vm_instance_t, name: *mut c_char, sparse: c_int, filename: *mut c_char, paddr: m_uint64_t, len: m_uint32_t) -> *mut vdevice {
    let mut ram_ptr: *mut u_char = null_mut();

    let dev: *mut vdevice = dev_create(name);
    if dev.is_null() {
        return null_mut();
    }

    (*dev).phys_addr = paddr;
    (*dev).phys_len = len;
    (*dev).flags = VDEVICE_FLAG_CACHING | VDEVICE_FLAG_GHOST;

    if sparse == 0 {
        (*dev).fd = memzone_open_cow_file(filename, (*dev).phys_len as size_t, addr_of_mut!(ram_ptr));
        if (*dev).fd == -1 {
            libc::perror(cstr!("dev_create_ghost_ram: mmap"));
            libc::free(dev.cast::<_>());
            return null_mut();
        }

        (*dev).host_addr = ram_ptr as m_iptr_t;
        if (*dev).host_addr == 0 {
            libc::free(dev.cast::<_>());
            return null_mut();
        }
    } else {
        if vm_ghost_image_get(filename, addr_of_mut!(ram_ptr), addr_of_mut!((*dev).fd)) == -1 {
            libc::free(dev.cast::<_>());
            return null_mut();
        }

        (*dev).host_addr = ram_ptr as m_iptr_t;
        dev_sparse_init(dev);
    }

    vm_bind_device(vm, dev);
    dev
}

/// Create a memory alias
#[no_mangle]
pub unsafe extern "C" fn dev_create_ram_alias(vm: *mut vm_instance_t, name: *mut c_char, orig: *mut c_char, paddr: m_uint64_t, len: m_uint32_t) -> *mut vdevice {
    // try to locate the device
    let orig_dev: *mut vdevice = dev_get_by_name(vm, orig);
    if orig_dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("VM%u: dev_create_ram_alias: unknown device '%s'.\n"), (*vm).instance_id, orig);
        return null_mut();
    }

    let dev: *mut vdevice = dev_remap(name, orig_dev, paddr, len);
    if dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("VM%u: dev_create_ram_alias: unable to create new device %s.\n"), (*vm).instance_id, name);
        return null_mut();
    }

    vm_bind_device(vm, dev);
    dev
}

/// Initialize a sparse device
#[no_mangle]
pub unsafe extern "C" fn dev_sparse_init(dev: *mut vdevice) -> c_int {
    // create the sparse mapping
    let nr_pages: u_int = normalize_size((*dev).phys_len, VM_PAGE_SIZE as u_int, VM_PAGE_SHIFT);
    let len: size_t = nr_pages as size_t * size_of::<m_iptr_t>();

    (*dev).sparse_map = libc::malloc(len).cast::<_>();
    if (*dev).sparse_map.is_null() {
        return -1;
    }

    if (*dev).host_addr == 0 {
        libc::memset((*dev).sparse_map.cast::<_>(), 0, len);
    } else {
        for i in 0..nr_pages {
            *(*dev).sparse_map.add(i as usize) = (*dev).host_addr + (i << VM_PAGE_SHIFT) as m_iptr_t;
        }
    }

    (*dev).flags |= VDEVICE_FLAG_SPARSE;
    0
}

/// Shutdown sparse device structures
#[no_mangle]
pub unsafe extern "C" fn dev_sparse_shutdown(dev: *mut vdevice) -> c_int {
    if ((*dev).flags & VDEVICE_FLAG_SPARSE) == 0 {
        return -1;
    }

    libc::free((*dev).sparse_map.cast::<_>());
    (*dev).sparse_map = null_mut();
    0
}

/// Show info about a sparse device
#[no_mangle]
pub unsafe extern "C" fn dev_sparse_show_info(dev: *mut vdevice) -> c_int {
    let mut dirty_pages: u_int;

    libc::printf(cstr!("Sparse information for device '%s':\n"), (*dev).name);

    if ((*dev).flags & VDEVICE_FLAG_SPARSE) == 0 {
        libc::printf(cstr!("This is not a sparse device.\n"));
        return -1;
    }

    if (*dev).sparse_map.is_null() {
        libc::printf(cstr!("No sparse map.\n"));
        return -1;
    }

    let nr_pages: u_int = normalize_size((*dev).phys_len, VM_PAGE_SIZE as u_int, VM_PAGE_SHIFT);
    dirty_pages = 0;

    for i in 0..nr_pages {
        if (*(*dev).sparse_map.add(i as usize) & VDEVICE_PTE_DIRTY) != 0 {
            dirty_pages += 1;
        }
    }

    libc::printf(cstr!("%u dirty pages on a total of %u pages.\n"), dirty_pages, nr_pages);
    0
}

/// Get an host address for a sparse device
#[no_mangle]
pub unsafe extern "C" fn dev_sparse_get_host_addr(vm: *mut vm_instance_t, dev: *mut vdevice, paddr: m_uint64_t, op_type: u_int, cow: *mut c_int) -> m_iptr_t {
    let mut ptr: m_iptr_t;

    let offset: u_int = ((paddr - (*dev).phys_addr) >> VM_PAGE_SHIFT) as u_int;
    ptr = *(*dev).sparse_map.add(offset as usize);
    *cow = 0;

    // If the device is not in COW mode, allocate a host page if the physical
    // page is requested for the first time.
    if (*dev).host_addr == 0 {
        if (ptr & VDEVICE_PTE_DIRTY) == 0 {
            ptr = vm_alloc_host_page(vm) as m_iptr_t;
            assert!(ptr != 0);

            *(*dev).sparse_map.add(offset as usize) = ptr | VDEVICE_PTE_DIRTY;
            return ptr;
        }

        return ptr & VM_PAGE_MASK as m_iptr_t;
    }

    // We have a "ghost" base. We apply the copy-on-write (COW) mechanism
    // ourselves.
    if (ptr & VDEVICE_PTE_DIRTY) != 0 {
        return ptr & VM_PAGE_MASK as m_iptr_t;
    }

    if op_type == MTS_READ {
        *cow = 1;
        return ptr & VM_PAGE_MASK as m_iptr_t;
    }

    // Write attempt on a "ghost" page. Duplicate it
    let ptr_new: m_iptr_t = vm_alloc_host_page(vm) as m_iptr_t;
    assert!(ptr_new != 0);

    libc::memcpy(ptr_new as *mut c_void, (ptr & VM_PAGE_MASK as m_iptr_t) as *mut c_void, VM_PAGE_SIZE);
    *(*dev).sparse_map.add(offset as usize) = ptr_new | VDEVICE_PTE_DIRTY;
    ptr_new
}

/// Get virtual address space used on host for the specified device
#[no_mangle]
pub unsafe extern "C" fn dev_get_vspace_size(dev: *mut vdevice) -> size_t {
    // if the device is simply remapped, don't count it
    if ((*dev).flags & VDEVICE_FLAG_REMAP) != 0 {
        return 0;
    }

    if (*dev).host_addr != 0 || ((*dev).flags & VDEVICE_FLAG_SPARSE) != 0 {
        return ((*dev).phys_len >> 10) as size_t;
    }

    0
}

/// dummy console handler
unsafe extern "C" fn dummy_console_handler(_cpu: *mut cpu_gen_t, _dev: *mut vdevice, offset: m_uint32_t, _op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    match offset {
        0x40c => {
            if op_type == MTS_READ {
                *data = 0x04; // tx ready
            }
        }

        0x41c => {
            if op_type == MTS_WRITE {
                libc::printf(cstr!("%c"), (*data & 0xff) as u_char as c_int);
                libc::fflush(c_stdout());
            }
        }

        _ => {}
    }

    null_mut()
}

/// Create a dummy console
#[no_mangle]
pub unsafe extern "C" fn dev_create_dummy_console(vm: *mut vm_instance_t) -> c_int {
    let dev: *mut vdevice = dev_create(cstr!("dummy_console"));
    if dev.is_null() {
        return -1;
    }

    (*dev).phys_addr = 0x1e840000; // 0x1f000000;
    (*dev).phys_len = 4096;
    (*dev).handler = Some(dummy_console_handler);

    vm_bind_device(vm, dev);
    0
}
