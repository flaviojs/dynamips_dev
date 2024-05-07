/*
 * Dynamips
 * Copyright (c) 2005 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * rbtree.c: Red/Black Trees.
 */

static const char rcsid[] = "$Id$";

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <unistd.h>
#include <errno.h>
#include <signal.h>
#include <fcntl.h>
#include <ctype.h>

#include "utils.h"
#include "rbtree.h"

#define rbtree_nil(tree) (&(tree)->nil)
#define NIL(tree,x)      (((x) == rbtree_nil(tree)) || !x)

/* Allocate memory for a new node */
static rbtree_node *rbtree_node_alloc(rbtree_tree *tree,void *key,void *value)
{
   rbtree_node *node;

   if (!(node = mp_alloc_n0(&tree->mp,sizeof(*node))))
      return NULL;

   node->key = key;
   node->value = value;
   node->left = rbtree_nil(tree);
   node->right = rbtree_nil(tree);
   node->parent = rbtree_nil(tree);
   node->color = -1;
   return node;
}

/* Free memory used by a node */
static inline void rbtree_node_free(rbtree_tree *tree,rbtree_node *node)
{
   mp_free(node);
}

/* Returns the node which represents the minimum value */
static inline rbtree_node *rbtree_min(rbtree_tree *tree,rbtree_node *x)
{
   while(!NIL(tree,x->left))
      x = x->left;

   return(x);
}

/* Returns the node which represents the maximum value */
_Unused static inline rbtree_node *rbtree_max(rbtree_tree *tree,rbtree_node *x)
{
   while(!NIL(tree,x->right))
      x = x->right;

   return(x);
}

/* Returns the successor of a node */
static inline rbtree_node *rbtree_successor(rbtree_tree *tree,rbtree_node *x)
{
   rbtree_node *y;

   if (!NIL(tree,x->right))
      return(rbtree_min(tree,x->right));

   y = x->parent;
   while(!NIL(tree,y) && (x == y->right)) {
      x = y;
      y = y->parent;
   }

   return(y);
}

/* Left rotation */
static inline void rbtree_left_rotate(rbtree_tree *tree,rbtree_node *x)
{
   rbtree_node *y;

   y = x->right;
   x->right = y->left;

   if (!NIL(tree,x->right))
      x->right->parent = x;

   y->parent = x->parent;
   
   if (NIL(tree,x->parent))
      tree->root = y;
   else {
      if (x == x->parent->left)
         x->parent->left = y;
      else
         x->parent->right = y;
   }

   y->left = x;
   x->parent = y;
}

/* Right rotation */
static inline void rbtree_right_rotate(rbtree_tree *tree,rbtree_node *y)
{
   rbtree_node *x;

   x = y->left;
   y->left = x->right;

   if (!NIL(tree,y->left))
      y->left->parent = y;

   x->parent = y->parent;

   if (NIL(tree,y->parent))
      tree->root = x;
   else {
      if (y->parent->left == y)
         y->parent->left = x;
      else
         y->parent->right = x;
   }

   x->right = y;
   y->parent = x;
}

/* Lookup for a node corresponding to "key" */
static inline rbtree_node *rbtree_lookup_node(rbtree_tree *tree,void *key)
{
   rbtree_node *node;
   int comp;

   node = tree->root;

   for (;;) {
      if (NIL(tree,node)) /* key not found */
         break;

      if (!(comp = tree->key_cmp(key,node->key,tree->opt_data)))
         break; /* exact match */

      node = (comp > 0) ? node->right : node->left;
   }
   
   return(node);
}

/* Restore Red/black tree properties after a removal */
static void rbtree_removal_fixup(rbtree_tree *tree,rbtree_node *x)
{
   rbtree_node *w;

   while((x != tree->root) && (x->color == RBTREE_BLACK))
   {
      if (x == x->parent->left)
      {
         w = x->parent->right;

         if (w->color == RBTREE_RED) {
            w->color = RBTREE_BLACK;
            x->parent->color = RBTREE_RED;
            rbtree_left_rotate(tree,x->parent);
            w = x->parent->right;
         }

         if ((w->left->color == RBTREE_BLACK) &&
             (w->right->color == RBTREE_BLACK))
         {
            w->color = RBTREE_RED;
            x = x->parent;
         }
         else
         {
            if (w->right->color == RBTREE_BLACK) {
               w->left->color = RBTREE_BLACK;
               w->color = RBTREE_RED;
               rbtree_right_rotate(tree,w);
               w = x->parent->right;
            }

            w->color = x->parent->color;
            x->parent->color = RBTREE_BLACK;
            w->right->color = RBTREE_BLACK;
            rbtree_left_rotate(tree,x->parent);
            x = tree->root;
         }
      }
      else
      {
         w = x->parent->left;

         if (w->color == RBTREE_RED) {
            w->color = RBTREE_BLACK;
            x->parent->color = RBTREE_RED;
            rbtree_right_rotate(tree,x->parent);
            w = x->parent->left;
         }

         if ((w->right->color == RBTREE_BLACK) &&
             (w->left->color == RBTREE_BLACK))
         {
            w->color = RBTREE_RED;
            x = x->parent;
         }
         else
         {
            if (w->left->color == RBTREE_BLACK) {
               w->right->color = RBTREE_BLACK;
               w->color = RBTREE_RED;
               rbtree_left_rotate(tree,w);
               w = x->parent->left;
            }

            w->color = x->parent->color;
            x->parent->color = RBTREE_BLACK;
            w->left->color = RBTREE_BLACK;
            rbtree_right_rotate(tree,x->parent);
            x = tree->root;
         }
      }
   }

   x->color = RBTREE_BLACK;
}

static void rbtree_foreach_node(rbtree_tree *tree,rbtree_node *node,
                                tree_fforeach user_fn,void *opt)
{
   if (!NIL(tree,node)) {
      rbtree_foreach_node(tree,node->left,user_fn,opt);
      user_fn(node->key,node->value,opt);
      rbtree_foreach_node(tree,node->right,user_fn,opt);
   }
}

/* Returns the maximum height of the right and left sub-trees */
static int rbtree_height_node(rbtree_tree *tree,rbtree_node *node)
{
   int lh,rh;

   lh = (!NIL(tree,node->left)) ? rbtree_height_node(tree,node->left) : 0;
   rh = (!NIL(tree,node->right)) ? rbtree_height_node(tree,node->right) : 0;
   return(1 + m_max(lh,rh));
}

/* Check a node */
static int rbtree_check_node(rbtree_tree *tree,rbtree_node *node)
{
   if (!NIL(tree,node)) return(0);

   if (!NIL(tree,node->left)) {
      if (tree->key_cmp(node->key,node->left->key,tree->opt_data) <= 0)
         return(-1);
      
      if (rbtree_check_node(tree,node->left) == -1)
         return(-1);
   }

   if (!NIL(tree,node->right)) {
      if (tree->key_cmp(node->key,node->right->key,tree->opt_data) >= 0)
         return(-1);

      if (rbtree_check_node(tree,node->right) == -1)
         return(-1);
   }

   return(0);
}

/* Create a new Red/Black tree */
rbtree_tree *rbtree_create(tree_fcompare key_cmp,void *opt_data)
{
   rbtree_tree *tree;

   if (!(tree = malloc(sizeof(*tree))))
      return NULL;

   memset(tree,0,sizeof(*tree));

   /* initialize the memory pool */
   if (!mp_create_fixed_pool(&tree->mp,"Red-Black Tree")) {
      free(tree);
      return NULL;
   }

   /* initialize the "nil" pointer */
   memset(rbtree_nil(tree),0,sizeof(rbtree_node));
   rbtree_nil(tree)->color = RBTREE_BLACK;

   tree->key_cmp = key_cmp;
   tree->opt_data = opt_data;
   tree->root = rbtree_nil(tree);
   return tree;
}

/* Delete a Red/Black tree */
void rbtree_delete(rbtree_tree *tree)
{
   if (tree) {
      mp_free_pool(&tree->mp);
      free(tree);
   }
}
