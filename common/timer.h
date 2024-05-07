/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * timer.h: Management of timers.
 */

#ifndef __TIMER_H__
#define __TIMER_H__  1

#include "rust_dynamips_c.h"

#include <sys/types.h>
#include <pthread.h>
#include "utils.h"

/* Remove a timer */
int timer_remove(timer_id id);

/* Create a new timer */
timer_id timer_create_entry(m_tmcnt_t interval,int boundary,int level,
                            timer_proc callback,void *user_arg);

/* Create a timer on boundary, with an offset */
timer_id timer_create_with_offset(m_tmcnt_t interval,m_tmcnt_t offset,
                                  int level,timer_proc callback,
                                  void *user_arg);

/* Set a new interval for a timer */
int timer_set_interval(timer_id id,long interval);

/* Create a new timer queue */
timer_queue_t *timer_create_queue(void);

/* Flush queues */
void timer_flush_queues(void);

/* Add a specified number of queues to the pool */
int timer_pool_add_queues(int nr_queues);

/* Initialize timer sub-system */
int timer_init(void);

#endif
