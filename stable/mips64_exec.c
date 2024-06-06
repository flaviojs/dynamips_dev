/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * MIPS64 Step-by-step execution.
 */

#if __GNUC__ > 2

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <assert.h>

#include "cpu.h"
#include "vm.h"
#include "mips64_exec.h"
#include "memory.h"
#include "rust_dynamips_c.h"
#include "dynamips.h"

/* Fetch an instruction */
static forced_inline int mips64_exec_fetch(cpu_mips_t *cpu,m_uint64_t pc,
                                           mips_insn_t *insn)
{   
   m_uint64_t exec_page;
   m_uint32_t offset;

   exec_page = pc & ~(m_uint64_t)MIPS_MIN_PAGE_IMASK;

   if (unlikely(exec_page != cpu->njm_exec_page)) {
      cpu->njm_exec_page = exec_page;
      cpu->njm_exec_ptr  = cpu->mem_op_lookup(cpu,exec_page);
   }

   offset = (pc & MIPS_MIN_PAGE_IMASK) >> 2;
   *insn = vmtoh32(cpu->njm_exec_ptr[offset]);
   return(0);
}

/* Single-step execution */
fastcall void mips64_exec_single_step(cpu_mips_t *cpu,mips_insn_t instruction)
{
   int res;

   res = mips64_exec_single_instruction(cpu,instruction);

   /* Normal flow ? */
   if (likely(!res)) cpu->pc += 4;
}

/* Run MIPS code in step-by-step mode */
void *mips64_exec_run_cpu(cpu_gen_t *gen)
{   
   cpu_mips_t *cpu = CPU_MIPS64(gen);
   pthread_t timer_irq_thread;
   int timer_irq_check = 0;
   mips_insn_t insn;
   int res;

   if (pthread_create(&timer_irq_thread,NULL,
                      (void *)mips64_timer_irq_run,cpu))
   {
      fprintf(stderr,"VM '%s': unable to create Timer IRQ thread for CPU%u.\n",
              cpu->vm->name,gen->id);
      cpu_stop(gen);
      return NULL;
   }

   gen->cpu_thread_running = TRUE;
   cpu_exec_loop_set(gen);

 start_cpu:
   gen->idle_count = 0;

   for(;;) {
      if (unlikely(gen->state != CPU_STATE_RUNNING))
         break;

      /* Handle virtual idle loop */
      if (unlikely(cpu->pc == cpu->idle_pc)) {
         if (++gen->idle_count == gen->idle_max) {
            cpu_idle_loop(gen);
            gen->idle_count = 0;
         }
      }

      /* Handle the virtual CPU clock */
      if (++timer_irq_check == cpu->timer_irq_check_itv) {
         timer_irq_check = 0;

         if (cpu->timer_irq_pending && !cpu->irq_disable) {
            mips64_trigger_timer_irq(cpu);
            mips64_trigger_irq(cpu);
            cpu->timer_irq_pending--;
         }
      }

      /* Reset "zero register" (for safety) */
      cpu->gpr[0] = 0;

      /* Check IRQ */
      if (unlikely(cpu->irq_pending)) {
         mips64_trigger_irq(cpu);
         continue;
      }

      /* Fetch and execute the instruction */      
      mips64_exec_fetch(cpu,cpu->pc,&insn);
      res = mips64_exec_single_instruction(cpu,insn);

      /* Normal flow ? */
      if (likely(!res)) cpu->pc += sizeof(mips_insn_t);
   }

   if (!cpu->pc) {
      cpu_stop(gen);
      cpu_log(gen,"SLOW_EXEC","PC=0, halting CPU.\n");
   }

   /* Check regularly if the CPU has been restarted */
   while(gen->cpu_thread_running) {
      gen->seq_state++;

      switch(gen->state) {
         case CPU_STATE_RUNNING:
            gen->state = CPU_STATE_RUNNING;
            goto start_cpu;

         case CPU_STATE_HALTED:     
            gen->cpu_thread_running = FALSE;
            pthread_join(timer_irq_thread,NULL);
            break;
      }
      
      /* CPU is paused */
      usleep(200000);
   }

   return NULL;
}

/* Execute the instruction in delay slot */
forced_inline void mips64_exec_bdslot(cpu_mips_t *cpu)
{
   mips_insn_t insn;

   /* Fetch the instruction in delay slot */
   mips64_exec_fetch(cpu,cpu->pc+4,&insn);

   /* Execute the instruction */
   mips64_exec_single_instruction(cpu,insn);
}

#endif
