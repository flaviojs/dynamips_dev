/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * ATM utility functions and Virtual ATM switch.
 *
 * HEC and AAL5 CRC computation functions are from Charles Michael Heard
 * and can be found at (no licence specified, this is to check!):
 *
 *    http://cell-relay.indiana.edu/cell-relay/publications/software/CRC/
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
#include "atm.h"

#define ATMSW_MAX_TOKENS  16

/* Handle an ATMSW configuration line */
int atmsw_handle_cfg_line(atmsw_table_t *t,char *str)
{  
   char *tokens[ATMSW_MAX_TOKENS];
   int count;

   if ((count = m_strsplit(str,':',tokens,ATMSW_MAX_TOKENS)) <= 1)
      return(-1);

   if (!strcmp(tokens[0],"IF"))
      return(atmsw_cfg_create_if(t,tokens,count));
   else if (!strcmp(tokens[0],"VP"))
      return(atmsw_cfg_create_vpc(t,tokens,count));
   else if (!strcmp(tokens[0],"VC"))
      return(atmsw_cfg_create_vcc(t,tokens,count));

   fprintf(stderr,"ATMSW: Unknown statement \"%s\" (allowed: IF,VP,VC)\n",
           tokens[0]);
   return(-1);
}

/* Read an ATMSW configuration file */
int atmsw_read_cfg_file(atmsw_table_t *t,char *filename)
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
         atmsw_handle_cfg_line(t,buffer);
   }
   
   fclose(fd);
   return(0);
}

/* Start a virtual ATM switch */
int atmsw_start(char *filename)
{
   atmsw_table_t *t;

   if (!(t = atmsw_create_table("default"))) {
      fprintf(stderr,"ATMSW: unable to create virtual fabric table.\n");
      return(-1);
   }

   if (atmsw_read_cfg_file(t,filename) == -1) {
      fprintf(stderr,"ATMSW: unable to parse configuration file.\n");
      return(-1);
   }

   atmsw_release("default");
   return(0);
}
