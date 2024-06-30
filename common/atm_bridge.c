/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 *
 * ATM bridge (RFC1483)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <errno.h>
#include <sys/select.h>
#include <sys/time.h>
#include <sys/types.h>

#include "utils.h"
#include "rust_dynamips_c.h"
#include "atm_bridge.h"

#define ATM_BRIDGE_LOCK(t)   pthread_mutex_lock(&(t)->lock)
#define ATM_BRIDGE_UNLOCK(t) pthread_mutex_unlock(&(t)->lock)

#define ATM_BRIDGE_MAX_TOKENS  16

/* Handle an ATMSW configuration line */
int atm_bridge_handle_cfg_line(atm_bridge_t *t,char *str)
{  
   char *tokens[ATM_BRIDGE_MAX_TOKENS];
   int count;

   if ((count = m_strsplit(str,':',tokens,ATM_BRIDGE_MAX_TOKENS)) <= 1)
      return(-1);

   if (!strcmp(tokens[0],"IF"))
      return(atm_bridge_cfg_create_if(t,tokens,count));
   else if (!strcmp(tokens[0],"BRIDGE"))
      return(atm_bridge_cfg_setup(t,tokens,count));

   fprintf(stderr,"ATM Bridge: "
           "Unknown statement \"%s\" (allowed: IF,BRIDGE)\n",
           tokens[0]);
   return(-1);
}


/* Read an ATM bridge configuration file */
int atm_bridge_read_cfg_file(atm_bridge_t *t,char *filename)
{
   char buffer[1024],*ptr;
   FILE *fd;

   if (!(fd = fopen(filename,"r"))) {
      perror("fopen");
      return(-1);
   }
   
   while(!feof(fd)) {
      if (!fgets(buffer,sizeof(buffer),fd))
         break;
      
      /* skip comments and end of line */
      if ((ptr = strpbrk(buffer,"#\r\n")) != NULL)
         *ptr = 0;

      /* analyze non-empty lines */
      if (strchr(buffer,':'))
         atm_bridge_handle_cfg_line(t,buffer);
   }
   
   fclose(fd);
   return(0);
}

/* Start a virtual ATM bridge */
int atm_bridge_start(char *filename)
{
   atm_bridge_t *t;

   if (!(t = atm_bridge_create("default"))) {
      fprintf(stderr,"ATM Bridge: unable to create virtual fabric table.\n");
      return(-1);
   }

   if (atm_bridge_read_cfg_file(t,filename) == -1) {
      fprintf(stderr,"ATM Bridge: unable to parse configuration file.\n");
      return(-1);
   }

   atm_bridge_release("default");
   return(0);
}
