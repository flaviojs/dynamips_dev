/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco 2600 routines and definitions (EEPROM,...).
 */

#ifndef __DEV_C2600_H__
#define __DEV_C2600_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"
#include "dev_mpc860.h"
#include "vm.h"

/* Get WIC device address for the specified onboard port */
int c2600_get_onboard_wic_addr(u_int slot,m_uint64_t *phys_addr);

/* Set EEPROM for the specified slot */
int c2600_set_slot_eeprom(c2600_t *router,u_int slot,
                          struct cisco_eeprom *eeprom);

/* Get network IRQ for specified slot/port */
u_int c2600_net_irq_for_slot_port(u_int slot,u_int port);

/* Set mainboard type */
int c2600_mainboard_set_type(c2600_t *router,char *mainboard_type);

/* Set chassis MAC address */
int c2600_chassis_set_mac_addr(c2600_t *router,char *mac_addr);

/* Set the system id */
int c2600_set_system_id(c2600_t *router,char *id);

/* Burn the system id into the appropriate eeprom if possible */
int c2600_refresh_systemid(c2600_t *router);

/* Show C2600 hardware info */
void c2600_show_hardware(c2600_t *router);

/* Initialize EEPROM groups */
void c2600_init_eeprom_groups(c2600_t *router);

/* Create the c2600 PCI controller device */
int dev_c2600_pci_init(vm_instance_t *vm,char *name,
                       m_uint64_t paddr,m_uint32_t len,
                       struct pci_bus *bus);

/* dev_c2600_iofpga_init() */
int dev_c2600_iofpga_init(c2600_t *router,m_uint64_t paddr,m_uint32_t len);

/* Register the c2600 platform */
int c2600_platform_register(void);

/* Hypervisor C2600 initialization */
extern int hypervisor_c2600_init(vm_platform_t *platform);

/* NM drivers */
extern struct cisco_card_driver dev_c2600_mb1e_eth_driver;
extern struct cisco_card_driver dev_c2600_mb2e_eth_driver;
extern struct cisco_card_driver dev_c2600_mb1fe_eth_driver;
extern struct cisco_card_driver dev_c2600_mb2fe_eth_driver;

extern struct cisco_card_driver dev_c2600_nm_1e_driver;
extern struct cisco_card_driver dev_c2600_nm_4e_driver;
extern struct cisco_card_driver dev_c2600_nm_1fe_tx_driver;
extern struct cisco_card_driver dev_c2600_nm_16esw_driver;

extern struct cisco_card_driver dev_c2600_nm_nam_driver;
extern struct cisco_card_driver dev_c2600_nm_cids_driver;

/* WIC drivers */
extern struct cisco_card_driver *dev_c2600_mb_wic_drivers[];

#endif
