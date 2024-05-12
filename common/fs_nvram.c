/** @file
 * @brief Cisco NVRAM filesystem.
 */

/*
 * Copyright (c) 2013 Flávio J. Saraiva <flaviojs2005@gmail.com>
 */

#include <assert.h>
#include <errno.h>
#include <stddef.h> // offsetof
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "fs_nvram.h"
