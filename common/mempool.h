/*
 * Copyright (c) 1999-2006 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * mempool.h: Simple Memory Pools.
 */

#ifndef __MEMPOOL_H__
#define __MEMPOOL_H__  1

#include "rust_dynamips_c.h"

#include <sys/types.h>
#include <sys/time.h>
#include <pthread.h>

#include "utils.h"

/* Allocate a new block in specified pool */
void *mp_alloc(mempool_t *pool,size_t size);

/* Allocate a new block which will not be zeroed */
void *mp_alloc_n0(mempool_t *pool,size_t size);

/* Reallocate a block */
void *mp_realloc(void *addr,size_t new_size);

/* Allocate a new memory block and copy data into it */
void *mp_dup(mempool_t *pool,void *data,size_t size);

/* Duplicate specified string and insert it in a memory pool */
char *mp_strdup(mempool_t *pool,char *str);

/* Free block at specified address */
int mp_free(void *addr);

/* Free block at specified address and clean pointer */
int mp_free_ptr(void *addr);

/* Free all blocks of specified pool */
void mp_free_all_blocks(mempool_t *pool);

/* Free specified memory pool */
void mp_free_pool(mempool_t *pool);

/* Create a new pool in a fixed memory area */
mempool_t *mp_create_fixed_pool(mempool_t *mp,char *name);

/* Create a new pool */
mempool_t *mp_create_pool(char *name);

#endif
