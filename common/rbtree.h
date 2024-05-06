/*
 * IPFlow Collector
 * Copyright (c) 2004 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * rbtree.c: Red/Black Trees.
 */

#ifndef __RBTREE_H__
#define __RBTREE_H__   1

#include "rust_dynamips_c.h"

static const char rcsid_rbtree[] = "$Id$";

#include <sys/types.h>

/* Insert a node in an Red/Black tree */
int rbtree_insert(rbtree_tree *tree,void *key,void *value);

/* Removes a node out of a tree */
void *rbtree_remove(rbtree_tree *tree,void *key);

/* 
 * Lookup for a node corresponding to "key". If node does not exist, 
 * function returns null pointer.
 */
void *rbtree_lookup(rbtree_tree *tree,void *key);

/* Call the specified function for each node */
int rbtree_foreach(rbtree_tree *tree,tree_fforeach user_fn,void *opt);

/* Compute the height of a Red/Black tree */
int rbtree_height(rbtree_tree *tree);

/* Returns the number of nodes */
int rbtree_node_count(rbtree_tree *tree);

/* Purge all nodes */
void rbtree_purge(rbtree_tree *tree);

/* Check tree consistency */
int rbtree_check(rbtree_tree *tree);

/* Create a new Red/Black tree */
rbtree_tree *rbtree_create(tree_fcompare key_cmp,void *opt_data);

/* Delete an Red/Black tree */
void rbtree_delete(rbtree_tree *tree);

#endif
