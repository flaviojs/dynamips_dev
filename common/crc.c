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

/* CRC tables */
m_uint32_t crc32_array[256];

/* Initialize CRC-32 algorithm */
static void crc32_init(void)
{
   unsigned long c;
   int n, k;
   
   for (n=0;n<256;n++) {
      c = (unsigned long) n;
      for (k = 0; k < 8; k++) {
         if (c & 1)
            c = CRC32_POLY ^ (c >> 1);
         else
            c = c >> 1;
      }
      crc32_array[n] = c;
   }
}

/* Initialize CRC algorithms */
void crc_init(void)
{
   crc12_init();
   crc16_init();
   crc32_init();
}
