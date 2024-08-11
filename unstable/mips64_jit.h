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
#include "tcb.h"

/* Check if there are pending IRQ */
extern void mips64_check_pending_irq(cpu_tc_t *tc);

/* Initialize instruction lookup table */
void mips64_jit_create_ilt(void);

/* Initialize the JIT structure */
int mips64_jit_init(cpu_mips_t *cpu);

/* Flush the JIT */
u_int mips64_jit_flush(cpu_mips_t *cpu,u_int threshold);

/* Shutdown the JIT */
void mips64_jit_shutdown(cpu_mips_t *cpu);

/* Check if an instruction is in a delay slot or not */
int mips64_jit_is_delay_slot(cpu_tc_t *tc,m_uint64_t pc);

/* Fetch a MIPS instruction and emit corresponding translated code */
struct mips64_insn_tag *mips64_jit_fetch_and_emit(cpu_mips_t *cpu,
                                                  cpu_tc_t *tc,
                                                  int delay_slot);

/* Record a patch to apply in a compiled block */
int mips64_jit_tcb_record_patch(cpu_mips_t *cpu,cpu_tc_t *tc,
                                u_char *jit_ptr,m_uint64_t vaddr);

/* Mark a block as containing self-modifying code */
void mips64_jit_mark_smc(cpu_mips_t *cpu,cpu_tb_t *tb);

/* Free an instruction block */
void mips64_jit_tcb_free(cpu_mips_t *cpu,cpu_tb_t *tb,int list_removal);

/* Execute compiled MIPS code */
void *mips64_jit_run_cpu(cpu_gen_t *cpu);

/* Set the Pointer Counter (PC) register */
void mips64_set_pc(cpu_tc_t *tc,m_uint64_t new_pc);

/* Set the Return Address (RA) register */
void mips64_set_ra(cpu_tc_t *tc,m_uint64_t ret_pc);

/* Single-step operation */
void mips64_emit_single_step(cpu_tc_t *tc,mips_insn_t insn);

/* Virtual Breakpoint */
void mips64_emit_breakpoint(cpu_tc_t *tc);

/* Emit unhandled instruction code */
int mips64_emit_invalid_delay_slot(cpu_tc_t *tc);

/* 
 * Increment count register and trigger the timer IRQ if value in compare 
 * register is the same.
 */
void mips64_inc_cp0_count_reg(cpu_tc_t *tc);

/* Increment the number of executed instructions (performance debugging) */
void mips64_inc_perf_counter(cpu_tc_t *tc);

#endif
