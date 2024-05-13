/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 */

#ifndef __JIT_OP_H__
#define __JIT_OP_H__

#include "rust_dynamips_c.h"

#include "utils.h"

typedef struct jit_op jit_op_t;
typedef struct jit_op_data jit_op_data_t;

#include "rust_dynamips_c.h"

extern u_int jit_op_blk_sizes[];

/* Get a JIT op (allocate one if necessary) */
jit_op_t *jit_op_get(jit_op_data_t *data,int size_index,u_int opcode);

/* Release a JIT op */
void jit_op_free(jit_op_data_t *data,jit_op_t *op);

/* Free a list of JIT ops */
void jit_op_free_list(jit_op_data_t *data,jit_op_t *op_list);

/* Initialize JIT op pools for the specified CPU */
int jit_op_init_cpu(jit_op_data_t *data);

/* Free memory used by pools */
void jit_op_free_pools(jit_op_data_t *data);

#endif
