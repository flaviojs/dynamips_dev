/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 */

#ifndef __JIT_OP_H__
#define __JIT_OP_H__

#include "utils.h"

typedef struct jit_op jit_op_t;
typedef struct jit_op_data jit_op_data_t;

#include "rust_dynamips_c.h"

/* Invalid register in op */
#define JIT_OP_INV_REG  -1

/* All flags */
#define JIT_OP_PPC_ALL_FLAGS  -1

/* All registers */
#define JIT_OP_ALL_REGS  -1

/* JIT opcodes */
enum {
   JIT_OP_INVALID = 0,
   JIT_OP_INSN_OUTPUT,
   JIT_OP_BRANCH_TARGET,
   JIT_OP_BRANCH_JUMP,
   JIT_OP_EOB,
   JIT_OP_LOAD_GPR,
   JIT_OP_STORE_GPR,
   JIT_OP_UPDATE_FLAGS,
   JIT_OP_REQUIRE_FLAGS,
   JIT_OP_TRASH_FLAGS,
   JIT_OP_ALTER_HOST_REG,
   JIT_OP_MOVE_HOST_REG,
   JIT_OP_SET_HOST_REG_IMM32,
};

extern u_int jit_op_blk_sizes[];

/* Find a specific opcode in a JIT op list */
static inline jit_op_t *jit_op_find_opcode(jit_op_t *op_list,u_int opcode)
{
   jit_op_t *op;

   for(op=op_list;op;op=op->next)
      if (op->opcode == opcode)
         return op;

   return NULL;
}

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
