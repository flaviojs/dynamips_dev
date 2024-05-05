/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Generic Hash Tables.
 */

#ifndef __HASH_H__
#define __HASH_H__  1

#include "rust_dynamips_c.h"

#include <sys/types.h>
#include "utils.h"

/* Create a new hash table */
hash_table_t *hash_table_create(hash_fcompute hash_func,hash_fcompare key_cmp,
                                int hash_size);

/* Delete an existing Hash Table */
void hash_table_delete(hash_table_t *ht);

/* Insert a new (key,value). If key already exist in table, replace value */
int hash_table_insert(hash_table_t *ht,void *key,void *value);

/* Remove a pair (key,value) from an hash table */
void *hash_table_remove(hash_table_t *ht,void *key);

/* Hash Table Lookup */
void *hash_table_lookup(hash_table_t *ht,void *key);

/* Call the specified function for each node found in hash table */
int hash_table_foreach(hash_table_t *ht,hash_fforeach user_fn,void *opt_arg);

/* Hash Table Lookup - key direct comparison */
void *hash_table_lookup_dcmp(hash_table_t *ht,void *key);

/* Hash Functions for strings */
int str_equal(void *s1,void *s2);
u_int str_hash(void *str);

/* Hash Functions for integers */
int int_equal(void *i1,void *i2);
u_int int_hash(void *i);

/* Hash Functions for u64 */
int u64_equal(void *i1,void *i2);
u_int u64_hash(void *i);

/* Hash Function for pointers */
int ptr_equal(void *i1,void *i2);
u_int ptr_hash(void *i);

#endif
