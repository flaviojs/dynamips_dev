/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __CPU_H__
#define __CPU_H__

#include "rust_dynamips_c.h"

#include <pthread.h>
#include <setjmp.h>
#include "utils.h"
#include "jit_op.h"

#include "mips64.h"
#include "mips64_cp0.h"
#include "ppc32.h"

/* Find a CPU in a group given its ID */
cpu_gen_t *cpu_group_find_id(cpu_group_t *group,u_int id);

/* Find the highest CPU ID in a CPU group */
int cpu_group_find_highest_id(cpu_group_t *group,u_int *highest_id);

/* Add a CPU in a CPU group */
int cpu_group_add(cpu_group_t *group,cpu_gen_t *cpu);

/* Create a new CPU group */
cpu_group_t *cpu_group_create(char *name);

/* Delete a CPU group */
void cpu_group_delete(cpu_group_t *group);

/* Rebuild the MTS subsystem for a CPU group */
int cpu_group_rebuild_mts(cpu_group_t *group);

/* Log a message for a CPU */
void cpu_log(cpu_gen_t *cpu,char *module,char *format,...);

/* Create a new CPU */
cpu_gen_t *cpu_create(vm_instance_t *vm,u_int type,u_int id);

/* Delete a CPU */
void cpu_delete(cpu_gen_t *cpu);

/* Start a CPU */
void cpu_start(cpu_gen_t *cpu);

/* Stop a CPU */
void cpu_stop(cpu_gen_t *cpu);

/* Start all CPUs of a CPU group */
void cpu_group_start_all_cpu(cpu_group_t *group);

/* Stop all CPUs of a CPU group */
void cpu_group_stop_all_cpu(cpu_group_t *group);

/* Set a state of all CPUs of a CPU group */
void cpu_group_set_state(cpu_group_t *group,u_int state);

/* Synchronize on CPUs (all CPUs must be inactive) */
int cpu_group_sync_state(cpu_group_t *group);

/* Save state of all CPUs */
int cpu_group_save_state(cpu_group_t *group);

/* Restore state of all CPUs */
int cpu_group_restore_state(cpu_group_t *group);

/* Virtual idle loop */
void cpu_idle_loop(cpu_gen_t *cpu);

/* Break idle wait state */
void cpu_idle_break_wait(cpu_gen_t *cpu);

#endif
