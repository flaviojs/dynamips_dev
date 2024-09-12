/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 * Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
 *
 * Many thanks to Nicolas Szalay for his patch
 * for the command line parsing and virtual machine
 * settings (RAM, ROM, NVRAM, ...)
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <signal.h>
#include <fcntl.h>
#include <assert.h>
#include <getopt.h>

#include "dynamips.h"
#include "cpu.h"
#include "vm.h"

#ifdef USE_UNSTABLE
#include "tcb.h"
#endif

#include "mips64_exec.h"
#include "mips64_jit.h"
#include "ppc32_exec.h"
#include "ppc32_jit.h"
#include "dev_c7200.h"
#include "dev_c3600.h"
#include "dev_c2691.h"
#include "dev_c3725.h"
#include "dev_c3745.h"
#include "dev_c2600.h"
#include "dev_c1700.h"
#include "dev_c6msfc1.h"
#include "dev_c6sup1.h"
#include "ppc32_vmtest.h"
#include "dev_vtty.h"
#include "hypervisor.h"
#include "net_io_bridge.h"
#include "atm.h"
#include "atm_bridge.h"
#include "frame_relay.h"
#include "eth_switch.h"
#ifdef PROFILE
#include "profiler.h"
#endif

int main(int argc,char *argv[])
{
   return(dynamips_main(argc, argv));
}
