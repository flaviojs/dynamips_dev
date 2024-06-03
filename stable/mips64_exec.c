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

/* Forward declaration of instruction array */
static insn_lookup_t *ilt = NULL;

/* ILT */
static forced_inline void *mips64_exec_get_insn(int index)
{
   return(&mips64_exec_tags[index]);
}

static int mips64_exec_chk_lo(struct mips64_insn_exec_tag *tag,int value)
{
   return((value & tag->mask) == (tag->value & 0xFFFF));
}

static int mips64_exec_chk_hi(struct mips64_insn_exec_tag *tag,int value)
{
   return((value & (tag->mask >> 16)) == (tag->value >> 16));
}

/* Destroy instruction lookup table */
static void destroy_ilt(void)
{
   assert(ilt);
   ilt_destroy(ilt);
   ilt = NULL;
}

/* Initialize instruction lookup table */
void mips64_exec_create_ilt(void)
{
   int i,count;

   for(i=0,count=0;mips64_exec_tags[i].exec;i++)
      count++;

   ilt = ilt_create("mips64e",count,
                    (ilt_get_insn_cbk_t)mips64_exec_get_insn,
                    (ilt_check_cbk_t)mips64_exec_chk_lo,
                    (ilt_check_cbk_t)mips64_exec_chk_hi);

   atexit(destroy_ilt);
}

/* Dump statistics */
void mips64_dump_stats(cpu_mips_t *cpu)
{
   int i;

#if NJM_STATS_ENABLE
   printf("\n");

   for(i=0;mips64_exec_tags[i].exec;i++)
      printf("  * %-10s : %10llu\n",
             mips64_exec_tags[i].name,mips64_exec_tags[i].count);

   printf("%llu instructions executed since startup.\n",cpu->insn_exec_count);
#else
   printf("Statistics support is not compiled in.\n");
#endif
}

/* Dump an instruction */
int mips64_dump_insn(char *buffer,size_t buf_size,size_t insn_name_size,
                     m_uint64_t pc,mips_insn_t instruction)
{
   char insn_name[64],insn_format[32],*name;
   int base,rs,rd,rt,sa,offset,imm;
   struct mips64_insn_exec_tag *tag;
   m_uint64_t new_pc;
   int index;

   /* Lookup for instruction */
   index = ilt_lookup(ilt,instruction);
   tag = mips64_exec_get_insn(index);

   if (!tag) {
      snprintf(buffer,buf_size,"%8.8x  (unknown)",instruction);
      return(-1);
   }
   
   if (!(name = tag->name))
      name = "[unknown]";

   if (!insn_name_size)
      insn_name_size = 10;

   snprintf(insn_format,sizeof(insn_format),"%%-%lus",(u_long)insn_name_size);
   snprintf(insn_name,sizeof(insn_name),insn_format,name);

   switch(tag->instr_type) {
      case 1:   /* instructions without operands */
         snprintf(buffer,buf_size,"%8.8x  %s",instruction,insn_name);
         break;

      case 2:   /* load/store instructions */
         base   = bits(instruction,21,25);
         rt     = bits(instruction,16,20);
         offset = (m_int16_t)bits(instruction,0,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%d(%s)",
                  instruction,insn_name,mips64_gpr_reg_names[rt],
                  offset,mips64_gpr_reg_names[base]);
         break;

      case 3:   /* GPR[rd] = GPR[rs] op GPR[rt] */
         rs = bits(instruction,21,25);
         rt = bits(instruction,16,20);
         rd = bits(instruction,11,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s,%s",
                  instruction,insn_name,mips64_gpr_reg_names[rd],
                  mips64_gpr_reg_names[rs],mips64_gpr_reg_names[rt]);
         break;

      case 4:   /* GPR[rd] = GPR[rt] op GPR[rs] */
         rs = bits(instruction,21,25);
         rt = bits(instruction,16,20);
         rd = bits(instruction,11,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s,%s",
                  instruction,insn_name,mips64_gpr_reg_names[rd],
                  mips64_gpr_reg_names[rt],mips64_gpr_reg_names[rs]);
         break;

      case 5:   /* GPR[rt] = GPR[rs] op immediate (hex) */
         rs  = bits(instruction,21,25);
         rt  = bits(instruction,16,20);
         imm = bits(instruction,0,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s,0x%x",
                  instruction,insn_name,mips64_gpr_reg_names[rt],
                  mips64_gpr_reg_names[rs],imm);
         break;

      case 6:   /* GPR[rt] = GPR[rs] op immediate (dec) */
         rs  = bits(instruction,21,25);
         rt  = bits(instruction,16,20);
         imm = bits(instruction,0,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s,%d",
                  instruction,insn_name,mips64_gpr_reg_names[rt],
                  mips64_gpr_reg_names[rs],(m_int16_t)imm);
         break;

      case 7:   /* GPR[rd] = GPR[rt] op sa */
         rt = bits(instruction,16,20);
         rd = bits(instruction,11,15);
         sa = bits(instruction,6,10);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s,%d",
                  instruction,insn_name,mips64_gpr_reg_names[rd],
                  mips64_gpr_reg_names[rt],sa);
         break;

      case 8:   /* Branch with: GPR[rs] / GPR[rt] / offset */
         rs = bits(instruction,21,25);
         rt = bits(instruction,16,20);
         offset = bits(instruction,0,15);
         new_pc = (pc + 4) + sign_extend(offset << 2,18);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s,0x%llx",
                  instruction,insn_name,mips64_gpr_reg_names[rs],
                  mips64_gpr_reg_names[rt],new_pc);
         break;

      case 9:   /* Branch with: GPR[rs] / offset */
         rs = bits(instruction,21,25);
         offset = bits(instruction,0,15);
         new_pc = (pc + 4) + sign_extend(offset << 2,18);
         snprintf(buffer,buf_size,"%8.8x  %s %s,0x%llx",
                  instruction,insn_name,mips64_gpr_reg_names[rs],new_pc);
         break;

      case 10:   /* Branch with: offset */
         offset = bits(instruction,0,15);
         new_pc = (pc + 4) + sign_extend(offset << 2,18);
         snprintf(buffer,buf_size,"%8.8x  %s 0x%llx",
                  instruction,insn_name,new_pc);
         break;

      case 11:   /* Jump */
         offset = bits(instruction,0,25);
         new_pc = (pc & ~((1 << 28) - 1)) | (offset << 2);
         snprintf(buffer,buf_size,"%8.8x  %s 0x%llx",
                  instruction,insn_name,new_pc);
         break;

      case 13:   /* op GPR[rs] */
         rs = bits(instruction,21,25);
         snprintf(buffer,buf_size,"%8.8x  %s %s",
                  instruction,insn_name,mips64_gpr_reg_names[rs]);
         break;

      case 14:   /* op GPR[rd] */
         rd = bits(instruction,11,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s",
                  instruction,insn_name,mips64_gpr_reg_names[rd]);
         break;

      case 15:   /* op GPR[rd], GPR[rs] */
         rs = bits(instruction,21,25);
         rd = bits(instruction,11,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s",
                  instruction,insn_name,mips64_gpr_reg_names[rd],
                  mips64_gpr_reg_names[rs]);
         break;

      case 16:   /* op GPR[rt], imm */
         rt  = bits(instruction,16,20);
         imm = bits(instruction,0,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,0x%x",
                  instruction,insn_name,mips64_gpr_reg_names[rt],imm);
         break;

      case 17:   /* op GPR[rs], GPR[rt] */
         rs = bits(instruction,21,25);
         rt = bits(instruction,16,20);         
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s",
                  instruction,insn_name,mips64_gpr_reg_names[rs],
                  mips64_gpr_reg_names[rt]);
         break;

      case 18:   /* op GPR[rt], CP0[rd] */
         rt = bits(instruction,16,20);
         rd = bits(instruction,11,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,%s",
                  instruction,insn_name,mips64_gpr_reg_names[rt],
                  mips64_cp0_reg_names[rd]);
         break;

      case 19:   /* op GPR[rt], $rd */
         rt = bits(instruction,16,20);
         rd = bits(instruction,11,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,$%d",
                  instruction,insn_name,mips64_gpr_reg_names[rt],rd);
         break;

      case 20:   /* op GPR[rs], imm */
         rs = bits(instruction,21,25);
         imm = bits(instruction,0,15);
         snprintf(buffer,buf_size,"%8.8x  %s %s,0x%x",
                  instruction,insn_name,mips64_gpr_reg_names[rs],imm);
         break;

      default:
         snprintf(buffer,buf_size,"%8.8x  %s (TO DEFINE - %d)",
                  instruction,insn_name,tag->instr_type);
         return(-1);
   }
   
   return(0);
}

/* Dump an instruction block */
void mips64_dump_insn_block(cpu_mips_t *cpu,m_uint64_t pc,u_int count,
                            size_t insn_name_size)
{
   mips_insn_t *ptr,insn;
   char buffer[80];
   int i;

   for(i=0;i<count;i++) {
      ptr = cpu->mem_op_lookup(cpu,pc);
      insn = vmtoh32(*ptr);

      mips64_dump_insn(buffer,sizeof(buffer),insn_name_size,pc,insn);
      printf("0x%llx: %s\n",pc,buffer);
      pc += sizeof(mips_insn_t);
   }
}

/* Execute a memory operation */
_Unused static forced_inline void mips64_exec_memop(cpu_mips_t *cpu,int memop,
                                            m_uint64_t vaddr,u_int dst_reg,
                                            int keep_ll_bit)
{     
   fastcall mips_memop_fn fn;

   if (!keep_ll_bit) cpu->ll_bit = 0;
   fn = cpu->mem_op_fn[memop];
   fn(cpu,vaddr,dst_reg);
}

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

/* Execute a single instruction */
static forced_inline int 
mips64_exec_single_instruction(cpu_mips_t *cpu,mips_insn_t instruction)
{
   register fastcall int (*exec)(cpu_mips_t *,mips_insn_t) = NULL;
   struct mips64_insn_exec_tag *tag;
   int index;

#if DEBUG_INSN_PERF_CNT
   cpu->perf_counter++;
#endif
   
   /* Increment CP0 count register */
   mips64_exec_inc_cp0_cnt(cpu);

   /* Lookup for instruction */
   index = ilt_lookup(ilt,instruction);
   tag = mips64_exec_get_insn(index);
   exec = tag->exec;

#if NJM_STATS_ENABLE
   cpu->insn_exec_count++;
   mips64_exec_tags[index].count++;
#endif
#if 0
   {
      char buffer[80];
      
      if (mips64_dump_insn(buffer,sizeof(buffer),0,cpu->pc,instruction)!=-1)
         cpu_log(cpu->gen,"EXEC","0x%llx: %s\n",cpu->pc,buffer);
   }
#endif
   return(exec(cpu,instruction));
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
