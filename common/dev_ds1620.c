/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Dallas DS1620 Temperature sensors.
 */

#include "dev_ds1620.h"

/* DS1620 commands */
#define DS1620_READ_TEMP     0xAA
#define DS1620_READ_COUNTER  0xA0
#define DS1620_READ_SLOPE    0xA9
#define DS1620_WRITE_TH      0x01
#define DS1620_WRITE_TL      0x02
#define DS1620_READ_TH       0xA1
#define DS1620_READ_TL       0xA2
#define DS1620_START_CONVT   0xEE
#define DS1620_STOP_CONVT    0x22
#define DS1620_WRITE_CONFIG  0x0C
#define DS1620_READ_CONFIG   0xAC

/* DS1620 config register */
#define DS1620_CONFIG_STATUS_DONE   0x80
#define DS1620_CONFIG_STATUS_THF    0x40
#define DS1620_CONFIG_STATUS_TLF    0x20
#define DS1620_CONFIG_STATUS_CPU    0x02
#define DS1620_CONFIG_STATUS_1SHOT  0x01

/* Size of various operations in bits (command, config and temp data) */
#define DS1620_CMD_SIZE      8
#define DS1620_CONFIG_SIZE   8
#define DS1620_TEMP_SIZE     9

/* Internal states */
enum {
   DS1620_STATE_CMD_IN,
   DS1620_STATE_DATA_IN,
   DS1620_STATE_DATA_OUT,
};

/* Read data bit */
u_int ds1620_read_data_bit(struct ds1620_data *d)
{
   u_int val;

   if (d->state != DS1620_STATE_DATA_OUT)
      return(1);

   val = (d->data >> d->data_pos) & 0x1;

   if (++d->data_pos == d->data_len) {
      /* return in command input state */
      d->state = DS1620_STATE_CMD_IN;
   }

   return(val);
}

/* Initialize a DS1620 */
void ds1620_init(struct ds1620_data *d,int temp)
{
   memset(d,0,sizeof(*d));

   /* reset state */
   ds1620_set_rst_bit(d,0);

   /* set initial temperature */
   ds1620_set_temp(d,temp);

   /* chip in CPU mode (3-wire communications) */
   d->reg_config = DS1620_CONFIG_STATUS_CPU;   
}
