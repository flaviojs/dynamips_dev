/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot.  All rights reserved.
 *
 * Intel Flash SIMM emulation.
 *
 * Intelligent ID Codes:
 *   28F008SA: 0x89A2 (1 Mb)
 *   28F016SA: 0x89A0 (2 Mb)
 *
 * Manuals:
 *    http://www.ortodoxism.ro/datasheets/Intel/mXvsysv.pdf
 *
 * TODO: A lot of commands are lacking. Doesn't work with NPE-G2.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <errno.h>

#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "memory.h"
#include "device.h"
