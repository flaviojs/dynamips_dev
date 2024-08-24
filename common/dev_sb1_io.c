/*
 * Cisco router simulation platform.
 * Copyright (c) 2005 Christophe Fillot (cf@utc.fr)
 *
 * SB-1 I/O devices.
 *
 * XXX: just for tests!
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <termios.h>
#include <fcntl.h>
#include <pthread.h>

#include "utils.h"
#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "memory.h"
#include "device.h"
#include "dev_c7200.h"
