/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __UTILS_H__
#define __UTILS_H__

#include "dynamips_c.h"

/* Forward declarations */
typedef struct cpu_gen cpu_gen_t;
typedef struct vm_instance vm_instance_t;
typedef struct vm_platform vm_platform_t;
typedef struct mips64_jit_tcb mips64_jit_tcb_t;
typedef struct ppc32_jit_tcb ppc32_jit_tcb_t;
typedef struct jit_op jit_op_t;

/* Dynamic sprintf */
char *dyn_sprintf(const char *fmt,...);

/* Logging function */
void m_flog(FILE *fd,char *module,char *fmt,va_list ap);

/* Logging function */
void m_log(char *module,char *fmt,...);

/* Equivalent to fprintf, but for a posix fd */
ssize_t fd_printf(int fd,int flags,char *fmt,...);

#endif
