/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __MIPS_64_H__
#define __MIPS_64_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h" 

/* MIPS "jr ra" instruction */
#define MIPS_INSN_JR_RA        0x03e00008

/* Addressing mode: Kernel, Supervisor and User */
#define MIPS_MODE_KERNEL  00

/* Segments in 32-bit User mode */
#define MIPS_USEG_BASE    0x00000000
#define MIPS_USEG_SIZE    0x80000000

/* Segments in 32-bit Supervisor mode */
#define MIPS_SUSEG_BASE   0x00000000
#define MIPS_SUSEG_SIZE   0x80000000
#define MIPS_SSEG_BASE    0xc0000000
#define MIPS_SSEG_SIZE    0x20000000

/* Segments in 32-bit Kernel mode */
#define MIPS_KUSEG_BASE   0x00000000
#define MIPS_KUSEG_SIZE   0x80000000

#define MIPS_KSEG0_BASE   0x80000000
#define MIPS_KSEG0_SIZE   0x20000000

#define MIPS_KSEG1_BASE   0xa0000000
#define MIPS_KSEG1_SIZE   0x20000000

#define MIPS_KSSEG_BASE   0xc0000000
#define MIPS_KSSEG_SIZE   0x20000000

#define MIPS_KSEG3_BASE   0xe0000000
#define MIPS_KSEG3_SIZE   0x20000000

/* xkphys mask (36-bit physical address) */
#define MIPS64_XKPHYS_ZONE_MASK    0xF800000000000000ULL
#define MIPS64_XKPHYS_PHYS_SIZE    (1ULL << 36)
#define MIPS64_XKPHYS_PHYS_MASK    (MIPS64_XKPHYS_PHYS_SIZE - 1)
#define MIPS64_XKPHYS_CCA_SHIFT    59

/* Initial Program Counter and Stack pointer for ROM */
#define MIPS_ROM_PC  0xffffffffbfc00000ULL
#define MIPS_ROM_SP  0xffffffff80004000ULL

/* Number of GPR (general purpose registers) */
#define MIPS64_GPR_NR  32

/* Number of registers in CP0 */
#define MIPS64_CP0_REG_NR   32

/* Number of registers in CP1 */
#define MIPS64_CP1_REG_NR   32

/* Number of instructions per page */
#define MIPS_INSN_PER_PAGE (MIPS_MIN_PAGE_SIZE/sizeof(mips_insn_t))

/* MIPS CPU Identifiers */
#define MIPS_PRID_R4600    0x00002012
#define MIPS_PRID_R4700    0x00002112
#define MIPS_PRID_R5000    0x00002312
#define MIPS_PRID_R7000    0x00002721
#define MIPS_PRID_R527x    0x00002812
#define MIPS_PRID_BCM1250  0x00040102

#define MIPS64_IRQ_LOCK(cpu)   pthread_mutex_lock(&(cpu)->irq_lock)
#define MIPS64_IRQ_UNLOCK(cpu) pthread_mutex_unlock(&(cpu)->irq_lock)

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

/* Determine an "idling" PC */
int mips64_get_idling_pc(cpu_gen_t *cpu);

/* Set an IRQ (VM IRQ standard routing) */
void mips64_vm_set_irq(vm_instance_t *vm,u_int irq);

/* Clear an IRQ (VM IRQ standard routing) */
void mips64_vm_clear_irq(vm_instance_t *vm,u_int irq);

/* Update the IRQ flag */
void mips64_update_irq_flag(cpu_mips_t *cpu);

/* Generate an exception */
void mips64_trigger_exception(cpu_mips_t *cpu,u_int exc_code,int bd_slot);

/*
 * Increment count register and trigger the timer IRQ if value in compare 
 * register is the same.
 */
fastcall void mips64_exec_inc_cp0_cnt(cpu_mips_t *cpu);

/* Trigger the Timer IRQ */
fastcall void mips64_trigger_timer_irq(cpu_mips_t *cpu);

/* Execute ERET instruction */
fastcall void mips64_exec_eret(cpu_mips_t *cpu);

/* Execute SYSCALL instruction */
fastcall void mips64_exec_syscall(cpu_mips_t *cpu);

/* Execute BREAK instruction */
fastcall void mips64_exec_break(cpu_mips_t *cpu,u_int code);

/* Trigger a Trap Exception */
fastcall void mips64_trigger_trap_exception(cpu_mips_t *cpu);

/* Trigger IRQs */
fastcall void mips64_trigger_irq(cpu_mips_t *cpu);

/* Set an IRQ */
void mips64_set_irq(cpu_mips_t *cpu,m_uint8_t irq);

/* Clear an IRQ */
void mips64_clear_irq(cpu_mips_t *cpu,m_uint8_t irq);

/* DMFC1 */
fastcall void mips64_exec_dmfc1(cpu_mips_t *cpu,u_int gp_reg,u_int cp1_reg);

/* DMTC1 */
fastcall void mips64_exec_dmtc1(cpu_mips_t *cpu,u_int gp_reg,u_int cp1_reg);

/* MFC1 */
fastcall void mips64_exec_mfc1(cpu_mips_t *cpu,u_int gp_reg,u_int cp1_reg);

/* MTC1 */
fastcall void mips64_exec_mtc1(cpu_mips_t *cpu,u_int gp_reg,u_int cp1_reg);

/* Virtual breakpoint */
fastcall void mips64_run_breakpoint(cpu_mips_t *cpu);

/* Add a virtual breakpoint */
int mips64_add_breakpoint(cpu_gen_t *cpu,m_uint64_t pc);

/* Remove a virtual breakpoint */
void mips64_remove_breakpoint(cpu_gen_t *cpu,m_uint64_t pc);

/* Debugging for register-jump to address 0 */
fastcall void mips64_debug_jr0(cpu_mips_t *cpu);

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
