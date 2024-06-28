/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * NetIO Filtering.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <unistd.h>
#include <errno.h>
#include <signal.h>
#include <fcntl.h>
#include <ctype.h>
#include <time.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <pthread.h>

#ifdef GEN_ETH
#include <pcap.h>
#endif

#include "rust_dynamips_c.h"
#include "net_io_filter.h"

/* ======================================================================== */
/* Initialization of packet filters.                                        */
/* ======================================================================== */

void netio_filter_load_all(void)
{
   netio_filter_add(&pf_freqdrop_def);
#ifdef GEN_ETH
   netio_filter_add(&pf_capture_def);
#endif
}
