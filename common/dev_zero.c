/*
 * Cisco router simulation platform.
 * Copyright (C) 2005,2006 Christophe Fillot.  All rights reserved.
 *
 * Zeroed memory zone.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <errno.h>

#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "memory.h"
#include "device.h"
