/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * MIPS64 JIT compiler.
 */

#ifndef __MIPS64_JIT_H__
#define __MIPS64_JIT_H__

#include "rust_dynamips_c.h"

#include "utils.h"

/* Check if there are pending IRQ */
extern void mips64_check_pending_irq(mips64_jit_tcb_t *b);

/* Initialize instruction lookup table */
void mips64_jit_create_ilt(void);

/* Initialize the JIT structure */
int mips64_jit_init(cpu_mips_t *cpu);

/* Flush the JIT */
u_int mips64_jit_flush(cpu_mips_t *cpu,u_int threshold);

/* Shutdown the JIT */
void mips64_jit_shutdown(cpu_mips_t *cpu);

/* Check if an instruction is in a delay slot or not */
int mips64_jit_is_delay_slot(mips64_jit_tcb_t *b,m_uint64_t pc);

/* Fetch a MIPS instruction and emit corresponding x86 translated code */
struct mips64_insn_tag *mips64_jit_fetch_and_emit(cpu_mips_t *cpu,
                                                  mips64_jit_tcb_t *block,
                                                  int delay_slot);

/* Record a patch to apply in a compiled block */
int mips64_jit_tcb_record_patch(mips64_jit_tcb_t *block,u_char *x86_ptr,
                                m_uint64_t vaddr);

/* Free an instruction block */
void mips64_jit_tcb_free(cpu_mips_t *cpu,mips64_jit_tcb_t *block,
                         int list_removal);

/* Execute compiled MIPS code */
void *mips64_jit_run_cpu(cpu_gen_t *cpu);

/* Set the Pointer Counter (PC) register */
void mips64_set_pc(mips64_jit_tcb_t *b,m_uint64_t new_pc);

/* Set the Return Address (RA) register */
void mips64_set_ra(mips64_jit_tcb_t *b,m_uint64_t ret_pc);

/* Single-step operation */
void mips64_emit_single_step(mips64_jit_tcb_t *b,mips_insn_t insn);

/* Virtual Breakpoint */
void mips64_emit_breakpoint(mips64_jit_tcb_t *b);

/* Emit unhandled instruction code */
int mips64_emit_invalid_delay_slot(mips64_jit_tcb_t *b);

/* 
 * Increment count register and trigger the timer IRQ if value in compare 
 * register is the same.
 */
void mips64_inc_cp0_count_reg(mips64_jit_tcb_t *b);

/* Increment the number of executed instructions (performance debugging) */
void mips64_inc_perf_counter(mips64_jit_tcb_t *b);

#endif
