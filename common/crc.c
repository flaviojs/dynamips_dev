/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * CRC functions.
 */

#include "dynamips_common.h"

#define CRC12_POLY  0x0f01
#define CRC16_POLY  0xa001
#define CRC32_POLY  0xedb88320L

/* Initialize CRC algorithms */
void crc_init(void)
{
   crc12_init();
   crc16_init();
   crc32_init();
}
