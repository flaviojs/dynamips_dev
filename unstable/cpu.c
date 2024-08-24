/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Management of CPU groups (for MP systems).
 */

#include "rust_dynamips_c.h"

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <stdarg.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <pthread.h>

#include "cpu.h"
#include "vm.h"
#include "tcb.h"
#include "memory.h"
#include "mips64.h"
#include "mips64_cp0.h"
#include "mips64_exec.h"
#include "mips64_jit.h"
#include "ppc32.h"
#include "ppc32_exec.h"
#include "ppc32_jit.h"
#include "dynamips.h"

/* Log a message for a CPU */
void cpu_log(cpu_gen_t *cpu,char *module,char *format,...)
{
   char buffer[256];
   va_list ap;

   va_start(ap,format);
   snprintf(buffer,sizeof(buffer),"CPU%u: %s",cpu->id,module);
   vm_flog(cpu->vm,buffer,format,ap);
   va_end(ap);
}
