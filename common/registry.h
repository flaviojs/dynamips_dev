/*
 * IPFlow Collector
 * Copyright (c) 2003 Christophe Fillot.
 * E-mail: cf@utc.fr
 * 
 * registry.h: Object Registry.
 */

#ifndef __REGISTRY_H__
#define __REGISTRY_H__  1

#include "rust_dynamips_c.h"

static const char rcsid_registry[] = "$Id$";

#include <sys/types.h>
#include <sys/time.h>
#include <pthread.h>

/* Initialize registry */
int registry_init(void);

/* Add a new entry to the registry */
int registry_add(char *name,int object_type,void *data);

/* Delete an entry from the registry */
int registry_delete(char *name,int object_type);

/* Rename an entry in the registry */
int registry_rename(char *name,char *newname,int object_type);

/* Find an entry (increment reference count) */
void *registry_find(char *name,int object_type);

/* Check if entry exists (does not change reference count) */
void *registry_exists(char *name,int object_type);

/* Release a reference of an entry (decrement the reference count) */
int registry_unref(char *name,int object_type);

/* 
 * Execute action on an object if its reference count is less or equal to
 * the specified count.
 */
int registry_exec_refcount(char *name,int object_type,int max_ref,int reg_del,
                           registry_exec obj_action,void *opt_arg);

/* Delete object if unused */
int registry_delete_if_unused(char *name,int object_type,
                              registry_exec obj_destructor,
                              void *opt_arg);

/* Execute a callback function for all objects of specified type */
int registry_foreach_type(int object_type,registry_foreach cb,
                          void *opt,int *err);

/* Delete all objects of the specified type */
int registry_delete_type(int object_type,registry_exec cb,void *opt);

/* Dump the registry */
void registry_dump(void);

#endif
