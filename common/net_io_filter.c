/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * NetIO Filtering.
 */

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
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <pthread.h>

#ifdef GEN_ETH
#include <pcap.h>
#endif

#include "rust_dynamips_c.h"
#include "net_io_filter.h"

/* ======================================================================== */
/* Frequency Dropping ("freq_drop").                                        */
/* ======================================================================== */

struct pf_freqdrop_data {
   int frequency;
   int current;
};

/* Setup filter ressources */
static int pf_freqdrop_setup(netio_desc_t *nio,void **opt,
                             int argc,char *argv[])
{
   struct pf_freqdrop_data *data = *opt;

   if (argc != 1)
      return(-1);

   if (!data) {
      if (!(data = malloc(sizeof(*data))))
         return(-1);

      *opt = data;
   }

   data->current = 0;
   data->frequency = atoi(argv[0]);
   return(0);
}

/* Free ressources used by filter */
static void pf_freqdrop_free(netio_desc_t *nio,void **opt)
{
   if (*opt)
      free(*opt);

   *opt = NULL;
}

/* Packet handler: drop 1 out of n packets */
static int pf_freqdrop_pkt_handler(netio_desc_t *nio,void *pkt,size_t len,
                                   void *opt)
{
   struct pf_freqdrop_data *data = opt;

   if (data != NULL) {
      switch(data->frequency) {
         case -1:
            return(NETIO_FILTER_ACTION_DROP);
         case 0:
            return(NETIO_FILTER_ACTION_PASS);
         default:
            data->current++;
         
            if (data->current == data->frequency) {
               data->current = 0;
               return(NETIO_FILTER_ACTION_DROP);
            }
      }
   }

   return(NETIO_FILTER_ACTION_PASS);
}

/* Packet dropping at 1/n frequency */
static netio_pktfilter_t pf_freqdrop_def = {
   "freq_drop",
   pf_freqdrop_setup,
   pf_freqdrop_free,
   pf_freqdrop_pkt_handler,
   NULL,
};

/* ======================================================================== */
/* Initialization of packet filters.                                        */
/* ======================================================================== */

void netio_filter_load_all(void)
{
   netio_filter_add(&pf_freqdrop_def);
#ifdef GEN_ETH
   netio_filter_add(&pf_capture_def);
#endif
}
