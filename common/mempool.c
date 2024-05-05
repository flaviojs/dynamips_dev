/*
 * Copyright (c) 1999-2006 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * mempool.c: Simple Memory Pools.
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
#include <sys/stat.h>
#include <sys/types.h>
#include <assert.h>

#include "utils.h"
#include "mempool.h"
