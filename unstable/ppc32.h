/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __PPC_32_H__
#define __PPC_32_H__

#include <pthread.h>

#include "utils.h" 
#include "rust_dynamips_c.h"

/* CPU identifiers */
#define PPC32_PVR_405     0x40110000

/* Minimum page size: 4 Kb */
#define PPC32_MIN_PAGE_SHIFT   12
#define PPC32_MIN_PAGE_SIZE    (1 << PPC32_MIN_PAGE_SHIFT)
#define PPC32_MIN_PAGE_IMASK   (PPC32_MIN_PAGE_SIZE - 1)
#define PPC32_MIN_PAGE_MASK    0xFFFFF000

/* Number of instructions per page */
#define PPC32_INSN_PER_PAGE    (PPC32_MIN_PAGE_SIZE/sizeof(ppc_insn_t))

/* Special Purpose Registers (SPR) */
#define PPC32_SPR_XER        1
#define PPC32_SPR_LR         8      /* Link Register */
#define PPC32_SPR_CTR        9      /* Count Register */
#define PPC32_SPR_DSISR      18
#define PPC32_SPR_DAR        19
#define PPC32_SPR_DEC        22     /* Decrementer */
#define PPC32_SPR_SDR1       25     /* Page Table Address */
#define PPC32_SPR_SRR0       26
#define PPC32_SPR_SRR1       27
#define PPC32_SPR_TBL_READ   268    /* Time Base Low (read) */
#define PPC32_SPR_TBU_READ   269    /* Time Base Up (read) */
#define PPC32_SPR_SPRG0      272
#define PPC32_SPR_SPRG1      273
#define PPC32_SPR_SPRG2      274
#define PPC32_SPR_SPRG3      275
#define PPC32_SPR_TBL_WRITE  284    /* Time Base Low (write) */
#define PPC32_SPR_TBU_WRITE  285    /* Time Base Up (write) */
#define PPC32_SPR_PVR        287    /* Processor Version Register */
#define PPC32_SPR_HID0       1008
#define PPC32_SPR_HID1       1009

#define PPC405_SPR_PID      945    /* Process Identifier */

/* Exception vectors */
#define PPC32_EXC_SYS_RST   0x00000100   /* System Reset */
#define PPC32_EXC_MC_CHK    0x00000200   /* Machine Check */
#define PPC32_EXC_DSI       0x00000300   /* Data memory access failure */
#define PPC32_EXC_ISI       0x00000400   /* Instruction fetch failure */
#define PPC32_EXC_EXT       0x00000500   /* External Interrupt */
#define PPC32_EXC_ALIGN     0x00000600   /* Alignment */
#define PPC32_EXC_PROG      0x00000700   /* FPU, Illegal instruction, ... */
#define PPC32_EXC_NO_FPU    0x00000800   /* FPU unavailable */
#define PPC32_EXC_DEC       0x00000900   /* Decrementer */
#define PPC32_EXC_SYSCALL   0x00000C00   /* System Call */
#define PPC32_EXC_TRACE     0x00000D00   /* Trace */
#define PPC32_EXC_FPU_HLP   0x00000E00   /* Floating-Point Assist */

/* Condition Register (CR) is accessed through 8 fields of 4 bits */
#define ppc32_get_cr_field(n)  ((n) >> 2)
#define ppc32_get_cr_bit(n)    (~(n) & 0x03)

/* Positions of LT, GT, EQ and SO bits in CR fields */
#define PPC32_CR_LT_BIT  3
#define PPC32_CR_GT_BIT  2
#define PPC32_CR_EQ_BIT  1
#define PPC32_CR_SO_BIT  0

/* CR0 (Condition Register Field 0) bits */
#define PPC32_CR0_LT_BIT    31
#define PPC32_CR0_LT        (1 << PPC32_CR0_LT_BIT)   /* Negative */
#define PPC32_CR0_GT_BIT    30
#define PPC32_CR0_GT        (1 << PPC32_CR0_GT_BIT)   /* Positive */
#define PPC32_CR0_EQ_BIT    29
#define PPC32_CR0_EQ        (1 << PPC32_CR0_EQ_BIT)   /* Zero */
#define PPC32_CR0_SO_BIT    28
#define PPC32_CR0_SO        (1 << PPC32_CR0_SO_BIT)   /* Summary overflow */

/* XER register */
#define PPC32_XER_SO_BIT    31
#define PPC32_XER_SO        (1 << PPC32_XER_SO_BIT) /* Summary Overflow */
#define PPC32_XER_OV        0x40000000              /* Overflow */
#define PPC32_XER_CA_BIT    29
#define PPC32_XER_CA        (1 << PPC32_XER_CA_BIT) /* Carry */
#define PPC32_XER_BC_MASK   0x0000007F              /* Byte cnt (lswx/stswx) */

#define PPC32_RFI_MSR_MASK  0x87c0ff73
#define PPC32_EXC_SRR1_MASK 0x0000ff73
#define PPC32_EXC_MSR_MASK  0x0006ef32

/* Upper BAT register */
#define PPC32_UBAT_BEPI_MASK   0xFFFE0000  /* Block Effective Page Index */
#define PPC32_UBAT_BEPI_SHIFT  17
#define PPC32_UBAT_BL_MASK     0x00001FFC  /* Block Length */
#define PPC32_UBAT_BL_SHIFT    2
#define PPC32_UBAT_XBL_MASK    0x0001FFFC  /* Block Length */
#define PPC32_UBAT_XBL_SHIFT   2
#define PPC32_UBAT_VS          0x00000002  /* Supervisor mode valid bit */
#define PPC32_UBAT_VP          0x00000001  /* User mode valid bit */
#define PPC32_UBAT_PROT_MASK   (PPC32_UBAT_VS|PPC32_UBAT_VP)

/* Lower BAT register */
#define PPC32_LBAT_BRPN_MASK   0xFFFE0000  /* Physical address */
#define PPC32_LBAT_BRPN_SHIFT  17
#define PPC32_LBAT_WIMG_MASK   0x00000078  /* Memory/cache access mode bits */
#define PPC32_LBAT_PP_MASK     0x00000003  /* Protection bits */

#define PPC32_BAT_ADDR_SHIFT   17

/* Segment Descriptor */
#define PPC32_SD_T          0x80000000
#define PPC32_SD_KS         0x40000000   /* Supervisor-state protection key */
#define PPC32_SD_KP         0x20000000   /* User-state protection key */
#define PPC32_SD_N          0x10000000   /* No-execute protection bit */
#define PPC32_SD_VSID_MASK  0x00FFFFFF   /* Virtual Segment ID */

/* SDR1 Register */
#define PPC32_SDR1_HTABORG_MASK  0xFFFF0000  /* Physical base address */
#define PPC32_SDR1_HTABEXT_MASK  0x0000E000  /* Extended base address */
#define PPC32_SDR1_HTABMASK      0x000001FF  /* Mask for page table address */
#define PPC32_SDR1_HTMEXT_MASK   0x00001FFF  /* Extended mask */

/* Page Table Entry (PTE) size: 64-bits */
#define PPC32_PTE_SIZE   8

/* PTE entry (Up and Lo) */
#define PPC32_PTEU_V           0x80000000    /* Valid entry */
#define PPC32_PTEU_VSID_MASK   0x7FFFFF80    /* Virtual Segment ID */
#define PPC32_PTEU_VSID_SHIFT  7 
#define PPC32_PTEU_H           0x00000040    /* Hash function */
#define PPC32_PTEU_API_MASK    0x0000003F    /* Abbreviated Page index */
#define PPC32_PTEL_RPN_MASK    0xFFFFF000    /* Physical Page Number */
#define PPC32_PTEL_XPN_MASK    0x00000C00    /* Extended Page Number (0-2) */
#define PPC32_PTEL_XPN_SHIFT   9
#define PPC32_PTEL_R           0x00000100    /* Referenced bit */
#define PPC32_PTEL_C           0x00000080    /* Changed bit */
#define PPC32_PTEL_WIMG_MASK   0x00000078    /* Mem/cache access mode bits */
#define PPC32_PTEL_WIMG_SHIFT  3
#define PPC32_PTEL_X_MASK      0x00000004    /* Extended Page Number (3) */
#define PPC32_PTEL_X_SHIFT     2
#define PPC32_PTEL_PP_MASK     0x00000003    /* Page Protection bits */

/* DSISR register */
#define PPC32_DSISR_NOTRANS    0x40000000    /* No valid translation */
#define PPC32_DSISR_STORE      0x02000000    /* Store operation */

/* PowerPC 405 TLB definitions */
#define PPC405_TLBHI_EPN_MASK    0xFFFFFC00    /* Effective Page Number */
#define PPC405_TLBHI_SIZE_MASK   0x00000380    /* Page Size */
#define PPC405_TLBHI_SIZE_SHIFT  7
#define PPC405_TLBHI_V           0x00000040    /* Valid TLB entry */
#define PPC405_TLBHI_E           0x00000020    /* Endianness */
#define PPC405_TLBHI_U0          0x00000010    /* User-Defined Attribute */

#define PPC405_TLBLO_RPN_MASK    0xFFFFFC00    /* Real Page Number */
#define PPC405_TLBLO_EX          0x00000200    /* Execute Enable */
#define PPC405_TLBLO_WR          0x00000100    /* Write Enable */
#define PPC405_TLBLO_ZSEL_MASK   0x000000F0    /* Zone Select */
#define PPC405_TLBLO_ZSEL_SHIFT  4
#define PPC405_TLBLO_W           0x00000008    /* Write-Through */
#define PPC405_TLBLO_I           0x00000004    /* Caching Inhibited */
#define PPC405_TLBLO_M           0x00000002    /* Memory Coherent */
#define PPC405_TLBLO_G           0x00000001    /* Guarded */

/* BAT type indexes */
enum {
   PPC32_IBAT_IDX = 0,
   PPC32_DBAT_IDX,
};

/* BAT register programming */
struct ppc32_bat_prog {
   int type,index;
   m_uint32_t hi,lo;
};

/* MTS Instruction Cache and Data Cache */
#define PPC32_MTS_ICACHE  PPC32_IBAT_IDX
#define PPC32_MTS_DCACHE  PPC32_DBAT_IDX

#define PPC32_CR_FIELD_OFFSET(f) \
   (OFFSET(cpu_ppc_t,cr_fields)+((f) * sizeof(u_int)))

/* Get the full CR register */
static forced_inline m_uint32_t ppc32_get_cr(cpu_ppc_t *cpu)
{
   m_uint32_t cr = 0;
   int i;

   for(i=0;i<8;i++)
      cr |= cpu->cr_fields[i] << (28 - (i << 2));

   return(cr);
}

/* Set the CR fields given a CR value */
static forced_inline void ppc32_set_cr(cpu_ppc_t *cpu,m_uint32_t cr)
{
   int i;

   for(i=0;i<8;i++)
      cpu->cr_fields[i] = (cr >> (28 - (i << 2))) & 0x0F;
}

/* Get a CR bit */
static forced_inline m_uint32_t ppc32_read_cr_bit(cpu_ppc_t *cpu,u_int bit)
{
   m_uint32_t res;

   res = cpu->cr_fields[ppc32_get_cr_field(bit)] >> ppc32_get_cr_bit(bit);
   return(res & 0x01);
}

/* Set a CR bit */
static forced_inline void ppc32_set_cr_bit(cpu_ppc_t *cpu,u_int bit)
{
   cpu->cr_fields[ppc32_get_cr_field(bit)] |= 1 << ppc32_get_cr_bit(bit);
}

/* Clear a CR bit */
static forced_inline void ppc32_clear_cr_bit(cpu_ppc_t *cpu,u_int bit)
{
   cpu->cr_fields[ppc32_get_cr_field(bit)] &= ~(1 << ppc32_get_cr_bit(bit));
}

/* Reset a PowerPC CPU */
int ppc32_reset(cpu_ppc_t *cpu);

/* Initialize a PowerPC processor */
int ppc32_init(cpu_ppc_t *cpu);

/* Delete a PowerPC processor */
void ppc32_delete(cpu_ppc_t *cpu);

/* Set the processor version register (PVR) */
void ppc32_set_pvr(cpu_ppc_t *cpu,m_uint32_t pvr);

/* Set idle PC value */
void ppc32_set_idle_pc(cpu_gen_t *cpu,m_uint64_t addr);

/* Timer IRQ */
void *ppc32_timer_irq_run(cpu_ppc_t *cpu);

/* Determine an "idling" PC */
int ppc32_get_idling_pc(cpu_gen_t *cpu);

/* Generate an exception */
void ppc32_trigger_exception(cpu_ppc_t *cpu,u_int exc_vector);

/* Trigger the decrementer exception */
void ppc32_trigger_timer_irq(cpu_ppc_t *cpu);

/* Trigger IRQs */
fastcall void ppc32_trigger_irq(cpu_ppc_t *cpu);

/* Virtual breakpoint */
fastcall void ppc32_run_breakpoint(cpu_ppc_t *cpu);

/* Add a virtual breakpoint */
int ppc32_add_breakpoint(cpu_gen_t *cpu,m_uint64_t ia);

/* Remove a virtual breakpoint */
void ppc32_remove_breakpoint(cpu_gen_t *cpu,m_uint64_t ia);

/* Set a register */
void ppc32_reg_set(cpu_gen_t *cpu,u_int reg,m_uint64_t val);

/* Dump registers of a PowerPC processor */
void ppc32_dump_regs(cpu_gen_t *cpu);

/* Dump MMU registers */
void ppc32_dump_mmu(cpu_gen_t *cpu);

/* Load a raw image into the simulated memory */
int ppc32_load_raw_image(cpu_ppc_t *cpu,char *filename,m_uint32_t vaddr);

/* Load an ELF image into the simulated memory */
int ppc32_load_elf_image(cpu_ppc_t *cpu,char *filename,int skip_load,
                         m_uint32_t *entry_point);

/* Run PowerPC code in step-by-step mode */
void *ppc32_exec_run_cpu(cpu_gen_t *gen);

#endif
