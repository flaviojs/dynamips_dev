/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Cisco MSFC1 routines and definitions (EEPROM,...).
 */

#ifndef __DEV_C6MSFC1_H__
#define __DEV_C6MSFC1_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"
#include "net_io.h"
#include "vm.h"

/* Initialize EEPROM groups */
void c6msfc1_init_eeprom_groups(c6msfc1_t *router);

/* Get network IRQ for specified slot/port */
u_int c6msfc1_net_irq_for_slot_port(u_int slot,u_int port);

/* Show the list of available PA drivers */
void c6msfc1_pa_show_drivers(void);

/* Set chassis MAC address */
int c6msfc1_midplane_set_mac_addr(c6msfc1_t *router,char *mac_addr);

/* Show MSFC1 hardware info */
void c6msfc1_show_hardware(c6msfc1_t *router);

/* dev_c6msfc1_iofpga_init() */
int dev_c6msfc1_iofpga_init(c6msfc1_t *router,m_uint64_t paddr,m_uint32_t len);

/* dev_mpfpga_init() */
int dev_c6msfc1_mpfpga_init(c6msfc1_t *router,m_uint64_t paddr,m_uint32_t len);

/* Register the c6msfc1 platform */
int c6msfc1_platform_register(void);

#endif
