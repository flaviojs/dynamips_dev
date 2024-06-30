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
