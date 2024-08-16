/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * PPC32 JIT compiler.
 */

#ifndef __PPC32_JIT_H__
#define __PPC32_JIT_H__

#include "rust_dynamips_c.h"

#include "utils.h"

/* Indicate registers modified by ppc32_update_cr() functions */
extern void ppc32_update_cr_set_altered_hreg(cpu_ppc_t *cpu);

/* ======================================================================== */
/* JIT operations with implementations specific to target CPU */
void ppc32_op_insn_output(ppc32_jit_tcb_t *b,jit_op_t *op);
void ppc32_op_load_gpr(ppc32_jit_tcb_t *b,jit_op_t *op);
void ppc32_op_store_gpr(ppc32_jit_tcb_t *b,jit_op_t *op);
void ppc32_op_update_flags(ppc32_jit_tcb_t *b,jit_op_t *op);
void ppc32_op_move_host_reg(ppc32_jit_tcb_t *b,jit_op_t *op);
void ppc32_op_set_host_reg_imm32(ppc32_jit_tcb_t *b,jit_op_t *op);

/* Set the Instruction Address (IA) register */
void ppc32_set_ia(u_char **ptr,m_uint32_t new_ia);

/* Jump to the next page */
void ppc32_set_page_jump(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b);

/* Increment the number of executed instructions (performance debugging) */
void ppc32_inc_perf_counter(cpu_ppc_t *cpu);

/* ======================================================================== */

/* Virtual Breakpoint */
void ppc32_emit_breakpoint(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b);

/* Initialize instruction lookup table */
void ppc32_jit_create_ilt(void);

/* Initialize the JIT structure */
int ppc32_jit_init(cpu_ppc_t *cpu);

/* Flush the JIT */
u_int ppc32_jit_flush(cpu_ppc_t *cpu,u_int threshold);

/* Shutdown the JIT */
void ppc32_jit_shutdown(cpu_ppc_t *cpu);

/* Fetch a PowerPC instruction and emit corresponding translated code */
struct ppc32_insn_tag *ppc32_jit_fetch_and_emit(cpu_ppc_t *cpu,
                                                ppc32_jit_tcb_t *block);

/* Record a patch to apply in a compiled block */
int ppc32_jit_tcb_record_patch(ppc32_jit_tcb_t *block,jit_op_t *iop,
                               u_char *jit_ptr,m_uint32_t vaddr);

/* Free an instruction block */
void ppc32_jit_tcb_free(cpu_ppc_t *cpu,ppc32_jit_tcb_t *block,
                        int list_removal);

/* Check if the specified address belongs to the specified block */
int ppc32_jit_tcb_local_addr(ppc32_jit_tcb_t *block,m_uint32_t vaddr,
                             u_char **jit_addr);

/* Recompile a page */
int ppc32_jit_tcb_recompile(cpu_ppc_t *cpu,ppc32_jit_tcb_t *block);

/* Execute compiled PowerPC code */
void *ppc32_jit_run_cpu(cpu_gen_t *gen);

/* Start register allocation sequence */
void ppc32_jit_start_hreg_seq(cpu_ppc_t *cpu,char *insn);

/* Close register allocation sequence */
void ppc32_jit_close_hreg_seq(cpu_ppc_t *cpu);

/* Insert a reg map as head of list (as MRU element) */
void ppc32_jit_insert_hreg_mru(cpu_ppc_t *cpu,struct hreg_map *map);

/* Allocate an host register */
int ppc32_jit_alloc_hreg(cpu_ppc_t *cpu,int ppc_reg);

/* Force allocation of an host register */
int ppc32_jit_alloc_hreg_forced(cpu_ppc_t *cpu,int hreg);

/* Initialize register mapping */
void ppc32_jit_init_hreg_mapping(cpu_ppc_t *cpu);

#endif
