/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __PPC_32_H__
#define __PPC_32_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h" 

/* Condition Register (CR) is accessed through 8 fields of 4 bits */
#define ppc32_get_cr_field(n)  ((n) >> 2)
#define ppc32_get_cr_bit(n)    (~(n) & 0x03)

#define PPC32_CR_FIELD_OFFSET(f) \
   (OFFSET(cpu_ppc_t,cr_fields)+((f) * sizeof(u_int)))

/* Get the full CR register */
static forced_inline m_uint32_t ppc32_get_cr(cpu_ppc_t *cpu)
{
   m_uint32_t cr = 0;
   int i;

   for(i=0;i<8;i++)
      cr |= cpu->cr_fields[i] << (28 - (i << 2));

   return(cr);
}

/* Set the CR fields given a CR value */
static forced_inline void ppc32_set_cr(cpu_ppc_t *cpu,m_uint32_t cr)
{
   int i;

   for(i=0;i<8;i++)
      cpu->cr_fields[i] = (cr >> (28 - (i << 2))) & 0x0F;
}

/* Get a CR bit */
static forced_inline m_uint32_t ppc32_read_cr_bit(cpu_ppc_t *cpu,u_int bit)
{
   m_uint32_t res;

   res = cpu->cr_fields[ppc32_get_cr_field(bit)] >> ppc32_get_cr_bit(bit);
   return(res & 0x01);
}

/* Set a CR bit */
static forced_inline void ppc32_set_cr_bit(cpu_ppc_t *cpu,u_int bit)
{
   cpu->cr_fields[ppc32_get_cr_field(bit)] |= 1 << ppc32_get_cr_bit(bit);
}

/* Clear a CR bit */
static forced_inline void ppc32_clear_cr_bit(cpu_ppc_t *cpu,u_int bit)
{
   cpu->cr_fields[ppc32_get_cr_field(bit)] &= ~(1 << ppc32_get_cr_bit(bit));
}

/* Reset a PowerPC CPU */
int ppc32_reset(cpu_ppc_t *cpu);

/* Initialize a PowerPC processor */
int ppc32_init(cpu_ppc_t *cpu);

/* Delete a PowerPC processor */
void ppc32_delete(cpu_ppc_t *cpu);

/* Set the processor version register (PVR) */
void ppc32_set_pvr(cpu_ppc_t *cpu,m_uint32_t pvr);

/* Set idle PC value */
void ppc32_set_idle_pc(cpu_gen_t *cpu,m_uint64_t addr);

/* Timer IRQ */
void *ppc32_timer_irq_run(cpu_ppc_t *cpu);

/* Determine an "idling" PC */
int ppc32_get_idling_pc(cpu_gen_t *cpu);

/* Generate an exception */
void ppc32_trigger_exception(cpu_ppc_t *cpu,u_int exc_vector);

/* Trigger the decrementer exception */
void ppc32_trigger_timer_irq(cpu_ppc_t *cpu);

/* Trigger IRQs */
fastcall void ppc32_trigger_irq(cpu_ppc_t *cpu);

/* Virtual breakpoint */
fastcall void ppc32_run_breakpoint(cpu_ppc_t *cpu);

/* Add a virtual breakpoint */
int ppc32_add_breakpoint(cpu_gen_t *cpu,m_uint64_t ia);

/* Remove a virtual breakpoint */
void ppc32_remove_breakpoint(cpu_gen_t *cpu,m_uint64_t ia);

/* Set a register */
void ppc32_reg_set(cpu_gen_t *cpu,u_int reg,m_uint64_t val);

/* Dump registers of a PowerPC processor */
void ppc32_dump_regs(cpu_gen_t *cpu);

/* Dump MMU registers */
void ppc32_dump_mmu(cpu_gen_t *cpu);

/* Load a raw image into the simulated memory */
int ppc32_load_raw_image(cpu_ppc_t *cpu,char *filename,m_uint32_t vaddr);

/* Load an ELF image into the simulated memory */
int ppc32_load_elf_image(cpu_ppc_t *cpu,char *filename,int skip_load,
                         m_uint32_t *entry_point);

/* Run PowerPC code in step-by-step mode */
void *ppc32_exec_run_cpu(cpu_gen_t *gen);

#endif
