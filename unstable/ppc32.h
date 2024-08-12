/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __PPC_32_H__
#define __PPC_32_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h" 

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
void ppc32_trigger_irq(cpu_ppc_t *cpu);

/* Virtual breakpoint */
void ppc32_run_breakpoint(cpu_ppc_t *cpu);

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
