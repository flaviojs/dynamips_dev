/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco card routines and definitions.
 */

#ifndef __CISCO_CARD_H__
#define __CISCO_CARD_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"

/* Set EEPROM definition for the specified Cisco card */
int cisco_card_set_eeprom(vm_instance_t *vm,struct cisco_card *card,
                          const struct cisco_eeprom *eeprom);

/* Unset EEPROM definition */
int cisco_card_unset_eeprom(struct cisco_card *card);

/* Check if a card has a valid EEPROM defined */
int cisco_card_check_eeprom(struct cisco_card *card);

/* Get slot info */
struct cisco_card *vm_slot_get_card_ptr(vm_instance_t *vm,u_int slot_id);

/* Check if a slot has an active card */
int vm_slot_active(vm_instance_t *vm,u_int slot_id,u_int port_id);

/* Set a flag for a card */
int vm_slot_set_flag(vm_instance_t *vm,u_int slot_id,u_int port_id,u_int flag);

/* Add a slot binding */
int vm_slot_add_binding(vm_instance_t *vm,char *dev_type,
                        u_int slot_id,u_int port_id);

/* Remove a slot binding */
int vm_slot_remove_binding(vm_instance_t *vm,u_int slot_id,u_int port_id);

/* Add a network IO binding */
int vm_slot_add_nio_binding(vm_instance_t *vm,u_int slot_id,u_int port_id,
                            char *nio_name);

/* Remove a NIO binding */
int vm_slot_remove_nio_binding(vm_instance_t *vm,u_int slot_id,u_int port_id);

/* Remove all NIO bindings for the specified slot (sub-slots included) */
int vm_slot_remove_all_nio_bindings(vm_instance_t *vm,u_int slot_id);

/* Enable a Network IO descriptor for the specified slot */
int vm_slot_enable_nio(vm_instance_t *vm,u_int slot_id,u_int port_id);

/* Disable Network IO descriptor for the specified slot */
int vm_slot_disable_nio(vm_instance_t *vm,u_int slot_id,u_int port_id);

/* Enable all NIO for the specified slot (sub-slots included) */
int vm_slot_enable_all_nio(vm_instance_t *vm,u_int slot_id);

/* Disable all NIO for the specified slot (sub-slots included) */
int vm_slot_disable_all_nio(vm_instance_t *vm,u_int slot_id);

/* Initialize the specified slot (sub-slots included) */
int vm_slot_init(vm_instance_t *vm,u_int slot_id);

/* Initialize all slots of a VM */
int vm_slot_init_all(vm_instance_t *vm);

/* Shutdown the specified slot (sub-slots included) */
int vm_slot_shutdown(vm_instance_t *vm,u_int slot_id);

/* Shutdown all slots of a VM */
int vm_slot_shutdown_all(vm_instance_t *vm);

/* Show info about the specified slot (sub-slots included) */
int vm_slot_show_info(vm_instance_t *vm,u_int slot_id);

/* Show info about all slots */
int vm_slot_show_all_info(vm_instance_t *vm);

/* Check if the specified slot has a valid EEPROM defined */
int vm_slot_check_eeprom(vm_instance_t *vm,u_int slot_id,u_int port_id);

/* Returns the EEPROM data of the specified slot */
struct cisco_eeprom *
vm_slot_get_eeprom(vm_instance_t *vm,u_int slot_id,u_int port_id);

/* Save config for the specified slot (sub-slots included) */
int vm_slot_save_config(vm_instance_t *vm,u_int slot_id,FILE *fd);

/* Save config for all slots */
int vm_slot_save_all_config(vm_instance_t *vm,FILE *fd);

/* Show slot drivers */
int vm_slot_show_drivers(vm_instance_t *vm);

/* Create a Network Module (command line) */
int vm_slot_cmd_create(vm_instance_t *vm,char *str);

/* Add a Network IO descriptor binding (command line) */
int vm_slot_cmd_add_nio(vm_instance_t *vm,char *str);

#endif
