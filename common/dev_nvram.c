/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot.  All rights reserved.
 *
 * Dallas DS1216 chip emulation:
 *   - NVRAM
 *   - Calendar
 *
 * Manuals:
 *    http://pdfserv.maxim-ic.com/en/ds/DS1216-DS1216H.pdf
 *
 * Calendar stuff written by Mtve.
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
