/*
 * Cisco 2691 simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco 2691 routines and definitions (EEPROM,...).
 */

#ifndef __DEV_C2691_H__
#define __DEV_C2691_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"
#include "device.h"
#include "net_io.h"
#include "vm.h"

/* Get WIC device address for the specified onboard port */
int c2691_get_onboard_wic_addr(u_int slot,m_uint64_t *phys_addr);

/* Set EEPROM for the specified slot */
int c2691_set_slot_eeprom(c2691_t *router,u_int slot,
                          struct cisco_eeprom *eeprom);

/* Get network IRQ for specified slot/port */
u_int c2691_net_irq_for_slot_port(u_int slot,u_int port);

/* Set chassis MAC address */
int c2691_chassis_set_mac_addr(c2691_t *router,char *mac_addr);

/* Set the system id */
int c2691_set_system_id(c2691_t *router,char *id);

/* Burn the system id into the appropriate eeprom if possible */
int c2691_refresh_systemid(c2691_t *router);

/* Show C2691 hardware info */
void c2691_show_hardware(c2691_t *router);

/* Initialize EEPROM groups */
void c2691_init_eeprom_groups(c2691_t *router);

/* dev_c2691_iofpga_init() */
int dev_c2691_iofpga_init(c2691_t *router,m_uint64_t paddr,m_uint32_t len);

/* Register the c2691 platform */
int c2691_platform_register(void);

/* Hypervisor C2691 initialization */
extern int hypervisor_c2691_init(vm_platform_t *platform);

/* NM drivers */
extern struct cisco_card_driver dev_c2691_nm_1fe_tx_driver;
extern struct cisco_card_driver dev_c2691_gt96100_fe_driver;
extern struct cisco_card_driver dev_c2691_nm_4t_driver;
extern struct cisco_card_driver dev_c2691_nm_16esw_driver;
extern struct cisco_card_driver dev_c2691_nm_nam_driver;
extern struct cisco_card_driver dev_c2691_nm_cids_driver;

/* WIC drivers */
extern struct cisco_card_driver *dev_c2691_mb_wic_drivers[];

#endif
