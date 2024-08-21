/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco 3725 routines and definitions (EEPROM,...).
 */

#ifndef __DEV_C3725_H__
#define __DEV_C3725_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"
#include "device.h"
#include "net_io.h"
#include "vm.h"

/* Get WIC device address for the specified onboard port */
int c3725_get_onboard_wic_addr(u_int slot,m_uint64_t *phys_addr);

/* Set EEPROM for the specified slot */
int c3725_set_slot_eeprom(c3725_t *router,u_int slot,
                          struct cisco_eeprom *eeprom);

/* Get network IRQ for specified slot/port */
u_int c3725_net_irq_for_slot_port(u_int slot,u_int port);

/* Get PCI device for the specified NM bay */
int c3725_nm_get_pci_device(u_int nm_bay);

/* Set chassis MAC address */
int c3725_chassis_set_mac_addr(c3725_t *router,char *mac_addr);

/* Set the system id */
int c3725_set_system_id(c3725_t *router,char *id);

/* Burn the system id into the appropriate eeprom if possible */
int c3725_refresh_systemid(c3725_t *router);

/* Show C3725 hardware info */
void c3725_show_hardware(c3725_t *router);

/* Initialize EEPROM groups */
void c3725_init_eeprom_groups(c3725_t *router);

/* Register the c3725 platform */
int c3725_platform_register(void);

/* Hypervisor C3725 initialization */
extern int hypervisor_c3725_init(vm_platform_t *platform);

/* NM drivers */
extern struct cisco_card_driver dev_c3725_nm_1fe_tx_driver;
extern struct cisco_card_driver dev_c3725_gt96100_fe_driver;
extern struct cisco_card_driver dev_c3725_nm_4t_driver;
extern struct cisco_card_driver dev_c3725_nm_16esw_driver;
extern struct cisco_card_driver dev_c3725_nmd_36esw_driver;
extern struct cisco_card_driver dev_c3725_nm_nam_driver;
extern struct cisco_card_driver dev_c3725_nm_cids_driver;

/* WIC drivers */
extern struct cisco_card_driver *dev_c3725_mb_wic_drivers[];

#endif
