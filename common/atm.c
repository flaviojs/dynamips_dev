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

/* Free resources used by an ATM switch */
static int atmsw_free(void *data,void *arg)
{
   atmsw_table_t *t = data;
   atmsw_vp_conn_t *vp;
   atmsw_vc_conn_t *vc;
   int i;

   /* Remove all VPs */
   for(i=0;i<ATMSW_VP_HASH_SIZE;i++)
      for(vp=t->vp_table[i];vp;vp=vp->next)
         atmsw_release_vpc(vp);

   /* Remove all VCs */
   for(i=0;i<ATMSW_VC_HASH_SIZE;i++)
      for(vc=t->vc_table[i];vc;vc=vc->next)
         atmsw_release_vcc(vc);

   mp_free_pool(&t->mp);
   free(t);
   return(TRUE);
}

/* Delete an ATM switch */
int atmsw_delete(char *name)
{
   return(registry_delete_if_unused(name,OBJ_TYPE_ATMSW,atmsw_free,NULL));
}

/* Delete all ATM switches */
int atmsw_delete_all(void)
{
   return(registry_delete_type(OBJ_TYPE_ATMSW,atmsw_free,NULL));
}

/* Save the configuration of an ATM switch */
void atmsw_save_config(atmsw_table_t *t,FILE *fd)
{
   atmsw_vp_conn_t *vp;
   atmsw_vc_conn_t *vc;
   int i;

   fprintf(fd,"atmsw create %s\n",t->name);

   ATMSW_LOCK(t);

   for(i=0;i<ATMSW_VP_HASH_SIZE;i++) {
      for(vp=t->vp_table[i];vp;vp=vp->next) {
         fprintf(fd,"atmsw create_vpc %s %s %u %s %u\n",
                 t->name,vp->input->name,vp->vpi_in,
                 vp->output->name,vp->vpi_out);
      }
   }

   for(i=0;i<ATMSW_VC_HASH_SIZE;i++) {
      for(vc=t->vc_table[i];vc;vc=vc->next) {
         fprintf(fd,"atmsw create_vcc %s %s %u %u %s %u %u\n",
                 t->name,vc->input->name,vc->vpi_in,vc->vci_in,
                 vc->output->name,vc->vpi_out,vc->vci_out);
      }
   }
   
   ATMSW_UNLOCK(t);

   fprintf(fd,"\n");
}

/* Save configurations of all ATM switches */
static void atmsw_reg_save_config(registry_entry_t *entry,void *opt,int *err)
{
   atmsw_save_config((atmsw_table_t *)entry->data,(FILE *)opt);
}

void atmsw_save_config_all(FILE *fd)
{
   registry_foreach_type(OBJ_TYPE_ATMSW,atmsw_reg_save_config,fd,NULL);
}

/* Create a new interface */
int atmsw_cfg_create_if(atmsw_table_t *t,char **tokens,int count)
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

/* Create a new Virtual Path Connection */
int atmsw_cfg_create_vpc(atmsw_table_t *t,char **tokens,int count)
{
   /* 5 parameters: "VP", InputIF, InVPI, OutputIF, OutVPI */
   if (count != 5) {
      fprintf(stderr,"ATMSW: invalid VPC descriptor.\n");
      return(-1);
   }

   return(atmsw_create_vpc(t,tokens[1],atoi(tokens[2]),
                           tokens[3],atoi(tokens[4])));
}

/* Create a new Virtual Channel Connection */
int atmsw_cfg_create_vcc(atmsw_table_t *t,char **tokens,int count)
{
   /* 7 parameters: "VP", InputIF, InVPI/VCI, OutputIF, OutVPI/VCI */
   if (count != 7) {
      fprintf(stderr,"ATMSW: invalid VCC descriptor.\n");
      return(-1);
   }

   return(atmsw_create_vcc(t,tokens[1],atoi(tokens[2]),atoi(tokens[3]),
                           tokens[4],atoi(tokens[5]),atoi(tokens[6])));
}

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
