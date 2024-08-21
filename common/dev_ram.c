/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot.  All rights reserved.
 * 
 * RAM emulation.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <errno.h>
#include <unistd.h>

#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "memory.h"
#include "device.h"
