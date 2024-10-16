/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#include "dynamips_c.h"

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

#include "cpu.h"
#include "jit_op.h"
#include "ppc32_jit.h"
#include "ppc32_amd64_trans.h"
#include "memory.h"

/* Macros for CPU structure access */
#define REG_OFFSET(reg)   (OFFSET(cpu_ppc_t,gpr[(reg)]))
#define MEMOP_OFFSET(op)  (OFFSET(cpu_ppc_t,mem_op_fn[(op)]))

#define DECLARE_INSN(name) \
   static int ppc32_emit_##name(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b, \
                                ppc_insn_t insn)

/* EFLAGS to Condition Register (CR) field - signed */
static m_uint32_t eflags_to_cr_signed[64] = {
   0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 
   0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 
   0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 
   0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 
   0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 
   0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 
   0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 
   0x08, 0x02, 0x04, 0x02, 0x08, 0x02, 0x04, 0x02, 
};

/* EFLAGS to Condition Register (CR) field - unsigned */
static m_uint32_t eflags_to_cr_unsigned[256] = {
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 0x04, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
   0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 0x02, 0x08, 
};

/* Load a 32 bit immediate value */
static inline void ppc32_load_imm(u_char **ptr,u_int reg,m_uint32_t val)
{
   if (val)
      amd64_mov_reg_imm_size(*ptr,reg,val,4);
   else
      amd64_alu_reg_reg_size(*ptr,X86_XOR,reg,reg,4);
}

/* Set the Instruction Address (IA) register */
void ppc32_set_ia(u_char **ptr,m_uint32_t new_ia)
{
   amd64_mov_membase_imm(*ptr,AMD64_R15,OFFSET(cpu_ppc_t,ia),new_ia,4);
}

/* Set the Link Register (LR) */
static void ppc32_set_lr(jit_op_t *iop,m_uint32_t new_lr)
{  
   amd64_mov_membase_imm(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,lr),new_lr,4);
}

/* 
 * Try to branch directly to the specified JIT block without returning to 
 * main loop.
 */
static void ppc32_try_direct_far_jump(cpu_ppc_t *cpu,jit_op_t *iop,
                                      m_uint32_t new_ia)
{
   m_uint32_t new_page,ia_hash,ia_offset;
   u_char *test1,*test2,*test3,*test4;

   /* Indicate that we throw %rbx, %rdx */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RBX);
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RSI);

   new_page = new_ia & PPC32_MIN_PAGE_MASK;
   ia_offset = (new_ia & PPC32_MIN_PAGE_IMASK) >> 2;
   ia_hash = ppc32_jit_get_virt_hash(new_ia);
   
   /* Get JIT block info in %rdx */
   amd64_mov_reg_membase(iop->ob_ptr,AMD64_RBX,
                         AMD64_R15,OFFSET(cpu_ppc_t,tcb_virt_hash),8);
   amd64_mov_reg_membase(iop->ob_ptr,AMD64_RDX,
                         AMD64_RBX,ia_hash*sizeof(void *),8);

   /* no JIT block found ? */
   amd64_test_reg_reg(iop->ob_ptr,AMD64_RDX,AMD64_RDX);
   test1 = iop->ob_ptr;
   amd64_branch8(iop->ob_ptr, X86_CC_Z, 0, 1);

   /* Check block IA */
   ppc32_load_imm(&iop->ob_ptr,AMD64_RSI,new_page);
   amd64_alu_reg_membase_size(iop->ob_ptr,X86_CMP,AMD64_RAX,AMD64_RDX,
                              OFFSET(ppc32_jit_tcb_t,start_ia),4);
   test2 = iop->ob_ptr;
   amd64_branch8(iop->ob_ptr, X86_CC_NE, 0, 1);

   /* Jump to the code */
   amd64_mov_reg_membase(iop->ob_ptr,AMD64_RSI,
                         AMD64_RDX,OFFSET(ppc32_jit_tcb_t,jit_insn_ptr),8);

   amd64_test_reg_reg(iop->ob_ptr,AMD64_RSI,AMD64_RSI);
   test3 = iop->ob_ptr;
   amd64_branch8(iop->ob_ptr, X86_CC_Z, 0, 1);

   amd64_mov_reg_membase(iop->ob_ptr,AMD64_RBX,
                         AMD64_RSI,ia_offset * sizeof(void *),8);
   
   amd64_test_reg_reg(iop->ob_ptr,AMD64_RBX,AMD64_RBX);
   test4 = iop->ob_ptr;
   amd64_branch8(iop->ob_ptr, X86_CC_Z, 0, 1);
   amd64_jump_reg(iop->ob_ptr,AMD64_RBX);

   /* Returns to caller... */
   amd64_patch(test1,iop->ob_ptr);
   amd64_patch(test2,iop->ob_ptr);
   amd64_patch(test3,iop->ob_ptr);
   amd64_patch(test4,iop->ob_ptr);

   ppc32_set_ia(&iop->ob_ptr,new_ia);
   ppc32_jit_tcb_push_epilog(&iop->ob_ptr);
}

/* Set Jump */
static void ppc32_set_jump(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b,jit_op_t *iop,
                           m_uint32_t new_ia,int local_jump)
{      
   int return_to_caller = FALSE;
   u_char *jump_ptr;

#if 0
   if (cpu->sym_trace && !local_jump)
      return_to_caller = TRUE;
#endif

   if (!return_to_caller && ppc32_jit_tcb_local_addr(b,new_ia,&jump_ptr)) {
      ppc32_jit_tcb_record_patch(b,iop,iop->ob_ptr,new_ia);
      amd64_jump32(iop->ob_ptr,0);
   } else {   
      if (cpu->exec_blk_direct_jump) {
         /* Block lookup optimization */
         ppc32_try_direct_far_jump(cpu,iop,new_ia);
      } else {
         ppc32_set_ia(&iop->ob_ptr,new_ia);
         ppc32_jit_tcb_push_epilog(&iop->ob_ptr);
      }
   }
}

/* Jump to the next page */
void ppc32_set_page_jump(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b)
{
   jit_op_t *iop,*op_list = NULL;

   cpu->gen->jit_op_current = &op_list;

   iop = ppc32_op_emit_insn_output(cpu,4,"set_page_jump");
   ppc32_set_jump(cpu,b,iop,b->start_ia + PPC32_MIN_PAGE_SIZE,FALSE);
   ppc32_op_insn_output(b,iop);

   jit_op_free_list(cpu->gen,op_list);
   cpu->gen->jit_op_current = NULL;
}

/* Load a GPR into the specified host register */
static forced_inline void ppc32_load_gpr(u_char **ptr,u_int host_reg,
                                         u_int ppc_reg)
{
   amd64_mov_reg_membase(*ptr,host_reg,AMD64_R15,REG_OFFSET(ppc_reg),4);
}

/* Store contents for a host register into a GPR register */
static forced_inline void ppc32_store_gpr(u_char **ptr,u_int ppc_reg,
                                          u_int host_reg)
{
   amd64_mov_membase_reg(*ptr,AMD64_R15,REG_OFFSET(ppc_reg),host_reg,4);
}

/* Apply an ALU operation on a GPR register and a host register */
static forced_inline void ppc32_alu_gpr(u_char **ptr,u_int op,
                                        u_int host_reg,u_int ppc_reg)
{
   amd64_alu_reg_membase_size(*ptr,op,host_reg,
                              AMD64_R15,REG_OFFSET(ppc_reg),4);
}

/* 
 * Update CR from %eflags
 * %rax, %rdx, %rsi are modified.
 */
static void ppc32_update_cr(ppc32_jit_tcb_t *b,int field,int is_signed)
{
   /* Get status bits from EFLAGS */
   amd64_pushfd_size(b->jit_ptr,8);
   amd64_pop_reg(b->jit_ptr,AMD64_RAX);

   if (!is_signed) {
      amd64_alu_reg_imm(b->jit_ptr,X86_AND,AMD64_RAX,0xFF);
      amd64_mov_reg_imm_size(b->jit_ptr,AMD64_RDX,eflags_to_cr_unsigned,8);
   } else {
      amd64_shift_reg_imm(b->jit_ptr,X86_SHR,AMD64_RAX,6);
      amd64_alu_reg_imm(b->jit_ptr,X86_AND,AMD64_RAX,0x3F);
      amd64_mov_reg_imm_size(b->jit_ptr,AMD64_RDX,eflags_to_cr_signed,8);
   }

   amd64_mov_reg_memindex(b->jit_ptr,AMD64_RAX,AMD64_RDX,0,AMD64_RAX,2,4);

#if 0
   /* Check XER Summary of Overflow and report it */
   amd64_mov_reg_membase(b->jit_ptr,AMD64_RCX,
                         AMD64_R15,OFFSET(cpu_ppc_t,xer),4);
   amd64_alu_reg_imm(b->jit_ptr,X86_AND,AMD64_RCX,PPC32_XER_SO);
   amd64_shift_reg_imm(b->jit_ptr,X86_SHR,AMD64_RCX,(field << 2) + 3);
   amd64_alu_reg_reg(b->jit_ptr,X86_OR,AMD64_RDX,AMD64_RCX);
#endif

   /* Store modified CR field */
   amd64_mov_membase_reg(b->jit_ptr,AMD64_R15,PPC32_CR_FIELD_OFFSET(field),
                         AMD64_RAX,4);
}

/* 
 * Update CR0 from %eflags
 * %eax, %ecx, %edx, %esi are modified.
 */
static void ppc32_update_cr0(ppc32_jit_tcb_t *b)
{
   ppc32_update_cr(b,0,TRUE);
}

/* Indicate registers modified by ppc32_update_cr() functions */
void ppc32_update_cr_set_altered_hreg(cpu_ppc_t *cpu)
{
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RAX);
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);
}

/* Basic C call */
static forced_inline void ppc32_emit_basic_c_call(u_char **ptr,void *f)
{
   amd64_mov_reg_imm(*ptr,AMD64_RBX,f);
   amd64_call_reg(*ptr,AMD64_RBX);
}

/* Emit a simple call to a C function without any parameter */
static void ppc32_emit_c_call(ppc32_jit_tcb_t *b,jit_op_t *iop,void *f)
{   
   ppc32_set_ia(&iop->ob_ptr,b->start_ia+(b->ppc_trans_pos << 2));
   ppc32_emit_basic_c_call(&iop->ob_ptr,f);
}

/* ======================================================================== */

/* Initialize register mapping */
void ppc32_jit_init_hreg_mapping(cpu_ppc_t *cpu)
{
   int avail_hregs[] = { AMD64_RSI, AMD64_RAX, AMD64_RCX, AMD64_RDX, 
                         AMD64_R13, AMD64_R14, AMD64_RDI, -1 };
   struct hreg_map *map;
   int i,hreg;

   cpu->hreg_map_list = cpu->hreg_lru = NULL;

   /* Add the available registers to the map list */
   for(i=0;avail_hregs[i]!=-1;i++) {
      hreg = avail_hregs[i];
      map = &cpu->hreg_map[hreg];

      /* Initialize mapping. At the beginning, no PPC reg is mapped */
      map->flags = 0;
      map->hreg  = hreg;
      map->vreg  = -1;
      ppc32_jit_insert_hreg_mru(cpu,map);
   }

   /* Clear PPC registers mapping */
   for(i=0;i<PPC32_GPR_NR;i++)
      cpu->ppc_reg_map[i] = -1;
}

/* Allocate a specific temp register */
static int ppc32_jit_get_tmp_hreg(cpu_ppc_t *cpu)
{
   return(AMD64_RBX);
}

/* ======================================================================== */
/* JIT operations (specific to target CPU).                                 */
/* ======================================================================== */

/* INSN_OUTPUT */
void ppc32_op_insn_output(ppc32_jit_tcb_t *b,jit_op_t *op)
{
   op->ob_final = b->jit_ptr;
   memcpy(b->jit_ptr,op->ob_data,op->ob_ptr - op->ob_data);
   b->jit_ptr += op->ob_ptr - op->ob_data;
}

/* LOAD_GPR: p[0] = %host_reg, p[1] = %ppc_reg */
void ppc32_op_load_gpr(ppc32_jit_tcb_t *b,jit_op_t *op)
{
   if (op->param[0] != JIT_OP_INV_REG)
      ppc32_load_gpr(&b->jit_ptr,op->param[0],op->param[1]);
}

/* STORE_GPR: p[0] = %host_reg, p[1] = %ppc_reg */
void ppc32_op_store_gpr(ppc32_jit_tcb_t *b,jit_op_t *op)
{
   if (op->param[0] != JIT_OP_INV_REG)
      ppc32_store_gpr(&b->jit_ptr,op->param[1],op->param[0]);
}

/* UPDATE_FLAGS: p[0] = cr_field, p[1] = is_signed */
void ppc32_op_update_flags(ppc32_jit_tcb_t *b,jit_op_t *op)
{
   if (op->param[0] != JIT_OP_INV_REG)
      ppc32_update_cr(b,op->param[0],op->param[1]);
}

/* MOVE_HOST_REG: p[0] = %host_dst_reg, p[1] = %host_src_reg */
void ppc32_op_move_host_reg(ppc32_jit_tcb_t *b,jit_op_t *op)
{
   if ((op->param[0] != JIT_OP_INV_REG) && (op->param[1] != JIT_OP_INV_REG))
      amd64_mov_reg_reg(b->jit_ptr,op->param[0],op->param[1],4);
}

/* SET_HOST_REG_IMM32: p[0] = %host_reg, p[1] = imm32 */
void ppc32_op_set_host_reg_imm32(ppc32_jit_tcb_t *b,jit_op_t *op)
{
   if (op->param[0] != JIT_OP_INV_REG)
      ppc32_load_imm(&b->jit_ptr,op->param[0],op->param[1]);
}

/* ======================================================================== */

/* Memory operation */
static void ppc32_emit_memop(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b,
                             int op,int base,int offset,int target,int update)
{
   m_uint32_t val = sign_extend(offset,16);
   jit_op_t *iop;

   /* 
    * Since an exception can be triggered, clear JIT state. This allows
    * to use branch target tag (we can directly branch on this instruction).
    */
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_BRANCH_TARGET);
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);

   iop = ppc32_op_emit_insn_output(cpu,5,"memop");

   /* Save PC for exception handling */
   ppc32_set_ia(&iop->ob_ptr,b->start_ia+(b->ppc_trans_pos << 2));

   /* RSI = sign-extended offset */
   ppc32_load_imm(&iop->ob_ptr,AMD64_RSI,val);

   /* RSI = GPR[base] + sign-extended offset */
   if (update || (base != 0))
      ppc32_alu_gpr(&iop->ob_ptr,X86_ADD,AMD64_RSI,base);

   if (update)
      amd64_mov_reg_reg(iop->ob_ptr,AMD64_R14,AMD64_RSI,4);

   /* RDX = target register */
   amd64_mov_reg_imm(iop->ob_ptr,AMD64_RDX,target);
   
   /* RDI = CPU instance pointer */
   amd64_mov_reg_reg(iop->ob_ptr,AMD64_RDI,AMD64_R15,8);

   /* Call memory function */
   amd64_alu_reg_imm(iop->ob_ptr,X86_SUB,AMD64_RSP,8);
   amd64_call_membase(iop->ob_ptr,AMD64_R15,MEMOP_OFFSET(op));
   amd64_alu_reg_imm(iop->ob_ptr,X86_ADD,AMD64_RSP,8);

   if (update)
      ppc32_store_gpr(&iop->ob_ptr,base,AMD64_R14);
}

/* Memory operation (indexed) */
static void ppc32_emit_memop_idx(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b,
                                 int op,int ra,int rb,int target,int update)
{
   jit_op_t *iop;

   /* 
    * Since an exception can be triggered, clear JIT state. This allows
    * to use branch target tag (we can directly branch on this instruction).
    */
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_BRANCH_TARGET);
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);

   iop = ppc32_op_emit_insn_output(cpu,5,"memop_idx");

   /* Save PC for exception handling */
   ppc32_set_ia(&iop->ob_ptr,b->start_ia+(b->ppc_trans_pos << 2));

   /* RSI = $rb */
   ppc32_load_gpr(&iop->ob_ptr,AMD64_RSI,rb);

   /* RSI = GPR[base] + sign-extended offset */
   if (update || (ra != 0))
      ppc32_alu_gpr(&iop->ob_ptr,X86_ADD,AMD64_RSI,ra);

   if (update)
      amd64_mov_reg_reg(iop->ob_ptr,AMD64_R14,AMD64_RSI,4);

   /* RDX = target register */
   amd64_mov_reg_imm(iop->ob_ptr,AMD64_RDX,target);
   
   /* RDI = CPU instance pointer */
   amd64_mov_reg_reg(iop->ob_ptr,AMD64_RDI,AMD64_R15,8);

   /* Call memory function */
   amd64_alu_reg_imm(iop->ob_ptr,X86_SUB,AMD64_RSP,8);
   amd64_call_membase(iop->ob_ptr,AMD64_R15,MEMOP_OFFSET(op));
   amd64_alu_reg_imm(iop->ob_ptr,X86_ADD,AMD64_RSP,8);

   if (update)
      ppc32_store_gpr(&iop->ob_ptr,ra,AMD64_R14);
}

typedef void (*memop_fast_access)(jit_op_t *iop,int target);

/* Fast LBZ */
static void ppc32_memop_fast_lbz(jit_op_t *iop,int target)
{
   amd64_clear_reg(iop->ob_ptr,AMD64_RCX);
   amd64_mov_reg_memindex(iop->ob_ptr,AMD64_RCX,AMD64_RBX,0,AMD64_RSI,0,1);
   ppc32_store_gpr(&iop->ob_ptr,target,AMD64_RCX);
}

/* Fast STB */
static void ppc32_memop_fast_stb(jit_op_t *iop,int target)
{
   ppc32_load_gpr(&iop->ob_ptr,AMD64_RDX,target);
   amd64_mov_memindex_reg(iop->ob_ptr,AMD64_RBX,0,AMD64_RSI,0,AMD64_RDX,1);
}

/* Fast LWZ */
static void ppc32_memop_fast_lwz(jit_op_t *iop,int target)
{
   amd64_mov_reg_memindex(iop->ob_ptr,AMD64_RAX,AMD64_RBX,0,AMD64_RSI,0,4);
   amd64_bswap32(iop->ob_ptr,AMD64_RAX);
   ppc32_store_gpr(&iop->ob_ptr,target,AMD64_RAX);
}

/* Fast STW */
static void ppc32_memop_fast_stw(jit_op_t *iop,int target)
{
   ppc32_load_gpr(&iop->ob_ptr,AMD64_RDX,target);
   amd64_bswap32(iop->ob_ptr,AMD64_RDX);
   amd64_mov_memindex_reg(iop->ob_ptr,AMD64_RBX,0,AMD64_RSI,0,AMD64_RDX,4);
}

/* Fast memory operation */
static void ppc32_emit_memop_fast(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b,
                                  int write_op,int opcode,
                                  int base,int offset,int target,
                                  memop_fast_access op_handler)
{   
   m_uint32_t val = sign_extend(offset,16);
   u_char *test1,*test2,*p_exit;
   jit_op_t *iop;

   /* 
    * Since an exception can be triggered, clear JIT state. This allows
    * to use branch target tag (we can directly branch on this instruction).
    */
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_BRANCH_TARGET);
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);

   iop = ppc32_op_emit_insn_output(cpu,5,"memop_fast");

   test2 = NULL;

   /* XXX */
   amd64_inc_membase(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,mts_lookups));

   /* RSI = GPR[base] + sign-extended offset */
   ppc32_load_imm(&iop->ob_ptr,AMD64_RSI,val);
   if (base != 0)
      ppc32_alu_gpr(&iop->ob_ptr,X86_ADD,AMD64_RSI,base);

   /* RBX = mts32_entry index */
   amd64_mov_reg_reg_size(iop->ob_ptr,X86_EBX,X86_ESI,4);
   amd64_mov_reg_reg_size(iop->ob_ptr,X86_EAX,X86_ESI,4);

   amd64_shift_reg_imm_size(iop->ob_ptr,X86_SHR,X86_EBX,MTS32_HASH_SHIFT1,4);
   amd64_shift_reg_imm_size(iop->ob_ptr,X86_SHR,X86_EAX,MTS32_HASH_SHIFT2,4);
   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,AMD64_RBX,AMD64_RAX);

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_AND,X86_EBX,MTS32_HASH_MASK,4);

   /* RCX = mts32 entry */
   amd64_mov_reg_membase(iop->ob_ptr,AMD64_RCX,
                         AMD64_R15,
                         OFFSET(cpu_ppc_t,mts_cache[PPC32_MTS_DCACHE]),8);
   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,AMD64_RBX,5);  /* TO FIX */
   amd64_alu_reg_reg(iop->ob_ptr,X86_ADD,AMD64_RCX,AMD64_RBX);

   /* Compare virtual page address (EAX = vpage) */
   amd64_mov_reg_reg(iop->ob_ptr,X86_EAX,X86_ESI,4);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,X86_EAX,PPC32_MIN_PAGE_MASK);

   amd64_alu_reg_membase_size(iop->ob_ptr,X86_CMP,X86_EAX,AMD64_RCX,
                              OFFSET(mts32_entry_t,gvpa),4);
   test1 = iop->ob_ptr;
   amd64_branch8(iop->ob_ptr, X86_CC_NZ, 0, 1);

   /* Test if we are writing to a COW page */
   if (write_op) {
      amd64_test_membase_imm_size(iop->ob_ptr,
                                  AMD64_RCX,OFFSET(mts32_entry_t,flags),
                                  MTS_FLAG_COW|MTS_FLAG_EXEC,4);
      test2 = iop->ob_ptr;
      amd64_branch8(iop->ob_ptr, X86_CC_NZ, 0, 1);
   }

   /* ESI = offset in page, RBX = Host Page Address */
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,X86_ESI,PPC32_MIN_PAGE_IMASK);
   amd64_mov_reg_membase(iop->ob_ptr,AMD64_RBX,
                         AMD64_RCX,OFFSET(mts32_entry_t,hpa),8);

   /* Memory access */
   op_handler(iop,target);

   p_exit = iop->ob_ptr;
   amd64_jump8(iop->ob_ptr,0);

   /* === Slow lookup === */
   amd64_patch(test1,iop->ob_ptr);
   if (test2)
      amd64_patch(test2,iop->ob_ptr);

   /* Save IA for exception handling */
   ppc32_set_ia(&iop->ob_ptr,b->start_ia+(b->ppc_trans_pos << 2));

   /* RDX = target register */
   amd64_mov_reg_imm(iop->ob_ptr,AMD64_RDX,target);

   /* RDI = CPU instance */
   amd64_mov_reg_reg(iop->ob_ptr,AMD64_RDI,AMD64_R15,8);

   /* Call memory access function */
   amd64_alu_reg_imm(iop->ob_ptr,X86_SUB,AMD64_RSP,8);
   amd64_call_membase(iop->ob_ptr,AMD64_R15,MEMOP_OFFSET(opcode));
   amd64_alu_reg_imm(iop->ob_ptr,X86_ADD,AMD64_RSP,8);

   amd64_patch(p_exit,iop->ob_ptr);
}

/* Emit unhandled instruction code */
static int ppc32_emit_unknown(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b,
                              ppc_insn_t opcode)
{
   u_char *test1;
   jit_op_t *iop;

   iop = ppc32_op_emit_insn_output(cpu,3,"unknown");

   /* Update IA */
   ppc32_set_ia(&iop->ob_ptr,b->start_ia+(b->ppc_trans_pos << 2));

   /* Fallback to non-JIT mode */
   amd64_mov_reg_reg(iop->ob_ptr,AMD64_RDI,AMD64_R15,8);
   amd64_mov_reg_imm(iop->ob_ptr,AMD64_RSI,opcode);

   amd64_alu_reg_imm(iop->ob_ptr,X86_SUB,AMD64_RSP,8);
   ppc32_emit_c_call(b,iop,ppc32_exec_single_insn_ext);
   amd64_alu_reg_imm(iop->ob_ptr,X86_ADD,AMD64_RSP,8);

   amd64_test_reg_reg_size(iop->ob_ptr,AMD64_RAX,AMD64_RAX,4);
   test1 = iop->ob_ptr;
   amd64_branch8(iop->ob_ptr, X86_CC_Z, 0, 1);
   ppc32_jit_tcb_push_epilog(&iop->ob_ptr);

   amd64_patch(test1,iop->ob_ptr);

   /* Signal this as an EOB to reset JIT state */
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);
   return(0);
}

/* Virtual Breakpoint */
void ppc32_emit_breakpoint(cpu_ppc_t *cpu,ppc32_jit_tcb_t *b)
{
   jit_op_t *iop;

   iop = ppc32_op_emit_insn_output(cpu,2,"breakpoint");

   amd64_mov_reg_reg(iop->ob_ptr,AMD64_RDI,AMD64_R15,8);

   amd64_alu_reg_imm(iop->ob_ptr,X86_SUB,AMD64_RSP,8);
   ppc32_emit_c_call(b,iop,ppc32_run_breakpoint);
   amd64_alu_reg_imm(iop->ob_ptr,X86_ADD,AMD64_RSP,8);

   /* Signal this as an EOB to to reset JIT state */
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);
}

/* Increment the number of executed instructions (performance debugging) */
void ppc32_inc_perf_counter(cpu_ppc_t *cpu)
{    
   jit_op_t *iop;
   
   iop = ppc32_op_emit_insn_output(cpu,1,"perf_cnt");
   amd64_inc_membase_size(iop->ob_ptr,
                          AMD64_R15,OFFSET(cpu_ppc_t,perf_counter),4);
}

/* ======================================================================== */

/* BLR - Branch to Link Register */
DECLARE_INSN(BLR)
{
   jit_op_t *iop;
   int hreg;

   ppc32_jit_start_hreg_seq(cpu,"blr");
   hreg = ppc32_jit_alloc_hreg(cpu,-1);
   ppc32_op_emit_alter_host_reg(cpu,hreg);

   iop = ppc32_op_emit_insn_output(cpu,2,"blr");

   amd64_mov_reg_membase(iop->ob_ptr,hreg,AMD64_R15,OFFSET(cpu_ppc_t,lr),4);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,ia),hreg,4);

   /* set the return address */
   if (insn & 1)
      ppc32_set_lr(iop,b->start_ia + ((b->ppc_trans_pos+1) << 2));

   ppc32_jit_tcb_push_epilog(&iop->ob_ptr);
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);
   ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1) << 2));

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* BCTR - Branch to Count Register */
DECLARE_INSN(BCTR)
{
   jit_op_t *iop;
   int hreg;

   ppc32_jit_start_hreg_seq(cpu,"bctr");
   hreg = ppc32_jit_alloc_hreg(cpu,-1);
   ppc32_op_emit_alter_host_reg(cpu,hreg);

   iop = ppc32_op_emit_insn_output(cpu,2,"bctr");

   amd64_mov_reg_membase(iop->ob_ptr,hreg,AMD64_R15,OFFSET(cpu_ppc_t,ctr),4);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,ia),hreg,4);

   /* set the return address */
   if (insn & 1)
      ppc32_set_lr(iop,b->start_ia + ((b->ppc_trans_pos+1) << 2));

   ppc32_jit_tcb_push_epilog(&iop->ob_ptr);
   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);
   ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1) << 2));

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MFLR - Move From Link Register */
DECLARE_INSN(MFLR)
{
   int rd = bits(insn,21,25);
   int hreg_rd;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mflr");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);
   iop = ppc32_op_emit_insn_output(cpu,1,"mflr");

   amd64_mov_reg_membase(iop->ob_ptr,hreg_rd,AMD64_R15,OFFSET(cpu_ppc_t,lr),4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MTLR - Move To Link Register */
DECLARE_INSN(MTLR)
{
   int rs = bits(insn,21,25);
   int hreg_rs;
   jit_op_t *iop;
   
   ppc32_jit_start_hreg_seq(cpu,"mtlr");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,1,"mtlr");
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,lr),hreg_rs,4);
   return(0);
}

/* MFCTR - Move From Counter Register */
DECLARE_INSN(MFCTR)
{
   int rd = bits(insn,21,25);
   int hreg_rd;
   jit_op_t *iop;
   
   ppc32_jit_start_hreg_seq(cpu,"mfctr");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   iop = ppc32_op_emit_insn_output(cpu,1,"mfctr");

   amd64_mov_reg_membase(iop->ob_ptr,hreg_rd,
                         AMD64_R15,OFFSET(cpu_ppc_t,ctr),4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MTCTR - Move To Counter Register */
DECLARE_INSN(MTCTR)
{
   int rs = bits(insn,21,25);
   int hreg_rs;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mtctr");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,1,"mtctr");

   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,ctr),
                         hreg_rs,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MFTBU - Move from Time Base (Up) */
DECLARE_INSN(MFTBU)
{
   int rd = bits(insn,21,25);
   int hreg_rd;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mftbu");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   iop = ppc32_op_emit_insn_output(cpu,1,"mftbu");

   amd64_mov_reg_membase(iop->ob_ptr,hreg_rd,
                         AMD64_R15,OFFSET(cpu_ppc_t,tb)+4,4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

#define PPC32_TB_INCREMENT  50

/* MFTBL - Move from Time Base (Lo) */
DECLARE_INSN(MFTBL)
{
   int rd = bits(insn,21,25);
   int hreg_rd;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mftbl");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   iop = ppc32_op_emit_insn_output(cpu,3,"mftbl");

   amd64_mov_reg_membase(iop->ob_ptr,hreg_rd,
                         AMD64_R15,OFFSET(cpu_ppc_t,tb),8);
   amd64_alu_reg_imm(iop->ob_ptr,X86_ADD,hreg_rd,PPC32_TB_INCREMENT);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,tb),
                         hreg_rd,8);

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* ADD */
DECLARE_INSN(ADD)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rd,hreg_ra,hreg_rb;
   jit_op_t *iop;

   /* $rd = $ra + $rb */
   ppc32_jit_start_hreg_seq(cpu,"add");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,2,"add");

   if (rd == ra)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_rd,hreg_rb,4);
   else if (rd == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_rd,hreg_ra,4);
   else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_ra,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_rd,hreg_rb,4);
   }

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);
   
   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* ADDC */
DECLARE_INSN(ADDC)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rd,hreg_ra,hreg_rb,hreg_t0;
   jit_op_t *iop;

   /* $rd = $ra + $rb */
   ppc32_jit_start_hreg_seq(cpu,"addc");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   /* store the carry flag */
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,2,"addc");

   if (rd == ra)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_rd,hreg_rb,4);
   else if (rd == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_rd,hreg_ra,4);
   else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_ra,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_rd,hreg_rb,4);
   }

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   /* store the carry flag */
   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t0,FALSE);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x1);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                         hreg_t0,4);

   if (insn & 1) {
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_rd,hreg_rd,4);
      ppc32_op_emit_update_flags(cpu,0,TRUE);
   }

   ppc32_jit_close_hreg_seq(cpu);      
   return(0);
}

/* ADDE - Add Extended */
DECLARE_INSN(ADDE)
{   
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_ra,hreg_rb,hreg_rd,hreg_t0,hreg_t1;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"adde");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   hreg_t0 = ppc32_jit_alloc_hreg(cpu,-1);
   hreg_t1 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_alter_host_reg(cpu,hreg_t0);
   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,3,"adde");

   /* $t0 = $ra + carry */
   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,hreg_t1,hreg_t1);
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_ra,4);

   amd64_alu_reg_membase_size(iop->ob_ptr,X86_ADD,hreg_t0,
                              AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),4);
   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t1,FALSE);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                         hreg_t1,4);

   /* $t0 += $rb */
   amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_t0,hreg_rb,4);
   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t1,FALSE);
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                              hreg_t1,4);

   /* update cr0 */
   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_t0,hreg_t0,4);

   amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_t0,4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* ADDI - ADD Immediate */
DECLARE_INSN(ADDI)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int imm = bits(insn,0,15);
   m_uint32_t tmp = sign_extend_32(imm,16);
   int hreg_rd,hreg_ra;
   jit_op_t *iop;

   /* $rd = $ra + imm */
   ppc32_jit_start_hreg_seq(cpu,"addi");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   if (ra != 0) {
      hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
      ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

      iop = ppc32_op_emit_insn_output(cpu,2,"addi");

      if (rd != ra)
         amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_ra,4);

      amd64_alu_reg_imm_size(iop->ob_ptr,X86_ADD,hreg_rd,tmp,4);
   } else {
      iop = ppc32_op_emit_insn_output(cpu,1,"addi");
      ppc32_load_imm(&iop->ob_ptr,hreg_rd,tmp);
   }

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* ADDIC - ADD Immediate with Carry */
DECLARE_INSN(ADDIC)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int imm = bits(insn,0,15);
   m_uint32_t tmp = sign_extend_32(imm,16);
   int hreg_rd,hreg_ra;
   jit_op_t *iop;

   /* $rd = $ra + imm */
   ppc32_jit_start_hreg_seq(cpu,"addic");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

   iop = ppc32_op_emit_insn_output(cpu,1,"addic");

   if (rd != ra)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_ra,4);

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_ADD,hreg_rd,tmp,4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   amd64_set_membase(iop->ob_ptr,X86_CC_C,
                     AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),FALSE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* ADDIC. */
DECLARE_INSN(ADDIC_dot)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int imm = bits(insn,0,15);
   m_uint32_t tmp = sign_extend_32(imm,16);
   int hreg_rd,hreg_ra;
   jit_op_t *iop;

   /* $rd = $ra + imm */
   ppc32_jit_start_hreg_seq(cpu,"addic.");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

   iop = ppc32_op_emit_insn_output(cpu,1,"addic.");

   if (rd != ra)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_ra,4);

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_ADD,hreg_rd,tmp,4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   amd64_set_membase(iop->ob_ptr,X86_CC_C,
                     AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),FALSE);

   ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* ADDIS - ADD Immediate Shifted */
DECLARE_INSN(ADDIS)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint32_t imm = bits(insn,0,15);
   m_uint32_t tmp = imm << 16;
   int hreg_rd,hreg_ra;
   jit_op_t *iop;

   /* $rd = $ra + (imm << 16) */
   ppc32_jit_start_hreg_seq(cpu,"addis");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   if (ra != 0) {
      hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
      ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

      iop = ppc32_op_emit_insn_output(cpu,1,"addis");

      if (rd != ra)
         amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_ra,4);

      amd64_alu_reg_imm_size(iop->ob_ptr,X86_ADD,hreg_rd,tmp,4);
   } else {
      iop = ppc32_op_emit_insn_output(cpu,1,"addis");
      amd64_mov_reg_imm(iop->ob_ptr,hreg_rd,tmp);
   }

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* ADDZE */
DECLARE_INSN(ADDZE)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int hreg_rd,hreg_ra,hreg_t0;
   jit_op_t *iop;

   /* $rd = $ra + xer_ca + set_carry */
   ppc32_jit_start_hreg_seq(cpu,"addze");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

   iop = ppc32_op_emit_insn_output(cpu,2,"addze");

   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,hreg_t0,hreg_t0);

   if (rd != ra)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_ra,4);

   amd64_alu_reg_membase_size(iop->ob_ptr,X86_ADD,hreg_rd,
                              AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),4);

   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t0,FALSE);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                         hreg_t0,4);

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* AND */
DECLARE_INSN(AND)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra,hreg_rb;
   jit_op_t *iop;

   /* $ra = $rs & $rb */
   ppc32_jit_start_hreg_seq(cpu,"and");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,1,"and");

   if (ra == rs)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_AND,hreg_ra,hreg_rb,4);
   else if (ra == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_AND,hreg_ra,hreg_rs,4);
   else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_AND,hreg_ra,hreg_rb,4);
   }

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* ANDC */
DECLARE_INSN(ANDC)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra,hreg_rb,hreg_t0;
   jit_op_t *iop;

   /* $ra = $rs & ~$rb */
   ppc32_jit_start_hreg_seq(cpu,"andc");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,1,"andc");

   /* $t0 = ~$rb */
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_rb,4);
   amd64_not_reg(iop->ob_ptr,hreg_t0);

   /* $ra = $rs & $t0 */
   if (ra == rs) 
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_AND,hreg_ra,hreg_t0,4);
   else {
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_AND,hreg_t0,hreg_rs,4);
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_t0,4);
   }

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* AND Immediate */
DECLARE_INSN(ANDI)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t imm = bits(insn,0,15);
   m_uint32_t tmp = imm;
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* $ra = $rs & imm */
   ppc32_jit_start_hreg_seq(cpu,"andi");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,2,"andi");

   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_AND,hreg_ra,tmp,4);
   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* AND Immediate Shifted */
DECLARE_INSN(ANDIS)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint32_t imm = bits(insn,0,15);
   m_uint32_t tmp = imm << 16;
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* $ra = $rs & imm */
   ppc32_jit_start_hreg_seq(cpu,"andis");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,2,"andis");

   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_AND,hreg_ra,tmp,4);
   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* B - Branch */
DECLARE_INSN(B)
{
   m_uint32_t offset = bits(insn,2,25);
   m_uint32_t new_ia;
   jit_op_t *iop;

   iop = ppc32_op_emit_insn_output(cpu,4,"b");

   /* compute the new ia */
   new_ia = b->start_ia + (b->ppc_trans_pos << 2);
   new_ia += sign_extend(offset << 2,26);
   ppc32_set_jump(cpu,b,iop,new_ia,TRUE);

   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);
   ppc32_op_emit_branch_target(cpu,b,new_ia);
   ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1) << 2));
   return(0);
}

/* BA - Branch Absolute */
DECLARE_INSN(BA)
{
   m_uint32_t offset = bits(insn,2,25);
   m_uint32_t new_ia;
   jit_op_t *iop;

   iop = ppc32_op_emit_insn_output(cpu,4,"ba");

   /* compute the new ia */
   new_ia = sign_extend(offset << 2,26);
   ppc32_set_jump(cpu,b,iop,new_ia,TRUE);

   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);
   ppc32_op_emit_branch_target(cpu,b,new_ia);
   ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1) << 2));
   return(0);
}

/* BL - Branch and Link */
DECLARE_INSN(BL)
{
   m_uint32_t offset = bits(insn,2,25);
   m_uint32_t new_ia;
   jit_op_t *iop;

   iop = ppc32_op_emit_insn_output(cpu,4,"bl");

   /* compute the new ia */
   new_ia = b->start_ia + (b->ppc_trans_pos << 2);
   new_ia += sign_extend(offset << 2,26);

   /* set the return address */
   ppc32_set_lr(iop,b->start_ia + ((b->ppc_trans_pos+1) << 2));
   ppc32_set_jump(cpu,b,iop,new_ia,TRUE);

   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);
   ppc32_op_emit_branch_target(cpu,b,new_ia);
   ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1) << 2));
   return(0);
}

/* BLA - Branch and Link Absolute */
DECLARE_INSN(BLA)
{
   m_uint32_t offset = bits(insn,2,25);
   m_uint32_t new_ia;
   jit_op_t *iop;

   iop = ppc32_op_emit_insn_output(cpu,4,"bla");

   /* compute the new ia */
   new_ia = sign_extend(offset << 2,26);

   /* set the return address */
   ppc32_set_lr(iop,b->start_ia + ((b->ppc_trans_pos+1) << 2));
   ppc32_set_jump(cpu,b,iop,new_ia,TRUE);

   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);
   ppc32_op_emit_branch_target(cpu,b,new_ia);
   ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1) << 2));
   return(0);
}

/* BC - Branch Conditional (Condition Check only) */
DECLARE_INSN(BCC)
{
   int bo = bits(insn,21,25);
   int bi = bits(insn,16,20);
   int bd = bits(insn,2,15);
   jit_op_t *iop;
   u_int cr_field,cr_bit;
   m_uint32_t new_ia;
   u_char *jump_ptr;
   int local_jump;
   int cond;

   ppc32_op_emit_basic_opcode(cpu,JIT_OP_BRANCH_JUMP);

   iop = ppc32_op_emit_insn_output(cpu,5,"bcc");

   /* Get the wanted value for the condition bit */
   cond = (bo >> 3) & 0x1;

   /* Set the return address */
   if (insn & 1) {
      ppc32_set_lr(iop,b->start_ia + ((b->ppc_trans_pos+1) << 2));
      ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1)<<2));
   }

   /* Compute the new ia */
   new_ia = sign_extend_32(bd << 2,16);
   if (!(insn & 0x02))
      new_ia += b->start_ia + (b->ppc_trans_pos << 2);

   /* Test the condition bit */
   cr_field = ppc32_get_cr_field(bi);
   cr_bit = ppc32_get_cr_bit(bi);

   ppc32_op_emit_require_flags(cpu,cr_field);

   amd64_test_membase_imm_size(iop->ob_ptr,
                               AMD64_R15,PPC32_CR_FIELD_OFFSET(cr_field),
                               (1 << cr_bit),4);

   local_jump = ppc32_jit_tcb_local_addr(b,new_ia,&jump_ptr);

   /* 
    * Optimize the jump, depending if the destination is in the same 
    * page or not.
    */
   if (local_jump) {
      ppc32_jit_tcb_record_patch(b,iop,iop->ob_ptr,new_ia);
      amd64_branch32(iop->ob_ptr,(cond) ? X86_CC_NZ : X86_CC_Z,0,FALSE);
   } else {   
      jump_ptr = iop->ob_ptr;
      amd64_branch32(iop->ob_ptr,(cond) ? X86_CC_Z : X86_CC_NZ,0,FALSE);
      ppc32_set_jump(cpu,b,iop,new_ia,TRUE);
      amd64_patch(jump_ptr,iop->ob_ptr);
   }

   ppc32_op_emit_branch_target(cpu,b,new_ia);
   return(0);
}

/* BC - Branch Conditional */
DECLARE_INSN(BC)
{
   int bo = bits(insn,21,25);
   int bi = bits(insn,16,20);
   int bd = bits(insn,2,15);
   int hreg_t0,hreg_t1;
   jit_op_t *iop;
   u_int cr_field,cr_bit;
   m_uint32_t new_ia;
   u_char *jump_ptr;
   int local_jump;
   int cond,ctr;

   ppc32_op_emit_basic_opcode(cpu,JIT_OP_BRANCH_JUMP);

   iop = ppc32_op_emit_insn_output(cpu,5,"bc");

   ppc32_jit_start_hreg_seq(cpu,"bc");
   hreg_t0 = ppc32_jit_alloc_hreg(cpu,-1);
   hreg_t1 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_alter_host_reg(cpu,hreg_t0);

   /* Get the wanted value for the condition bit and CTR value */
   cond = (bo >> 3) & 0x1;
   ctr  = (bo >> 1) & 0x1;

   /* Set the return address */
   if (insn & 1) {
      ppc32_set_lr(iop,b->start_ia + ((b->ppc_trans_pos+1) << 2));
      ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1)<<2));
   }

   /* Compute the new ia */
   new_ia = sign_extend_32(bd << 2,16);
   if (!(insn & 0x02))
      new_ia += b->start_ia + (b->ppc_trans_pos << 2);

   amd64_mov_reg_imm(iop->ob_ptr,hreg_t0,1);

   /* Decrement the count register */
   if (!(bo & 0x04)) {
      amd64_dec_membase_size(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,ctr),4);
      amd64_set_reg(iop->ob_ptr,(ctr) ? X86_CC_Z : X86_CC_NZ,hreg_t1,FALSE);
      amd64_alu_reg_reg(iop->ob_ptr,X86_AND,hreg_t0,hreg_t1);
   }

   /* Test the condition bit */
   if (!((bo >> 4) & 0x01)) {
      cr_field = ppc32_get_cr_field(bi);
      cr_bit = ppc32_get_cr_bit(bi);

      ppc32_op_emit_require_flags(cpu,cr_field);

      amd64_test_membase_imm_size(iop->ob_ptr,
                                  AMD64_R15,PPC32_CR_FIELD_OFFSET(cr_field),
                                  (1 << cr_bit),4);

      amd64_set_reg(iop->ob_ptr,(cond) ? X86_CC_NZ : X86_CC_Z,hreg_t1,FALSE);
      amd64_alu_reg_reg(iop->ob_ptr,X86_AND,hreg_t0,hreg_t1);
   }

   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);

   local_jump = ppc32_jit_tcb_local_addr(b,new_ia,&jump_ptr);

   /* 
    * Optimize the jump, depending if the destination is in the same 
    * page or not.
    */
   if (local_jump) {
      ppc32_jit_tcb_record_patch(b,iop,iop->ob_ptr,new_ia);
      amd64_branch32(iop->ob_ptr,X86_CC_NZ,0,FALSE);
   } else {   
      jump_ptr = iop->ob_ptr;
      amd64_branch32(iop->ob_ptr,X86_CC_Z,0,FALSE);
      ppc32_set_jump(cpu,b,iop,new_ia,TRUE);
      amd64_patch(jump_ptr,iop->ob_ptr);
   }

   ppc32_op_emit_branch_target(cpu,b,new_ia);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* BCLR - Branch Conditional to Link register */
DECLARE_INSN(BCLR)
{   
   int bo = bits(insn,21,25);
   int bi = bits(insn,16,20);
   int bd = bits(insn,2,15);
   int hreg_t0,hreg_t1;
   jit_op_t *iop;
   u_int cr_field,cr_bit;
   m_uint32_t new_ia;
   u_char *jump_ptr;
   int cond,ctr;

   ppc32_jit_start_hreg_seq(cpu,"bclr");
   hreg_t0 = ppc32_jit_alloc_hreg(cpu,-1);
   hreg_t1 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_alter_host_reg(cpu,hreg_t0);

   iop = ppc32_op_emit_insn_output(cpu,5,"bclr");

   /* Get the wanted value for the condition bit and CTR value */
   cond = (bo >> 3) & 0x1;
   ctr  = (bo >> 1) & 0x1;

   /* Compute the new ia */
   new_ia = sign_extend_32(bd << 2,16);
   if (!(insn & 0x02))
      new_ia += b->start_ia + (b->ppc_trans_pos << 2);

   amd64_mov_reg_imm(iop->ob_ptr,hreg_t0,1);

   /* Decrement the count register */
   if (!(bo & 0x04)) {
      amd64_dec_membase_size(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,ctr),4);
      amd64_set_reg(iop->ob_ptr,(ctr) ? X86_CC_Z : X86_CC_NZ,hreg_t1,FALSE);
      amd64_alu_reg_reg(iop->ob_ptr,X86_AND,hreg_t0,hreg_t1);
   }

   /* Test the condition bit */
   if (!((bo >> 4) & 0x01)) {
      cr_field = ppc32_get_cr_field(bi);
      cr_bit = ppc32_get_cr_bit(bi);

      ppc32_op_emit_require_flags(cpu,cr_field);

      amd64_test_membase_imm_size(iop->ob_ptr,
                                  AMD64_R15,PPC32_CR_FIELD_OFFSET(cr_field),
                                  (1 << cr_bit),4);

      amd64_set_reg(iop->ob_ptr,(cond) ? X86_CC_NZ : X86_CC_Z,hreg_t1,FALSE);
      amd64_alu_reg_reg(iop->ob_ptr,X86_AND,hreg_t0,hreg_t1);
   }

   /* Set the return address */
   amd64_mov_reg_membase(iop->ob_ptr,hreg_t1,AMD64_R15,OFFSET(cpu_ppc_t,lr),4);

   if (insn & 1) {
      ppc32_set_lr(iop,b->start_ia + ((b->ppc_trans_pos+1) << 2));
      ppc32_op_emit_branch_target(cpu,b,b->start_ia+((b->ppc_trans_pos+1)<<2));
   }

   /* Branching */
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);

   jump_ptr = iop->ob_ptr;
   amd64_branch32(iop->ob_ptr,X86_CC_Z,0,FALSE);

   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t1,0xFFFFFFFC);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,ia),hreg_t1,4);
   ppc32_jit_tcb_push_epilog(&iop->ob_ptr);

   amd64_patch(jump_ptr,iop->ob_ptr);

   ppc32_op_emit_basic_opcode(cpu,JIT_OP_EOB);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CMP - Compare */
DECLARE_INSN(CMP)
{
   int rd = bits(insn,23,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_ra,hreg_rb;
   jit_op_t *iop;
   
   ppc32_jit_start_hreg_seq(cpu,"cmp");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,1,"cmp");

   amd64_alu_reg_reg_size(iop->ob_ptr,X86_CMP,hreg_ra,hreg_rb,4);
   ppc32_op_emit_update_flags(cpu,rd,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CMPI - Compare Immediate */
DECLARE_INSN(CMPI)
{
   int rd = bits(insn,23,25);
   int ra = bits(insn,16,20);
   m_uint16_t imm = bits(insn,0,15);
   m_uint32_t tmp = sign_extend_32(imm,16);
   int hreg_ra;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"cmpi");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

   iop = ppc32_op_emit_insn_output(cpu,1,"cmpi");

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_CMP,hreg_ra,tmp,4);
   ppc32_op_emit_update_flags(cpu,rd,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CMPL - Compare Logical */
DECLARE_INSN(CMPL)
{
   int rd = bits(insn,23,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_ra,hreg_rb;
   jit_op_t *iop;
   
   ppc32_jit_start_hreg_seq(cpu,"cmpl");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,1,"cmpl");

   amd64_alu_reg_reg_size(iop->ob_ptr,X86_CMP,hreg_ra,hreg_rb,4);
   ppc32_op_emit_update_flags(cpu,rd,FALSE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CMPLI - Compare Immediate */
DECLARE_INSN(CMPLI)
{
   int rd = bits(insn,23,25);
   int ra = bits(insn,16,20);
   m_uint32_t imm = bits(insn,0,15);
   int hreg_ra;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"cmpli");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

   iop = ppc32_op_emit_insn_output(cpu,1,"cmpli");

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_CMP,hreg_ra,imm,4);
   ppc32_op_emit_update_flags(cpu,rd,FALSE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CRAND - Condition Register AND */
DECLARE_INSN(CRAND)
{
   int bd = bits(insn,21,25);
   int bb = bits(insn,16,20);
   int ba = bits(insn,11,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_start_hreg_seq(cpu,"crand");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);

   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(ba));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bb));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bd));

   iop = ppc32_op_emit_insn_output(cpu,3,"crand");

   /* test $ba bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba)),
                          (1 << ppc32_get_cr_bit(ba)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,AMD64_RDX,FALSE);

   /* test $bb bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb)),
                          (1 << ppc32_get_cr_bit(bb)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,hreg_t0,FALSE);
   
   /* result of AND between $ba and $bb */
   amd64_alu_reg_reg(iop->ob_ptr,X86_AND,hreg_t0,AMD64_RDX);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);
   
   /* set/clear $bd bit depending on the result */
   amd64_alu_membase_imm_size(iop->ob_ptr,X86_AND,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              ~(1 << ppc32_get_cr_bit(bd)),4);

   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_t0,ppc32_get_cr_bit(bd));
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CRANDC - Condition Register AND with Complement */
DECLARE_INSN(CRANDC)
{
   int bd = bits(insn,21,25);
   int bb = bits(insn,16,20);
   int ba = bits(insn,11,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_start_hreg_seq(cpu,"crandc");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);

   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(ba));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bb));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bd));

   iop = ppc32_op_emit_insn_output(cpu,3,"crandc");

   /* test $ba bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba)),
                          (1 << ppc32_get_cr_bit(ba)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,AMD64_RDX,FALSE);

   /* test $bb bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb)),
                          (1 << ppc32_get_cr_bit(bb)));
   amd64_set_reg(iop->ob_ptr,X86_CC_Z,hreg_t0,FALSE);
   
   /* result of AND between $ba and $bb */
   amd64_alu_reg_reg(iop->ob_ptr,X86_AND,hreg_t0,AMD64_RDX);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);
   
   /* set/clear $bd bit depending on the result */
   amd64_alu_membase_imm_size(iop->ob_ptr,X86_AND,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              ~(1 << ppc32_get_cr_bit(bd)),4);

   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_t0,ppc32_get_cr_bit(bd));
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CREQV - Condition Register EQV */
DECLARE_INSN(CREQV)
{
   int bd = bits(insn,21,25);
   int bb = bits(insn,16,20);
   int ba = bits(insn,11,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_start_hreg_seq(cpu,"creqv");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);

   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(ba));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bb));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bd));

   iop = ppc32_op_emit_insn_output(cpu,3,"creqv");

   /* test $ba bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba)),
                          (1 << ppc32_get_cr_bit(ba)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,AMD64_RDX,FALSE);

   /* test $bb bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb)),
                          (1 << ppc32_get_cr_bit(bb)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,hreg_t0,FALSE);
   
   /* result of XOR between $ba and $bb */
   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,hreg_t0,AMD64_RDX);
   amd64_not_reg(iop->ob_ptr,hreg_t0);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);
   
   /* set/clear $bd bit depending on the result */
   amd64_alu_membase_imm_size(iop->ob_ptr,X86_AND,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              ~(1 << ppc32_get_cr_bit(bd)),4);

   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_t0,ppc32_get_cr_bit(bd));
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CRNAND - Condition Register NAND */
DECLARE_INSN(CRNAND)
{
   int bd = bits(insn,21,25);
   int bb = bits(insn,16,20);
   int ba = bits(insn,11,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_start_hreg_seq(cpu,"crnand");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);

   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(ba));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bb));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bd));

   iop = ppc32_op_emit_insn_output(cpu,3,"crnand");

   /* test $ba bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba)),
                          (1 << ppc32_get_cr_bit(ba)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,AMD64_RDX,FALSE);

   /* test $bb bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb)),
                          (1 << ppc32_get_cr_bit(bb)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,hreg_t0,FALSE);
   
   /* result of NAND between $ba and $bb */
   amd64_alu_reg_reg(iop->ob_ptr,X86_AND,hreg_t0,AMD64_RDX);
   amd64_not_reg(iop->ob_ptr,hreg_t0);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);
   
   /* set/clear $bd bit depending on the result */
   amd64_alu_membase_imm_size(iop->ob_ptr,X86_AND,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              ~(1 << ppc32_get_cr_bit(bd)),4);

   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_t0,ppc32_get_cr_bit(bd));
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CRNOR - Condition Register NOR */
DECLARE_INSN(CRNOR)
{
   int bd = bits(insn,21,25);
   int bb = bits(insn,16,20);
   int ba = bits(insn,11,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_start_hreg_seq(cpu,"crnor");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);

   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(ba));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bb));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bd));

   iop = ppc32_op_emit_insn_output(cpu,3,"crnor");

   /* test $ba bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba)),
                          (1 << ppc32_get_cr_bit(ba)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,AMD64_RDX,FALSE);

   /* test $bb bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb)),
                          (1 << ppc32_get_cr_bit(bb)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,hreg_t0,FALSE);
   
   /* result of NOR between $ba and $bb */
   amd64_alu_reg_reg(iop->ob_ptr,X86_OR,hreg_t0,AMD64_RDX);
   amd64_not_reg(iop->ob_ptr,hreg_t0);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);
   
   /* set/clear $bd bit depending on the result */
   amd64_alu_membase_imm_size(iop->ob_ptr,X86_AND,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              ~(1 << ppc32_get_cr_bit(bd)),4);

   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_t0,ppc32_get_cr_bit(bd));
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CROR - Condition Register OR */
DECLARE_INSN(CROR)
{
   int bd = bits(insn,21,25);
   int bb = bits(insn,16,20);
   int ba = bits(insn,11,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_start_hreg_seq(cpu,"cror");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);

   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(ba));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bb));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bd));

   iop = ppc32_op_emit_insn_output(cpu,3,"cror");

   /* test $ba bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba)),
                          (1 << ppc32_get_cr_bit(ba)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,AMD64_RDX,FALSE);

   /* test $bb bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb)),
                          (1 << ppc32_get_cr_bit(bb)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,hreg_t0,FALSE);
   
   /* result of NOR between $ba and $bb */
   amd64_alu_reg_reg(iop->ob_ptr,X86_OR,hreg_t0,AMD64_RDX);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);
   
   /* set/clear $bd bit depending on the result */
   amd64_alu_membase_imm_size(iop->ob_ptr,X86_AND,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              ~(1 << ppc32_get_cr_bit(bd)),4);

   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_t0,ppc32_get_cr_bit(bd));
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CRORC - Condition Register OR with Complement */
DECLARE_INSN(CRORC)
{
   int bd = bits(insn,21,25);
   int bb = bits(insn,16,20);
   int ba = bits(insn,11,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_start_hreg_seq(cpu,"crorc");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);

   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(ba));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bb));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bd));

   iop = ppc32_op_emit_insn_output(cpu,3,"crorc");

   /* test $ba bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba)),
                          (1 << ppc32_get_cr_bit(ba)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,AMD64_RDX,FALSE);

   /* test $bb bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb)),
                          (1 << ppc32_get_cr_bit(bb)));
   amd64_set_reg(iop->ob_ptr,X86_CC_Z,hreg_t0,FALSE);
   
   /* result of ORC between $ba and $bb */
   amd64_alu_reg_reg(iop->ob_ptr,X86_OR,hreg_t0,AMD64_RDX);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);
   
   /* set/clear $bd bit depending on the result */
   amd64_alu_membase_imm_size(iop->ob_ptr,X86_AND,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              ~(1 << ppc32_get_cr_bit(bd)),4);

   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_t0,ppc32_get_cr_bit(bd));
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* CRXOR - Condition Register XOR */
DECLARE_INSN(CRXOR)
{
   int bd = bits(insn,21,25);
   int bb = bits(insn,16,20);
   int ba = bits(insn,11,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_start_hreg_seq(cpu,"crxor");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);

   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(ba));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bb));
   ppc32_op_emit_require_flags(cpu,ppc32_get_cr_field(bd));

   iop = ppc32_op_emit_insn_output(cpu,3,"crxor");

   /* test $ba bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(ba)),
                          (1 << ppc32_get_cr_bit(ba)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,AMD64_RDX,FALSE);

   /* test $bb bit */
   amd64_test_membase_imm(iop->ob_ptr,
                          AMD64_R15,
                          PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bb)),
                          (1 << ppc32_get_cr_bit(bb)));
   amd64_set_reg(iop->ob_ptr,X86_CC_NZ,hreg_t0,FALSE);
   
   /* result of XOR between $ba and $bb */
   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,hreg_t0,AMD64_RDX);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x01);
   
   /* set/clear $bd bit depending on the result */
   amd64_alu_membase_imm_size(iop->ob_ptr,X86_AND,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              ~(1 << ppc32_get_cr_bit(bd)),4);

   amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_t0,ppc32_get_cr_bit(bd));
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,
                              PPC32_CR_FIELD_OFFSET(ppc32_get_cr_field(bd)),
                              hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* DIVWU - Divide Word Unsigned */
DECLARE_INSN(DIVWU)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rb;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"divwu");
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RAX);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   /* $rd = $ra / $rb */
   ppc32_op_emit_load_gpr(cpu,AMD64_RAX,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,2,"divwu");
   ppc32_load_imm(&iop->ob_ptr,AMD64_RDX,0);

   amd64_div_reg_size(iop->ob_ptr,hreg_rb,0,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,AMD64_RAX,AMD64_RAX,4);

   ppc32_op_emit_store_gpr(cpu,rd,AMD64_RAX);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   /* edx:eax are directly modified: throw them */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RAX);
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* EQV */
DECLARE_INSN(EQV)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra,hreg_rb;
   jit_op_t *iop;

   /* $ra = ~($rs ^ $rb) */
   ppc32_jit_start_hreg_seq(cpu,"eqv");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,1,"eqv");

   if (ra == rs)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_XOR,hreg_ra,hreg_rb,4);
   else if (ra == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_XOR,hreg_ra,hreg_rs,4);
   else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_XOR,hreg_ra,hreg_rb,4);
   }

   amd64_not_reg(iop->ob_ptr,hreg_ra);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* EXTSB - Extend Sign Byte */
DECLARE_INSN(EXTSB)
{   
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* $ra = extsb($rs) */
   ppc32_jit_start_hreg_seq(cpu,"extsb");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,2,"extsb");

   if (rs != ra)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_shift_reg_imm_size(iop->ob_ptr,X86_SHL,hreg_ra,24,4);
   amd64_shift_reg_imm_size(iop->ob_ptr,X86_SAR,hreg_ra,24,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* EXTSH - Extend Sign Word */
DECLARE_INSN(EXTSH)
{   
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* $ra = extsh($rs) */
   ppc32_jit_start_hreg_seq(cpu,"extsh");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,2,"extsh");

   if (rs != ra)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_shift_reg_imm_size(iop->ob_ptr,X86_SHL,hreg_ra,16,4);
   amd64_shift_reg_imm_size(iop->ob_ptr,X86_SAR,hreg_ra,16,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* LBZ - Load Byte and Zero */
DECLARE_INSN(LBZ)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   //ppc32_emit_memop(b,PPC_MEMOP_LBZ,ra,offset,rs,0);
   ppc32_emit_memop_fast(cpu,b,0,PPC_MEMOP_LBZ,ra,offset,rs,
                         ppc32_memop_fast_lbz);
   return(0);
}

/* LBZU - Load Byte and Zero with Update */
DECLARE_INSN(LBZU)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_LBZ,ra,offset,rs,1);
   return(0);
}

/* LBZUX - Load Byte and Zero with Update Indexed */
DECLARE_INSN(LBZUX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_LBZ,ra,rb,rs,1);
   return(0);
}

/* LBZX - Load Byte and Zero Indexed */
DECLARE_INSN(LBZX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_LBZ,ra,rb,rs,0);
   return(0);
}

/* LHA - Load Half-Word Algebraic */
DECLARE_INSN(LHA)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_LHA,ra,offset,rs,0);
   return(0);
}

/* LHAU - Load Half-Word Algebraic with Update */
DECLARE_INSN(LHAU)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_LHA,ra,offset,rs,1);
   return(0);
}

/* LHAUX - Load Half-Word Algebraic with Update Indexed */
DECLARE_INSN(LHAUX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_LHA,ra,rb,rs,1);
   return(0);
}

/* LHAX - Load Half-Word Algebraic Indexed */
DECLARE_INSN(LHAX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_LHA,ra,rb,rs,0);
   return(0);
}

/* LHZ - Load Half-Word and Zero */
DECLARE_INSN(LHZ)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_LHZ,ra,offset,rs,0);
   return(0);
}

/* LHZU - Load Half-Word and Zero with Update */
DECLARE_INSN(LHZU)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_LHZ,ra,offset,rs,1);
   return(0);
}

/* LHZUX - Load Half-Word and Zero with Update Indexed */
DECLARE_INSN(LHZUX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_LHZ,ra,rb,rs,1);
   return(0);
}

/* LHZX - Load Half-Word and Zero Indexed */
DECLARE_INSN(LHZX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_LHZ,ra,rb,rs,0);
   return(0);
}

/* LWZ - Load Word and Zero */
DECLARE_INSN(LWZ)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   //ppc32_emit_memop(b,PPC_MEMOP_LWZ,ra,offset,rs,0);
   ppc32_emit_memop_fast(cpu,b,0,PPC_MEMOP_LWZ,ra,offset,rs,
                         ppc32_memop_fast_lwz);
   return(0);
}

/* LWZU - Load Word and Zero with Update */
DECLARE_INSN(LWZU)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_LWZ,ra,offset,rs,1);
   return(0);
}

/* LWZUX - Load Word and Zero with Update Indexed */
DECLARE_INSN(LWZUX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_LWZ,ra,rb,rs,1);
   return(0);
}

/* LWZX - Load Word and Zero Indexed */
DECLARE_INSN(LWZX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_LWZ,ra,rb,rs,0);
   return(0);
}

/* MCRF - Move Condition Register Field */
DECLARE_INSN(MCRF)
{
   int rd = bits(insn,23,25);
   int rs = bits(insn,18,20);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mcrf");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);   
   ppc32_op_emit_require_flags(cpu,rs);

   iop = ppc32_op_emit_insn_output(cpu,1,"mcrf");

   /* Load "rs" field in %edx */
   amd64_mov_reg_membase(iop->ob_ptr,hreg_t0,
                         AMD64_R15,PPC32_CR_FIELD_OFFSET(rs),4);

   /* Store it in "rd" field */
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,PPC32_CR_FIELD_OFFSET(rd),
                         hreg_t0,4);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MFCR - Move from Condition Register */
DECLARE_INSN(MFCR)
{
   int rd = bits(insn,21,25);
   int hreg_rd,hreg_t0;
   jit_op_t *iop;
   int i;

   ppc32_jit_start_hreg_seq(cpu,"mfcr");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   ppc32_op_emit_require_flags(cpu,JIT_OP_PPC_ALL_FLAGS);

   iop = ppc32_op_emit_insn_output(cpu,3,"mfcr");

   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,hreg_rd,hreg_rd);

   for(i=0;i<8;i++) {
      /* load field in %edx */
      amd64_mov_reg_membase(iop->ob_ptr,hreg_t0,
                            AMD64_R15,PPC32_CR_FIELD_OFFSET(i),4);
      amd64_shift_reg_imm(iop->ob_ptr,X86_SHL,hreg_rd,4);
      amd64_alu_reg_reg(iop->ob_ptr,X86_OR,hreg_rd,hreg_t0);
   }

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MFMSR - Move from Machine State Register */
DECLARE_INSN(MFMSR)
{
   int rd = bits(insn,21,25);
   int hreg_rd;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mfmsr");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   iop = ppc32_op_emit_insn_output(cpu,1,"mfmsr");
   amd64_mov_reg_membase(iop->ob_ptr,hreg_rd,
                         AMD64_R15,OFFSET(cpu_ppc_t,msr),4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MFSR - Move From Segment Register */
DECLARE_INSN(MFSR)
{
   int rd = bits(insn,21,25);
   int sr = bits(insn,16,19);
   int hreg_rd;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mfsr");
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   iop = ppc32_op_emit_insn_output(cpu,1,"mfsr");

   amd64_mov_reg_membase(iop->ob_ptr,hreg_rd,
                         AMD64_R15,(OFFSET(cpu_ppc_t,sr) + (sr << 2)),4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MTCRF - Move to Condition Register Fields */
DECLARE_INSN(MTCRF)
{
   int rs = bits(insn,21,25);
   int crm = bits(insn,12,19);
   int hreg_rs,hreg_t0;
   jit_op_t *iop;
   int i;

   ppc32_jit_start_hreg_seq(cpu,"mtcrf");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,4,"mtcrf");

   for(i=0;i<8;i++)
      if (crm & (1 << (7 - i))) {
         amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_rs,4);

         if (i != 7)
            amd64_shift_reg_imm(iop->ob_ptr,X86_SHR,hreg_t0,28 - (i << 2));

         amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x0F);
         amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,PPC32_CR_FIELD_OFFSET(i),
                               hreg_t0,4);
      }

   ppc32_op_emit_basic_opcode(cpu,JIT_OP_TRASH_FLAGS);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MULHW - Multiply High Word */
DECLARE_INSN(MULHW)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rb;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mulhw");
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RAX);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,AMD64_RAX,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   /* rd = hi(ra * rb) */
   iop = ppc32_op_emit_insn_output(cpu,2,"mulhw");
   amd64_mul_reg_size(iop->ob_ptr,hreg_rb,1,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,AMD64_RDX,AMD64_RDX,4);

   ppc32_op_emit_store_gpr(cpu,rd,AMD64_RDX);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   /* edx:eax are directly modified: throw them */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RAX);
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MULHWU - Multiply High Word Unsigned */
DECLARE_INSN(MULHWU)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rb;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mulhwu");
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RAX);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,AMD64_RAX,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   /* rd = hi(ra * rb) */
   iop = ppc32_op_emit_insn_output(cpu,2,"mulhwu");
   amd64_mul_reg_size(iop->ob_ptr,hreg_rb,0,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,AMD64_RDX,AMD64_RDX,4);

   ppc32_op_emit_store_gpr(cpu,rd,AMD64_RDX);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   /* edx:eax are directly modified: throw them */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RAX);
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MULLI - Multiply Low Immediate */
DECLARE_INSN(MULLI)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint32_t imm = bits(insn,0,15);
   int hreg_t0;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mulli");
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RAX);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_load_gpr(cpu,AMD64_RAX,ra);

   /* rd = lo(ra * imm) */
   iop = ppc32_op_emit_insn_output(cpu,2,"mulli");

   ppc32_load_imm(&iop->ob_ptr,hreg_t0,sign_extend_32(imm,16));
   amd64_mul_reg_size(iop->ob_ptr,hreg_t0,1,4);
   ppc32_op_emit_store_gpr(cpu,rd,AMD64_RAX);

   /* edx:eax are directly modified: throw them */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RAX);
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* MULLW - Multiply Low Word */
DECLARE_INSN(MULLW)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rb;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"mullw");
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RAX);
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RDX);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,AMD64_RAX,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   /* rd = lo(ra * rb) */
   iop = ppc32_op_emit_insn_output(cpu,2,"mullw");
   amd64_mul_reg_size(iop->ob_ptr,hreg_rb,1,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,AMD64_RAX,AMD64_RAX,4);

   ppc32_op_emit_store_gpr(cpu,rd,AMD64_RAX);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   /* edx:eax are directly modified: throw them */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RAX);
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RDX);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* NAND */
DECLARE_INSN(NAND)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra,hreg_rb;
   jit_op_t *iop;

   /* $ra = ~($rs & $rb) */
   ppc32_jit_start_hreg_seq(cpu,"nand");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,2,"nand");

   if (ra == rs)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_AND,hreg_ra,hreg_rb,4);
   else if (ra == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_AND,hreg_ra,hreg_rs,4);
   else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_AND,hreg_ra,hreg_rb,4);
   }

   amd64_not_reg(iop->ob_ptr,hreg_ra);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* NEG */
DECLARE_INSN(NEG)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int hreg_rd,hreg_ra;
   jit_op_t *iop;

   /* $rd = neg($ra) */
   ppc32_jit_start_hreg_seq(cpu,"neg");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

   iop = ppc32_op_emit_insn_output(cpu,1,"neg");

   if (rd != ra)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_ra,4);

   amd64_neg_reg(iop->ob_ptr,hreg_rd);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_rd,hreg_rd,4);

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* NOR */
DECLARE_INSN(NOR)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra,hreg_rb;
   jit_op_t *iop;

   /* $ra = ~($rs | $rb) */
   ppc32_jit_start_hreg_seq(cpu,"nor");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,2,"nor");

   if (ra == rs)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_ra,hreg_rb,4);
   else if (ra == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_ra,hreg_rs,4);
   else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_ra,hreg_rb,4);
   }

   amd64_not_reg(iop->ob_ptr,hreg_ra);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* OR */
DECLARE_INSN(OR)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra,hreg_rb;
   jit_op_t *iop;

   /* $ra = $rs | $rb */
   ppc32_jit_start_hreg_seq(cpu,"or");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   /* special optimization for move/nop operation */
   if (rs == rb) {
      ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
      iop = ppc32_op_emit_insn_output(cpu,2,"or");

      if (ra != rs)
         amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

      if (insn & 1)
         amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

      ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

      if (insn & 1)
         ppc32_op_emit_update_flags(cpu,0,TRUE);

      ppc32_jit_close_hreg_seq(cpu);
      return(0);
   }

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,2,"or");

   if (ra == rs) {
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_ra,hreg_rb,4);
   } else if (ra == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_ra,hreg_rs,4);
   else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_ra,hreg_rb,4);
   }

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* OR with Complement */
DECLARE_INSN(ORC)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra,hreg_rb,hreg_t0;
   jit_op_t *iop;

   /* $ra = $rs & ~$rb */
   ppc32_jit_start_hreg_seq(cpu,"orc");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,1,"orc");

   /* $t0 = ~$rb */
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_rb,4);
   amd64_not_reg(iop->ob_ptr,hreg_t0);

   /* $ra = $rs | $t0 */
   if (ra == rs) 
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_ra,hreg_t0,4);
   else {
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_t0,hreg_rs,4);
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_t0,4);
   }

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* OR Immediate */
DECLARE_INSN(ORI)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t imm = bits(insn,0,15);
   m_uint32_t tmp = imm;
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* $ra = $rs | imm */
   ppc32_jit_start_hreg_seq(cpu,"ori");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,1,"ori");

   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_OR,hreg_ra,tmp,4);
   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   ppc32_jit_close_hreg_seq(cpu);
   return(0); 
}

/* OR Immediate Shifted */
DECLARE_INSN(ORIS)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t imm = bits(insn,0,15);
   m_uint32_t tmp = imm << 16;
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* $ra = $rs | imm */
   ppc32_jit_start_hreg_seq(cpu,"oris");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,1,"oris");

   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_alu_reg_imm_size(iop->ob_ptr,X86_OR,hreg_ra,tmp,4);
   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* RLWIMI - Rotate Left Word Immediate then Mask Insert */
DECLARE_INSN(RLWIMI)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int sh = bits(insn,11,15);
   int mb = bits(insn,6,10);
   int me = bits(insn,1,5);
   register m_uint32_t mask;
   int hreg_rs,hreg_ra,hreg_t0;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"rlwimi");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

   mask = ppc32_rotate_mask(mb,me);

   iop = ppc32_op_emit_insn_output(cpu,2,"rlwimi");

   /* Apply inverse mask to $ra */
   if (mask != 0)
      amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_ra,~mask);

   /* Rotate $rs of "sh" bits and apply the mask */
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_rs,4);

   if (sh != 0)
      amd64_shift_reg_imm_size(iop->ob_ptr,X86_ROL,hreg_t0,sh,4);

   if (mask != 0xFFFFFFFF)
      amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,mask);

   /* Store the result */
   amd64_alu_reg_reg_size(iop->ob_ptr,X86_OR,hreg_ra,hreg_t0,4);
   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* RLWINM - Rotate Left Word Immediate AND with Mask */
DECLARE_INSN(RLWINM)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int sh = bits(insn,11,15);
   int mb = bits(insn,6,10);
   int me = bits(insn,1,5);
   register m_uint32_t mask;
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"rlwinm");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,2,"rlwinm");

   /* Rotate $rs of "sh" bits and apply the mask */
   mask = ppc32_rotate_mask(mb,me);

   if (rs != ra)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   if (sh != 0)
      amd64_shift_reg_imm_size(iop->ob_ptr,X86_ROL,hreg_ra,sh,4);

   if (mask != 0xFFFFFFFF)
      amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_ra,mask);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* RLWNM - Rotate Left Word then Mask Insert */
DECLARE_INSN(RLWNM)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int mb = bits(insn,6,10);
   int me = bits(insn,1,5);
   register m_uint32_t mask;
   int hreg_rs,hreg_ra,hreg_t0;
   jit_op_t *iop;

   /* ecx is directly modified: throw it */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RCX);

   ppc32_jit_start_hreg_seq(cpu,"rlwnm");
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RCX);

   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,AMD64_RCX,rb);

   iop = ppc32_op_emit_insn_output(cpu,2,"rlwnm");

   /* Load the shift register ("sh") */
   mask = ppc32_rotate_mask(mb,me);

   /* Rotate $rs and apply the mask */
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_rs,4);

   amd64_shift_reg_size(iop->ob_ptr,X86_ROL,hreg_t0,4);

   if (mask != 0xFFFFFFFF)
      amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,mask);

   amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_t0,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* Shift Left Word */
DECLARE_INSN(SLW)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* ecx is directly modified: throw it */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RCX);

   ppc32_jit_start_hreg_seq(cpu,"slw");
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RCX);
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   /* $ra = $rs << $rb. If count >= 32, then null result */
   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,AMD64_RCX,rb);

   iop = ppc32_op_emit_insn_output(cpu,3,"slw");

   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,AMD64_RCX,0x3f);

   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_shift_reg(iop->ob_ptr,X86_SHL,hreg_ra);
   
   /* store the result */
   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);
   
   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* SRAWI - Shift Right Algebraic Word Immediate */
DECLARE_INSN(SRAWI)
{   
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int sh = bits(insn,11,15);
   register m_uint32_t mask;
   int hreg_rs,hreg_ra,hreg_t0;
   jit_op_t *iop;

   ppc32_jit_start_hreg_seq(cpu,"srawi");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   /* $ra = (int32)$rs >> sh */
   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,3,"srawi");
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_rs,4);
   
   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);
   amd64_shift_reg_imm_size(iop->ob_ptr,X86_SAR,hreg_ra,sh,4);

   /* set XER_CA depending on the result */
   mask = ~(0xFFFFFFFFU << sh) | 0x80000000;

   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,mask);
   amd64_alu_reg_imm_size(iop->ob_ptr,X86_CMP,hreg_t0,0x80000000,4);
   amd64_set_reg(iop->ob_ptr,X86_CC_A,hreg_t0,FALSE);
   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,hreg_t0,0x1);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                         hreg_t0,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);
   
   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* Shift Right Word */
DECLARE_INSN(SRW)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* ecx is directly modified: throw it */
   ppc32_op_emit_alter_host_reg(cpu,AMD64_RCX);

   ppc32_jit_start_hreg_seq(cpu,"srw");
   ppc32_jit_alloc_hreg_forced(cpu,AMD64_RCX);
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   /* $ra = $rs >> $rb. If count >= 32, then null result */
   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,AMD64_RCX,rb);

   iop = ppc32_op_emit_insn_output(cpu,3,"srw");

   amd64_alu_reg_imm(iop->ob_ptr,X86_AND,AMD64_RCX,0x3f);

   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_shift_reg(iop->ob_ptr,X86_SHR,hreg_ra);
   
   /* store the result */
   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_ra,hreg_ra,4);

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);
   
   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* STB - Store Byte */
DECLARE_INSN(STB)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   //ppc32_emit_memop(b,PPC_MEMOP_STB,ra,offset,rs,0);
   ppc32_emit_memop_fast(cpu,b,1,PPC_MEMOP_STB,ra,offset,rs,
                         ppc32_memop_fast_stb);
   return(0);
}

/* STBU - Store Byte with Update */
DECLARE_INSN(STBU)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_STB,ra,offset,rs,1);
   return(0);
}

/* STBUX - Store Byte with Update Indexed */
DECLARE_INSN(STBUX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_STB,ra,rb,rs,1);
   return(0);
}

/* STBUX - Store Byte Indexed */
DECLARE_INSN(STBX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_STB,ra,rb,rs,0);
   return(0);
}

/* STH - Store Half-Word */
DECLARE_INSN(STH)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_STH,ra,offset,rs,0);
   return(0);
}

/* STHU - Store Half-Word with Update */
DECLARE_INSN(STHU)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_STH,ra,offset,rs,1);
   return(0);
}

/* STHUX - Store Half-Word with Update Indexed */
DECLARE_INSN(STHUX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_STH,ra,rb,rs,1);
   return(0);
}

/* STHUX - Store Half-Word Indexed */
DECLARE_INSN(STHX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_STH,ra,rb,rs,0);
   return(0);
}

/* STW - Store Word */
DECLARE_INSN(STW)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   //ppc32_emit_memop(b,PPC_MEMOP_STW,ra,offset,rs,0);
   ppc32_emit_memop_fast(cpu,b,1,PPC_MEMOP_STW,ra,offset,rs,
                         ppc32_memop_fast_stw);
   return(0);
}

/* STWU - Store Word with Update */
DECLARE_INSN(STWU)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t offset = bits(insn,0,15);

   ppc32_emit_memop(cpu,b,PPC_MEMOP_STW,ra,offset,rs,1);
   return(0);
}

/* STWUX - Store Word with Update Indexed */
DECLARE_INSN(STWUX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_STW,ra,rb,rs,1);
   return(0);
}

/* STWUX - Store Word Indexed */
DECLARE_INSN(STWX)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);

   ppc32_emit_memop_idx(cpu,b,PPC_MEMOP_STW,ra,rb,rs,0);
   return(0);
}

/* SUBF - Subtract From */
DECLARE_INSN(SUBF)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rd,hreg_ra,hreg_rb,hreg_t0;
   jit_op_t *iop;

   /* $rd = $rb - $ra */
   ppc32_jit_start_hreg_seq(cpu,"subf");
   hreg_t0 = ppc32_jit_get_tmp_hreg(cpu);

   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,2,"subf");

   if (rd == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_SUB,hreg_rd,hreg_ra,4);
   else if (rd == ra) {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_rb,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_SUB,hreg_t0,hreg_ra,4);
      amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_t0,4);
   } else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_rb,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_SUB,hreg_rd,hreg_ra,4);
   }

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* SUBFC - Subtract From Carrying */
DECLARE_INSN(SUBFC)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_ra,hreg_rb,hreg_rd,hreg_t0,hreg_t1;
   jit_op_t *iop;

   /* $rd = ~$ra + 1 + $rb */
   ppc32_jit_start_hreg_seq(cpu,"subfc");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   hreg_t0 = ppc32_jit_alloc_hreg(cpu,-1);
   hreg_t1 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_alter_host_reg(cpu,hreg_t0);
   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,3,"subfc");

   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,hreg_t1,hreg_t1);

   /* $t0 = ~$ra + 1 */
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_ra,4);
   amd64_not_reg(iop->ob_ptr,hreg_t0);
   amd64_alu_reg_imm_size(iop->ob_ptr,X86_ADD,hreg_t0,1,4);
   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t1,FALSE);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                         hreg_t1,4);

   /* $t0 += $rb */
   amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_t0,hreg_rb,4);
   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t1,FALSE);
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                              hreg_t1,4);

   amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_t0,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_rd,hreg_rd,4);

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   /* update cr0 */
   if (insn & 1)
      ppc32_update_cr0(b);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* SUBFE - Subtract From Extended */
DECLARE_INSN(SUBFE)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_ra,hreg_rb,hreg_rd,hreg_t0,hreg_t1;
   jit_op_t *iop;

   /* $rd = ~$ra + $carry (xer_ca) + $rb */
   ppc32_jit_start_hreg_seq(cpu,"subfe");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   hreg_t0 = ppc32_jit_alloc_hreg(cpu,-1);
   hreg_t1 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_alter_host_reg(cpu,hreg_t0);
   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,3,"subfe");

   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,hreg_t1,hreg_t1);

   /* $t0 = ~$ra + $carry */
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_ra,4);
   amd64_not_reg(iop->ob_ptr,hreg_t0);
   amd64_alu_reg_membase_size(iop->ob_ptr,X86_ADD,hreg_t0,
                              AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),4);

   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t1,FALSE);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                         hreg_t1,4);

   /* $t0 += $rb */
   amd64_alu_reg_reg_size(iop->ob_ptr,X86_ADD,hreg_t0,hreg_rb,4);
   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t1,FALSE);
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                              hreg_t1,4);

   amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_t0,4);

   if (insn & 1)
      amd64_test_reg_reg_size(iop->ob_ptr,hreg_rd,hreg_rd,4);

   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);

   /* update cr0 */
   if (insn & 1)
      ppc32_update_cr0(b);

   ppc32_jit_close_hreg_seq(cpu); 
   return(0);
}

/* SUBFIC - Subtract From Immediate Carrying */
DECLARE_INSN(SUBFIC)
{
   int rd = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t imm = bits(insn,0,15);
   m_uint32_t tmp = sign_extend_32(imm,16);
   int hreg_ra,hreg_rd,hreg_t0,hreg_t1;
   jit_op_t *iop;

   /* $rd = ~$ra + 1 + sign_extend(imm,16) */
   ppc32_jit_start_hreg_seq(cpu,"subfic");
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rd = ppc32_jit_alloc_hreg(cpu,rd);

   hreg_t0 = ppc32_jit_alloc_hreg(cpu,-1);
   hreg_t1 = ppc32_jit_get_tmp_hreg(cpu);

   ppc32_op_emit_alter_host_reg(cpu,hreg_t0);
   ppc32_op_emit_load_gpr(cpu,hreg_ra,ra);

   iop = ppc32_op_emit_insn_output(cpu,3,"subfic");

   amd64_alu_reg_reg(iop->ob_ptr,X86_XOR,hreg_t1,hreg_t1);

   /* $t0 = ~$ra + 1 */
   amd64_mov_reg_reg(iop->ob_ptr,hreg_t0,hreg_ra,4);
   amd64_not_reg(iop->ob_ptr,hreg_t0);
   amd64_alu_reg_imm_size(iop->ob_ptr,X86_ADD,hreg_t0,1,4);

   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t1,FALSE);
   amd64_mov_membase_reg(iop->ob_ptr,AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                         hreg_t1,4);

   /* $t0 += sign_extend(imm,16) */
   amd64_alu_reg_imm_size(iop->ob_ptr,X86_ADD,hreg_t0,tmp,4);
   amd64_set_reg(iop->ob_ptr,X86_CC_C,hreg_t1,FALSE);
   amd64_alu_membase_reg_size(iop->ob_ptr,X86_OR,
                              AMD64_R15,OFFSET(cpu_ppc_t,xer_ca),
                              hreg_t1,4);

   amd64_mov_reg_reg(iop->ob_ptr,hreg_rd,hreg_t0,4);
   ppc32_op_emit_store_gpr(cpu,rd,hreg_rd);
   
   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* SYNC - Synchronize */
DECLARE_INSN(SYNC)
{
   return(0);
}

/* XOR */
DECLARE_INSN(XOR)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   int rb = bits(insn,11,15);
   int hreg_rs,hreg_ra,hreg_rb;
   jit_op_t *iop;

   /* $ra = $rs ^ $rb */
   ppc32_jit_start_hreg_seq(cpu,"xor");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);
   hreg_rb = ppc32_jit_alloc_hreg(cpu,rb);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);
   ppc32_op_emit_load_gpr(cpu,hreg_rb,rb);

   iop = ppc32_op_emit_insn_output(cpu,1,"xor");

   if (ra == rs)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_XOR,hreg_ra,hreg_rb,4);
   else if (ra == rb)
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_XOR,hreg_ra,hreg_rs,4);
   else {
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);
      amd64_alu_reg_reg_size(iop->ob_ptr,X86_XOR,hreg_ra,hreg_rb,4);
   }

   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   if (insn & 1)
      ppc32_op_emit_update_flags(cpu,0,TRUE);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* XORI - XOR Immediate */
DECLARE_INSN(XORI)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint32_t imm = bits(insn,0,15);
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* $ra = $rs ^ imm */
   ppc32_jit_start_hreg_seq(cpu,"xori");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,1,"xori");

   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_alu_reg_imm(iop->ob_ptr,X86_XOR,hreg_ra,imm);
   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* XORIS - XOR Immediate Shifted */
DECLARE_INSN(XORIS)
{
   int rs = bits(insn,21,25);
   int ra = bits(insn,16,20);
   m_uint16_t imm = bits(insn,0,15);
   m_uint32_t tmp = imm << 16;
   int hreg_rs,hreg_ra;
   jit_op_t *iop;

   /* $ra = $rs ^ (imm << 16) */
   ppc32_jit_start_hreg_seq(cpu,"xoris");
   hreg_rs = ppc32_jit_alloc_hreg(cpu,rs);
   hreg_ra = ppc32_jit_alloc_hreg(cpu,ra);

   ppc32_op_emit_load_gpr(cpu,hreg_rs,rs);

   iop = ppc32_op_emit_insn_output(cpu,1,"xoris");

   if (ra != rs)
      amd64_mov_reg_reg(iop->ob_ptr,hreg_ra,hreg_rs,4);

   amd64_alu_reg_imm(iop->ob_ptr,X86_XOR,hreg_ra,tmp);
   ppc32_op_emit_store_gpr(cpu,ra,hreg_ra);

   ppc32_jit_close_hreg_seq(cpu);
   return(0);
}

/* PPC instruction array */
struct ppc32_insn_tag ppc32_insn_tags[] = {
   { ppc32_emit_BLR        , 0xfffffffe , 0x4e800020 },
   { ppc32_emit_BCTR       , 0xfffffffe , 0x4e800420 },
   { ppc32_emit_MFLR       , 0xfc1fffff , 0x7c0802a6 },
   { ppc32_emit_MTLR       , 0xfc1fffff , 0x7c0803a6 },
   { ppc32_emit_MFCTR      , 0xfc1fffff , 0x7c0902a6 },
   { ppc32_emit_MTCTR      , 0xfc1fffff , 0x7c0903a6 },
   { ppc32_emit_MFTBL      , 0xfc1ff7ff , 0x7c0c42e6 },
   { ppc32_emit_MFTBU      , 0xfc1ff7ff , 0x7c0d42e6 },
   { ppc32_emit_ADD        , 0xfc0007fe , 0x7c000214 },
   { ppc32_emit_ADDC       , 0xfc0007fe , 0x7c000014 },
   { ppc32_emit_ADDE       , 0xfc0007fe , 0x7c000114 },
   { ppc32_emit_ADDI       , 0xfc000000 , 0x38000000 },
   { ppc32_emit_ADDIC      , 0xfc000000 , 0x30000000 },
   { ppc32_emit_ADDIC_dot  , 0xfc000000 , 0x34000000 },
   { ppc32_emit_ADDIS      , 0xfc000000 , 0x3c000000 },
   { ppc32_emit_ADDZE      , 0xfc00fffe , 0x7c000194 },
   { ppc32_emit_AND        , 0xfc0007fe , 0x7c000038 },
   { ppc32_emit_ANDC       , 0xfc0007fe , 0x7c000078 },
   { ppc32_emit_ANDI       , 0xfc000000 , 0x70000000 },
   { ppc32_emit_ANDIS      , 0xfc000000 , 0x74000000 },
   { ppc32_emit_B          , 0xfc000003 , 0x48000000 },
   { ppc32_emit_BA         , 0xfc000003 , 0x48000002 },
   { ppc32_emit_BL         , 0xfc000003 , 0x48000001 },
   { ppc32_emit_BLA        , 0xfc000003 , 0x48000003 },
   { ppc32_emit_BCC        , 0xfe800000 , 0x40800000 },
   { ppc32_emit_BC         , 0xfc000000 , 0x40000000 },
   { ppc32_emit_BCLR       , 0xfc00fffe , 0x4c000020 },
   { ppc32_emit_CMP        , 0xfc6007ff , 0x7c000000 },
   { ppc32_emit_CMPI       , 0xfc600000 , 0x2c000000 },
   { ppc32_emit_CMPL       , 0xfc6007ff , 0x7c000040 },
   { ppc32_emit_CMPLI      , 0xfc600000 , 0x28000000 },
   { ppc32_emit_CRAND      , 0xfc0007ff , 0x4c000202 },
   { ppc32_emit_CRANDC     , 0xfc0007ff , 0x4c000102 },
   { ppc32_emit_CREQV      , 0xfc0007ff , 0x4c000242 },
   { ppc32_emit_CRNAND     , 0xfc0007ff , 0x4c0001c2 },
   { ppc32_emit_CRNOR      , 0xfc0007ff , 0x4c000042 },
   { ppc32_emit_CROR       , 0xfc0007ff , 0x4c000382 },
   { ppc32_emit_CRORC      , 0xfc0007ff , 0x4c000342 },
   { ppc32_emit_CRXOR      , 0xfc0007ff , 0x4c000182 },
   { ppc32_emit_DIVWU      , 0xfc0007fe , 0x7c000396 },
   { ppc32_emit_EQV        , 0xfc0007fe , 0x7c000238 },
   { ppc32_emit_EXTSB      , 0xfc00fffe , 0x7c000774 },
   { ppc32_emit_EXTSH      , 0xfc00fffe , 0x7c000734 },
   { ppc32_emit_LBZ        , 0xfc000000 , 0x88000000 },
   { ppc32_emit_LBZU       , 0xfc000000 , 0x8c000000 },
   { ppc32_emit_LBZUX      , 0xfc0007ff , 0x7c0000ee },
   { ppc32_emit_LBZX       , 0xfc0007ff , 0x7c0000ae },
   { ppc32_emit_LHA        , 0xfc000000 , 0xa8000000 },
   { ppc32_emit_LHAU       , 0xfc000000 , 0xac000000 },
   { ppc32_emit_LHAUX      , 0xfc0007ff , 0x7c0002ee },
   { ppc32_emit_LHAX       , 0xfc0007ff , 0x7c0002ae },
   { ppc32_emit_LHZ        , 0xfc000000 , 0xa0000000 },
   { ppc32_emit_LHZU       , 0xfc000000 , 0xa4000000 },
   { ppc32_emit_LHZUX      , 0xfc0007ff , 0x7c00026e },
   { ppc32_emit_LHZX       , 0xfc0007ff , 0x7c00022e },
   { ppc32_emit_LWZ        , 0xfc000000 , 0x80000000 },
   { ppc32_emit_LWZU       , 0xfc000000 , 0x84000000 },
   { ppc32_emit_LWZUX      , 0xfc0007ff , 0x7c00006e },
   { ppc32_emit_LWZX       , 0xfc0007ff , 0x7c00002e },
   { ppc32_emit_MCRF       , 0xfc63ffff , 0x4c000000 },
   { ppc32_emit_MFCR       , 0xfc1fffff , 0x7c000026 },
   { ppc32_emit_MFMSR      , 0xfc1fffff , 0x7c0000a6 },
   { ppc32_emit_MFSR       , 0xfc10ffff , 0x7c0004a6 },
   { ppc32_emit_MTCRF      , 0xfc100fff , 0x7c000120 },
   { ppc32_emit_MULHW      , 0xfc0007fe , 0x7c000096 },
   { ppc32_emit_MULHWU     , 0xfc0007fe , 0x7c000016 },
   { ppc32_emit_MULLI      , 0xfc000000 , 0x1c000000 },
   { ppc32_emit_MULLW      , 0xfc0007fe , 0x7c0001d6 },
   { ppc32_emit_NAND       , 0xfc0007fe , 0x7c0003b8 },
   { ppc32_emit_NEG        , 0xfc00fffe , 0x7c0000d0 },
   { ppc32_emit_NOR        , 0xfc0007fe , 0x7c0000f8 },
   { ppc32_emit_OR         , 0xfc0007fe , 0x7c000378 },
   { ppc32_emit_ORC        , 0xfc0007fe , 0x7c000338 },
   { ppc32_emit_ORI        , 0xfc000000 , 0x60000000 },
   { ppc32_emit_ORIS       , 0xfc000000 , 0x64000000 },
   { ppc32_emit_RLWIMI     , 0xfc000000 , 0x50000000 },
   { ppc32_emit_RLWINM     , 0xfc000000 , 0x54000000 },
   { ppc32_emit_RLWNM      , 0xfc000000 , 0x5c000000 },
   { ppc32_emit_SLW        , 0xfc0007fe , 0x7c000030 },
   { ppc32_emit_SRAWI      , 0xfc0007fe , 0x7c000670 },
   { ppc32_emit_SRW        , 0xfc0007fe , 0x7c000430 },
   { ppc32_emit_STB        , 0xfc000000 , 0x98000000 },
   { ppc32_emit_STBU       , 0xfc000000 , 0x9c000000 },
   { ppc32_emit_STBUX      , 0xfc0007ff , 0x7c0001ee },
   { ppc32_emit_STBX       , 0xfc0007ff , 0x7c0001ae },
   { ppc32_emit_STH        , 0xfc000000 , 0xb0000000 },
   { ppc32_emit_STHU       , 0xfc000000 , 0xb4000000 },
   { ppc32_emit_STHUX      , 0xfc0007ff , 0x7c00036e },
   { ppc32_emit_STHX       , 0xfc0007ff , 0x7c00032e },
   { ppc32_emit_STW        , 0xfc000000 , 0x90000000 },
   { ppc32_emit_STWU       , 0xfc000000 , 0x94000000 },
   { ppc32_emit_STWUX      , 0xfc0007ff , 0x7c00016e },
   { ppc32_emit_STWX       , 0xfc0007ff , 0x7c00012e },
   { ppc32_emit_SUBF       , 0xfc0007fe , 0x7c000050 },
   { ppc32_emit_SUBFC      , 0xfc0007fe , 0x7c000010 },
   { ppc32_emit_SUBFE      , 0xfc0007fe , 0x7c000110 },
   { ppc32_emit_SUBFIC     , 0xfc000000 , 0x20000000 },
   { ppc32_emit_SYNC       , 0xffffffff , 0x7c0004ac },
   { ppc32_emit_XOR        , 0xfc0007fe , 0x7c000278 },
   { ppc32_emit_XORI       , 0xfc000000 , 0x68000000 },
   { ppc32_emit_XORIS      , 0xfc000000 , 0x6c000000 },
   { ppc32_emit_unknown    , 0x00000000 , 0x00000000 },
   { NULL                  , 0x00000000 , 0x00000000 },
};
