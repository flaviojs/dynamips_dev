/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Virtual machine abstraction.
 */

#include "rust_dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/types.h>
#include <assert.h>
#include <glob.h>

#include "cpu.h"
#include "vm.h"
#include "mips64_jit.h"

#include MIPS64_ARCH_INC_FILE

/* Log a message */
void vm_flog(vm_instance_t *vm,char *module,char *format,va_list ap)
{
   if (vm->log_fd)
      m_flog(vm->log_fd,module,format,ap);
}

/* Log a message */
void vm_log(vm_instance_t *vm,char *module,char *format,...)
{ 
   va_list ap;

   if (vm->log_fd) {
      va_start(ap,format);
      vm_flog(vm,module,format,ap);
      va_end(ap);
   }
}

/* Error message */
void vm_error(vm_instance_t *vm,char *format,...)
{ 
   char buffer[2048];
   va_list ap;

   va_start(ap,format);
   vsnprintf(buffer,sizeof(buffer),format,ap);
   va_end(ap);

   fprintf(stderr,"%s '%s': %s",vm_get_log_name(vm),vm->name,buffer);
}
