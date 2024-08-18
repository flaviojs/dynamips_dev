/*
 * Cisco 3745 simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco 3745 routines and definitions (EEPROM,...).
 */

#ifndef __DEV_C3745_H__
#define __DEV_C3745_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"
#include "device.h"
#include "pci_dev.h"
#include "dev_gt.h"
#include "net_io.h"
#include "vm.h"

/* Get WIC device address for the specified onboard port */
int c3745_get_onboard_wic_addr(u_int slot,m_uint64_t *phys_addr);

/* Set EEPROM for the specified slot */
int c3745_set_slot_eeprom(c3745_t *router,u_int slot,
                          struct cisco_eeprom *eeprom);

/* Get network IRQ for specified slot/port */
u_int c3745_net_irq_for_slot_port(u_int slot,u_int port);

/* Set chassis MAC address */
int c3745_chassis_set_mac_addr(c3745_t *router,char *mac_addr);

/* Set the system id */
int c3745_set_system_id(c3745_t *router,char *id);

/* Burn the system id into the appropriate eeprom if possible */
int c3745_refresh_systemid(c3745_t *router);

/* Show C3745 hardware info */
void c3745_show_hardware(c3745_t *router);

/* Initialize EEPROM groups */
void c3745_init_eeprom_groups(c3745_t *router);

/* dev_c3745_iofpga_init() */
int dev_c3745_iofpga_init(c3745_t *router,m_uint64_t paddr,m_uint32_t len);

/* Register the c3745 platform */
int c3745_platform_register(void);

/* Hypervisor C3745 initialization */
extern int hypervisor_c3745_init(vm_platform_t *platform);

/* NM drivers */
extern struct cisco_card_driver dev_c3745_nm_1fe_tx_driver;
extern struct cisco_card_driver dev_c3745_gt96100_fe_driver;
extern struct cisco_card_driver dev_c3745_nm_4t_driver;
extern struct cisco_card_driver dev_c3745_nm_16esw_driver;
extern struct cisco_card_driver dev_c3745_nmd_36esw_driver;
extern struct cisco_card_driver dev_c3745_nm_nam_driver;
extern struct cisco_card_driver dev_c3745_nm_cids_driver;

/* WIC drivers */
extern struct cisco_card_driver *dev_c3745_mb_wic_drivers[];

#endif
