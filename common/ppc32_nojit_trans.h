/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __PPC32_NOJIT_TRANS_H__
#define __PPC32_NOJIT_TRANS_H__

#include "rust_dynamips_c.h"

#include "utils.h"
#include "x86-codegen.h"
#include "cpu.h"
#include "ppc32_exec.h"
#include "dynamips.h"

#define JIT_SUPPORT 0

/* Push epilog for an x86 instruction block */
void ppc32_jit_tcb_push_epilog(u_char **ptr);

/* Execute JIT code */
void ppc32_jit_tcb_exec(cpu_ppc_t *cpu,ppc32_jit_tcb_t *block);

#endif
