/*  
 * Cisco router simulation platform.
 * Copyright (C) 2006 Christophe Fillot.  All rights reserved.
 *
 * AMD Am79c971 FastEthernet chip emulation.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <unistd.h>
#include <time.h>
#include <errno.h>
#include <assert.h>

#include "utils.h"
#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "dev_am79c971.h"
