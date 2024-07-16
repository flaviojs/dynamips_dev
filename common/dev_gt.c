/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Galileo GT64010/GT64120A/GT96100A system controller.
 *
 * The DMA stuff is not complete, only "normal" transfers are working
 * (source and destination addresses incrementing).
 *
 * Also, these transfers are "instantaneous" from a CPU point-of-view: when
 * a channel is enabled, the transfer is immediately done. So, this is not 
 * very realistic.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stddef.h>

#include "utils.h"
#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "dev_gt.h"
