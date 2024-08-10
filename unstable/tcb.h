/*
 * Cisco router simulation platform.
 * Copyright (c) 2008 Christophe Fillot (cf@utc.fr)
 *
 * Translation Sharing Groups.
 */

#ifndef __TSG_H__
#define __TSG_H__

#include "rust_dynamips_c.h"

#include "utils.h"
#include "vm.h"

/* Allocate a new TCB descriptor */
cpu_tc_t *tc_alloc(cpu_gen_t *cpu,m_uint64_t vaddr,m_uint32_t exec_state);

/* Bind a CPU to a TSG - If the group isn't specified, create one */
int tsg_bind_cpu(cpu_gen_t *cpu);

/* Unbind a CPU from a TSG - release all resources used */
int tsg_unbind_cpu(cpu_gen_t *cpu);

/* Create a JIT chunk */
int tc_alloc_jit_chunk(cpu_gen_t *cpu,cpu_tc_t *tc);

/* Allocate a new TC descriptor */
cpu_tc_t *tc_alloc(cpu_gen_t *cpu,m_uint64_t vaddr,m_uint32_t exec_state);

/* Compute a checksum on a page */
tsg_checksum_t tsg_checksum_page(void *page,ssize_t size);

/* Try to find a TC descriptor to share generated code */
int tc_find_shared(cpu_gen_t *cpu,cpu_tb_t *tb);

/* Register a newly compiled TCB descriptor */
void tc_register(cpu_gen_t *cpu,cpu_tb_t *tb,cpu_tc_t *tc);

/* Remove all TC descriptors belonging to a single CPU (ie not shared) */
int tsg_remove_single_desc(cpu_gen_t *cpu);

/* Dump a TCB */
void tb_dump(cpu_tb_t *tb);

/* Dump a TCB descriptor */
void tc_dump(cpu_tc_t *tc);

/* Show statistics about all translation groups */
void tsg_show_stats(void);

/* Adjust the JIT buffer if its size is not sufficient */
int tc_adjust_jit_buffer(cpu_gen_t *cpu,cpu_tc_t *tc,
                         void (*set_jump)(u_char **insn,u_char *dst));

/* Record a patch to apply in a compiled block */
struct insn_patch *tc_record_patch(cpu_gen_t *cpu,cpu_tc_t *tc,
                                   u_char *jit_ptr,m_uint64_t vaddr);

/* Apply all patches */
int tc_apply_patches(cpu_tc_t *tc,void (*set_patch)(u_char *insn,u_char *dst));

/* Free the patch table */
void tc_free_patches(cpu_tc_t *tc);

/* Initialize the JIT structures of a CPU */
int cpu_jit_init(cpu_gen_t *cpu,size_t virt_hash_size,size_t phys_hash_size);

/* Shutdown the JIT structures of a CPU */
void cpu_jit_shutdown(cpu_gen_t *cpu);

/* Allocate a new Translation Block */
cpu_tb_t *tb_alloc(cpu_gen_t *cpu,m_uint64_t vaddr,u_int exec_state);

/* Free a Translation Block */
void tb_free(cpu_gen_t *cpu,cpu_tb_t *tb);

/* Enable a Tranlsation Block  */
void tb_enable(cpu_gen_t *cpu,cpu_tb_t *tb);

/* Flush all TCB of a virtual CPU */
void cpu_jit_tcb_flush_all(cpu_gen_t *cpu);

/* Mark a TB as containing self-modifying code */
void tb_mark_smc(cpu_gen_t *cpu,cpu_tb_t *tb);

/* Handle write access on an executable page */
void cpu_jit_write_on_exec_page(cpu_gen_t *cpu,
                                m_uint32_t wr_phys_page,
                                m_uint32_t wr_hp,
                                m_uint32_t ip_phys_page);

#endif
