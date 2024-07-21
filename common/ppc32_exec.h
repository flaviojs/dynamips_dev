/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __PPC32_EXEC_H__
#define __PPC32_EXEC_H__

#include "rust_dynamips_c.h"

#include "utils.h"

/* Initialize instruction lookup table */
void ppc32_exec_create_ilt(void);

/* Dump statistics */
void ppc32_dump_stats(cpu_ppc_t *cpu);

/* Execute a page */
fastcall int ppc32_exec_page(cpu_ppc_t *cpu);

/* Execute a single instruction (external) */
fastcall int ppc32_exec_single_insn_ext(cpu_ppc_t *cpu,ppc_insn_t insn);

#endif
