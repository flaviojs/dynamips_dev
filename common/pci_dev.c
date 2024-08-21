/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * PCI devices.
 *
 * Very interesting docs:
 *   http://www.science.unitn.it/~fiorella/guidelinux/tlk/node72.html
 *   http://www.science.unitn.it/~fiorella/guidelinux/tlk/node76.html
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>

#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "memory.h"
#include "device.h"
