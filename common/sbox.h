/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 *
 * S-box functions.
 */

#ifndef __SBOX_H__
#define __SBOX_H__

#include "dynamips_common.h"

static forced_inline m_uint32_t sbox_u32(m_uint32_t val)
{
   m_uint32_t hash = 0;

   hash ^= sbox_array[(m_uint8_t)val];
   hash *= 3;
   val >>= 8;

   hash ^= sbox_array[(m_uint8_t)val];
   hash *= 3;
   val >>= 8;

   hash ^= sbox_array[(m_uint8_t)val];
   hash *= 3;
   val >>= 8;

   hash ^= sbox_array[(m_uint8_t)val];
   return(hash);
}

#endif
