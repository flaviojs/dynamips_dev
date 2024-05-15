/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Instruction Lookup Tables.
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <assert.h>

#include "utils.h"
#include "rust_dynamips_c.h"
#include "insn_lookup.h"
#include "dynamips.h"

/* Load an RFC array from disk */
static int ilt_load_rfct(FILE *fd,insn_lookup_t *ilt)
{
   u_int id,nr_elements,nr_eqid;
   rfc_array_t *rfct;
   size_t len;

   /* Read ID and number of elements */
   if ((fread(&id,sizeof(id),1,fd) != 1) ||
       (fread(&nr_elements,sizeof(nr_elements),1,fd) != 1) ||
       (fread(&nr_eqid,sizeof(nr_eqid),1,fd) != 1))
      return(-1);
      
   if ((id >= RFC_ARRAY_NUMBER) || (nr_elements > RFC_ARRAY_MAXSIZE))
      return(-1);

   /* Allocate the RFC array with the eqID table */
   len = sizeof(*rfct) + (nr_elements * sizeof(int));

   if (!(rfct = malloc(len)))
      return(-1);

   memset(rfct,0,sizeof(*rfct));
   rfct->nr_elements = nr_elements;
   rfct->nr_eqid = nr_eqid;
   
   /* Read the equivalent ID array */
   if (fread(rfct->eqID,sizeof(int),nr_elements,fd) != nr_elements) {
      free(rfct);
      return(-1);
   }

   ilt->rfct[id] = rfct;
   return(0);
}

/* Check an instruction table loaded from disk */
static int ilt_check_cached_table(insn_lookup_t *ilt)
{
   int i;

   /* All arrays must have been loaded */
   for(i=0;i<RFC_ARRAY_NUMBER;i++)
      if (!ilt->rfct[i])
         return(-1);

   return(0);
}

/* Load a full instruction table from disk */
static insn_lookup_t *ilt_load_table(FILE *fd)
{
   insn_lookup_t *ilt;
   int i;
   
   if (!(ilt = malloc(sizeof(*ilt))))
      return NULL;

   memset(ilt,0,sizeof(*ilt));
   fseek(fd,0,SEEK_SET);

   for(i=0;i<RFC_ARRAY_NUMBER;i++) {
      if (ilt_load_rfct(fd,ilt) == -1) {
         ilt_destroy(ilt);
         return NULL;
      }
   }

   if (ilt_check_cached_table(ilt) == -1) {
      ilt_destroy(ilt);
      return NULL;
   }

   return ilt;
}

/* Build a filename for a cached ILT table on disk */
static char *ilt_build_filename(char *table_name)
{
   return(dyn_sprintf("ilt_%s_%s",sw_version_tag,table_name));
}

/* Try to load a cached ILT table from disk */
static insn_lookup_t *ilt_cache_load(char *table_name)
{
   insn_lookup_t *ilt;
   char *filename;
   FILE *fd;

   if (!(filename = ilt_build_filename(table_name)))
      return NULL;

   if (!(fd = fopen(filename,"rb"))) {
      free(filename);
      return NULL;
   }

   ilt = ilt_load_table(fd);
   fclose(fd);
   free(filename);
   return ilt;
}

/* Store the specified ILT table on disk for future use (cache) */
static int ilt_cache_store(char *table_name,insn_lookup_t *ilt)
{
   char *filename;
   FILE *fd;

   if (!(filename = ilt_build_filename(table_name)))
      return(-1);

   if (!(fd = fopen(filename,"wb"))) {
      free(filename);
      return(-1);
   }

   ilt_store_table(fd,ilt);
   fclose(fd);
   free(filename);
   return(0);
}

/* Create an instruction lookup table */
insn_lookup_t *ilt_create(char *table_name,
                          int nr_insn,ilt_get_insn_cbk_t get_insn,
                          ilt_check_cbk_t chk_lo,ilt_check_cbk_t chk_hi)
{
   insn_lookup_t *ilt;
   
   /* Try to load a cached table from disk */
   if ((ilt = ilt_cache_load(table_name))) {
      printf("ILT: loaded table \"%s\" from cache.\n",table_name);
      return ilt;
   }

   /* We have to build the full table... */
   ilt = malloc(sizeof(*ilt));
   assert(ilt);
   memset(ilt,0,sizeof(*ilt));

   ilt->cbm_size = normalize_size(nr_insn,CBM_SIZE,CBM_SHIFT);
   ilt->nr_insn  = nr_insn;
   ilt->get_insn = get_insn;
   ilt->chk_lo   = chk_lo;
   ilt->chk_hi   = chk_hi;

   /* Compile the instruction opcodes */
   ilt_compile(ilt);
   
   /* Store the result on disk for future exec */
   ilt_cache_store(table_name,ilt);
   return(ilt);
}

/* Destroy an instruction lookup table */
void ilt_destroy(insn_lookup_t *ilt)
{
   int i;

   assert(ilt);

   /* Free instruction opcodes */
   for (i = 0; i < RFC_ARRAY_NUMBER; i++) {
      if (ilt->rfct[i])
         rfc_free_array(ilt->rfct[i]);
   }

   /* Free instruction lookup table */
   free(ilt);
}
