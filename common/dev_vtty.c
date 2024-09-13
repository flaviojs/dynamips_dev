/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Virtual console TTY.
 *
 * "Interactive" part idea by Mtve.
 * TCP console added by Mtve.
 * Serial console by Peter Ross (suxen_drol@hotmail.com)
 */

#include "rust_dynamips_c.h"

#include "dynamips_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/tcp.h>
#include <termios.h>
#include <netdb.h>
#include <fcntl.h>
#include <errno.h>
#include <assert.h>

#include <arpa/telnet.h>
#include <arpa/inet.h>

#include "utils.h"
#include "cpu.h"
#include "vm.h"
#include "dynamips.h"
#include "mips64_exec.h"
#include "ppc32_exec.h"
#include "dev_vtty.h"

#ifdef USE_UNSTABLE
#include "tcb.h"
#endif

#ifndef SOL_TCP
#define SOL_TCP 6
#endif
