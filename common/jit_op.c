/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 *
 * JIT operations.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <signal.h>
#include <fcntl.h>
#include <assert.h>

#include "cpu.h"
#include "jit_op.h"

/* Get a JIT op (allocate one if necessary) */
jit_op_t *jit_op_get(jit_op_data_t *data,int size_index,u_int opcode)
{
   jit_op_t *op;
   size_t len;

   assert(size_index < JIT_OP_POOL_NR);
   op = data->pool[size_index];

   if (op != NULL) {
      assert(op->ob_size_index == size_index);
      data->pool[size_index] = op->next;
   } else {
      /* no block found, allocate one */
      len = sizeof(*op) + jit_op_blk_sizes[size_index];

      op = malloc(len);
      assert(op != NULL);
      op->ob_size_index = size_index;
   }

   op->opcode = opcode;
   op->param[0] = op->param[1] = op->param[2] = -1;
   op->next = NULL;
   op->ob_ptr = op->ob_data;
   op->arg_ptr = NULL;
   op->insn_name = NULL;
   return op;
}
