/*
 * Cisco Simulation Platform.
 * Copyright (c) 2006 Christophe Fillot.  All rights reserved.
 *
 * 23-Oct-2006: only basic code at this time.
 *
 * Considering the access pattern, this might be emulating SST39VF1681/SST39VF1682.
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
