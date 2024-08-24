/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco 7200 routines and definitions (EEPROM,...).
 *
 * Notes on IRQs (see "show stack"):
 *
 *   - triggering IRQ 3: we get indefinitely (for each slot):
 *        "Error: Unexpected NM Interrupt received from slot: 6"
 *
 *   - triggering IRQ 4: GT64010 reg access: probably "DMA/Timer Interrupt"
 *
 *   - triggering IRQ 6: we get (probably "OIR/Error Interrupt")
 *        %ERR-1-PERR: PCI bus parity error
 *        %ERR-1-SERR: PCI bus system/parity error
 *        %ERR-1-FATAL: Fatal error interrupt, No reloading
 *        err_stat=0x0, err_enable=0x0, mgmt_event=0xFFFFFFFF
 *
 */

#ifndef __DEV_C7200_H__
#define __DEV_C7200_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"
#include "dev_mv64460.h"
#include "net_io.h"
#include "vm.h"

/* Initialize system EEPROM groups */
void c7200_init_sys_eeprom_groups(c7200_t *router);

/* Initialize midplane EEPROM groups */
void c7200_init_mp_eeprom_groups(c7200_t *router);

/* Returns TRUE if the specified card in slot 0 is an I/O card */
int c7200_slot0_iocard_present(c7200_t *router);

/* Set EEPROM for the specified slot */
int c7200_set_slot_eeprom(c7200_t *router,u_int slot,
                          struct cisco_eeprom *eeprom);

/* Get network IRQ for specified slot/port */
u_int c7200_net_irq_for_slot_port(u_int slot,u_int port);

/* Get register offset for the specified slot */
u_int dev_c7200_net_get_reg_offset(u_int slot);

/* Update network interrupt status */
void dev_c7200_net_update_irq(c7200_t *router);

/* Show the list of available PA drivers */
void c7200_pa_show_drivers(void);

/* Get an NPE driver */
struct c7200_npe_driver *c7200_npe_get_driver(char *npe_type);

/* Set the NPE type */
int c7200_npe_set_type(c7200_t *router,char *npe_type);

/* Set Midplane type */
int c7200_midplane_set_type(c7200_t *router,char *midplane_type);

/* Set chassis MAC address */
int c7200_midplane_set_mac_addr(c7200_t *router,char *mac_addr);

/* Show C7200 hardware info */
void c7200_show_hardware(c7200_t *router);

/* dev_c7200_iofpga_init() */
int dev_c7200_iofpga_init(c7200_t *router,m_uint64_t paddr,m_uint32_t len);

/* Register the c7200 platform */
int c7200_platform_register(void);

/* Set the system id */
int c7200_set_system_id(c7200_t *router,char *id);

/* Burn the system id into the appropriate eeprom if possible */
int c7200_refresh_systemid(c7200_t *router);

/* Hypervisor C7200 initialization */
extern int hypervisor_c7200_init(vm_platform_t *platform);

/* PA drivers */
extern struct cisco_card_driver dev_c7200_npeg2_driver;
extern struct cisco_card_driver dev_c7200_iocard_fe_driver;
extern struct cisco_card_driver dev_c7200_iocard_2fe_driver;
extern struct cisco_card_driver dev_c7200_iocard_ge_e_driver;
extern struct cisco_card_driver dev_c7200_pa_fe_tx_driver;
extern struct cisco_card_driver dev_c7200_pa_2fe_tx_driver;
extern struct cisco_card_driver dev_c7200_pa_ge_driver;
extern struct cisco_card_driver dev_c7200_pa_4e_driver;
extern struct cisco_card_driver dev_c7200_pa_8e_driver;
extern struct cisco_card_driver dev_c7200_pa_4t_driver;
extern struct cisco_card_driver dev_c7200_pa_8t_driver;
extern struct cisco_card_driver dev_c7200_pa_a1_driver;
extern struct cisco_card_driver dev_c7200_pa_pos_oc3_driver;
extern struct cisco_card_driver dev_c7200_pa_4b_driver;
extern struct cisco_card_driver dev_c7200_pa_mc8te1_driver;
extern struct cisco_card_driver dev_c7200_jcpa_driver;

#endif
