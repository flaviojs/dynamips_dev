/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * MIPS Coprocessor 0 (System Coprocessor) implementation.
 * We don't use the JIT here, since there is no high performance needed.
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

#include "device.h"
#include "mips64.h"
#include "mips64_cp0.h"
#include "dynamips.h"
#include "memory.h"

/* Raw dump of the TLB */
void mips64_tlb_raw_dump(cpu_gen_t *cpu)
{
   cpu_mips_t *mcpu = CPU_MIPS64(cpu);
   tlb_entry_t *entry;
   u_int i;

   printf("TLB dump:\n");

   for(i=0;i<mcpu->cp0.tlb_entries;i++) {
      entry = &mcpu->cp0.tlb[i];
      printf(" %2d: mask=0x%16.16llx hi=0x%16.16llx "
             "lo0=0x%16.16llx lo1=0x%16.16llx\n",
             i, entry->mask, entry->hi, entry->lo0, entry->lo1);
   }

   printf("\n");
}
