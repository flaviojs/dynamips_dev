//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//!
//! PCI devices.
//!
//! Very interesting docs:
//!   http://www.science.unitn.it/~fiorella/guidelinux/tlk/node72.html
//!   http://www.science.unitn.it/~fiorella/guidelinux/tlk/node76.html

use crate::_private::*;
use crate::cpu::*;
use crate::device::*;
use crate::dynamips_common::*;
use crate::memory::*;
use crate::utils::*;
use crate::vm::*;

pub type pci_dev_t = pci_device;

pub const PCI_BUS_ADDR: m_uint32_t = 0xcf8;
pub const PCI_BUS_DATA: m_uint32_t = 0xcfc;

/// PCI ID (Vendor + Device) register
pub const PCI_REG_ID: c_int = 0x00;

/// PCI Base Address Registers (BAR)
pub const PCI_REG_BAR0: c_int = 0x10;
pub const PCI_REG_BAR1: c_int = 0x14;
pub const PCI_REG_BAR2: c_int = 0x18;
pub const PCI_REG_BAR3: c_int = 0x1c;
pub const PCI_REG_BAR4: c_int = 0x20;
pub const PCI_REG_BAR5: c_int = 0x24;

/// PCI function prototypes
pub type pci_init_t = Option<unsafe extern "C" fn(dev: *mut pci_dev_t)>;
pub type pci_reg_read_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, dev: *mut pci_dev_t, reg: c_int) -> m_uint32_t>;
pub type pci_reg_write_t = Option<unsafe extern "C" fn(cpu: *mut cpu_gen_t, dev: *mut pci_dev_t, reg: c_int, value: m_uint32_t)>;

/// PCI device
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_device {
    pub name: *mut c_char,
    pub vendor_id: u_int,
    pub product_id: u_int,
    pub device: c_int,
    pub function: c_int,
    pub irq: c_int,
    pub priv_data: *mut c_void,

    /// Parent bus
    pub pci_bus: *mut pci_bus,

    pub init: pci_init_t,
    pub read_register: pci_reg_read_t,
    pub write_register: pci_reg_write_t,

    pub next: *mut pci_device,
    pub pprev: *mut *mut pci_device,
}

/// PCI bus
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_bus {
    pub name: *mut c_char,
    pub pci_addr: m_uint32_t,

    /// Bus number
    pub bus: c_int,

    /// PCI device list on this bus
    pub dev_list: *mut pci_device,

    /// PCI bridges to access other busses
    pub bridge_list: *mut pci_bridge,
}

/// PCI bridge
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_bridge {
    pub pri_bus: c_int, // Primary Bus
    pub sec_bus: c_int, // Secondary Bus
    pub sub_bus: c_int, // Subordinate Bus

    pub skip_bus_check: c_int,

    /// Bus configuration register
    pub cfg_reg_bus: m_uint32_t,

    /// PCI bridge device
    pub pci_dev: *mut pci_device,

    /// Secondary PCI bus
    pub pci_bus: *mut pci_bus,

    /// Fallback handlers to read/write config registers
    pub fallback_read: pci_reg_read_t,
    pub fallback_write: pci_reg_write_t,

    pub next: *mut pci_bridge,
    pub pprev: *mut *mut pci_bridge,
}

/// PCI IO device
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pci_io_device {
    pub start: m_uint32_t,
    pub end: m_uint32_t,
    pub real_dev: *mut vdevice,
    pub handler: dev_handler_t,
    pub next: *mut pci_io_device,
    pub pprev: *mut *mut pci_io_device,
}

const DEBUG_PCI: u_int = 1;

unsafe fn GET_PCI_ADDR(pci_bus: *mut pci_bus, offset: c_int, mask: m_uint32_t) -> c_int {
    (((*pci_bus).pci_addr >> offset) & mask) as c_int
}

/// Trigger a PCI device IRQ
#[no_mangle]
pub unsafe extern "C" fn pci_dev_trigger_irq(vm: *mut vm_instance_t, dev: *mut pci_device) {
    if (*dev).irq != -1 {
        vm_set_irq(vm, (*dev).irq as u_int);
    }
}

/// Clear a PCI device IRQ
#[no_mangle]
pub unsafe extern "C" fn pci_dev_clear_irq(vm: *mut vm_instance_t, dev: *mut pci_device) {
    if (*dev).irq != -1 {
        vm_clear_irq(vm, (*dev).irq as u_int);
    }
}

/// Swapping function
#[inline]
unsafe fn pci_swap(val: m_uint32_t, swap: c_int) -> m_uint32_t {
    if swap != 0 {
        swap32(val)
    } else {
        val
    }
}

/// PCI bus lookup
#[no_mangle]
pub unsafe extern "C" fn pci_bus_lookup(pci_bus_root: *mut pci_bus, bus: c_int) -> *mut pci_bus {
    let mut next_bus: *mut pci_bus;
    let mut cur_bus: *mut pci_bus = pci_bus_root;
    let mut bridge: *mut pci_bridge;

    while !cur_bus.is_null() {
        if (*cur_bus).bus == bus {
            return cur_bus;
        }

        // Try busses behind PCI bridges
        next_bus = null_mut();

        bridge = (*cur_bus).bridge_list;
        while !bridge.is_null() {
            // Specific case: final bridge with no checking of secondary
            // bus number. Dynamically programming.
            if (*bridge).skip_bus_check != 0 {
                pci_bridge_set_bus_info(bridge, (*cur_bus).bus, bus, bus);
                (*bridge).skip_bus_check = FALSE;
                return (*bridge).pci_bus;
            }

            if (bus >= (*bridge).sec_bus) && (bus <= (*bridge).sub_bus) {
                next_bus = (*bridge).pci_bus;
                break;
            }
            bridge = (*bridge).next;
        }

        cur_bus = next_bus;
    }

    null_mut()
}

/// PCI device local lookup
#[no_mangle]
pub unsafe extern "C" fn pci_dev_lookup_local(pci_bus: *mut pci_bus, device: c_int, function: c_int) -> *mut pci_device {
    let mut dev: *mut pci_device;

    dev = (*pci_bus).dev_list;
    while !dev.is_null() {
        if ((*dev).device == device) && ((*dev).function == function) {
            return dev;
        }
        dev = (*dev).next
    }

    null_mut()
}

/// PCI Device lookup
#[no_mangle]
pub unsafe extern "C" fn pci_dev_lookup(pci_bus_root: *mut pci_bus, bus: c_int, device: c_int, function: c_int) -> *mut pci_device {
    // Find, try to find the request bus
    let req_bus: *mut pci_bus = pci_bus_lookup(pci_bus_root, bus);
    if req_bus.is_null() {
        return null_mut();
    }

    // Walk through devices present on this bus
    pci_dev_lookup_local(req_bus, device, function)
}

/// Handle the address register access
#[no_mangle]
pub unsafe extern "C" fn pci_dev_addr_handler(_cpu: *mut cpu_gen_t, pci_bus: *mut pci_bus, op_type: u_int, swap: c_int, data: *mut m_uint64_t) {
    if op_type == MTS_WRITE {
        (*pci_bus).pci_addr = pci_swap(*data as m_uint32_t, swap);
    } else {
        *data = pci_swap((*pci_bus).pci_addr, swap) as m_uint64_t;
    }
}

/// Handle the data register access.
///
/// The address of requested register is first written at address 0xcf8
/// (with pci_dev_addr_handler).
///
/// The data is read/written at address 0xcfc.
#[no_mangle]
pub unsafe extern "C" fn pci_dev_data_handler(cpu: *mut cpu_gen_t, pci_bus: *mut pci_bus, op_type: u_int, swap: c_int, data: *mut m_uint64_t) {
    if op_type == MTS_READ {
        *data = 0x0;
    }

    // http://www.mega-tokyo.com/osfaq2/index.php/PciSectionOfPentiumVme
    //
    // 31      : Enable Bit
    // 30 - 24 : Reserved
    // 23 - 16 : Bus Number
    // 15 - 11 : Device Number
    // 10 -  8 : Function Number
    //  7 -  2 : Register Number
    //  1 -  0 : always 00
    let bus: c_int = GET_PCI_ADDR(pci_bus, 16, 0xff);
    let device: c_int = GET_PCI_ADDR(pci_bus, 11, 0x1f);
    let function: c_int = GET_PCI_ADDR(pci_bus, 8, 0x7);
    let reg: c_int = GET_PCI_ADDR(pci_bus, 0, 0xff);

    // Find the corresponding PCI device
    let dev: *mut pci_device = pci_dev_lookup(pci_bus, bus, device, function);

    if dev.is_null() {
        if op_type == MTS_READ {
            cpu_log!(cpu, cstr!("PCI"), cstr!("read request for unknown device at pc=0x%llx (bus=%d,device=%d,function=%d,reg=0x%2.2x).\n"), cpu_get_pc(cpu), bus, device, function, reg);
        } else {
            cpu_log!(cpu, cstr!("PCI"), cstr!("write request (data=0x%8.8x) for unknown device at pc=0x%llx (bus=%d,device=%d,function=%d,reg=0x%2.2x).\n"), pci_swap(*data as m_uint32_t, swap), cpu_get_pc(cpu), bus, device, function, reg);
        }

        // Returns an invalid device ID
        if (op_type == MTS_READ) && (reg == PCI_REG_ID) {
            *data = 0xffffffff;
        }
    } else {
        if DEBUG_PCI != 0 {
            if op_type == MTS_READ {
                cpu_log!(cpu, cstr!("PCI"), cstr!("read request for device '%s' at pc=0x%llx: bus=%d,device=%d,function=%d,reg=0x%2.2x\n"), (*dev).name, cpu_get_pc(cpu), bus, device, function, reg);
            } else {
                cpu_log!(cpu, cstr!("PCI"), cstr!("write request (data=0x%8.8x) for device '%s' at pc=0x%llx: bus=%d,device=%d,function=%d,reg=0x%2.2x\n"), pci_swap(*data as m_uint32_t, swap), (*dev).name, cpu_get_pc(cpu), bus, device, function, reg);
            }
        }
        if op_type == MTS_WRITE {
            if (*dev).write_register.is_some() {
                (*dev).write_register.unwrap()(cpu, dev, reg, pci_swap(*data as m_uint32_t, swap));
            }
        } else if reg == PCI_REG_ID {
            *data = pci_swap(((*dev).product_id << 16) | (*dev).vendor_id, swap) as m_uint64_t;
        } else if (*dev).read_register.is_some() {
            *data = pci_swap((*dev).read_register.unwrap()(cpu, dev, reg), swap) as m_uint64_t;
        }
    }
}

/// Add a PCI bridge
#[no_mangle]
pub unsafe extern "C" fn pci_bridge_add(pci_bus: *mut pci_bus) -> *mut pci_bridge {
    if pci_bus.is_null() {
        return null_mut();
    }

    let bridge: *mut pci_bridge = libc::malloc(size_of::<pci_bridge>()).cast::<_>();
    if bridge.is_null() {
        libc::fprintf(c_stderr(), cstr!("pci_bridge_add: unable to create new PCI bridge.\n"));
        return null_mut();
    }

    libc::memset(bridge.cast::<_>(), 0, size_of::<pci_bridge>());
    (*bridge).pri_bus = (*pci_bus).bus;
    (*bridge).sec_bus = -1;
    (*bridge).sub_bus = -1;
    (*bridge).pci_bus = null_mut();

    // Insert the bridge in the double-linked list
    (*bridge).next = (*pci_bus).bridge_list;
    (*bridge).pprev = addr_of_mut!((*pci_bus).bridge_list);

    if !(*pci_bus).bridge_list.is_null() {
        (*(*pci_bus).bridge_list).pprev = addr_of_mut!((*bridge).next);
    }

    (*pci_bus).bridge_list = bridge;
    bridge
}

/// Remove a PCI bridge from the double-linked list
#[inline]
unsafe fn pci_bridge_remove_from_list(bridge: *mut pci_bridge) {
    if !(*bridge).next.is_null() {
        (*(*bridge).next).pprev = (*bridge).pprev;
    }

    if !(*bridge).pprev.is_null() {
        *((*bridge).pprev) = (*bridge).next;
    }
}

/// Remove a PCI bridge
#[no_mangle]
pub unsafe extern "C" fn pci_bridge_remove(bridge: *mut pci_bridge) {
    if !bridge.is_null() {
        pci_bridge_remove_from_list(bridge);
        libc::free(bridge.cast::<_>());
    }
}

/// Map secondary bus to a PCI bridge
#[no_mangle]
pub unsafe extern "C" fn pci_bridge_map_bus(bridge: *mut pci_bridge, pci_bus: *mut pci_bus) {
    if !bridge.is_null() {
        (*bridge).pci_bus = pci_bus;

        if !(*bridge).pci_bus.is_null() {
            (*(*bridge).pci_bus).bus = (*bridge).sec_bus;
        }
    }
}

/// Set PCI bridge bus info
#[no_mangle]
pub unsafe extern "C" fn pci_bridge_set_bus_info(bridge: *mut pci_bridge, pri_bus: c_int, sec_bus: c_int, sub_bus: c_int) {
    if !bridge.is_null() {
        (*bridge).pri_bus = pri_bus;
        (*bridge).sec_bus = sec_bus;
        (*bridge).sub_bus = sub_bus;

        if !(*bridge).pci_bus.is_null() {
            (*(*bridge).pci_bus).bus = (*bridge).sec_bus;
        }
    }
}

/// Add a PCI device
#[no_mangle]
pub unsafe extern "C" fn pci_dev_add(pci_bus: *mut pci_bus, name: *mut c_char, vendor_id: u_int, product_id: u_int, device: c_int, function: c_int, irq: c_int, priv_data: *mut c_void, init: pci_init_t, read_register: pci_reg_read_t, write_register: pci_reg_write_t) -> *mut pci_device {
    let mut dev: *mut pci_device;

    if pci_bus.is_null() {
        return null_mut();
    }

    dev = pci_dev_lookup_local(pci_bus, device, function);
    if !dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("pci_dev_add: bus %s, device %d, function %d already registered (device '%s').\n"), (*pci_bus).name, device, function, (*dev).name);
        return null_mut();
    }

    // we can create safely the new device
    dev = libc::malloc(size_of::<pci_device>()).cast::<_>();
    if dev.is_null() {
        libc::fprintf(c_stderr(), cstr!("pci_dev_add: unable to create new PCI device.\n"));
        return null_mut();
    }

    libc::memset(dev.cast::<_>(), 0, size_of::<pci_device>());
    (*dev).name = name;
    (*dev).vendor_id = vendor_id;
    (*dev).product_id = product_id;
    (*dev).pci_bus = pci_bus;
    (*dev).device = device;
    (*dev).function = function;
    (*dev).irq = irq;
    (*dev).priv_data = priv_data;
    (*dev).init = init;
    (*dev).read_register = read_register;
    (*dev).write_register = write_register;

    // Insert the device in the double-linked list
    (*dev).next = (*pci_bus).dev_list;
    (*dev).pprev = addr_of_mut!((*pci_bus).dev_list);

    if !(*pci_bus).dev_list.is_null() {
        (*(*pci_bus).dev_list).pprev = addr_of_mut!((*dev).next);
    }

    (*pci_bus).dev_list = dev;

    #[allow(clippy::unnecessary_unwrap)]
    if init.is_some() {
        init.unwrap()(dev);
    }
    dev
}

/// Add a basic PCI device that just returns a Vendor/Product ID
#[no_mangle]
pub unsafe extern "C" fn pci_dev_add_basic(pci_bus: *mut pci_bus, name: *mut c_char, vendor_id: u_int, product_id: u_int, device: c_int, function: c_int) -> *mut pci_device {
    pci_dev_add(pci_bus, name, vendor_id, product_id, device, function, -1, null_mut(), None, None, None)
}

/// Remove a device from the double-linked list
#[inline]
unsafe fn pci_dev_remove_from_list(dev: *mut pci_device) {
    if !(*dev).next.is_null() {
        (*(*dev).next).pprev = (*dev).pprev;
    }

    if !(*dev).pprev.is_null() {
        *((*dev).pprev) = (*dev).next;
    }
}

/// Remove a PCI device
#[no_mangle]
pub unsafe extern "C" fn pci_dev_remove(dev: *mut pci_device) {
    if !dev.is_null() {
        pci_dev_remove_from_list(dev);
        libc::free(dev.cast::<_>());
    }
}

/// Remove a PCI device given its ID (bus,device,function)
#[no_mangle]
pub unsafe extern "C" fn pci_dev_remove_by_id(pci_bus: *mut pci_bus, bus: c_int, device: c_int, function: c_int) -> c_int {
    let dev: *mut pci_device = pci_dev_lookup(pci_bus, bus, device, function);
    if dev.is_null() {
        return -1;
    }

    pci_dev_remove(dev);
    0
}

/// Remove a PCI device given its name
#[no_mangle]
pub unsafe extern "C" fn pci_dev_remove_by_name(pci_bus: *mut pci_bus, name: *mut c_char) -> c_int {
    let mut dev: *mut pci_device;
    let mut next: *mut pci_device;
    let mut count: c_int = 0;

    dev = (*pci_bus).dev_list;
    while !dev.is_null() {
        next = (*dev).next;

        if libc::strcmp((*dev).name, name) == 0 {
            pci_dev_remove(dev);
            count += 1;
        }
        dev = next;
    }

    count
}

/// Create a PCI bus
#[no_mangle]
pub unsafe extern "C" fn pci_bus_create(name: *mut c_char, bus: c_int) -> *mut pci_bus {
    let d: *mut pci_bus = libc::malloc(size_of::<pci_bus>()).cast::<_>();
    if d.is_null() {
        libc::fprintf(c_stderr(), cstr!("pci_bus_create: unable to create PCI info.\n"));
        return null_mut();
    }

    libc::memset(d.cast::<_>(), 0, size_of::<pci_bus>());
    (*d).name = libc::strdup(name);
    (*d).bus = bus;
    d
}

/// Delete a PCI bus
#[no_mangle]
pub unsafe extern "C" fn pci_bus_remove(pci_bus: *mut pci_bus) {
    let mut dev: *mut pci_device;
    let mut next: *mut pci_device;
    let mut bridge: *mut pci_bridge;
    let mut next_bridge: *mut pci_bridge;

    if !pci_bus.is_null() {
        // Remove all devices
        dev = (*pci_bus).dev_list;
        while !dev.is_null() {
            next = (*dev).next;
            libc::free(dev.cast::<_>());
            dev = next;
        }

        // Remove all bridges
        bridge = (*pci_bus).bridge_list;
        while !bridge.is_null() {
            next_bridge = (*bridge).next;
            libc::free(bridge.cast::<_>());
            bridge = next_bridge;
        }

        // Free the structure itself
        libc::free((*pci_bus).name.cast::<_>());
        libc::free(pci_bus.cast::<_>());
    }
}

/// Read a configuration register of a PCI bridge
unsafe extern "C" fn pci_bridge_read_reg(cpu: *mut cpu_gen_t, dev: *mut pci_device, reg: c_int) -> m_uint32_t {
    let bridge: *mut pci_bridge = (*dev).priv_data.cast::<_>();
    let mut val: m_uint32_t = 0;

    match reg {
        0x18 => (*bridge).cfg_reg_bus,
        _ => {
            if (*bridge).fallback_read.is_some() {
                val = (*bridge).fallback_read.unwrap()(cpu, dev, reg);
            }

            // Returns appropriate PCI bridge class code if nothing defined
            if (reg == 0x08) && val == 0 {
                val = 0x06040000;
            }

            val
        }
    }
}

/// Write a configuration register of a PCI bridge
unsafe extern "C" fn pci_bridge_write_reg(cpu: *mut cpu_gen_t, dev: *mut pci_device, reg: c_int, value: m_uint32_t) {
    let bridge: *mut pci_bridge = (*dev).priv_data.cast::<_>();
    let pri_bus: u_int;
    let sec_bus: u_int;
    let sub_bus: u_int;

    match reg {
        0x18 => {
            (*bridge).cfg_reg_bus = value;
            sub_bus = (value >> 16) & 0xFF;
            sec_bus = (value >> 8) & 0xFF;
            pri_bus = value & 0xFF;

            // Modify the PCI bridge settings
            vm_log!((*cpu).vm, cstr!("PCI"), cstr!("PCI bridge %d,%d,%d -> pri: %2.2u, sec: %2.2u, sub: %2.2u\n"), (*(*dev).pci_bus).bus, (*dev).device, (*dev).function, pri_bus, sec_bus, sub_bus);

            pci_bridge_set_bus_info(bridge, pri_bus as c_int, sec_bus as c_int, sub_bus as c_int);
        }

        _ => {
            if (*bridge).fallback_write.is_some() {
                (*bridge).fallback_write.unwrap()(cpu, dev, reg, value);
            }
        }
    }
}

/// Create a PCI bridge device
#[no_mangle]
pub unsafe extern "C" fn pci_bridge_create_dev(pci_bus: *mut pci_bus, name: *mut c_char, vendor_id: u_int, product_id: u_int, device: c_int, function: c_int, sec_bus: *mut pci_bus, fallback_read: pci_reg_read_t, fallback_write: pci_reg_write_t) -> *mut pci_device {
    // Create the PCI bridge structure
    let bridge: *mut pci_bridge = pci_bridge_add(pci_bus);
    if bridge.is_null() {
        return null_mut();
    }

    // Create the PCI device corresponding to the bridge
    let dev: *mut pci_device = pci_dev_add(pci_bus, name, vendor_id, product_id, device, function, -1, bridge.cast::<_>(), None, Some(pci_bridge_read_reg), Some(pci_bridge_write_reg));

    if dev.is_null() {
        pci_bridge_remove(bridge);
        return null_mut();
    }

    // Keep the associated PCI device for this bridge
    (*bridge).pci_dev = dev;

    // Set the fallback functions
    (*bridge).fallback_read = fallback_read;
    (*bridge).fallback_write = fallback_write;

    // Map the secondary bus (disabled at startup)
    pci_bridge_map_bus(bridge, sec_bus);
    dev
}

/// Show PCI device list of the specified bus
unsafe fn pci_bus_show_dev_list(pci_bus: *mut pci_bus) {
    let mut dev: *mut pci_device;
    let mut bridge: *mut pci_bridge;
    let mut bus_id: [c_char; 32] = [0; 32];

    if pci_bus.is_null() {
        return;
    }

    if (*pci_bus).bus != -1 {
        libc::snprintf(bus_id.as_c_mut(), bus_id.len(), cstr!("%2d"), (*pci_bus).bus);
    } else {
        libc::strcpy(bus_id.as_c_mut(), cstr!("XX"));
    }

    dev = (*pci_bus).dev_list;
    while !dev.is_null() {
        libc::printf(cstr!("   %-18s: ID %4.4x:%4.4x, Bus %s, Dev. %2d, Func. %2d"), (*dev).name, (*dev).vendor_id, (*dev).product_id, bus_id.as_c(), (*dev).device, (*dev).function);

        if (*dev).irq != -1 {
            libc::printf(cstr!(", IRQ: %d\n"), (*dev).irq);
        } else {
            libc::printf(cstr!("\n"));
        }
        dev = (*dev).next;
    }

    bridge = (*pci_bus).bridge_list;
    while !bridge.is_null() {
        pci_bus_show_dev_list((*bridge).pci_bus);
        bridge = (*bridge).next;
    }
}

/// Show PCI device list
#[no_mangle]
pub unsafe extern "C" fn pci_dev_show_list(pci_bus: *mut pci_bus) {
    if pci_bus.is_null() {
        return;
    }

    libc::printf(cstr!("PCI Bus \"%s\" Device list:\n"), (*pci_bus).name);
    pci_bus_show_dev_list(pci_bus);
    libc::printf(cstr!("\n"));
}
