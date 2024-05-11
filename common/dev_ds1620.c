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
