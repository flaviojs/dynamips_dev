//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! Packet SRAM. This is a fast memory zone for packets on NPE150/NPE200.

use crate::_private::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::pci_dev::*;
use crate::utils::*;
use crate::vm::*;

const PCI_VENDOR_SRAM: u_int = 0x1137;
const PCI_PRODUCT_SRAM: u_int = 0x0005;

/// SRAM structure
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sram_data {
    /// VM object info
    pub vm_obj: vm_obj_t,

    /// SRAM main device
    pub dev: *mut vdevice,

    /// Aliased device
    pub alias_dev_name: *mut c_char,
    pub alias_dev: *mut vdevice,

    /// Byte-swapped device
    pub bs_dev_name: *mut c_char,
    pub bs_obj: *mut vm_obj_t,

    /// PCI device
    pub pci_dev: *mut pci_device,

    /// Filename used to virtualize SRAM
    pub filename: *mut c_char,
}

/// Shutdown an SRAM device
unsafe extern "C" fn dev_c7200_sram_shutdown(vm: *mut vm_instance_t, d: *mut c_void) -> *mut c_void {
    let d: *mut sram_data = d.cast::<_>();
    if !d.is_null() {
        // Remove the PCI device
        pci_dev_remove((*d).pci_dev);

        // Remove the byte-swapped device
        vm_object_remove(vm, (*d).bs_obj);

        // Remove the alias and the main device
        dev_remove(vm, (*d).alias_dev);
        dev_remove(vm, (*d).dev);

        // Free devices
        libc::free((*d).alias_dev.cast::<_>());
        libc::free((*d).dev.cast::<_>());

        // Free device names
        libc::free((*d).alias_dev_name.cast::<_>());
        libc::free((*d).bs_dev_name.cast::<_>());

        // Remove filename used to virtualize SRAM
        if !(*d).filename.is_null() {
            libc::unlink((*d).filename);
            libc::free((*d).filename.cast::<_>());
        }

        // Free the structure itself
        libc::free(d.cast::<_>());
    }

    null_mut()
}

/// Initialize an SRAM device
#[no_mangle]
pub unsafe extern "C" fn dev_c7200_sram_init(vm: *mut vm_instance_t, name: *mut c_char, paddr: m_uint64_t, len: m_uint32_t, pci_bus: *mut pci_bus, pci_device: c_int) -> c_int {
    // Allocate the private data structure for SRAM
    let d: *mut sram_data = libc::malloc(size_of::<sram_data>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("dev_c7200_sram_init (%s): out of memory\n"), name);
        return -1;
    }

    libc::memset(d.cast::<_>(), 0, size_of::<sram_data>());

    vm_object_init(addr_of_mut!((*d).vm_obj));
    (*d).vm_obj.name = name;
    (*d).vm_obj.data = d.cast::<_>();
    (*d).vm_obj.shutdown = Some(dev_c7200_sram_shutdown);

    (*d).filename = vm_build_filename(vm, name);
    if (*d).filename.is_null() {
        libc::free(d.cast::<_>());
        return -1;
    }

    // add as a pci device
    (*d).pci_dev = pci_dev_add_basic(pci_bus, name, PCI_VENDOR_SRAM, PCI_PRODUCT_SRAM, pci_device, 0);

    if (*d).pci_dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("dev_c7200_sram_init: unable to create basic device.\n"));
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    let alias_paddr: m_uint64_t = 0x100000000_u64 + paddr;

    // create the standard RAM zone
    (*d).dev = dev_create_ram(vm, name, FALSE, (*d).filename, paddr, len);
    if (*d).dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("dev_c7200_sram_init: unable to create '%s' file.\n"), (*d).filename);
        pci_dev_remove((*d).pci_dev);
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    // create the RAM alias
    (*d).alias_dev_name = dyn_sprintf!(cstr!("%s_alias"), name);
    if (*d).alias_dev_name.is_null() {
        libc::fprintf(c_stderr(), cstr!("dev_c7200_sram_init: unable to create alias name.\n"));
        dev_remove(vm, (*d).dev);
        libc::free((*d).dev.cast::<_>());
        libc::unlink((*d).filename);
        pci_dev_remove((*d).pci_dev);
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    (*d).alias_dev = dev_create_ram_alias(vm, (*d).alias_dev_name, name, alias_paddr, len);

    if (*d).alias_dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("dev_c7200_sram_init: unable to create alias device.\n"));
        libc::free((*d).alias_dev_name.cast::<_>());
        dev_remove(vm, (*d).dev);
        libc::free((*d).dev.cast::<_>());
        libc::unlink((*d).filename);
        pci_dev_remove((*d).pci_dev);
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    // create the byte-swapped zone (used with Galileo DMA)
    (*d).bs_dev_name = dyn_sprintf!(cstr!("%s_bswap"), name);
    if (*d).bs_dev_name.is_null() {
        libc::fprintf(c_stderr(), cstr!("dev_c7200_sram_init: unable to create BS name.\n"));
        dev_remove(vm, (*d).alias_dev);
        libc::free((*d).alias_dev.cast::<_>());
        libc::free((*d).alias_dev_name.cast::<_>());
        dev_remove(vm, (*d).dev);
        libc::free((*d).dev.cast::<_>());
        libc::unlink((*d).filename);
        pci_dev_remove((*d).pci_dev);
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    if dev_bswap_init(vm, (*d).bs_dev_name, paddr + 0x800000, len, paddr) == -1 {
        libc::fprintf(c_stderr(), cstr!("dev_c7200_sram_init: unable to create BS device.\n"));
        libc::free((*d).bs_dev_name.cast::<_>());
        dev_remove(vm, (*d).alias_dev);
        libc::free((*d).alias_dev.cast::<_>());
        libc::free((*d).alias_dev_name.cast::<_>());
        dev_remove(vm, (*d).dev);
        libc::free((*d).dev.cast::<_>());
        libc::unlink((*d).filename);
        pci_dev_remove((*d).pci_dev);
        libc::free((*d).filename.cast::<_>());
        libc::free(d.cast::<_>());
        return -1;
    }

    (*d).bs_obj = vm_object_find(vm, (*d).bs_dev_name);
    0
}
