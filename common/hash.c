/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Hash Tables.
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <assert.h>

#include "utils.h"
#include "hash.h"

/* Free memory used by a node */
static inline void hash_node_free(hash_node_t *node)
{
   free(node);
}

/* Allocate memory for a new node */
static hash_node_t *hash_node_alloc(hash_table_t *ht,void *key,void *value)
{
   hash_node_t *node;

   node = malloc(sizeof(*node));
   assert(node!=NULL);
   node->key = key;
   node->value = value;
   node->next = NULL;
   return node;
}
