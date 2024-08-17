/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot.
 *
 * NMC93C46/NMC93C56 Serial EEPROM.
 */

#ifndef __NMC93CX6_H__
#define __NMC93CX6_H__

#include "rust_dynamips_c.h"

#include <sys/types.h>
#include "utils.h"

/* Handle write */
void nmc93cX6_write(struct nmc93cX6_group *g,u_int data);

/* Returns the TRUE if the EEPROM is active */
u_int nmc93cX6_is_active(struct nmc93cX6_group *g,u_int group_id);

/* Returns the DOUT bit value */
u_int nmc93cX6_get_dout(struct nmc93cX6_group *g,u_int group_id);

/* Handle read */
u_int nmc93cX6_read(struct nmc93cX6_group *p);

#endif /* __NMC93CX6_H__ */
