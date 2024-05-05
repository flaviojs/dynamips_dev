/*
 * IPFlow Collector
 * Copyright (c) 2003 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * registry.c: Object Registry.
 */

#include "rust_dynamips_c.h"

#define _GNU_SOURCE
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
#include <pthread.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <assert.h>

#include "utils.h"
#include "registry.h"
