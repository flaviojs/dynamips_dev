/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * MIPS64 Step-by-step execution.
 */

#if __GNUC__ > 2

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <assert.h>

#include "cpu.h"
#include "vm.h"
#include "mips64_exec.h"
#include "memory.h"
#include "rust_dynamips_c.h"
#include "dynamips.h"

#endif
