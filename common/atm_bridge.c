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

/* Release NIO used by an ATM bridge */
static void atm_bridge_clear_config(atm_bridge_t *t)
{
   if (t != NULL) {
      /* release ethernet NIO */
      if (t->eth_nio) {
         netio_rxl_remove(t->eth_nio);
         netio_release(t->eth_nio->name);
      }

      /* release ATM NIO */
      if (t->atm_nio) {
         netio_rxl_remove(t->atm_nio);
         netio_release(t->atm_nio->name);
      }

      t->eth_nio = t->atm_nio = NULL;
   }
}

/* Unconfigure an ATM bridge */
int atm_bridge_unconfigure(atm_bridge_t *t)
{
   ATM_BRIDGE_LOCK(t);
   atm_bridge_clear_config(t);
   ATM_BRIDGE_UNLOCK(t);
   return(0);
}

/* Free resources used by an ATM bridge */
static int atm_bridge_free(void *data,void *arg)
{
   atm_bridge_t *t = data;

   atm_bridge_clear_config(t);
   free(t->name);
   free(t);
   return(TRUE);
}

/* Delete an ATM bridge */
int atm_bridge_delete(char *name)
{
   return(registry_delete_if_unused(name,OBJ_TYPE_ATM_BRIDGE,
                                    atm_bridge_free,NULL));
}

/* Delete all ATM switches */
int atm_bridge_delete_all(void)
{
   return(registry_delete_type(OBJ_TYPE_ATM_BRIDGE,atm_bridge_free,NULL));
}

/* Create a new interface */
int atm_bridge_cfg_create_if(atm_bridge_t *t,char **tokens,int count)
{
   netio_desc_t *nio = NULL;
   int nio_type;

   /* at least: IF, interface name, NetIO type */
   if (count < 3) {
      fprintf(stderr,"atmsw_cfg_create_if: invalid interface description\n");
      return(-1);
   }
   
   nio_type = netio_get_type(tokens[2]);
   switch(nio_type) {
      case NETIO_TYPE_UNIX:
         if (count != 5) {
            fprintf(stderr,"ATMSW: invalid number of arguments "
                    "for UNIX NIO '%s'\n",tokens[1]);
            break;
         }

         nio = netio_desc_create_unix(tokens[1],tokens[3],tokens[4]);
         break;

      case NETIO_TYPE_UDP:
         if (count != 6) {
            fprintf(stderr,"ATMSW: invalid number of arguments "
                    "for UDP NIO '%s'\n",tokens[1]);
            break;
         }

         nio = netio_desc_create_udp(tokens[1],atoi(tokens[3]),
                                     tokens[4],atoi(tokens[5]));
         break;

      case NETIO_TYPE_TCP_CLI:
         if (count != 5) {
            fprintf(stderr,"ATMSW: invalid number of arguments "
                    "for TCP CLI NIO '%s'\n",tokens[1]);
            break;
         }

         nio = netio_desc_create_tcp_cli(tokens[1],tokens[3],tokens[4]);
         break;

      case NETIO_TYPE_TCP_SER:
         if (count != 4) {
            fprintf(stderr,"ATMSW: invalid number of arguments "
                    "for TCP SER NIO '%s'\n",tokens[1]);
            break;
         }

         nio = netio_desc_create_tcp_ser(tokens[1],tokens[3]);
         break;

      default:
         fprintf(stderr,"ATMSW: unknown/invalid NETIO type '%s'\n",
                 tokens[2]);
   }

   if (!nio) {
      fprintf(stderr,"ATMSW: unable to create NETIO descriptor of "
              "interface %s\n",tokens[1]);
      return(-1);
   }

   netio_release(nio->name);
   return(0);
}

/* Bridge setup */
int atm_bridge_cfg_setup(atm_bridge_t *t,char **tokens,int count)
{
   /* 5 parameters: "BRIDGE", Eth_IF, ATM_IF, VPI, VCI */
   if (count != 5) {
      fprintf(stderr,"ATM Bridge: invalid VPC descriptor.\n");
      return(-1);
   }

   return(atm_bridge_configure(t,tokens[1],tokens[2],
                               atoi(tokens[3]),atoi(tokens[4])));
}

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
