/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Virtual console TTY.
 *
 * "Interactive" part idea by Mtve.
 * TCP console added by Mtve.
 * Serial console by Peter Ross (suxen_drol@hotmail.com)
 */

#include "rust_dynamips_c.h"

#include "dynamips_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/tcp.h>
#include <termios.h>
#include <netdb.h>
#include <fcntl.h>
#include <errno.h>
#include <assert.h>

#include <arpa/telnet.h>
#include <arpa/inet.h>

#include "utils.h"
#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "ppc32_exec.h"
#include "device.h"
#include "memory.h"
#include "dev_vtty.h"

#ifdef USE_UNSTABLE
#include "tcb.h"
#endif

#ifndef SOL_TCP
#define SOL_TCP 6
#endif

/* VTTY list */
static pthread_t vtty_thread;

#define VTTY_LIST_LOCK()   pthread_mutex_lock(&vtty_list_mutex);
#define VTTY_LIST_UNLOCK() pthread_mutex_unlock(&vtty_list_mutex);
 
/* VTTY TCP input */
static void vtty_tcp_input(int *fd_slot,void *opt)
{
   vtty_read_and_store((vtty_t *)opt,fd_slot);
}

/* VTTY thread */
static void *vtty_thread_main(void *arg)
{
   vtty_t *vtty;
   struct timeval tv;
   int fd_max,fd_tcp,res;
   fd_set rfds;
   int i;

   for(;;) {
      VTTY_LIST_LOCK();

      /* Build the FD set */
      FD_ZERO(&rfds);
      fd_max = -1;
      for(vtty=vtty_list;vtty;vtty=vtty->next) {

          switch(vtty->type_) {
              case VTTY_TYPE_TCP:

                  for(i=0;i<vtty->fd_count;i++)
                      if (vtty->fd_array[i] != -1) {
                          FD_SET(vtty->fd_array[i],&rfds);
                          if (vtty->fd_array[i] > fd_max)
                              fd_max = vtty->fd_array[i];
                      }

                  fd_tcp = fd_pool_set_fds(&vtty->fd_pool,&rfds);
                  fd_max = m_max(fd_tcp,fd_max);
                  break;

              default:
                  if (vtty->fd_array[0] != -1) {
                      FD_SET(vtty->fd_array[0],&rfds);
                      fd_max = m_max(vtty->fd_array[0],fd_max);
                  }
          }

      }
      VTTY_LIST_UNLOCK();

      /* Wait for incoming data */
      tv.tv_sec  = 0;
      tv.tv_usec = 50 * 1000;  /* 50 ms */
      res = select(fd_max+1,&rfds,NULL,NULL,&tv);

      if (res == -1) {
         if (errno != EINTR) {
            perror("vtty_thread: select");
         }
         continue;
      }

      /* Examine active FDs and call user handlers */
      VTTY_LIST_LOCK();
      for(vtty=vtty_list;vtty;vtty=vtty->next) {

         switch(vtty->type_) {
            case VTTY_TYPE_TCP:

               /* check incoming connection */
               for(i=0;i<vtty->fd_count;i++) {
                   
                   if (vtty->fd_array[i] == -1)
                       continue;
                   
                   if (!FD_ISSET(vtty->fd_array[i],&rfds))
                       continue;
                   
                   vtty_tcp_conn_accept(vtty, i);
               }

               /* check established connection */
               fd_pool_check_input(&vtty->fd_pool,&rfds,vtty_tcp_input,vtty);
               break;
      
            /* Term, Serial */
            default:
               if (vtty->fd_array[0] != -1 && FD_ISSET(vtty->fd_array[0],&rfds)) {
                  vtty_read_and_store(vtty,&vtty->fd_array[0]);
                  vtty->input_pending = TRUE;
               }
         }
         
         if (vtty->input_pending) {
            if (vtty->read_notifier != NULL)
               vtty->read_notifier(vtty);

            vtty->input_pending = FALSE;
         }

         /* Flush any pending output */
         if (!vtty->managed_flush)
            vtty_flush(vtty);
      }
      VTTY_LIST_UNLOCK();
   }
   
   return NULL;
}

/* Initialize the VTTY thread */
int vtty_init(void)
{
   if (pthread_create(&vtty_thread,NULL,vtty_thread_main,NULL)) {
      perror("vtty: pthread_create");
      return(-1);
   }

   return(0);
}
