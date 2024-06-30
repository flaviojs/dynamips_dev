/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 *
 * ATM bridge (RFC1483)
 */

#include "rust_dynamips_c.h"

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
#include "atm_bridge.h"

#define ATM_BRIDGE_LOCK(t)   pthread_mutex_lock(&(t)->lock)
#define ATM_BRIDGE_UNLOCK(t) pthread_mutex_unlock(&(t)->lock)
