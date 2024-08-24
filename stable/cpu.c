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
#include "mips64.h"
#include "mips64_cp0.h"
#include "mips64_exec.h"
#include "mips64_jit.h"
#include "ppc32.h"
#include "ppc32_exec.h"
#include "ppc32_jit.h"
#include "dynamips.h"
#include "vm.h"

/* Log a message for a CPU */
void cpu_log(cpu_gen_t *cpu,char *module,char *format,...)
{
   char buffer[256];
   va_list ap;
   char *i;
   char *buf;

   buffer[0] = 'C';
   buffer[1] = 'P';
   buffer[2] = 'U';

   switch (cpu->id){
       case 0:
           buffer[3] = '0';
           break;
       case 1:
           buffer[3] = '1';
           break;
       case 2:
           buffer[3] = '2';
           break;
       case 3:
           buffer[3] = '3';
           break;
       case 4:
           buffer[3] = '4';
           break;
       case 5:
           buffer[3] = '5';
           break;
       case 6:
           buffer[3] = '6';
           break;
       case 7:
           buffer[3] = '7';
           break;
       case 8:
           buffer[3] = '8';
           break;
       case 9:
           buffer[3] = '9';
           break;
       default:
           buffer[3] = '-';
           break;
   }

   buffer[4] = ':';
   buffer[5] = ' ';

   buf = &buffer[6];
   for(i = module; *i != '\0'; ++i) {
       *buf = *i;
       ++buf;
   }

   *buf = '\0';

   va_start(ap,format);
   vm_flog(cpu->vm,buffer,format,ap);
   va_end(ap);
}
