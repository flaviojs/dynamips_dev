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

/* RFC Chunk preprocessing: phase 0 */
static rfc_array_t *rfc_phase_0(insn_lookup_t *ilt,ilt_check_cbk_t pcheck)
{
   rfc_eqclass_t *eqcl;
   rfc_array_t *rfct;
   cbm_array_t *bmp;
   int i;

   /* allocate a temporary class bitmap */
   bmp = cbm_create(ilt);
   assert(bmp);

   /* Allocate a new RFC array of 16-bits entries */
   rfct = rfc_alloc_array(RFC_ARRAY_MAXSIZE);
   assert(rfct);

   for(i=0;i<RFC_ARRAY_MAXSIZE;i++)
   {
      /* determine all instructions that match this value */
      rfc_check_insn(ilt,bmp,pcheck,i);

      /* get equivalent class for this bitmap */
      eqcl = cbm_get_eqclass(rfct,bmp);
      assert(eqcl);

      /* fill the RFC table */
      rfct->eqID[i] = eqcl->eqID;
   }

   free(bmp);
   return rfct;
}

/* RFC Chunk preprocessing: phase j (j > 0) */
static rfc_array_t *rfc_phase_j(insn_lookup_t *ilt,rfc_array_t *p0,
                                rfc_array_t *p1)
{
   rfc_eqclass_t *eqcl;
   rfc_array_t *rfct;
   cbm_array_t *bmp;
   int nr_elements;
   int index = 0;
   int i,j;

   /* allocate a temporary class bitmap */
   bmp = cbm_create(ilt);
   assert(bmp);

   /* compute number of elements */
   nr_elements = p0->nr_eqid * p1->nr_eqid;

   /* allocate a new RFC array */
   rfct = rfc_alloc_array(nr_elements);
   assert(rfct);
   rfct->parent0 = p0;
   rfct->parent1 = p1;

   /* make a cross product between p0 and p1 */
   for(i=0;i<p0->nr_eqid;i++)
      for(j=0;j<p1->nr_eqid;j++)
      {
         /* compute bitwise AND */
         cbm_bitwise_and(bmp,p0->id2cbm[i],p1->id2cbm[j]);

         /* get equivalent class for this bitmap */
         eqcl = cbm_get_eqclass(rfct,bmp);
         assert(eqcl);

         /* fill RFC table */
         rfct->eqID[index++] = eqcl->eqID;
      }

   free(bmp);
   return rfct;
}

/* Compute RFC phase 0 */
static void ilt_phase_0(insn_lookup_t *ilt,int idx,ilt_check_cbk_t pcheck)
{
   rfc_array_t *rfct;

   rfct = rfc_phase_0(ilt,pcheck);
   assert(rfct);
   ilt->rfct[idx] = rfct;
}

/* Compute RFC phase j */
static void ilt_phase_j(insn_lookup_t *ilt,int p0,int p1,int res)
{
   rfc_array_t *rfct;

   rfct = rfc_phase_j(ilt,ilt->rfct[p0],ilt->rfct[p1]);
   assert(rfct);
   ilt->rfct[res] = rfct;
}

/* Postprocessing */
static void ilt_postprocessing(insn_lookup_t *ilt)
{
   rfc_array_t *rfct = ilt->rfct[2];
   int i;

   for(i=0;i<rfct->nr_elements;i++)
      rfct->eqID[i] = cbm_first_match(ilt,rfct->id2cbm[rfct->eqID[i]]);
}

/* Instruction lookup table compilation */
static void ilt_compile(insn_lookup_t *ilt)
{  
   ilt_phase_0(ilt,0,ilt->chk_hi);
   ilt_phase_0(ilt,1,ilt->chk_lo);
   ilt_phase_j(ilt,0,1,2);
   ilt_postprocessing(ilt);
}

/* Dump an instruction lookup table */
_Unused static int ilt_dump(char *table_name,insn_lookup_t *ilt)
{
   rfc_array_t *rfct;
   char *filename;
   FILE *fd;
   int i,j;
   
   filename = dyn_sprintf("ilt_dump_%s_%s.txt",sw_version_tag,table_name);
   assert(filename != NULL);

   fd = fopen(filename,"w");
   assert(fd != NULL);
   
   fprintf(fd,"ILT %p: nr_insn=%d, cbm_size=%d\n",
         ilt,ilt->nr_insn,ilt->cbm_size);

   for(i=0;i<RFC_ARRAY_NUMBER;i++) {
      rfct = ilt->rfct[i];
      
      fprintf(fd,"RFCT %d: nr_elements=%d, nr_eqid=%d\n",
              i,rfct->nr_elements,rfct->nr_eqid);
      
      for(j=0;j<rfct->nr_elements;j++)
         fprintf(fd,"  (0x%4.4x,0x%4.4x) = 0x%4.4x\n",i,j,rfct->eqID[j]);
   }
   
   fclose(fd);
   free(filename);
   return(0);
}

/* Write the specified RFC array to disk */
static void ilt_store_rfct(FILE *fd,int id,rfc_array_t *rfct)
{
   /* Store RFC array ID + number of elements */
   fwrite(&id,sizeof(id),1,fd);
   fwrite(&rfct->nr_elements,sizeof(rfct->nr_elements),1,fd);
   fwrite(&rfct->nr_eqid,sizeof(rfct->nr_eqid),1,fd);

   fwrite(rfct->eqID,sizeof(int),rfct->nr_elements,fd);
}

/* Write the full instruction lookup table */
static void ilt_store_table(FILE *fd,insn_lookup_t *ilt)
{
   int i;

   for(i=0;i<RFC_ARRAY_NUMBER;i++)
      if (ilt->rfct[i] != NULL)
         ilt_store_rfct(fd,i,ilt->rfct[i]);
}

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
