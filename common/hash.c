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

/* Remove a pair (key,value) from an hash table */
void *hash_table_remove(hash_table_t *ht,void *key)
{
   hash_node_t **node,*tmp;
   u_int hash_val;
   void *value;

   assert(ht!=NULL);

   hash_val = ht->hash_func(key) % ht->size;

   for(node=&ht->nodes[hash_val];*node;node=&(*node)->next)
      if (ht->key_cmp((*node)->key,key)) {
         tmp = *node;
         value = tmp->value;
         *node = tmp->next;

         hash_node_free(tmp);
         return(value);
      }

   return NULL;
}

/* Hash Table Lookup */
void *hash_table_lookup(hash_table_t *ht,void *key)
{
   hash_node_t *node;
   u_int hash_val;

   assert(ht!=NULL);

   hash_val = ht->hash_func(key) % ht->size;

   for(node=ht->nodes[hash_val];node;node=node->next)
      if (ht->key_cmp(node->key,key))
         return node->value;

   return NULL;
}

/* Hash Table Lookup - key direct comparison */
void *hash_table_lookup_dcmp(hash_table_t *ht,void *key)
{
   hash_node_t *node;
   u_int hash_val;

   assert(ht!=NULL);

   hash_val = ht->hash_func(key) % ht->size;

   for(node=ht->nodes[hash_val];node;node=node->next)
      if (node->key == key)
         return node->value;

   return NULL;
}

/* Call the specified function for each node found in hash table */
int hash_table_foreach(hash_table_t *ht,hash_fforeach user_fn,void *opt_arg)
{
   hash_node_t *node;
   int i;

   assert(ht!=NULL);

   for(i=0;i<ht->size;i++)
      for(node=ht->nodes[i];node;node=node->next)
         user_fn(node->key,node->value,opt_arg);
   
   return(0);
}
