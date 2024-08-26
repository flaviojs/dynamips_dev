/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco 1700 routines and definitions (EEPROM,...).
 */

#ifndef __DEV_C1700_H__
#define __DEV_C1700_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"
#include "dev_mpc860.h"
#include "vm.h"

/* Get WIC device address for the specified onboard port */
int c1700_get_onboard_wic_addr(u_int slot,m_uint64_t *phys_addr);

/* Set EEPROM for the specified slot */
int c1700_set_slot_eeprom(c1700_t *router,u_int slot,
                          struct cisco_eeprom *eeprom);

/* Get network IRQ for specified slot/port */
u_int c1700_net_irq_for_slot_port(u_int slot,u_int port);

/* Set mainboard type */
int c1700_mainboard_set_type(c1700_t *router,char *mainboard_type);

/* Set chassis MAC address */
int c1700_chassis_set_mac_addr(c1700_t *router,char *mac_addr);

/* Set the system id */
int c1700_set_system_id(c1700_t *router,char *id);

/* Burn the system id into the appropriate eeprom if possible */
int c1700_refresh_systemid(c1700_t *router);

/* Show C1700 hardware info */
void c1700_show_hardware(c1700_t *router);

/* Initialize EEPROM groups */
void c1700_init_eeprom_groups(c1700_t *router);

/* dev_c1700_iofpga_init() */
int dev_c1700_iofpga_init(c1700_t *router,m_uint64_t paddr,m_uint32_t len);

/* Register the c1700 platform */
int c1700_platform_register(void);

/* Hypervisor C1700 initialization */
extern int hypervisor_c1700_init(vm_platform_t *platform);

/* c1700 Motherboard drivers */
extern struct cisco_card_driver dev_c1700_mb_eth_driver;
extern struct cisco_card_driver dev_c1710_mb_eth_driver;

/* WIC drivers */
extern struct cisco_card_driver *dev_c1700_mb_wic_drivers[];

#endif
