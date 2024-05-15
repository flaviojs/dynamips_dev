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
