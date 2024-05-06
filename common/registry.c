/*
 * IPFlow Collector
 * Copyright (c) 2003 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * registry.c: Object Registry.
 */

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <unistd.h>
#include <errno.h>
#include <signal.h>
#include <fcntl.h>
#include <ctype.h>
#include <time.h>
#include <pthread.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <assert.h>

#include "utils.h"
#include "rust_dynamips_c.h"
#include "registry.h"

#define DEBUG_REGISTRY  0

#define REGISTRY_LOCK()    pthread_mutex_lock(&registry->lock)
#define REGISTRY_UNLOCK()  pthread_mutex_unlock(&registry->lock)

/* Insert a new entry */
static void registry_insert_entry(registry_entry_t *entry)
{      
   registry_entry_t *bucket;
   u_int h_index;

   /* insert new entry in hash table for names */
   h_index = str_hash(entry->name) % registry->ht_name_entries;
   bucket = &registry->ht_names[h_index];

   entry->hname_next = bucket->hname_next;
   entry->hname_prev = bucket;
   bucket->hname_next->hname_prev = entry;
   bucket->hname_next = entry;

   /* insert new entry in hash table for object types */
   bucket = &registry->ht_types[entry->object_type];

   entry->htype_next = bucket->htype_next;
   entry->htype_prev = bucket;
   bucket->htype_next->htype_prev = entry;
   bucket->htype_next = entry;
}

/* Detach a registry entry */
static void registry_detach_entry(registry_entry_t *entry)
{
   entry->hname_prev->hname_next = entry->hname_next;
   entry->hname_next->hname_prev = entry->hname_prev;

   entry->htype_prev->htype_next = entry->htype_next;
   entry->htype_next->htype_prev = entry->htype_prev;
}

/* Remove a registry entry */
static void registry_remove_entry(registry_entry_t *entry)
{
   registry_detach_entry(entry);

   mp_free(entry);
}

/* Locate an entry */
static inline registry_entry_t *registry_find_entry(char *name,int object_type)
{
   registry_entry_t *entry,*bucket;
   u_int h_index;

   h_index = str_hash(name) % registry->ht_name_entries;
   bucket = &registry->ht_names[h_index];

   for(entry=bucket->hname_next;entry!=bucket;entry=entry->hname_next)
      if (!strcmp(entry->name,name) && (entry->object_type == object_type))
         return entry;

   return NULL;
}

/* Execute a callback function for all objects of specified type */
int registry_foreach_type(int object_type,registry_foreach cb,
                          void *opt,int *err)
{
   registry_entry_t *p,*bucket,*next;
   int count = 0;

   REGISTRY_LOCK();

   bucket = &registry->ht_types[object_type];

   for(p=bucket->htype_next;p!=bucket;p=next) {
      next = p->htype_next;
      if (cb) cb(p,opt,err);
      count++;
   }

   REGISTRY_UNLOCK();
   return(count);
}

/* Delete all objects of the specified type */
int registry_delete_type(int object_type,registry_exec cb,void *opt)
{
   registry_entry_t *p,*bucket,*next;
   int count = 0;
   int status;

   REGISTRY_LOCK();

   bucket = &registry->ht_types[object_type];

   for(p=bucket->htype_next;p!=bucket;p=next) {
      next = p->htype_next;

      if (p->ref_count == 0) {
         status = TRUE;
         
         if (cb != NULL) 
            status = cb(p->data,opt);

         if (status) {
            registry_remove_entry(p);
            count++;
         }
      } else {
         fprintf(stderr,"registry_delete_type: object \"%s\" (type %d) still "
                 "referenced (count=%d)\n",p->name,object_type,p->ref_count);
      }
   }

   REGISTRY_UNLOCK();
   return(count);
}

/* Dump the registry */
void registry_dump(void)
{
   registry_entry_t *p,*bucket;
   int i;

   REGISTRY_LOCK();

   printf("Registry dump:\n");

   printf("  Objects (from name hash table):\n");

   /* dump hash table of names */
   for(i=0;i<registry->ht_name_entries;i++)
   {
      bucket = &registry->ht_names[i];

      for(p=bucket->hname_next;p!=bucket;p=p->hname_next)
         printf("     %s (type %d, ref_count=%d)\n",
                p->name,p->object_type,p->ref_count);
   }

   printf("\n  Objects classed by types:\n");

   /* dump hash table of types */
   for(i=0;i<registry->ht_type_entries;i++)
   {         
      printf("     Type %d: ",i);

      bucket = &registry->ht_types[i];
      for(p=bucket->htype_next;p!=bucket;p=p->htype_next)
         printf("%s(%d) ",p->name,p->ref_count);
         
      printf("\n");
   }

   REGISTRY_UNLOCK();
}
