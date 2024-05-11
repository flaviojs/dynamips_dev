/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 *
 * ROMMON Environment Variables.
 */

#include "utils.h"
#include "rommon_var.h"

/* Write a file with all ROMMON variables */
int rommon_var_update_file(struct rommon_var_list *rvl)
{
   struct rommon_var *var;
   FILE *fd;

   if (!rvl->filename)
      return(-1);

   if (!(fd = fopen(rvl->filename,"w"))) {
      fprintf(stderr,"%s: unable to create file %s (%s)\n",
              __func__,rvl->filename,strerror(errno));
      return(-1);
   }

   for(var=rvl->var_list;var;var=var->next)
      fprintf(fd,"%s=%s\n",var->name,var->value ? var->value : "");

   fclose(fd);
   return(0);
}

/* Find the specified variable */
struct rommon_var *rommon_var_find(struct rommon_var_list *rvl,char *name)
{
   struct rommon_var *var;

   for(var=rvl->var_list;var;var=var->next)
      if (!strcmp(var->name,name))
         return var;

   return NULL;
}

/* Create a new variable */
static struct rommon_var *rommon_var_create(char *name)
{
   struct rommon_var *var;

   if (!(var = malloc(sizeof(*var))))
      return NULL;

   var->next  = NULL;
   var->value = NULL;
   var->name  = strdup(name);

   if (!var->name) {
      free(var);
      return NULL;
   }

   return var;
}

/* Delete a variable */
static struct rommon_var *rommon_var_delete(struct rommon_var *var)
{
   struct rommon_var *next_var;

   next_var = var->next;
   free(var->value);
   free(var->name);
   free(var);
   return(next_var);
}

/* Set value for a variable */
static int rommon_var_set(struct rommon_var *var,char *value)
{
   char *new_value;

   if (!(new_value = strdup(value)))
      return(-1);

   /* free old value */
   if (var->value)
      free(var->value);

   var->value = new_value;
   return(0);
}
