/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Periodic tasks centralization.
 */

#ifndef __PTASK_H__
#define __PTASK_H__

#include "rust_dynamips_c.h"

#include <sys/types.h>
#include <sys/socket.h>
#include <sys/un.h>
#include "utils.h"

/* Add a new task */
ptask_id_t ptask_add(ptask_callback cbk,void *object,void *arg);

/* Remove a task */
int ptask_remove(ptask_id_t id);

/* Initialize ptask module */
int ptask_init(u_int sleep_time);

#endif
