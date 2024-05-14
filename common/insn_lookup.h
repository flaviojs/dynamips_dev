/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * MIPS Instruction Lookup Tables.
 */

#ifndef __INSN_LOOKUP_H__
#define __INSN_LOOKUP_H__

#include "rust_dynamips_c.h"

#include "utils.h"

/* Forward declaration for instruction lookup table */
typedef struct insn_lookup insn_lookup_t;

#define CBM_ARRAY(array,i) ((array)->tab[(i)])
#define CBM_CSIZE(count)   (((count)*sizeof(int))+sizeof(cbm_array_t))

/* Create an instruction lookup table */
insn_lookup_t *ilt_create(char *table_name,
                          int nr_insn,ilt_get_insn_cbk_t get_insn,
                          ilt_check_cbk_t chk_lo,ilt_check_cbk_t chk_hi);

/* Destroy an instruction lookup table */
void ilt_destroy(insn_lookup_t *ilt);

#endif
