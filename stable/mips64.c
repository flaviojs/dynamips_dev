/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * XXX TODO: proper context save/restore for CPUs.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <assert.h>

#include "cpu.h"
#include "mips64_mem.h"
#include "mips64_exec.h"
#include "mips64_jit.h"
#include "dynamips.h"
