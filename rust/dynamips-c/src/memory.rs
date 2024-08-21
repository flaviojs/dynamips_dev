//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::utils::*;
use crate::vm::*;
#[cfg(feature = "USE_UNSTABLE")]
use std::ops::Shr;

/// MTS operation
pub const MTS_READ: u_int = 0;
pub const MTS_WRITE: u_int = 1;

/// 0.5GB value
pub const MTS_SIZE_512M: u_int = 0x20000000;

/// MTS flag bits: D (device), ACC (memory access), C (chain)
pub const MTS_FLAG_BITS: c_int = 4;
pub const MTS_FLAG_MASK: u_long = 0x0000000f_u64 as u_long;

/// Masks for MTS entries
pub const MTS_CHAIN_MASK: u_int = 0x00000001;
pub const MTS_ACC_MASK: u_int = 0x00000006;
pub const MTS_DEV_MASK: u_int = 0x00000008;
pub const MTS_ADDR_MASK: u_long = !MTS_FLAG_MASK;

/// Device ID mask and shift, device offset mask
pub const MTS_DEVID_MASK: u_int = 0xfc000000;
pub const MTS_DEVID_SHIFT: c_int = 26;
pub const MTS_DEVOFF_MASK: u_int = 0x03ffffff;

/// Memory access flags
pub const MTS_ACC_AE: u_int = 0x00000002; // Address Error
pub const MTS_ACC_T: u_int = 0x00000004; // TLB Exception
pub const MTS_ACC_U: u_int = 0x00000006; // Unexistent

/// Macro for easy hash computing
#[cfg(feature = "USE_UNSTABLE")]
#[inline]
unsafe fn MTS_SHR<T: Shr<c_int, Output = T>>(v: T, sr: c_int) -> T {
    v >> sr
}

/// Hash table size for MTS64 (default: [shift:16,bits:12])
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS64_HASH_SHIFT: c_int = 12;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS64_HASH_BITS: c_int = 14;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS64_HASH_SIZE: m_uint32_t = 1 << MTS64_HASH_BITS;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS64_HASH_MASK: m_uint32_t = MTS64_HASH_SIZE - 1;

/// Hash table size for MTS64 (default: [shift:16,bits:12])
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_SHIFT1: c_int = 12;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_SHIFT2: c_int = 20;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_BITS: c_int = 8;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_SIZE: m_uint32_t = 1 << MTS64_HASH_BITS;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS64_HASH_MASK: m_uint32_t = MTS64_HASH_SIZE - 1;

/// MTS64 hash on virtual addresses
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn MTS64_HASH(vaddr: m_uint64_t) -> m_uint32_t {
    (vaddr >> MTS64_HASH_SHIFT) as m_uint32_t & MTS64_HASH_MASK
}

/// MTS64 hash on virtual addresses
#[cfg(feature = "USE_UNSTABLE")]
macro_rules! MTS64_SHR {
    ($v:expr, $i:expr) => {
        paste! {
            MTS_SHR($v, [<MTS64_HASH_SHIFT $i>])
        }
    };
}
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn MTS64_HASH(vaddr: m_uint64_t) -> m_uint32_t {
    (MTS64_SHR!(vaddr, 1) ^ MTS64_SHR!(vaddr, 2)) as m_uint32_t & MTS64_HASH_MASK
}

/// Hash table size for MTS32 (default: [shift:15,bits:15])
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS32_HASH_SHIFT: c_int = 12;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS32_HASH_BITS: c_int = 14;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS32_HASH_SIZE: m_uint32_t = 1 << MTS32_HASH_BITS;
#[cfg(not(feature = "USE_UNSTABLE"))]
pub const MTS32_HASH_MASK: m_uint32_t = MTS32_HASH_SIZE - 1;

/// Hash table size for MTS32 (default: [shift:15,bits:15])
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_SHIFT1: c_int = 12;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_SHIFT2: c_int = 20;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_BITS: c_int = 8;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_SIZE: m_uint32_t = 1 << MTS32_HASH_BITS;
#[cfg(feature = "USE_UNSTABLE")]
pub const MTS32_HASH_MASK: m_uint32_t = MTS32_HASH_SIZE - 1;

/// MTS32 hash on virtual addresses
#[cfg(not(feature = "USE_UNSTABLE"))]
#[no_mangle]
pub unsafe extern "C" fn MTS32_HASH(vaddr: m_uint32_t) -> m_uint32_t {
    (vaddr >> MTS32_HASH_SHIFT) & MTS32_HASH_MASK
}

/// MTS32 hash on virtual addresses
#[cfg(feature = "USE_UNSTABLE")]
macro_rules! MTS32_SHR {
    ($v:expr, $i:expr) => {
        paste! {
            MTS_SHR($v, [<MTS32_HASH_SHIFT $i>])
        }
    };
}
#[cfg(feature = "USE_UNSTABLE")]
#[no_mangle]
pub unsafe extern "C" fn MTS32_HASH(vaddr: m_uint32_t) -> m_uint32_t {
    (MTS32_SHR!(vaddr, 1) ^ MTS32_SHR!(vaddr, 2)) & MTS32_HASH_MASK
}

/// Number of entries per chunk
pub const MTS64_CHUNK_SIZE: usize = 256;
pub const MTS32_CHUNK_SIZE: usize = 256;

/// MTS64: chunk definition
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct mts64_chunk {
    pub entry: [mts64_entry_t; MTS64_CHUNK_SIZE],
    pub next: *mut mts64_chunk,
    pub count: u_int,
}

/// MTS32: chunk definition
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
pub struct mts32_chunk {
    pub entry: [mts32_entry_t; MTS32_CHUNK_SIZE],
    pub next: *mut mts32_chunk,
    pub count: u_int,
}

/// Record a memory access
#[no_mangle]
pub unsafe extern "C" fn memlog_rec_access(cpu: *mut cpu_gen_t, vaddr: m_uint64_t, data: m_uint64_t, op_size: m_uint32_t, op_type: m_uint32_t) {
    let acc: *mut memlog_access_t = addr_of_mut!((*cpu).memlog_array[(*cpu).memlog_pos as usize]);
    (*acc).iaddr = cpu_get_pc(cpu);
    (*acc).vaddr = vaddr;
    (*acc).data = data;
    (*acc).op_size = op_size;
    (*acc).op_type = op_type;
    (*acc).data_valid = (op_type == MTS_WRITE) as m_uint32_t;

    (*cpu).memlog_pos = ((*cpu).memlog_pos + 1) & (MEMLOG_COUNT as m_uint32_t - 1);
}

/// Show the latest memory accesses
#[no_mangle]
pub unsafe extern "C" fn memlog_dump(cpu: *mut cpu_gen_t) {
    let mut acc: *mut memlog_access_t;
    let mut s_data: [c_char; 64] = [0; 64];
    let mut pos: u_int;

    for i in 0..MEMLOG_COUNT as u_int {
        pos = (*cpu).memlog_pos + i;
        pos &= MEMLOG_COUNT as u_int - 1;
        acc = addr_of_mut!((*cpu).memlog_array[pos as usize]);

        if cpu_get_pc(cpu) != 0 {
            if (*acc).data_valid != 0 {
                libc::snprintf(s_data.as_c_mut(), s_data.len(), cstr!("0x%llx"), (*acc).data);
            } else {
                libc::snprintf(s_data.as_c_mut(), s_data.len(), cstr!("XXXXXXXX"));
            }

            libc::printf(cstr!("CPU%u: pc=0x%8.8llx, vaddr=0x%8.8llx, size=%u, type=%s, data=%s\n"), (*cpu).id, (*acc).iaddr, (*acc).vaddr, (*acc).op_size, if (*acc).op_type == MTS_READ { cstr!("read ") } else { cstr!("write") }, s_data.as_c());
        }
    }
}

/// Update the data obtained by a read access
#[no_mangle]
pub unsafe extern "C" fn memlog_update_read(cpu: *mut cpu_gen_t, raddr: m_iptr_t) {
    let acc: *mut memlog_access_t = addr_of_mut!((*cpu).memlog_array[((*cpu).memlog_pos - 1) as usize & (MEMLOG_COUNT - 1)]);

    if (*acc).op_type == MTS_READ {
        match (*acc).op_size {
            1 => (*acc).data = *(raddr as *mut m_uint8_t) as m_uint64_t,
            2 => (*acc).data = vmtoh16(*(raddr as *mut m_uint16_t)) as m_uint64_t,
            4 => (*acc).data = vmtoh32(*(raddr as *mut m_uint32_t)) as m_uint64_t,
            8 => (*acc).data = vmtoh64(*(raddr as *mut m_uint64_t)),
            _ => {}
        }

        (*acc).data_valid = TRUE as m_uint32_t;
    }
}

// === Operations on physical memory ======================================

/// Get host pointer for the physical address
#[inline]
unsafe fn physmem_get_hptr(vm: *mut vm_instance_t, paddr: m_uint64_t, op_size: u_int, op_type: u_int, data: *mut m_uint64_t) -> *mut c_void {
    let mut cow: c_int = 0;

    let dev: *mut vdevice = dev_lookup(vm, paddr, FALSE);
    if dev.is_null() {
        return null_mut();
    }

    if ((*dev).flags & VDEVICE_FLAG_SPARSE) != 0 {
        let ptr: *mut c_void = dev_sparse_get_host_addr(vm, dev, paddr, op_type, addr_of_mut!(cow)) as *mut c_void;
        if ptr.is_null() {
            return null_mut();
        }

        return ptr.add((paddr & VM_PAGE_IMASK) as usize);
    }

    if ((*dev).host_addr != 0) && ((*dev).flags & VDEVICE_FLAG_NO_MTS_MMAP) == 0 {
        return ((*dev).host_addr as *mut c_void).add((paddr - (*dev).phys_addr) as usize);
    }

    if op_size == 0 {
        return null_mut();
    }

    let offset: m_uint32_t = (paddr - (*dev).phys_addr) as m_uint32_t;
    (*dev).handler.unwrap()((*vm).boot_cpu, dev, offset, op_size, op_type, data)
}

/// Copy a memory block from VM physical RAM to real host
#[no_mangle]
pub unsafe extern "C" fn physmem_copy_from_vm(vm: *mut vm_instance_t, mut real_buffer: *mut c_void, mut paddr: m_uint64_t, mut len: size_t) {
    let mut dummy: m_uint64_t = 0;
    let mut r: m_uint32_t;
    let mut ptr: *mut u_char;

    while len > 0 {
        r = m_min(VM_PAGE_SIZE - (paddr & VM_PAGE_IMASK) as size_t, len) as m_uint32_t;
        ptr = physmem_get_hptr(vm, paddr, 0, MTS_READ, addr_of_mut!(dummy)).cast::<_>();

        if likely(!ptr.is_null()) {
            libc::memcpy(real_buffer, ptr.cast::<_>(), r as size_t);
        } else {
            r = m_min(len, 4) as m_uint32_t;
            match r {
                4 => {
                    *real_buffer.cast::<m_uint32_t>() = htovm32(physmem_copy_u32_from_vm(vm, paddr));
                }
                2 => {
                    *real_buffer.cast::<m_uint16_t>() = htovm16(physmem_copy_u16_from_vm(vm, paddr));
                }
                1 => {
                    *real_buffer.cast::<m_uint8_t>() = physmem_copy_u8_from_vm(vm, paddr);
                }
                _ => {}
            }
        }

        real_buffer = real_buffer.byte_add(r as usize);
        paddr += r as m_uint64_t;
        len -= r as size_t;
    }
}

/// Copy a memory block to VM physical RAM from real host
#[no_mangle]
pub unsafe extern "C" fn physmem_copy_to_vm(vm: *mut vm_instance_t, mut real_buffer: *mut c_void, mut paddr: m_uint64_t, mut len: size_t) {
    let mut dummy: m_uint64_t = 0;
    let mut r: m_uint32_t;
    let mut ptr: *mut u_char;

    while len > 0 {
        r = m_min(VM_PAGE_SIZE - (paddr & VM_PAGE_IMASK) as size_t, len) as m_uint32_t;
        ptr = physmem_get_hptr(vm, paddr, 0, MTS_WRITE, addr_of_mut!(dummy)).cast::<_>();

        if likely(!ptr.is_null()) {
            libc::memcpy(ptr.cast::<_>(), real_buffer, r as size_t);
        } else {
            r = m_min(len, 4) as m_uint32_t;
            match r {
                4 => {
                    physmem_copy_u32_to_vm(vm, paddr, htovm32(*real_buffer.cast::<m_uint32_t>()));
                }
                2 => {
                    physmem_copy_u16_to_vm(vm, paddr, htovm16(*real_buffer.cast::<m_uint16_t>()));
                }
                1 => {
                    physmem_copy_u8_to_vm(vm, paddr, *real_buffer.cast::<m_uint8_t>());
                }
                _ => {}
            }
        }

        real_buffer = real_buffer.byte_add(r as usize);
        paddr += r as m_uint64_t;
        len -= r as size_t;
    }
}

/// Copy a 32-bit word from the VM physical RAM to real host
#[no_mangle]
pub unsafe extern "C" fn physmem_copy_u32_from_vm(vm: *mut vm_instance_t, paddr: m_uint64_t) -> m_uint32_t {
    let mut tmp: m_uint64_t = 0;

    let ptr: *mut m_uint32_t = physmem_get_hptr(vm, paddr, 4, MTS_READ, addr_of_mut!(tmp)).cast::<_>();
    if ptr.is_null() {
        return vmtoh32(*ptr);
    }

    tmp as m_uint32_t
}

/// Copy a 32-bit word to the VM physical RAM from real host
#[no_mangle]
pub unsafe extern "C" fn physmem_copy_u32_to_vm(vm: *mut vm_instance_t, paddr: m_uint64_t, val: m_uint32_t) {
    let mut tmp: m_uint64_t = val as m_uint64_t;

    let ptr: *mut m_uint32_t = physmem_get_hptr(vm, paddr, 4, MTS_WRITE, addr_of_mut!(tmp)).cast::<_>();
    if !ptr.is_null() {
        *ptr = htovm32(val);
    }
}

/// Copy a 16-bit word from the VM physical RAM to real host
#[no_mangle]
pub unsafe extern "C" fn physmem_copy_u16_from_vm(vm: *mut vm_instance_t, paddr: m_uint64_t) -> m_uint16_t {
    let mut tmp: m_uint64_t = 0;

    let ptr: *mut m_uint16_t = physmem_get_hptr(vm, paddr, 2, MTS_READ, addr_of_mut!(tmp)).cast::<_>();
    if !ptr.is_null() {
        return vmtoh16(*ptr);
    }

    tmp as m_uint16_t
}

/// Copy a 16-bit word to the VM physical RAM from real host
#[no_mangle]
pub unsafe extern "C" fn physmem_copy_u16_to_vm(vm: *mut vm_instance_t, paddr: m_uint64_t, val: m_uint16_t) {
    let mut tmp: m_uint64_t = val as m_uint64_t;

    let ptr: *mut m_uint16_t = physmem_get_hptr(vm, paddr, 2, MTS_WRITE, addr_of_mut!(tmp)).cast::<_>();
    if !ptr.is_null() {
        *ptr = htovm16(val);
    }
}

/// Copy a byte from the VM physical RAM to real host
#[no_mangle]
pub unsafe extern "C" fn physmem_copy_u8_from_vm(vm: *mut vm_instance_t, paddr: m_uint64_t) -> m_uint8_t {
    let mut tmp: m_uint64_t = 0;

    let ptr: *mut m_uint8_t = physmem_get_hptr(vm, paddr, 1, MTS_READ, addr_of_mut!(tmp)).cast::<_>();
    if !ptr.is_null() {
        return *ptr;
    }

    tmp as m_uint8_t
}

/// Copy a 16-bit word to the VM physical RAM from real host
#[no_mangle]
pub unsafe extern "C" fn physmem_copy_u8_to_vm(vm: *mut vm_instance_t, paddr: m_uint64_t, val: m_uint8_t) {
    let mut tmp: m_uint64_t = val as m_uint64_t;

    let ptr: *mut m_uint8_t = physmem_get_hptr(vm, paddr, 1, MTS_WRITE, addr_of_mut!(tmp)).cast::<_>();
    if !ptr.is_null() {
        *ptr = val;
    }
}

/// DMA transfer operation
#[no_mangle]
pub unsafe extern "C" fn physmem_dma_transfer(vm: *mut vm_instance_t, mut src: m_uint64_t, mut dst: m_uint64_t, mut len: size_t) {
    let mut dummy: m_uint64_t = 0;
    let mut sptr: *mut u_char;
    let mut dptr: *mut u_char;
    let mut clen: size_t;
    let mut sl: size_t;
    let mut dl: size_t;

    while len > 0 {
        sptr = physmem_get_hptr(vm, src, 0, MTS_READ, addr_of_mut!(dummy)).cast::<_>();
        dptr = physmem_get_hptr(vm, dst, 0, MTS_WRITE, addr_of_mut!(dummy)).cast::<_>();

        if sptr.is_null() || dptr.is_null() {
            vm_log!(vm, cstr!("DMA"), cstr!("unable to transfer from 0x%llx to 0x%llx\n"), src, dst);
            return;
        }

        sl = VM_PAGE_SIZE - (src & VM_PAGE_IMASK) as size_t;
        dl = VM_PAGE_SIZE - (dst & VM_PAGE_IMASK) as size_t;
        clen = m_min(sl, dl);
        clen = m_min(clen, len);

        libc::memcpy(dptr.cast::<_>(), sptr.cast::<_>(), clen);

        src += clen as m_uint64_t;
        dst += clen as m_uint64_t;
        len -= clen;
    }
}

/// strlen in VM physical memory
#[no_mangle]
pub unsafe extern "C" fn physmem_strlen(vm: *mut vm_instance_t, paddr: m_uint64_t) -> size_t {
    let mut len: size_t = 0;
    let ptr: *mut c_char;

    let vm_ram: *mut vdevice = dev_lookup(vm, paddr, TRUE);
    if !vm_ram.is_null() {
        ptr = ((*vm_ram).host_addr as *mut c_char).add((paddr - (*vm_ram).phys_addr) as usize);
        len = libc::strlen(ptr);
    }

    len
}

/// find sequence of bytes in VM cacheable physical memory interval [first,last]
#[no_mangle]
pub unsafe extern "C" fn physmem_cfind(vm: *mut vm_instance_t, bytes: *mut m_uint8_t, len: size_t, mut first: m_uint64_t, last: m_uint64_t, paddr: *mut m_uint64_t) -> c_int {
    let mut dev: *mut vdevice;
    let mut i: size_t;
    let mut buflen: size_t;
    let mut last_dev_addr: m_uint64_t;

    #[allow(clippy::absurd_extreme_comparisons)]
    if len <= 0 || first > last || len as m_uint64_t + 1 > last - first {
        return -1; // nothing to find
    }

    let buffer: *mut m_uint8_t = libc::malloc(len).cast::<_>();
    if buffer.is_null() {
        libc::perror(cstr!("physmem_cfind: malloc"));
        return -1;
    }
    i = 0;
    buflen = 0;
    dev = (*vm).dev_list;
    while !dev.is_null() {
        // each device
        if dev.is_null() || (*dev).phys_addr > last {
            break; // no more devices
        }
        if ((*dev).flags & VDEVICE_FLAG_CACHING) == 0 {
            dev = (*dev).next;
            continue; // not cacheable
        }

        // reset buffer if previous device is not continuous in memory
        if first + buflen as m_uint64_t != (*dev).phys_addr {
            i = 0;
            buflen = 0;
        }
        last_dev_addr = (*dev).phys_addr + (*dev).phys_len as m_uint64_t - 1;
        if last_dev_addr > last {
            last_dev_addr = last;
        }

        // fill buffer
        while buflen < len && (first + buflen as m_uint64_t) <= last_dev_addr {
            *buffer.add(buflen) = physmem_copy_u8_from_vm(vm, first + buflen as m_uint64_t);
            buflen += 1;
        }
        if buflen < len {
            dev = (*dev).next;
            continue; // not enough data
        }

        // test each possible match
        while first + len as m_uint64_t <= last_dev_addr {
            if i >= len {
                i = 0;
            }
            if i == 0 || libc::memcmp(buffer.cast::<_>(), bytes.add(len).sub(i).cast::<_>(), i) == 0 && libc::memcmp(buffer.add(i).cast::<_>(), bytes.cast::<_>(), len - i) == 0 {
                // match found
                if !paddr.is_null() {
                    *paddr = first;
                }
                libc::free(buffer.cast::<_>());
                return 0;
            }

            *buffer.add(i) = physmem_copy_u8_from_vm(vm, first + len as m_uint64_t);
            i += 1;
            first += 1;
        }
        dev = (*dev).next;
    }

    libc::free(buffer.cast::<_>());
    -1 // not found
}

/// Physical memory dump (32-bit words)
#[no_mangle]
pub unsafe extern "C" fn physmem_dump_vm(vm: *mut vm_instance_t, paddr: m_uint64_t, u32_count: m_uint32_t) {
    for i in 0..u32_count {
        vm_log!(vm, cstr!("physmem_dump"), cstr!("0x%8.8llx: 0x%8.8x\n"), paddr + (i << 2) as m_uint64_t, physmem_copy_u32_from_vm(vm, paddr + (i << 2) as m_uint64_t));
    }
}
