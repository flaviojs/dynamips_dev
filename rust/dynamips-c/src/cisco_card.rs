//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! Generic Cisco card routines and definitions.

use crate::_private::*;
use crate::cisco_eeprom::*;
use crate::dynamips_common::*;
use crate::net_io::*;
use crate::pci_dev::*;
use crate::utils::*;
use crate::vm::*;

pub const CISCO_CARD_MAX_WIC: usize = 8;
pub const CISCO_CARD_MAX_SUBSLOTS: usize = 16;

/// Card types // TODO enum
pub const CISCO_CARD_TYPE_UNDEF: u_int = 0;
pub const CISCO_CARD_TYPE_PA: u_int = 1;
pub const CISCO_CARD_TYPE_NM: u_int = 2;
pub const CISCO_CARD_TYPE_WIC: u_int = 3;

/// Card flags // TODO enum
pub const CISCO_CARD_FLAG_OVERRIDE: u_int = 1;

/// Prototype of card driver initialization function
pub type cisco_card_init_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card) -> c_int>;

/// Prototype of card driver shutdown function
pub type cisco_card_shutdown_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card) -> c_int>;

/// Prototype of card NIO get sub-slot info function
pub type cisco_card_get_sub_info_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card, port_id: u_int, drv_array: *mut *mut *mut cisco_card_driver, subcard_type: *mut u_int) -> c_int>;

/// Prototype of card NIO set function
pub type cisco_card_set_nio_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card, port_id: u_int, nio: *mut netio_desc_t) -> c_int>;

/// Prototype of card NIO unset function
pub type cisco_card_unset_nio_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card, port_id: u_int) -> c_int>;

/// Prototype of card NIO show info function
pub type cisco_card_show_info_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card) -> c_int>;

/// Cisco NIO binding to a slot/port
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cisco_nio_binding {
    pub nio: *mut netio_desc_t,
    pub port_id: u_int,
    pub orig_port_id: u_int,
    pub prev: *mut cisco_nio_binding,
    pub next: *mut cisco_nio_binding,
}

/// Generic Cisco card driver
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cisco_card_driver {
    pub dev_type: *mut c_char,
    pub supported: c_int,
    pub wic_slots: c_int,
    pub card_init: cisco_card_init_fn,
    pub card_shutdown: cisco_card_shutdown_fn,
    pub card_get_sub_info: cisco_card_get_sub_info_fn,
    pub card_set_nio: cisco_card_set_nio_fn,
    pub card_unset_nio: cisco_card_unset_nio_fn,
    pub card_show_info: cisco_card_show_info_fn,
}

/// Generic Cisco card
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cisco_card {
    pub dev_name: *mut c_char, // Device name
    pub dev_type: *mut c_char, // Device Type
    pub card_type: u_int,      // Card type (NM,PA,WIC,...)
    pub card_flags: u_int,     // Card flags
    pub card_id: u_int,        // Card ID (slot or sub-slot)
    pub slot_id: u_int,        // Slot and Sub-slot ID
    pub subslot_id: u_int,
    pub eeprom: cisco_eeprom,                                  // EEPROM
    pub pci_bus: *mut pci_bus,                                 // PCI bus
    pub driver: *mut cisco_card_driver,                        // Driver
    pub drv_info: *mut c_void,                                 // Private driver info
    pub nio_list: *mut cisco_nio_binding,                      // NIO bindings to ports
    pub parent: *mut cisco_card,                               // Parent card
    pub sub_slots: [*mut cisco_card; CISCO_CARD_MAX_SUBSLOTS], // Sub-slots
}

/// Get cisco card type description
#[no_mangle]
pub unsafe extern "C" fn cisco_card_get_type_desc(dev_type: c_int) -> *mut c_char {
    match dev_type as u_int {
        CISCO_CARD_TYPE_PA => cstr!("Port Adapter (PA)"),
        CISCO_CARD_TYPE_NM => cstr!("Network Module (NM)"),
        CISCO_CARD_TYPE_WIC => cstr!("WAN Interface Card (WIC)"),
        _ => cstr!("Unknown"),
    }
}

/// Set EEPROM definition for the specified Cisco card
#[no_mangle]
pub unsafe extern "C" fn cisco_card_set_eeprom(vm: *mut vm_instance_t, card: *mut cisco_card, eeprom: *const cisco_eeprom) -> c_int {
    if eeprom.is_null() {
        return 0;
    }

    if cisco_eeprom_copy(addr_of_mut!((*card).eeprom), eeprom) == -1 {
        vm_error!(vm, cstr!("cisco_card_set_eeprom: no memory (eeprom=%p).\n"), eeprom);
        return -1;
    }

    0
}

/// Unset EEPROM definition
#[no_mangle]
pub unsafe extern "C" fn cisco_card_unset_eeprom(card: *mut cisco_card) -> c_int {
    cisco_eeprom_free(addr_of_mut!((*card).eeprom));
    0
}

/// Check if a card has a valid EEPROM defined
#[no_mangle]
pub unsafe extern "C" fn cisco_card_check_eeprom(card: *mut cisco_card) -> c_int {
    cisco_eeprom_valid(addr_of_mut!((*card).eeprom))
}

/// Create a card structure
#[inline]
unsafe fn cisco_card_create(card_type: u_int) -> *mut cisco_card {
    let card: *mut cisco_card = libc::malloc(size_of::<cisco_card>()).cast::<_>();
    if !card.is_null() {
        libc::memset(card.cast::<_>(), 0, size_of::<cisco_card>());
        (*card).card_type = card_type;
    }

    card
}

/// Find a NIO binding
unsafe fn cisco_card_find_nio_binding(card: *mut cisco_card, port_id: u_int) -> *mut cisco_nio_binding {
    if card.is_null() {
        return null_mut();
    }

    let mut nb: *mut cisco_nio_binding = (*card).nio_list;
    while !nb.is_null() {
        if (*nb).port_id == port_id {
            return nb;
        }
        nb = (*nb).next;
    }

    null_mut()
}

/// Remove all NIO bindings
unsafe fn cisco_card_remove_all_nio_bindings(vm: *mut vm_instance_t, card: *mut cisco_card) {
    let mut nb: *mut cisco_nio_binding = (*card).nio_list;
    while !nb.is_null() {
        let next: *mut cisco_nio_binding = (*nb).next;

        // tell the slot driver to stop using this NIO
        if !(*card).driver.is_null() {
            (*(*card).driver).card_unset_nio.unwrap()(vm, card, (*nb).port_id);
        }

        // unreference NIO object
        netio_release((*(*nb).nio).name);
        libc::free(nb.cast::<_>());
        nb = next;
    }

    (*card).nio_list = null_mut();
}

/// Enable all NIO for the specified card
#[inline]
unsafe fn cisco_card_enable_all_nio(vm: *mut vm_instance_t, card: *mut cisco_card) {
    if !card.is_null() && !(*card).driver.is_null() && !(*card).drv_info.is_null() {
        let mut nb: *mut cisco_nio_binding = (*card).nio_list;
        while !nb.is_null() {
            (*(*card).driver).card_set_nio.unwrap()(vm, card, (*nb).port_id, (*nb).nio);
            nb = (*nb).next;
        }
    }
}

/// Disable all NIO for the specified card
#[inline]
unsafe fn cisco_card_disable_all_nio(vm: *mut vm_instance_t, card: *mut cisco_card) {
    if !card.is_null() && !(*card).driver.is_null() && !(*card).drv_info.is_null() {
        let mut nb: *mut cisco_nio_binding = (*card).nio_list;
        while !nb.is_null() {
            (*(*card).driver).card_unset_nio.unwrap()(vm, card, (*nb).port_id);
            nb = (*nb).next;
        }
    }
}

/// Initialize a card
#[inline]
unsafe fn cisco_card_init(vm: *mut vm_instance_t, card: *mut cisco_card, id: u_int) -> c_int {
    // Check that a device type is defined for this card
    if card.is_null() || (*card).dev_type.is_null() || (*card).driver.is_null() {
        return -1;
    }

    // Allocate device name
    let len: size_t = libc::strlen((*card).dev_type) + 10;
    (*card).dev_name = libc::malloc(len).cast::<_>();
    if (*card).dev_name.is_null() {
        vm_error!(vm, cstr!("unable to allocate device name.\n"));
        return -1;
    }

    libc::snprintf((*card).dev_name, len, cstr!("%s(%u)"), (*card).dev_type, id);

    // Initialize card driver
    if (*(*card).driver).card_init.unwrap()(vm, card) == -1 {
        vm_error!(vm, cstr!("unable to initialize card type '%s' (id %u)\n"), (*card).dev_type, id);
        return -1;
    }

    0
}

/// Shutdown card
unsafe fn cisco_card_shutdown(vm: *mut vm_instance_t, card: *mut cisco_card) -> c_int {
    // Check that a device type is defined for this card
    if card.is_null() || (*card).dev_type.is_null() || (*card).driver.is_null() {
        return -1;
    }

    // Shutdown the NM driver
    if !(*card).drv_info.is_null() && (*(*card).driver).card_shutdown.unwrap()(vm, card) == -1 {
        vm_error!(vm, cstr!("unable to shutdown card type '%s' (slot %u/%u)\n"), (*card).dev_type, (*card).slot_id, (*card).subslot_id);
        return -1;
    }

    libc::free((*card).dev_name.cast::<_>());
    (*card).dev_name = null_mut();
    (*card).drv_info = null_mut();
    0
}

/// Show info for the specified card
unsafe fn cisco_card_show_info(vm: *mut vm_instance_t, card: *mut cisco_card) -> c_int {
    // Check that a device type is defined for this card
    if card.is_null() || (*card).driver.is_null() || (*(*card).driver).card_show_info.is_none() {
        return -1;
    }

    (*(*card).driver).card_show_info.unwrap()(vm, card);
    0
}

/// Save config for the specified card
unsafe fn cisco_card_save_config(vm: *mut vm_instance_t, card: *mut cisco_card, fd: *mut libc::FILE) -> c_int {
    if !card.is_null() {
        libc::fprintf(fd, cstr!("vm slot_add_binding %s %u %u %s\n"), (*vm).name, (*card).slot_id, (*card).subslot_id, (*card).dev_type);

        let mut nb: *mut cisco_nio_binding = (*card).nio_list;
        while !nb.is_null() {
            libc::fprintf(fd, cstr!("vm add_nio_binding %s %u %u %s\n"), (*vm).name, (*card).slot_id, (*nb).orig_port_id, (*(*nb).nio).name);
            nb = (*nb).next;
        }
    }

    0
}

/// Find a driver in a driver array
unsafe fn cisco_card_find_driver(array: *mut *mut cisco_card_driver, dev_type: *mut c_char) -> *mut cisco_card_driver {
    for i in 0.. {
        if !(*array.add(i)).is_null() && libc::strcmp((*(*array.add(i))).dev_type, dev_type) == 0 {
            return *array.add(i);
        }
    }

    null_mut()
}

// ========================================================================
// High level routines for managing VM slots.
// ========================================================================

/// Get slot info
#[no_mangle]
pub unsafe extern "C" fn vm_slot_get_card_ptr(vm: *mut vm_instance_t, slot_id: u_int) -> *mut cisco_card {
    if slot_id >= (*vm).nr_slots {
        return null_mut();
    }

    (*vm).slots[slot_id as usize]
}

/// Get info for a slot/port (with sub-cards)
unsafe fn vm_slot_get_info(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int, rc: *mut *mut *mut cisco_card, real_port_id: *mut u_int) -> c_int {
    if slot_id >= VM_MAX_SLOTS as u_int {
        *rc = null_mut();
        return -1;
    }

    *rc = addr_of_mut!((*vm).slots[slot_id as usize]);
    let card: *mut cisco_card = (*vm).slots[slot_id as usize];

    let card_type: u_int = if !card.is_null() { (*card).card_type } else { CISCO_CARD_TYPE_UNDEF };

    match card_type {
        // Handle WICs which are sub-slots for Network Modules (NM).
        // Numbering: wic #0 => port_id = 0x10
        //            wic #1 => port_id = 0x20
        CISCO_CARD_TYPE_NM => {
            if (*(*card).driver).wic_slots > 0 {
                let wic_id: u_int = port_id >> 4;

                if wic_id >= (CISCO_CARD_MAX_WIC as u_int + 1) {
                    vm_error!(vm, cstr!("Invalid wic_id %u (slot %u)\n"), wic_id, slot_id);
                    return -1;
                }

                if wic_id >= 0x01 {
                    // wic card
                    *rc = addr_of_mut!((*card).sub_slots[wic_id as usize - 1]);
                    *real_port_id = port_id & 0x0F;
                } else {
                    // main card
                    *real_port_id = port_id;
                }
            } else {
                *real_port_id = port_id;
            }
            0
        }

        // No translation for Cisco 7200 Port Adapters and WICs
        CISCO_CARD_TYPE_PA | CISCO_CARD_TYPE_WIC => {
            *real_port_id = port_id;
            0
        }

        // Not initialized yet
        _ => {
            *real_port_id = port_id;
            0
        }
    }
}

/// Translate a port ID (for sub-cards)
unsafe fn vm_slot_translate_port_id(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int, rc: *mut *mut cisco_card) -> u_int {
    let mut tmp: *mut *mut cisco_card = null_mut();
    let mut real_port_id: u_int = 0;

    if vm_slot_get_info(vm, slot_id, port_id, addr_of_mut!(tmp), addr_of_mut!(real_port_id)) == -1 {
        *rc = null_mut();
        return port_id;
    }

    *rc = *tmp;
    real_port_id
}

/// Check if a slot has an active card
#[no_mangle]
pub unsafe extern "C" fn vm_slot_active(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int) -> c_int {
    let mut rc: *mut *mut cisco_card = null_mut();
    let mut real_port_id: u_int = 0;

    if vm_slot_get_info(vm, slot_id, port_id, addr_of_mut!(rc), addr_of_mut!(real_port_id)) == -1 {
        return FALSE;
    }

    if (*rc).is_null() || (*(*rc)).dev_type.is_null() {
        return FALSE;
    }

    TRUE
}

/// Set a flag for a card
#[no_mangle]
pub unsafe extern "C" fn vm_slot_set_flag(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int, flag: u_int) -> c_int {
    let mut rc: *mut *mut cisco_card = null_mut();
    let mut real_port_id: u_int = 0;

    if vm_slot_get_info(vm, slot_id, port_id, addr_of_mut!(rc), addr_of_mut!(real_port_id)) == -1 {
        return FALSE;
    }

    if (*rc).is_null() {
        return FALSE;
    }

    (*(*rc)).card_flags |= flag;
    TRUE
}

/// Add a slot binding
#[no_mangle]
pub unsafe extern "C" fn vm_slot_add_binding(vm: *mut vm_instance_t, dev_type: *mut c_char, slot_id: u_int, port_id: u_int) -> c_int {
    let mut drv_array: *mut *mut cisco_card_driver = null_mut();
    let mut rc: *mut *mut cisco_card = null_mut();
    let parent: *mut cisco_card;
    let mut real_port_id: u_int = 0;
    let mut card_type: u_int = 0;
    let card_id: u_int;

    if vm_slot_get_info(vm, slot_id, port_id, addr_of_mut!(rc), addr_of_mut!(real_port_id)) == -1 {
        return -1;
    }

    // check that this bay is empty
    if !(*rc).is_null() {
        if ((*(*rc)).card_flags & CISCO_CARD_FLAG_OVERRIDE) != 0 {
            vm_slot_remove_binding(vm, slot_id, port_id);
        } else {
            vm_error!(vm, cstr!("a card already exists in slot %u/%u (%s)\n"), slot_id, port_id, (*(*rc)).dev_type);
            return -1;
        }
    }

    let card: *mut cisco_card = (*vm).slots[slot_id as usize];

    if card.is_null() || card == (*rc) {
        // Main slot
        drv_array = (*vm).slots_drivers;
        card_type = (*vm).slots_type;
        card_id = slot_id;
        parent = null_mut();
    } else {
        // Subslot
        if (*(*card).driver).card_get_sub_info.is_none() {
            vm_error!(vm, cstr!("no sub-slot possible for slot %u/%u.\n"), slot_id, port_id);
            return -1;
        }

        if (*(*card).driver).card_get_sub_info.unwrap()(vm, card, port_id, addr_of_mut!(drv_array), addr_of_mut!(card_type)) == -1 {
            vm_error!(vm, cstr!("no sub-slot info for slot %u/%u.\n"), slot_id, port_id);
            return -1;
        }

        card_id = port_id;
        parent = card;
    }

    assert!(!drv_array.is_null());

    // Find the card driver
    let driver: *mut cisco_card_driver = cisco_card_find_driver(drv_array, dev_type);
    if driver.is_null() {
        vm_error!(vm, cstr!("unknown card type '%s' for slot %u/%u.\n"), dev_type, slot_id, port_id);
        return -1;
    }

    // Allocate new card info
    let nc: *mut cisco_card = cisco_card_create(card_type);
    if nc.is_null() {
        return -1;
    }

    (*nc).slot_id = slot_id;
    (*nc).subslot_id = port_id;
    (*nc).card_id = card_id;
    (*nc).dev_type = (*driver).dev_type;
    (*nc).driver = driver;
    (*nc).parent = parent;
    *rc = nc;
    0
}

/// Remove a slot binding
#[no_mangle]
pub unsafe extern "C" fn vm_slot_remove_binding(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int) -> c_int {
    let mut rc: *mut *mut cisco_card = null_mut();
    let mut sc: *mut cisco_card;
    let mut real_port_id: u_int = 0;

    if vm_slot_get_info(vm, slot_id, port_id, addr_of_mut!(rc), addr_of_mut!(real_port_id)) == -1 {
        return -1;
    }

    if (*rc).is_null() {
        return -1;
    }

    if !(*(*rc)).drv_info.is_null() {
        vm_error!(vm, cstr!("slot %u/%u is still active\n"), slot_id, port_id);
        return -1;
    }

    for i in 0..CISCO_CARD_MAX_SUBSLOTS {
        sc = (*(*rc)).sub_slots[i];
        if !sc.is_null() {
            vm_error!(vm, cstr!("sub-slot %u/%u is still active\n"), slot_id, (*sc).subslot_id);
            return -1;
        }
    }

    // Remove all NIOs bindings
    vm_slot_remove_all_nio_bindings(vm, slot_id);

    // Free the card info structure
    libc::free((*rc).cast::<_>());
    *rc = null_mut();
    0
}

/// Add a network IO binding
#[no_mangle]
pub unsafe extern "C" fn vm_slot_add_nio_binding(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int, nio_name: *mut c_char) -> c_int {
    let mut rc: *mut cisco_card = null_mut();

    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Get the real card (in case this is a sub-slot)
    let real_port_id: u_int = vm_slot_translate_port_id(vm, slot_id, port_id, addr_of_mut!(rc));

    if rc.is_null() {
        return -1;
    }

    // check that a NIO is not already bound to this port
    if !cisco_card_find_nio_binding(rc, real_port_id).is_null() {
        vm_error!(vm, cstr!("a NIO already exists for interface %u/%u.\n"), slot_id, port_id);
        return -1;
    }

    // acquire a reference on the NIO object
    let nio: *mut netio_desc_t = netio_acquire(nio_name);
    if nio.is_null() {
        vm_error!(vm, cstr!("unable to find NIO '%s'.\n"), nio_name);
        return -1;
    }

    // create a new binding
    let nb: *mut cisco_nio_binding = libc::malloc(size_of::<cisco_nio_binding>()).cast::<_>();
    if nb.is_null() {
        vm_error!(vm, cstr!("unable to create NIO binding for interface %u/%u.\n"), slot_id, port_id);
        netio_release(nio_name);
        return -1;
    }

    libc::memset(nb.cast::<_>(), 0, size_of::<cisco_nio_binding>());
    (*nb).nio = nio;
    (*nb).port_id = real_port_id;
    (*nb).orig_port_id = port_id;

    (*nb).next = (*rc).nio_list;
    if !(*nb).next.is_null() {
        (*(*nb).next).prev = nb;
    }
    (*rc).nio_list = nb;
    0
}

/// Remove a NIO binding
#[no_mangle]
pub unsafe extern "C" fn vm_slot_remove_nio_binding(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int) -> c_int {
    let mut rc: *mut cisco_card = null_mut();

    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Get the real card (in case this is a sub-slot)
    let real_port_id: u_int = vm_slot_translate_port_id(vm, slot_id, port_id, addr_of_mut!(rc));

    if rc.is_null() {
        return -1;
    }

    // no nio binding for this slot/port ?
    let nb: *mut cisco_nio_binding = cisco_card_find_nio_binding(rc, real_port_id);
    if nb.is_null() {
        return -1;
    }

    // tell the NM driver to stop using this NIO
    if !(*rc).driver.is_null() {
        (*(*rc).driver).card_unset_nio.unwrap()(vm, rc, port_id);
    }

    // remove this entry from the double linked list
    if !(*nb).next.is_null() {
        (*(*nb).next).prev = (*nb).prev;
    }

    if !(*nb).prev.is_null() {
        (*(*nb).prev).next = (*nb).next;
    } else {
        (*rc).nio_list = (*nb).next;
    }

    // unreference NIO object
    netio_release((*(*nb).nio).name);
    libc::free(nb.cast::<_>());
    0
}

/// Remove all NIO bindings for the specified slot (sub-slots included)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_remove_all_nio_bindings(vm: *mut vm_instance_t, slot_id: u_int) -> c_int {
    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Remove NIO bindings for the main slot
    cisco_card_remove_all_nio_bindings(vm, card);

    // Remove NIO bindings for all sub-slots
    for i in 0..CISCO_CARD_MAX_SUBSLOTS {
        let sc: *mut cisco_card = (*card).sub_slots[i];
        if !sc.is_null() {
            cisco_card_remove_all_nio_bindings(vm, sc);
        }
    }

    0
}

/// Enable a Network IO descriptor for the specified slot
#[no_mangle]
pub unsafe extern "C" fn vm_slot_enable_nio(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int) -> c_int {
    let mut rc: *mut cisco_card = null_mut();

    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Get the real card (in case this is a sub-slot)
    let real_port_id: u_int = vm_slot_translate_port_id(vm, slot_id, port_id, addr_of_mut!(rc));

    if rc.is_null() {
        return -1;
    }

    // no nio binding for this slot/port ?
    let nb: *mut cisco_nio_binding = cisco_card_find_nio_binding(rc, real_port_id);
    if !nb.is_null() {
        return -1;
    }

    // check that the driver is defined and successfully initialized
    if (*rc).driver.is_null() || (*rc).drv_info.is_null() {
        return -1;
    }

    (*(*rc).driver).card_set_nio.unwrap()(vm, rc, real_port_id, (*nb).nio)
}

/// Disable Network IO descriptor for the specified slot
#[no_mangle]
pub unsafe extern "C" fn vm_slot_disable_nio(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int) -> c_int {
    let mut rc: *mut cisco_card = null_mut();

    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Get the real card (in case this is a sub-slot)
    let real_port_id: u_int = vm_slot_translate_port_id(vm, slot_id, port_id, addr_of_mut!(rc));

    if rc.is_null() {
        return -1;
    }

    // no nio binding for this slot/port ?
    let nb: *mut cisco_nio_binding = cisco_card_find_nio_binding(rc, real_port_id);
    if nb.is_null() {
        return -1;
    }

    // check that the driver is defined and successfully initialized
    if (*rc).driver.is_null() || (*rc).drv_info.is_null() {
        return -1;
    }

    (*(*rc).driver).card_unset_nio.unwrap()(vm, rc, real_port_id)
}

/// Enable all NIO for the specified slot (sub-slots included)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_enable_all_nio(vm: *mut vm_instance_t, slot_id: u_int) -> c_int {
    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Enable slot NIOs
    cisco_card_enable_all_nio(vm, card);

    // Enable NIO of sub-slots
    for i in 0..CISCO_CARD_MAX_SUBSLOTS {
        cisco_card_enable_all_nio(vm, (*card).sub_slots[i]);
    }

    0
}

/// Disable all NIO for the specified slot (sub-slots included)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_disable_all_nio(vm: *mut vm_instance_t, slot_id: u_int) -> c_int {
    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Disable slot NIOs
    cisco_card_disable_all_nio(vm, card);

    // Disable NIO of sub-slots
    for i in 0..CISCO_CARD_MAX_SUBSLOTS {
        cisco_card_disable_all_nio(vm, (*card).sub_slots[i]);
    }

    0
}

/// Initialize the specified slot (sub-slots included)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_init(vm: *mut vm_instance_t, slot_id: u_int) -> c_int {
    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return 0;
    }

    // Initialize card main module
    cisco_card_init(vm, card, slot_id);

    // Initialize sub-slots
    for i in 0..CISCO_CARD_MAX_SUBSLOTS {
        cisco_card_init(vm, (*card).sub_slots[i], slot_id);
    }

    // Enable all NIO
    vm_slot_enable_all_nio(vm, slot_id);
    0
}

/// Initialize all slots of a VM
#[no_mangle]
pub unsafe extern "C" fn vm_slot_init_all(vm: *mut vm_instance_t) -> c_int {
    for i in 0..(*vm).nr_slots {
        if vm_slot_init(vm, i) == -1 {
            vm_error!(vm, cstr!("unable to initialize slot %u\n"), i);
            return -1;
        }
    }

    0
}

/// Shutdown the specified slot (sub-slots included)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_shutdown(vm: *mut vm_instance_t, slot_id: u_int) -> c_int {
    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Disable all NIO
    vm_slot_disable_all_nio(vm, slot_id);

    // Shutdown sub-slots
    for i in 0..CISCO_CARD_MAX_SUBSLOTS {
        cisco_card_shutdown(vm, (*card).sub_slots[i]);
    }

    // Shutdown card main module
    cisco_card_shutdown(vm, card);
    0
}

/// Shutdown all slots of a VM
#[no_mangle]
pub unsafe extern "C" fn vm_slot_shutdown_all(vm: *mut vm_instance_t) -> c_int {
    for i in 0..(*vm).nr_slots {
        vm_slot_shutdown(vm, i);
    }

    0
}

/// Show info about the specified slot (sub-slots included)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_show_info(vm: *mut vm_instance_t, slot_id: u_int) -> c_int {
    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    cisco_card_show_info(vm, card);
    0
}

/// Show info about all slots
#[no_mangle]
pub unsafe extern "C" fn vm_slot_show_all_info(vm: *mut vm_instance_t) -> c_int {
    for i in 0..(*vm).nr_slots {
        vm_slot_show_info(vm, i);
    }

    0
}

/// Check if the specified slot has a valid EEPROM defined
#[no_mangle]
pub unsafe extern "C" fn vm_slot_check_eeprom(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int) -> c_int {
    let mut rc: *mut cisco_card = null_mut();

    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return FALSE;
    }

    // Get the real card (in case this is a sub-slot)
    vm_slot_translate_port_id(vm, slot_id, port_id, addr_of_mut!(rc));

    if rc.is_null() {
        return FALSE;
    }

    cisco_card_check_eeprom(rc)
}

/// Returns the EEPROM data of the specified slot
#[no_mangle]
pub unsafe extern "C" fn vm_slot_get_eeprom(vm: *mut vm_instance_t, slot_id: u_int, port_id: u_int) -> *mut cisco_eeprom {
    let mut rc: *mut cisco_card = null_mut();

    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return null_mut();
    }

    // Get the real card (in case this is a sub-slot)
    vm_slot_translate_port_id(vm, slot_id, port_id, addr_of_mut!(rc));

    if rc.is_null() {
        return null_mut();
    }

    addr_of_mut!((*rc).eeprom)
}

/// Save config for the specified slot (sub-slots included)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_save_config(vm: *mut vm_instance_t, slot_id: u_int, fd: *mut libc::FILE) -> c_int {
    let card: *mut cisco_card = vm_slot_get_card_ptr(vm, slot_id);
    if card.is_null() {
        return -1;
    }

    // Main slot info
    cisco_card_save_config(vm, card, fd);

    // Shutdown sub-slots
    for i in 0..CISCO_CARD_MAX_SUBSLOTS {
        cisco_card_save_config(vm, (*card).sub_slots[i], fd);
    }

    0
}

/// Save config for all slots
#[no_mangle]
pub unsafe extern "C" fn vm_slot_save_all_config(vm: *mut vm_instance_t, fd: *mut libc::FILE) -> c_int {
    for i in 0..(*vm).nr_slots {
        vm_slot_save_config(vm, i, fd);
    }

    0
}

/// Show slot drivers
#[no_mangle]
pub unsafe extern "C" fn vm_slot_show_drivers(vm: *mut vm_instance_t) -> c_int {
    if (*vm).slots_drivers.is_null() {
        return -1;
    }

    let slot_type: *mut c_char = cisco_card_get_type_desc((*vm).slots_type as c_int);

    libc::printf(cstr!("Available %s %s drivers:\n"), (*(*vm).platform).log_name, slot_type);

    for i in 0.. {
        if (*(*vm).slots_drivers.add(i)).is_null() {
            break;
        }
        libc::printf(cstr!("  * %s %s\n"), (*(*(*vm).slots_drivers.add(i))).dev_type, if (*(*(*vm).slots_drivers.add(i))).supported == 0 { cstr!("(NOT WORKING)") } else { cstr!("") });
    }

    libc::printf(cstr!("\n"));
    0
}

/// Maximum number of tokens in a slot description
const SLOT_DESC_MAX_TOKENS: usize = 8;

/// Create a Network Module (command line)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_cmd_create(vm: *mut vm_instance_t, str_: *mut c_char) -> c_int {
    let mut tokens: [*mut c_char; SLOT_DESC_MAX_TOKENS] = [null_mut(); SLOT_DESC_MAX_TOKENS];

    // A port adapter description is like "1:0:NM-1FE"
    let count: c_int = m_strsplit(str_, b':' as c_char, tokens.as_c_mut(), SLOT_DESC_MAX_TOKENS as c_int);

    #[allow(clippy::manual_range_contains)]
    if (count < 2) || (count > 3) {
        vm_error!(vm, cstr!("unable to parse slot description '%s'.\n"), str_);
        return -1;
    }

    // Parse the slot id
    let slot_id: u_int = libc::atoi(tokens[0]) as u_int;

    // Parse the sub-slot id
    let port_id: u_int = if count == 3 { libc::atoi(tokens[1]) as u_int } else { 0 };

    // Add this new slot to the current slot list
    let res: c_int = vm_slot_add_binding(vm, tokens[count as usize - 1], slot_id, port_id);

    // The complete array was cleaned by strsplit
    #[allow(clippy::needless_range_loop)]
    for i in 0..SLOT_DESC_MAX_TOKENS {
        libc::free(tokens[i].cast::<_>());
    }

    res
}

/// Add a Network IO descriptor binding (command line)
#[no_mangle]
pub unsafe extern "C" fn vm_slot_cmd_add_nio(vm: *mut vm_instance_t, str_: *mut c_char) -> c_int {
    let mut tokens: [*mut c_char; SLOT_DESC_MAX_TOKENS] = [null_mut(); SLOT_DESC_MAX_TOKENS];
    let mut res: c_int = -1;
    let mut nio_name: [c_char; 128] = [0; 128];

    // A NIO binding description is like "1:3:tap:tap0"
    let count: c_int = m_strsplit(str_, b':' as c_char, tokens.as_c_mut(), SLOT_DESC_MAX_TOKENS as c_int);
    if count < 3 {
        vm_error!(vm, cstr!("unable to parse NIO description '%s'.\n"), str_);
        return -1;
    }

    // Parse the slot id
    let slot_id: u_int = libc::atoi(tokens[0]) as u_int;

    // Parse the port id
    let port_id: u_int = libc::atoi(tokens[1]) as u_int;

    // Autogenerate a NIO name
    libc::snprintf(nio_name.as_c_mut(), nio_name.len(), cstr!("%s-i%u/%u/%u"), vm_get_type(vm), (*vm).instance_id, slot_id, port_id);

    // Create the Network IO descriptor
    let mut nio: *mut netio_desc_t = null_mut();
    let nio_type: c_int = netio_get_type(tokens[2]);

    'done: {
        match nio_type as u_int {
            NETIO_TYPE_UNIX => {
                if count != 5 {
                    vm_error!(vm, cstr!("invalid number of arguments for UNIX NIO '%s'\n"), str_);
                    break 'done;
                }

                nio = netio_desc_create_unix(nio_name.as_c_mut(), tokens[3], tokens[4]);
            }

            NETIO_TYPE_VDE => {
                if count != 5 {
                    vm_error!(vm, cstr!("invalid number of arguments for VDE NIO '%s'\n"), str_);
                    break 'done;
                }

                nio = netio_desc_create_vde(nio_name.as_c_mut(), tokens[3], tokens[4]);
            }

            NETIO_TYPE_TAP => {
                if count != 4 {
                    vm_error!(vm, cstr!("invalid number of arguments for TAP NIO '%s'\n"), str_);
                    break 'done;
                }

                nio = netio_desc_create_tap(nio_name.as_c_mut(), tokens[3]);
            }

            NETIO_TYPE_UDP => {
                if count != 6 {
                    vm_error!(vm, cstr!("invalid number of arguments for UDP NIO '%s'\n"), str_);
                    break 'done;
                }

                nio = netio_desc_create_udp(nio_name.as_c_mut(), libc::atoi(tokens[3]), tokens[4], libc::atoi(tokens[5]));
            }

            NETIO_TYPE_TCP_CLI => {
                if count != 5 {
                    vm_error!(vm, cstr!("invalid number of arguments for TCP CLI NIO '%s'\n"), str_);
                    break 'done;
                }

                nio = netio_desc_create_tcp_cli(nio_name.as_c_mut(), tokens[3], tokens[4]);
            }

            NETIO_TYPE_TCP_SER => {
                if count != 4 {
                    vm_error!(vm, cstr!("invalid number of arguments for TCP SER NIO '%s'\n"), str_);
                    break 'done;
                }

                nio = netio_desc_create_tcp_ser(nio_name.as_c_mut(), tokens[3]);
            }

            NETIO_TYPE_NULL => {
                nio = netio_desc_create_null(nio_name.as_c_mut());
            }

            #[cfg(feature = "ENABLE_LINUX_ETH")]
            NETIO_TYPE_LINUX_ETH => {
                if count != 4 {
                    vm_error!(vm, cstr!("invalid number of arguments for Linux Eth NIO '%s'\n"), str_);
                    break 'done;
                }

                nio = netio_desc_create_lnxeth(nio_name.as_c_mut(), tokens[3]);
            }

            #[cfg(feature = "ENABLE_GEN_ETH")]
            NETIO_TYPE_GEN_ETH => {
                if count != 4 {
                    vm_error!(vm, cstr!("invalid number of arguments for Generic Eth NIO '%s'\n"), str_);
                    break 'done;
                }

                nio = netio_desc_create_geneth(nio_name.as_c_mut(), tokens[3]);
            }

            _ => {
                vm_error!(vm, cstr!("unknown NETIO type '%s'\n"), tokens[2]);
                break 'done;
            }
        }

        if nio.is_null() {
            vm_error!(vm, cstr!("unable to create NETIO descriptor for slot %u\n"), slot_id);
            break 'done;
        }

        if vm_slot_add_nio_binding(vm, slot_id, port_id, nio_name.as_c_mut()) == -1 {
            vm_error!(vm, cstr!("unable to add NETIO binding for slot %u\n"), slot_id);
            netio_release(nio_name.as_c_mut());
            netio_delete(nio_name.as_c_mut());
            break 'done;
        }

        netio_release(nio_name.as_c_mut());
        res = 0;
    }

    let _ = nio;
    // The complete array was cleaned by strsplit
    #[allow(clippy::needless_range_loop)]
    for i in 0..SLOT_DESC_MAX_TOKENS {
        libc::free(tokens[i].cast::<_>());
    }

    res
}
