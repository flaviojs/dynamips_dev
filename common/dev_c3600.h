/*
 * Cisco 3600 simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco 3600 routines and definitions (EEPROM,...).
 */

#ifndef __DEV_C3600_H__
#define __DEV_C3600_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"
#include "device.h"
#include "net_io.h"
#include "vm.h"

/* Set EEPROM for the specified slot */
int c3600_set_slot_eeprom(c3600_t *router,u_int slot,
                          struct cisco_eeprom *eeprom);

/* Get network IRQ for specified slot/port */
u_int c3600_net_irq_for_slot_port(u_int slot,u_int port);

/* Show the list of available NM drivers */
void c3600_nm_show_drivers(void);

/* Set chassis MAC address */
int c3600_chassis_set_mac_addr(c3600_t *router,char *mac_addr);

/* Set the system id */
int c3600_set_system_id(c3600_t *router,char *id);

/* Burn the system id into the appropriate eeprom if possible */
int c3600_refresh_systemid(c3600_t *router);

/* Set the chassis type */
int c3600_chassis_set_type(c3600_t *router,char *chassis_type);

/* Get the chassis ID */
int c3600_chassis_get_id(c3600_t *router);

/* Show C3600 hardware info */
void c3600_show_hardware(c3600_t *router);

/* Initialize EEPROM groups */
void c3600_init_eeprom_groups(c3600_t *router);

/* dev_c3600_iofpga_init() */
int dev_c3600_iofpga_init(c3600_t *router,m_uint64_t paddr,m_uint32_t len);

/* Register the c3600 platform */
int c3600_platform_register(void);

/* Hypervisor C3600 initialization */
extern int hypervisor_c3600_init(vm_platform_t *platform);

/* NM drivers */
extern struct cisco_card_driver dev_c3600_nm_1e_driver;
extern struct cisco_card_driver dev_c3600_nm_4e_driver;
extern struct cisco_card_driver dev_c3600_nm_1fe_tx_driver;
extern struct cisco_card_driver dev_c3600_nm_4t_driver;
extern struct cisco_card_driver dev_c3600_leopard_2fe_driver;
extern struct cisco_card_driver dev_c3600_nm_16esw_driver;
extern struct cisco_card_driver dev_c3600_nmd_36esw_driver;

#endif
