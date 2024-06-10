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

/* Execute a callback for the specified entry */
static inline void mips64_cp0_tlb_callback(cpu_mips_t *cpu,tlb_entry_t *entry,
                                           int action)
{
   _maybe_used m_uint64_t vaddr,paddr0,paddr1;
   _maybe_used m_uint32_t psize;

   vaddr = entry->hi & mips64_cp0_get_vpn2_mask(cpu);
   psize = get_page_size(entry->mask);

   if (entry->lo0 & MIPS_TLB_V_MASK) {
      paddr0 = (entry->lo0 & MIPS_TLB_PFN_MASK) << 6;
      
      /*printf("TLB: vaddr=0x%8.8llx -> paddr0=0x%10.10llx (size=0x%8.8x), "
             "action=%s\n",
             vaddr,paddr0,psize,
             (action == 0) ? "ADD" : "DELETE");*/
   }

   if (entry->lo1 & MIPS_TLB_V_MASK) {
      paddr1 = (entry->lo1 & MIPS_TLB_PFN_MASK) << 6;

      /*printf("TLB: vaddr=0x%8.8llx -> paddr1=0x%10.10llx (size=0x%8.8x), "
             "action=%s\n",
             vaddr,paddr1,psize,
             (action == 0) ? "ADD" : "DELETE");*/
   }
}

/* TLBW: Write a TLB entry */
static inline void mips64_cp0_exec_tlbw(cpu_mips_t *cpu,u_int index)
{
   mips_cp0_t *cp0 = &cpu->cp0;
   tlb_entry_t *entry;

#if DEBUG_TLB_ACTIVITY
   cpu_log(cpu,"TLB","CP0_TLBWI: writing entry %u "
           "[mask=0x%8.8llx,hi=0x%8.8llx,lo0=0x%8.8llx,lo1=0x%8.8llx]\n",
           index,cp0->reg[MIPS_CP0_PAGEMASK],cp0->reg[MIPS_CP0_TLB_HI],
           cp0->reg[MIPS_CP0_TLB_LO_0],cp0->reg[MIPS_CP0_TLB_LO_1]);
#endif

   if (index < cp0->tlb_entries)
   {
      entry = &cp0->tlb[index];

      mips64_cp0_tlb_callback(cpu,entry,TLB_ZONE_ADD);

      entry->mask = cp0->reg[MIPS_CP0_PAGEMASK] & MIPS_TLB_PAGE_MASK;
      entry->hi   = cp0->reg[MIPS_CP0_TLB_HI];
      entry->lo0  = cp0->reg[MIPS_CP0_TLB_LO_0];
      entry->lo1  = cp0->reg[MIPS_CP0_TLB_LO_1];

      /* if G bit is set in lo0 and lo1, set it in hi */
      if ((entry->lo0 & entry->lo1) & MIPS_CP0_LO_G_MASK)
         entry->hi |= MIPS_TLB_G_MASK;
      else
         entry->hi &= ~MIPS_TLB_G_MASK;

      /* Clear G bit in TLB lo0 and lo1 */
      entry->lo0 &= ~MIPS_CP0_LO_G_MASK;
      entry->lo1 &= ~MIPS_CP0_LO_G_MASK;

      /* Inform the MTS subsystem */
      cpu->mts_invalidate(cpu);

      mips64_cp0_tlb_callback(cpu,entry,TLB_ZONE_DELETE);

#if DEBUG_TLB_ACTIVITY
      mips64_tlb_dump_entry(cpu,index);
#endif
   }
}

/* TLBWI: Write Indexed TLB entry */
fastcall void mips64_cp0_exec_tlbwi(cpu_mips_t *cpu)
{
   mips64_cp0_exec_tlbw(cpu,cpu->cp0.reg[MIPS_CP0_INDEX]);
}

/* TLBWR: Write Random TLB entry */
fastcall void mips64_cp0_exec_tlbwr(cpu_mips_t *cpu)
{
   mips64_cp0_exec_tlbw(cpu,mips64_cp0_get_random_reg(cpu));
}

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
