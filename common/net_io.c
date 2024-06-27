/*
 * Cisco router) simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Network Input/Output Abstraction Layer.
 */

#include "dynamips_common.h"

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

#ifdef __linux__
#include <net/if.h>
#include <linux/if_tun.h>
#endif

#include "rust_dynamips_c.h"
#include "net_io.h"
#include "net_io_filter.h"

/* Free a NetIO descriptor */
// TODO static
int netio_free(void *data,void *arg);

/* NIO RX listener */
static pthread_mutex_t netio_rxl_mutex = PTHREAD_MUTEX_INITIALIZER;
static pthread_mutex_t netio_rxq_mutex = PTHREAD_MUTEX_INITIALIZER;
static struct netio_rx_listener *netio_rxl_list = NULL;
static struct netio_rx_listener *netio_rxl_add_list = NULL;
static netio_desc_t *netio_rxl_remove_list = NULL;
static pthread_t netio_rxl_thread;
static pthread_cond_t netio_rxl_cond;

#define NETIO_RXL_LOCK()   pthread_mutex_lock(&netio_rxl_mutex);
#define NETIO_RXL_UNLOCK() pthread_mutex_unlock(&netio_rxl_mutex);

#define NETIO_RXQ_LOCK()   pthread_mutex_lock(&netio_rxq_mutex);
#define NETIO_RXQ_UNLOCK() pthread_mutex_unlock(&netio_rxq_mutex);

/*
 * =========================================================================
 * NULL Driver (does nothing, used for debugging)
 * =========================================================================
 */
static ssize_t netio_null_send(void *null_ptr,void *pkt,size_t pkt_len)
{
   return(pkt_len);
}

static ssize_t netio_null_recv(void *null_ptr,void *pkt,size_t max_len)
{
   usleep(200000);
   return(-1);
}

static void netio_null_save_cfg(netio_desc_t *nio,FILE *fd)
{
   fprintf(fd,"nio create_null %s\n",nio->name);
}

/* Create a new NetIO descriptor with NULL method */
netio_desc_t *netio_desc_create_null(char *nio_name)
{
   netio_desc_t *nio;
   
   if (!(nio = netio_create(nio_name)))
      return NULL;

   nio->type_    = NETIO_TYPE_NULL;
   nio->send     = (void *)netio_null_send;
   nio->recv     = (void *)netio_null_recv;
   nio->save_cfg = netio_null_save_cfg;
   nio->dptr     = NULL;

   if (netio_record(nio) == -1) {
      netio_free(nio,NULL);
      return NULL;
   }

   return nio;
}

/* Free a NetIO descriptor */
// TODO static
int netio_free(void *data,void *arg)
{
   netio_desc_t *nio = data;

   if (nio) {
      netio_filter_unbind(nio,NETIO_FILTER_DIR_RX);
      netio_filter_unbind(nio,NETIO_FILTER_DIR_TX);
      netio_filter_unbind(nio,NETIO_FILTER_DIR_BOTH);

      if (nio->free != NULL)
         nio->free(nio->dptr);

      free(nio->name);
      free(nio);
   }

   return(TRUE);
}

/* Reset NIO statistics */
void netio_reset_stats(netio_desc_t *nio)
{
   nio->stats_pkts_in = nio->stats_pkts_out = 0;
   nio->stats_bytes_in = nio->stats_bytes_out = 0;
}

/* Indicate if a NetIO can transmit a packet */
int netio_can_transmit(netio_desc_t *nio)
{
   u_int bw_current;

   /* No bandwidth constraint applied, can always transmit */
   if (!nio->bandwidth)
      return(TRUE);

   /* Check that we verify the bandwidth constraint */
   bw_current = nio->bw_cnt_total * 8 * 1000;
   bw_current /= 1024 * NETIO_BW_SAMPLE_ITV * NETIO_BW_SAMPLES;

   return(bw_current < nio->bandwidth);
}

/* Update bandwidth counter */
void netio_update_bw_stat(netio_desc_t *nio,m_uint64_t bytes)
{
   nio->bw_cnt[nio->bw_pos] += bytes;
   nio->bw_cnt_total += bytes;
}

/* Reset NIO bandwidth counter */
void netio_clear_bw_stat(netio_desc_t *nio)
{
   if (++nio->bw_ptask_cnt == (NETIO_BW_SAMPLE_ITV / ptask_sleep_time)) {
      nio->bw_ptask_cnt = 0;

      if (++nio->bw_pos == NETIO_BW_SAMPLES)
         nio->bw_pos = 0;

      nio->bw_cnt_total -= nio->bw_cnt[nio->bw_pos];
      nio->bw_cnt[nio->bw_pos] = 0;
   }
}

/* Set the bandwidth constraint */
void netio_set_bandwidth(netio_desc_t *nio,u_int bandwidth)
{
   nio->bandwidth = bandwidth;
}

/*
 * =========================================================================
 * RX Listeners
 * =========================================================================
 */

/* Find a RX listener */
static inline struct netio_rx_listener *netio_rxl_find(netio_desc_t *nio)
{
   struct netio_rx_listener *rxl;

   for(rxl=netio_rxl_list;rxl;rxl=rxl->next)
      if (rxl->nio == nio)
         return rxl;

   return NULL;
}

/* Remove a NIO from the listener list */
static int netio_rxl_remove_internal(netio_desc_t *nio)
{
   struct netio_rx_listener *rxl;
   int res = -1;

   if ((rxl = netio_rxl_find(nio))) {
      /* we suppress this NIO only when the ref count hits 0 */
      rxl->ref_count--;

      if (!rxl->ref_count) {
         /* remove this listener from the double linked list */
         if (rxl->next)
            rxl->next->prev = rxl->prev;
      
         if (rxl->prev)
            rxl->prev->next = rxl->next;
         else
            netio_rxl_list = rxl->next;

         /* if this is non-FD NIO, wait for thread to terminate */
         if (netio_get_fd(rxl->nio) == -1) {
            rxl->running = FALSE;
            pthread_join(rxl->spec_thread,NULL);
         }
         
         free(rxl);
      }

      res = 0;
   }
   
   return(res);
}

/* Add a RXL listener to the listener list */
static void netio_rxl_add_internal(struct netio_rx_listener *rxl)
{  
   struct netio_rx_listener *tmp;
   
   if ((tmp = netio_rxl_find(rxl->nio))) {
      tmp->ref_count++;
      free(rxl);
   } else {
      rxl->prev = NULL;
      rxl->next = netio_rxl_list;
      if (rxl->next) rxl->next->prev = rxl;
      netio_rxl_list = rxl;
   }
}

/* RX Listener dedicated thread (for non-FD NIO) */
static void *netio_rxl_spec_thread(void *arg)
{
   struct netio_rx_listener *rxl = arg;
   netio_desc_t *nio = rxl->nio;
   ssize_t pkt_len;

   while(rxl->running) {
      pkt_len = netio_recv(nio,nio->rx_pkt,sizeof(nio->rx_pkt));

      if (pkt_len > 0)
         rxl->rx_handler(nio,nio->rx_pkt,pkt_len,rxl->arg1,rxl->arg2);
   }

   return NULL;
}

/* RX Listener General Thread */
void *netio_rxl_gen_thread(void *arg)
{ 
   struct netio_rx_listener *rxl;
   ssize_t pkt_len;
   netio_desc_t *nio;
   struct timeval tv;
   int fd,fd_max,res;
   fd_set rfds;

   for(;;) {
      NETIO_RXL_LOCK();

      NETIO_RXQ_LOCK();
      /* Add the new waiting NIO to the active list */
      while(netio_rxl_add_list != NULL) {
         rxl = netio_rxl_add_list;
         netio_rxl_add_list = netio_rxl_add_list->next;
         netio_rxl_add_internal(rxl);
      }

      /* Delete the NIO present in the remove list */
      while(netio_rxl_remove_list != NULL) {
         nio = netio_rxl_remove_list;
         netio_rxl_remove_list = netio_rxl_remove_list->rxl_next;
         netio_rxl_remove_internal(nio);
      }

      pthread_cond_broadcast(&netio_rxl_cond);
      NETIO_RXQ_UNLOCK();

      /* Build the FD set */
      FD_ZERO(&rfds);
      fd_max = -1;
      for(rxl=netio_rxl_list;rxl;rxl=rxl->next) {
         if ((fd = netio_get_fd(rxl->nio)) == -1)
            continue;

         if (fd > fd_max) fd_max = fd;
         FD_SET(fd,&rfds);
      }
      NETIO_RXL_UNLOCK();

      /* Wait for incoming packets */
      tv.tv_sec = 0;
      tv.tv_usec = 20 * 1000;  /* 200 ms */
      res = select(fd_max+1,&rfds,NULL,NULL,&tv);

      if (res == -1) {
         if (errno != EINTR)
            perror("netio_rxl_thread: select");
         continue;
      }

      /* Examine active FDs and call user handlers */
      NETIO_RXL_LOCK();

      for(rxl=netio_rxl_list;rxl;rxl=rxl->next) {
         nio = rxl->nio;

         if ((fd = netio_get_fd(nio)) == -1)
            continue;

         if (FD_ISSET(fd,&rfds)) {
            pkt_len = netio_recv(nio,nio->rx_pkt,sizeof(nio->rx_pkt));

            if (pkt_len > 0)
               rxl->rx_handler(nio,nio->rx_pkt,pkt_len,rxl->arg1,rxl->arg2);
         }
      }

      NETIO_RXL_UNLOCK();
   }
   
   return NULL;
}

/* Add a RX listener in the listener list */
int netio_rxl_add(netio_desc_t *nio,netio_rx_handler_t rx_handler,
                  void *arg1,void *arg2)
{
   struct netio_rx_listener *rxl;

   NETIO_RXQ_LOCK();

   if (!(rxl = malloc(sizeof(*rxl)))) {
      NETIO_RXQ_UNLOCK();
      fprintf(stderr,"netio_rxl_add: unable to create structure.\n");
      return(-1);
   }

   memset(rxl,0,sizeof(*rxl));
   rxl->nio = nio;
   rxl->ref_count = 1;
   rxl->rx_handler = rx_handler;
   rxl->arg1 = arg1;
   rxl->arg2 = arg2;
   rxl->running = TRUE;

   if ((netio_get_fd(rxl->nio) == -1) &&
       pthread_create(&rxl->spec_thread,NULL,netio_rxl_spec_thread,rxl)) 
   {
      NETIO_RXQ_UNLOCK();
      fprintf(stderr,"netio_rxl_add: unable to create specific thread.\n");
      free(rxl);
      return(-1);
   }

   rxl->next = netio_rxl_add_list;
   netio_rxl_add_list = rxl;
   while(netio_rxl_add_list != NULL) {
      pthread_cond_wait(&netio_rxl_cond,&netio_rxq_mutex);
   }
   NETIO_RXQ_UNLOCK();
   return(0);
}

/* Remove a NIO from the listener list */
int netio_rxl_remove(netio_desc_t *nio)
{
   NETIO_RXQ_LOCK();
   nio->rxl_next = netio_rxl_remove_list;
   netio_rxl_remove_list = nio;
   while(netio_rxl_remove_list != NULL) {
      pthread_cond_wait(&netio_rxl_cond,&netio_rxq_mutex);
   }
   NETIO_RXQ_UNLOCK();
   return(0);
}

/* Initialize the RXL thread */
int netio_rxl_init(void)
{
   pthread_cond_init(&netio_rxl_cond,NULL);

   if (pthread_create(&netio_rxl_thread,NULL,netio_rxl_gen_thread,NULL)) {
      perror("netio_rxl_init: pthread_create");
      return(-1);
   }

   return(0);
}
