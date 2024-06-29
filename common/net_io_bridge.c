/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * NetIO bridges.
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
#include "net_io_bridge.h"

#define PKT_MAX_SIZE 2048

/* Create a new interface */
static int netio_bridge_cfg_create_if(netio_bridge_t *t,
                                      char **tokens,int count)
{
   netio_desc_t *nio = NULL;
   int nio_type;

   nio_type = netio_get_type(tokens[1]);
   switch(nio_type) {
      case NETIO_TYPE_UNIX:
         if (count != 4) {
            fprintf(stderr,"NETIO_BRIDGE: invalid number of arguments "
                    "for UNIX NIO\n");
            break;
         }

         nio = netio_desc_create_unix(tokens[0],tokens[2],tokens[3]);
         break;

      case NETIO_TYPE_TAP:
         if (count != 3) {
            fprintf(stderr,"NETIO_BRIDGE: invalid number of arguments "
                    "for TAP NIO\n");
            break;
         }

         nio = netio_desc_create_tap(tokens[0],tokens[2]);
         break;

      case NETIO_TYPE_UDP:
         if (count != 5) {
            fprintf(stderr,"NETIO_BRIDGE: invalid number of arguments "
                    "for UDP NIO\n");
            break;
         }

         nio = netio_desc_create_udp(tokens[0],atoi(tokens[2]),
                                     tokens[3],atoi(tokens[4]));
         break;

      case NETIO_TYPE_TCP_CLI:
         if (count != 4) {
            fprintf(stderr,"NETIO_BRIDGE: invalid number of arguments "
                    "for TCP CLI NIO\n");
            break;
         }

         nio = netio_desc_create_tcp_cli(tokens[0],tokens[2],tokens[3]);
         break;

      case NETIO_TYPE_TCP_SER:
         if (count != 3) {
            fprintf(stderr,"NETIO_BRIDGE: invalid number of arguments "
                    "for TCP SER NIO\n");
            break;
         }

         nio = netio_desc_create_tcp_ser(tokens[0],tokens[2]);
         break;

#ifdef GEN_ETH
      case NETIO_TYPE_GEN_ETH:
         if (count != 3) {
            fprintf(stderr,"NETIO_BRIDGE: invalid number of arguments "
                    "for Generic Ethernet NIO\n");
            break;
         }
         
         nio = netio_desc_create_geneth(tokens[0],tokens[2]);
         break;
#endif

#ifdef LINUX_ETH
      case NETIO_TYPE_LINUX_ETH:
         if (count != 3) {
            fprintf(stderr,"NETIO_BRIDGE: invalid number of arguments "
                    "for Linux Ethernet NIO\n");
            break;
         }
         
         nio = netio_desc_create_lnxeth(tokens[0],tokens[2]);
         break;
#endif

      default:
         fprintf(stderr,"NETIO_BRIDGE: unknown/invalid NETIO type '%s'\n",
                 tokens[1]);
   }

   if (!nio) {
      fprintf(stderr,"NETIO_BRIDGE: unable to create NETIO descriptor\n");
      return(-1);
   }

   if (netio_bridge_add_netio(t,tokens[0]) == -1) {
      fprintf(stderr,"NETIO_BRIDGE: unable to add NETIO descriptor.\n");
      netio_release(nio->name);
      return(-1);
   }

   netio_release(nio->name);
   return(0);
}

#define NETIO_BRIDGE_MAX_TOKENS  16

/* Handle a configuration line */
static int netio_bridge_handle_cfg_line(netio_bridge_t *t,char *str)
{  
   char *tokens[NETIO_BRIDGE_MAX_TOKENS];
   int count;

   if ((count = m_strsplit(str,':',tokens,NETIO_BRIDGE_MAX_TOKENS)) <= 2)
      return(-1);

   return(netio_bridge_cfg_create_if(t,tokens,count));
}

/* Read a configuration file */
static int netio_bridge_read_cfg_file(netio_bridge_t *t,char *filename)
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
         netio_bridge_handle_cfg_line(t,buffer);
   }
   
   fclose(fd);
   return(0);
}

/* Start a virtual bridge */
int netio_bridge_start(char *filename)
{
   netio_bridge_t *t;

   if (!(t = netio_bridge_create("default"))) {
      fprintf(stderr,"NETIO_BRIDGE: unable to create virtual fabric table.\n");
      return(-1);
   }

   if (netio_bridge_read_cfg_file(t,filename) == -1) {
      fprintf(stderr,"NETIO_BRIDGE: unable to parse configuration file.\n");
      return(-1);
   }
   
   netio_bridge_release("default");
   return(0);
}
