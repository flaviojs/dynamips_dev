/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __MIPS_64_H__
#define __MIPS_64_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h" 

/* Register names */
extern char *mips64_gpr_reg_names[];

/* Get cacheability info */
int mips64_cca_cached(m_uint8_t val);

/* Reset a MIPS64 CPU */
int mips64_reset(cpu_mips_t *cpu);

/* Initialize a MIPS64 processor */
int mips64_init(cpu_mips_t *cpu);

/* Delete a MIPS64 processor */
void mips64_delete(cpu_mips_t *cpu);

/* Set the CPU PRID register */
void mips64_set_prid(cpu_mips_t *cpu,m_uint32_t prid);

/* Set idle PC value */
void mips64_set_idle_pc(cpu_gen_t *cpu,m_uint64_t addr);

/* Timer IRQ */
void *mips64_timer_irq_run(cpu_mips_t *cpu);

/* Determine an "idling" PC */
int mips64_get_idling_pc(cpu_gen_t *cpu);

/* Set an IRQ (VM IRQ standard routing) */
void mips64_vm_set_irq(vm_instance_t *vm,u_int irq);

/* Clear an IRQ (VM IRQ standard routing) */
void mips64_vm_clear_irq(vm_instance_t *vm,u_int irq);

/* Update the IRQ flag */
void mips64_update_irq_flag(cpu_mips_t *cpu);

/* Generate a general exception */
void mips64_general_exception(cpu_mips_t *cpu,u_int exc_code);

/* Generate a general exception that updates BadVaddr */
void mips64_gen_exception_badva(cpu_mips_t *cpu,u_int exc_code,
                                m_uint64_t bad_vaddr);

/* Generate a TLB/XTLB exception */
void mips64_tlb_miss_exception(cpu_mips_t *cpu,u_int exc_code,
                               m_uint64_t bad_vaddr);

/* Prepare a TLB exception */
void mips64_prepare_tlb_exception(cpu_mips_t *cpu,m_uint64_t vaddr);

/*
 * Increment count register and trigger the timer IRQ if value in compare 
 * register is the same.
 */
void mips64_exec_inc_cp0_cnt(cpu_mips_t *cpu);

/* Trigger the Timer IRQ */
void mips64_trigger_timer_irq(cpu_mips_t *cpu);

/* Execute ERET instruction */
void mips64_exec_eret(cpu_mips_t *cpu);

/* Execute SYSCALL instruction */
void mips64_exec_syscall(cpu_mips_t *cpu);

/* Execute BREAK instruction */
void mips64_exec_break(cpu_mips_t *cpu,u_int code);

/* Trigger a Trap Exception */
void mips64_trigger_trap_exception(cpu_mips_t *cpu);

/* Trigger IRQs */
void mips64_trigger_irq(cpu_mips_t *cpu);

/* Set an IRQ */
void mips64_set_irq(cpu_mips_t *cpu,m_uint8_t irq);

/* Clear an IRQ */
void mips64_clear_irq(cpu_mips_t *cpu,m_uint8_t irq);

/* DMFC1 */
void mips64_exec_dmfc1(cpu_mips_t *cpu,u_int gp_reg,u_int cp1_reg);

/* DMTC1 */
void mips64_exec_dmtc1(cpu_mips_t *cpu,u_int gp_reg,u_int cp1_reg);

/* MFC1 */
void mips64_exec_mfc1(cpu_mips_t *cpu,u_int gp_reg,u_int cp1_reg);

/* MTC1 */
void mips64_exec_mtc1(cpu_mips_t *cpu,u_int gp_reg,u_int cp1_reg);

/* Virtual breakpoint */
void mips64_run_breakpoint(cpu_mips_t *cpu);

/* Add a virtual breakpoint */
int mips64_add_breakpoint(cpu_gen_t *cpu,m_uint64_t pc);

/* Remove a virtual breakpoint */
void mips64_remove_breakpoint(cpu_gen_t *cpu,m_uint64_t pc);

/* Debugging for register-jump to address 0 */
void mips64_debug_jr0(cpu_mips_t *cpu);

/* Set a register */
void mips64_reg_set(cpu_gen_t *cpu,u_int reg,m_uint64_t val);

/* Dump registers of a MIPS64 processor */
void mips64_dump_regs(cpu_gen_t *cpu);

/* Dump a memory block */
void mips64_dump_memory(cpu_mips_t *cpu,m_uint64_t vaddr,u_int count);

/* Dump the stack */
void mips64_dump_stack(cpu_mips_t *cpu,u_int count);

/* Save the CPU state into a file */
int mips64_save_state(cpu_mips_t *cpu,char *filename);

/* Load a raw image into the simulated memory */
int mips64_load_raw_image(cpu_mips_t *cpu,char *filename,m_uint64_t vaddr);

/* Load an ELF image into the simulated memory */
int mips64_load_elf_image(cpu_mips_t *cpu,char *filename,int skip_load,
                          m_uint32_t *entry_point);

/* Symbol lookup */
struct symbol *mips64_sym_lookup(cpu_mips_t *cpu,m_uint64_t addr);

/* Insert a new symbol */
struct symbol *mips64_sym_insert(cpu_mips_t *cpu,char *name,m_uint64_t addr);

/* Create the symbol tree */
int mips64_sym_create_tree(cpu_mips_t *cpu);

/* Load a symbol file */
int mips64_sym_load_file(cpu_mips_t *cpu,char *filename);

#endif
