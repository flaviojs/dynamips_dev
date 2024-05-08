/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * CRC functions.
 */

#ifndef __CRC_H__
#define __CRC_H__

#include "dynamips_common.h"

extern m_uint32_t crc32_array[];

/* Compute a CRC-32 on the specified block */
static forced_inline 
m_uint32_t crc32_compute(m_uint32_t crc_accum,m_uint8_t *ptr,int len)
{
   register m_uint32_t c = crc_accum;
   int n;
   
   for (n = 0; n < len; n++) {
      c = crc32_array[(c ^ ptr[n]) & 0xff] ^ (c >> 8);
   }

   return(~c);
}


/* Initialize CRC algorithms */
void crc_init(void);

#endif
