//! Cisco router simulation platform.
//! Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
//!
//! Generic Cisco card routines and definitions.

use crate::_extra::*;
use crate::cisco_eeprom::*;
use crate::net_io::*;
use crate::pci_dev::*;
use crate::vm::*;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;

pub const CISCO_CARD_MAX_WIC: usize = 8;
pub const CISCO_CARD_MAX_SUBSLOTS: usize = 16;

// Card types
pub type _CISCO_CARD_TYPE_ENUM = c_int; // TODO enum
pub const CISCO_CARD_TYPE_UNDEF: _CISCO_CARD_TYPE_ENUM = 0;
pub const CISCO_CARD_TYPE_PA: _CISCO_CARD_TYPE_ENUM = 1;
pub const CISCO_CARD_TYPE_NM: _CISCO_CARD_TYPE_ENUM = 2;
pub const CISCO_CARD_TYPE_WIC: _CISCO_CARD_TYPE_ENUM = 3;

// Card flags
pub type _CISCO_CARD_FLAG_ENUM = c_int; // TODO enum
pub const CISCO_CARD_FLAG_OVERRIDE: _CISCO_CARD_FLAG_ENUM = 1;

// Prototype of card driver initialization function
pub type cisco_card_init_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card) -> c_int>;

// Prototype of card driver shutdown function
pub type cisco_card_shutdown_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card) -> c_int>;

// Prototype of card NIO get sub-slot info function
pub type cisco_card_get_sub_info_fn =
    Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card, port_id: u_int, drv_array: *mut *mut *mut cisco_card_driver, subcard_type: *mut u_int) -> c_int>;

// Prototype of card NIO set function
pub type cisco_card_set_nio_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card, port_id: u_int, nio: *mut netio_desc_t) -> c_int>;

// Prototype of card NIO unset function
pub type cisco_card_unset_nio_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card, port_id: u_int) -> c_int>;

// Prototype of card NIO show info function
pub type cisco_card_show_info_fn = Option<unsafe extern "C" fn(vm: *mut vm_instance_t, card: *mut cisco_card) -> c_int>;

// Cisco NIO binding to a slot/port
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cisco_nio_binding {
    pub nio: *mut netio_desc_t,
    pub port_id: u_int,
    pub orig_port_id: u_int,
    pub prev: *mut cisco_nio_binding,
    pub next: *mut cisco_nio_binding,
}

// Generic Cisco card driver
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

// Generic Cisco card
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
