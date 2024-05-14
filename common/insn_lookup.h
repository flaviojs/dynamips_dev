/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * MIPS Instruction Lookup Tables.
 */

#ifndef __INSN_LOOKUP_H__
#define __INSN_LOOKUP_H__

#include "utils.h"
#include "rust_dynamips_c.h"

/* Forward declaration for instruction lookup table */
typedef struct insn_lookup insn_lookup_t;

#define CBM_ARRAY(array,i) ((array)->tab[(i)])
#define CBM_CSIZE(count)   (((count)*sizeof(int))+sizeof(cbm_array_t))

/* Equivalent Classes */
typedef struct rfc_eqclass rfc_eqclass_t;
struct rfc_eqclass {
   cbm_array_t *cbm;   /* Class Bitmap */
   int eqID;           /* Index associated to this class */
};

/* Instruction lookup */
static forced_inline int ilt_get_index(rfc_array_t *a1,rfc_array_t *a2,
                                       int i1,int i2)
{
   return((a1->eqID[i1]*a2->nr_eqid) + a2->eqID[i2]);
}

static forced_inline int ilt_get_idx(insn_lookup_t *ilt,int a1,int a2,
                                     int i1,int i2)
{
   return(ilt_get_index(ilt->rfct[a1],ilt->rfct[a2],i1,i2));
}

static forced_inline int ilt_lookup(insn_lookup_t *ilt,mips_insn_t insn)
{
   int id_i;

   id_i = ilt_get_idx(ilt,0,1,insn >> 16,insn & 0xFFFF);
   return(ilt->rfct[2]->eqID[id_i]);
}

/* Create an instruction lookup table */
insn_lookup_t *ilt_create(char *table_name,
                          int nr_insn,ilt_get_insn_cbk_t get_insn,
                          ilt_check_cbk_t chk_lo,ilt_check_cbk_t chk_hi);

/* Destroy an instruction lookup table */
void ilt_destroy(insn_lookup_t *ilt);

#endif
