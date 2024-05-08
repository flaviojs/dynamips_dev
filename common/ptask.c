/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Periodic tasks centralization. Used for TX part of network devices.
 */

#include "rust_dynamips_c.h"

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
