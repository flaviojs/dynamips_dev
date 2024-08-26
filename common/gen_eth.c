/*
 * Copyright (c) 2006 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * gen_eth.c: module used to send/receive Ethernet packets.
 *
 * Use libpcap (0.9+) or WinPcap (0.4alpha1+) to receive and send packets.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <unistd.h>
#include <errno.h>
#include <signal.h>
#include <fcntl.h>
#include <ctype.h>
#include <netdb.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <sys/ioctl.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <pthread.h>

#ifdef CYGWIN
/* Needed for pcap_open() flags */
#define HAVE_REMOTE
#endif

#include "pcap.h"
#include "utils.h"
#include "gen_eth.h"
