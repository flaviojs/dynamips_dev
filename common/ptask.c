/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Periodic tasks centralization. Used for TX part of network devices.
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
#include <sys/types.h>
#include <pthread.h>
#include <assert.h>

#include "ptask.h"

static pthread_t ptask_thread;

#define PTASK_LOCK() pthread_mutex_lock(&ptask_mutex)
#define PTASK_UNLOCK() pthread_mutex_unlock(&ptask_mutex)

/* Periodic task thread */
static void *ptask_run(void *arg)
{
   pthread_mutex_t umutex = PTHREAD_MUTEX_INITIALIZER;
   pthread_cond_t ucond = PTHREAD_COND_INITIALIZER;

   ptask_t *task;

   for(;;) {
      PTASK_LOCK();
      for(task=ptask_list;task;task=task->next)
         task->cbk(task->object,task->arg);
      PTASK_UNLOCK();

      /* For testing! */
      {
         struct timespec t_spc;
         m_tmcnt_t expire;

         expire = m_gettime_usec() + (ptask_sleep_time * 1000);

         pthread_mutex_lock(&umutex);
         t_spc.tv_sec = expire / 1000000;
         t_spc.tv_nsec = (expire % 1000000) * 1000;
         pthread_cond_timedwait(&ucond,&umutex,&t_spc);
         pthread_mutex_unlock(&umutex);
      }

      /* Old method... */
      //usleep(ptask_sleep_time*1000);
   }

   return NULL;
}

/* Initialize ptask module */
int ptask_init(u_int sleep_time)
{
   if (sleep_time)
      ptask_sleep_time = sleep_time;

   if (pthread_create(&ptask_thread,NULL,ptask_run,NULL) != 0) {
      fprintf(stderr,"ptask_init: unable to create thread.\n");
      return(-1);
   }

   return(0);
}
