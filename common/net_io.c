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
 * UDP sockets
 * =========================================================================
 */

/* Free a NetIO UDP descriptor */
static void netio_udp_free(netio_inet_desc_t *nid)
{
   if (nid->remote_host) {
      free(nid->remote_host);
      nid->remote_host = NULL;
   }

   if (nid->fd != -1) 
      close(nid->fd);
}

/* Send a packet to an UDP socket */
static ssize_t netio_udp_send(netio_inet_desc_t *nid,void *pkt,size_t pkt_len)
{
   return(send(nid->fd,pkt,pkt_len,0));
}

/* Receive a packet from an UDP socket */
static ssize_t netio_udp_recv(netio_inet_desc_t *nid,void *pkt,size_t max_len)
{
   return(recvfrom(nid->fd,pkt,max_len,0,NULL,NULL));
}

/* Save the NIO configuration */
static void netio_udp_save_cfg(netio_desc_t *nio,FILE *fd)
{
   netio_inet_desc_t *nid = nio->dptr;
   fprintf(fd,"nio create_udp %s %d %s %d\n",
           nio->name,nid->local_port,nid->remote_host,nid->remote_port);
}

/* Create a new NetIO descriptor with UDP method */
netio_desc_t *netio_desc_create_udp(char *nio_name,int local_port,
                                    char *remote_host,int remote_port)
{
   netio_inet_desc_t *nid;
   netio_desc_t *nio;
   
   if (!(nio = netio_create(nio_name)))
      return NULL;

   nid = &nio->u.nid;
   nid->local_port  = local_port;
   nid->remote_port = remote_port;

   if (!(nid->remote_host = strdup(remote_host))) {
      fprintf(stderr,"netio_desc_create_udp: insufficient memory\n");
      goto error;
   }

   if ((nid->fd = udp_connect(local_port,remote_host,remote_port)) < 0) {
      fprintf(stderr,"netio_desc_create_udp: unable to connect to %s:%d\n",
              remote_host,remote_port);
      goto error;
   }

   nio->type_    = NETIO_TYPE_UDP;
   nio->send     = (void *)netio_udp_send;
   nio->recv     = (void *)netio_udp_recv;
   nio->free     = (void *)netio_udp_free;
   nio->save_cfg = netio_udp_save_cfg;
   nio->dptr     = &nio->u.nid;

   if (netio_record(nio) == -1)
      goto error;

   return nio;

 error:
   netio_free(nio,NULL);
   return NULL;
}

/*
 * =========================================================================
 * UDP sockets with auto allocation
 * =========================================================================
 */

/* Get local port */
int netio_udp_auto_get_local_port(netio_desc_t *nio)
{
   if (nio->type_ != NETIO_TYPE_UDP_AUTO)
      return(-1);
   
   return(nio->u.nid.local_port);
}

/* Connect to a remote host/port */
int netio_udp_auto_connect(netio_desc_t *nio,char *host,int port)
{
   netio_inet_desc_t *nid = nio->dptr;

   /* NIO already connected */
   if (nid->remote_host != NULL)
      return(-1);
   
   if (!(nid->remote_host = strdup(host))) {
      fprintf(stderr,"netio_desc_create_udp_auto: insufficient memory\n");
      return(-1);
   }
   
   nid->remote_port = port;
   
   if (ip_connect_fd(nid->fd,nid->remote_host,nid->remote_port) < 0) {
      free(nid->remote_host);
      nid->remote_host = NULL;
      return(-1);
   }
   
   return(0);
}

/* Create a new NetIO descriptor with auto UDP method */
netio_desc_t *netio_desc_create_udp_auto(char *nio_name,char *local_addr,
                                         int port_start,int port_end)
{
   netio_inet_desc_t *nid;
   netio_desc_t *nio;
   
   if (!(nio = netio_create(nio_name)))
      return NULL;
   
   nid = &nio->u.nid;
   nid->local_port  = -1;
   nid->remote_host = NULL;
   nid->remote_port = -1;
      
   if ((nid->fd = udp_listen_range(local_addr,port_start,port_end,
                                   &nid->local_port)) < 0) 
   {
      fprintf(stderr,
              "netio_desc_create_udp_auto: unable to create socket "
              "(addr=%s,port_start=%d,port_end=%d)\n",
              local_addr,port_start,port_end);
      goto error;
   }
   
   nio->type_    = NETIO_TYPE_UDP_AUTO;
   nio->send     = (void *)netio_udp_send;
   nio->recv     = (void *)netio_udp_recv;
   nio->free     = (void *)netio_udp_free;
   nio->save_cfg = netio_udp_save_cfg;
   nio->dptr     = &nio->u.nid;
   
   if (netio_record(nio) == -1)
      goto error;
   
   return nio;
   
error:
   netio_free(nio,NULL);
   return NULL;
}

/*
 * =========================================================================
 * Linux RAW Ethernet driver
 * =========================================================================
 */
#ifdef LINUX_ETH
/* Free a NetIO raw ethernet descriptor */
static void netio_lnxeth_free(netio_lnxeth_desc_t *nled)
{
   if (nled->fd != -1) 
      close(nled->fd);
}

/* Send a packet to a raw Ethernet socket */
static ssize_t netio_lnxeth_send(netio_lnxeth_desc_t *nled,
                                 void *pkt,size_t pkt_len)
{
   return(lnx_eth_send(nled->fd,nled->dev_id,pkt,pkt_len));
}

/* Receive a packet from an raw Ethernet socket */
static ssize_t netio_lnxeth_recv(netio_lnxeth_desc_t *nled,
                                 void *pkt,size_t max_len)
{
   return(lnx_eth_recv(nled->fd,pkt,max_len));
}

/* Save the NIO configuration */
static void netio_lnxeth_save_cfg(netio_desc_t *nio,FILE *fd)
{
   netio_lnxeth_desc_t *nled = nio->dptr;
   fprintf(fd,"nio create_linux_eth %s %s\n",nio->name,nled->dev_name);
}

/* Create a new NetIO descriptor with raw Ethernet method */
netio_desc_t *netio_desc_create_lnxeth(char *nio_name,char *dev_name)
{
   netio_lnxeth_desc_t *nled;
   netio_desc_t *nio;
   
   if (!(nio = netio_create(nio_name)))
      return NULL;

   nled = &nio->u.nled;

   if (strlen(dev_name) >= NETIO_DEV_MAXLEN) {
      fprintf(stderr,"netio_desc_create_lnxeth: bad Ethernet device string "
              "specified.\n");
      netio_free(nio,NULL);
      return NULL;
   }

   strcpy(nled->dev_name,dev_name);

   nled->fd = lnx_eth_init_socket(dev_name);
   nled->dev_id = lnx_eth_get_dev_index(dev_name);

   if (nled->fd < 0) {
      netio_free(nio,NULL);
      return NULL;
   }

   nio->type_    = NETIO_TYPE_LINUX_ETH;
   nio->send     = (void *)netio_lnxeth_send;
   nio->recv     = (void *)netio_lnxeth_recv;
   nio->free     = (void *)netio_lnxeth_free;
   nio->save_cfg = netio_lnxeth_save_cfg;
   nio->dptr     = &nio->u.nled;

   if (netio_record(nio) == -1) {
      netio_free(nio,NULL);
      return NULL;
   }

   return nio;
}
#endif /* LINUX_ETH */

/*
 * =========================================================================
 * Generic RAW Ethernet driver
 * =========================================================================
 */
#ifdef GEN_ETH
/* Free a NetIO raw ethernet descriptor */
static void netio_geneth_free(netio_geneth_desc_t *nged)
{
   gen_eth_close(nged->pcap_dev);
}

/* Send a packet to an Ethernet device */
static ssize_t netio_geneth_send(netio_geneth_desc_t *nged,
                                 void *pkt,size_t pkt_len)
{
   return(gen_eth_send(nged->pcap_dev,pkt,pkt_len));
}

/* Receive a packet from an Ethernet device */
static ssize_t netio_geneth_recv(netio_geneth_desc_t *nged,
                                 void *pkt,size_t max_len)
{
   return(gen_eth_recv(nged->pcap_dev,pkt,max_len));
}

/* Save the NIO configuration */
static void netio_geneth_save_cfg(netio_desc_t *nio,FILE *fd)
{
   netio_geneth_desc_t *nged = nio->dptr;
   fprintf(fd,"nio create_gen_eth %s %s\n",nio->name,nged->dev_name);
}

/* Create a new NetIO descriptor with generic raw Ethernet method */
netio_desc_t *netio_desc_create_geneth(char *nio_name,char *dev_name)
{
   netio_geneth_desc_t *nged;
   netio_desc_t *nio;
      
   if (!(nio = netio_create(nio_name)))
      return NULL;

   nged = &nio->u.nged;

   if (strlen(dev_name) >= NETIO_DEV_MAXLEN) {
      fprintf(stderr,"netio_desc_create_geneth: bad Ethernet device string "
              "specified.\n");
      netio_free(nio,NULL);
      return NULL;
   }

   strcpy(nged->dev_name,dev_name);

   if (!(nged->pcap_dev = gen_eth_init(dev_name))) {
      netio_free(nio,NULL);
      return NULL;
   }

   nio->type_    = NETIO_TYPE_GEN_ETH;
   nio->send     = (void *)netio_geneth_send;
   nio->recv     = (void *)netio_geneth_recv;
   nio->free     = (void *)netio_geneth_free;
   nio->save_cfg = netio_geneth_save_cfg;
   nio->dptr     = &nio->u.nged;

   if (netio_record(nio) == -1) {
      netio_free(nio,NULL);
      return NULL;
   }

   return nio;
}
#endif /* GEN_ETH */

/*
 * =========================================================================
 * FIFO Driver (intra-hypervisor communications)
 * =========================================================================
 */

/* Extract the first packet of the FIFO */
static netio_fifo_pkt_t *netio_fifo_extract_pkt(netio_fifo_desc_t *nfd)
{ 
   netio_fifo_pkt_t *p;
      
   if (!(p = nfd->head))
      return NULL;

   nfd->pkt_count--;
   nfd->head = p->next;

   if (!nfd->head)
      nfd->last = NULL;

   return p;
}

/* Insert a packet into the FIFO (in tail) */
static void netio_fifo_insert_pkt(netio_fifo_desc_t *nfd,netio_fifo_pkt_t *p)
{   
   pthread_mutex_lock(&nfd->lock);

   nfd->pkt_count++;
   p->next = NULL;

   if (nfd->last) {
      nfd->last->next = p;
   } else {
      nfd->head = p;
   }

   nfd->last = p;
   pthread_mutex_unlock(&nfd->lock);
}

/* Free the packet list */
static void netio_fifo_free_pkt_list(netio_fifo_desc_t *nfd)
{
   netio_fifo_pkt_t *p,*next;
   
   for(p=nfd->head;p;p=next) {
      next = p->next;
      free(p);
   }

   nfd->head = nfd->last = NULL;
   nfd->pkt_count = 0;
}

/* Establish a cross-connect between two FIFO NetIO */
int netio_fifo_crossconnect(netio_desc_t *a,netio_desc_t *b)
{
   netio_fifo_desc_t *pa,*pb;

   if ((a->type_ != NETIO_TYPE_FIFO) || (b->type_ != NETIO_TYPE_FIFO))
      return(-1);

   pa = &a->u.nfd;
   pb = &b->u.nfd;

   /* A => B */
   pthread_mutex_lock(&pa->endpoint_lock);
   pthread_mutex_lock(&pa->lock);
   pa->endpoint = pb;
   netio_fifo_free_pkt_list(pa);
   pthread_mutex_unlock(&pa->lock);
   pthread_mutex_unlock(&pa->endpoint_lock);

   /* B => A */
   pthread_mutex_lock(&pb->endpoint_lock);
   pthread_mutex_lock(&pb->lock);
   pb->endpoint = pa;
   netio_fifo_free_pkt_list(pb);
   pthread_mutex_unlock(&pb->lock);
   pthread_mutex_unlock(&pb->endpoint_lock);
   return(0);
}

/* Unbind an endpoint */
static void netio_fifo_unbind_endpoint(netio_fifo_desc_t *nfd)
{
   pthread_mutex_lock(&nfd->endpoint_lock);
   nfd->endpoint = NULL;
   pthread_mutex_unlock(&nfd->endpoint_lock);
}

/* Free a NetIO FIFO descriptor */
static void netio_fifo_free(netio_fifo_desc_t *nfd)
{
   if (nfd->endpoint)
      netio_fifo_unbind_endpoint(nfd->endpoint);

   netio_fifo_free_pkt_list(nfd);
   pthread_mutex_destroy(&nfd->lock);
   pthread_cond_destroy(&nfd->cond);
}

/* Send a packet (to the endpoint FIFO) */
static ssize_t netio_fifo_send(netio_fifo_desc_t *nfd,void *pkt,size_t pkt_len)
{
   netio_fifo_pkt_t *p;
   size_t len;

   pthread_mutex_lock(&nfd->endpoint_lock);

   /* The cross-connect must have been established before */
   if (!nfd->endpoint)
      goto error;

   /* Allocate a a new packet and insert it into the endpoint FIFO */
   len = sizeof(netio_fifo_pkt_t) + pkt_len;
   if (!(p = malloc(len)))
      goto error;

   memcpy(p->pkt,pkt,pkt_len);
   p->pkt_len = pkt_len;
   netio_fifo_insert_pkt(nfd->endpoint,p);
   pthread_cond_signal(&nfd->endpoint->cond);
   pthread_mutex_unlock(&nfd->endpoint_lock);
   return(pkt_len);

 error:
   pthread_mutex_unlock(&nfd->endpoint_lock);
   return(-1);
}

/* Read a packet from the local FIFO queue */
static ssize_t netio_fifo_recv(netio_fifo_desc_t *nfd,void *pkt,size_t max_len)
{
   struct timespec ts; 
   m_tmcnt_t expire;
   netio_fifo_pkt_t *p;
   size_t len = -1;

   /* Wait for the endpoint to signal a new arriving packet */
   expire = m_gettime_usec() + 50000;
   ts.tv_sec = expire / 1000000;
   ts.tv_nsec = (expire % 1000000) * 1000;

   pthread_mutex_lock(&nfd->lock);
   pthread_cond_timedwait(&nfd->cond,&nfd->lock,&ts);

   /* Extract a packet from the list */
   p = netio_fifo_extract_pkt(nfd);
   pthread_mutex_unlock(&nfd->lock);

   if (p) {
      len = m_min(p->pkt_len,max_len);
      memcpy(pkt,p->pkt,len);
      free(p);
   }
   
   return(len);
}

/* Create a new NetIO descriptor with FIFO method */
netio_desc_t *netio_desc_create_fifo(char *nio_name)
{
   netio_fifo_desc_t *nfd;
   netio_desc_t *nio;
   
   if (!(nio = netio_create(nio_name)))
      return NULL;

   nfd = &nio->u.nfd;
   pthread_mutex_init(&nfd->lock,NULL);
   pthread_mutex_init(&nfd->endpoint_lock,NULL);
   pthread_cond_init(&nfd->cond,NULL);

   nio->type_ = NETIO_TYPE_FIFO;
   nio->send = (void *)netio_fifo_send;
   nio->recv = (void *)netio_fifo_recv;
   nio->free = (void *)netio_fifo_free;
   nio->dptr = nfd;

   if (netio_record(nio) == -1) {
      netio_free(nio,NULL);
      return NULL;
   }

   return nio;
}

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
