
/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Dallas DS1620 Temperature sensors.
 */

#ifndef __DEV_DS1620_H__
#define __DEV_DS1620_H__

#include "rust_dynamips_c.h"

#include "dynamips_common.h"

/* Set temperature */
void ds1620_set_temp(struct ds1620_data *d,int temp);

/* Set reset bit */
void ds1620_set_rst_bit(struct ds1620_data *d,u_int rst_bit);

/* Write data bit */
void ds1620_write_data_bit(struct ds1620_data *d,u_int data_bit);

/* Read data bit */
u_int ds1620_read_data_bit(struct ds1620_data *d);

/* Initialize a DS1620 */
void ds1620_init(struct ds1620_data *d,int temp);

#endif
