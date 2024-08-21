/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __MEMORY_H__
#define __MEMORY_H__

#include "rust_dynamips_c.h"

#include <sys/types.h>
#include "utils.h"

/* Record a memory access */
void memlog_rec_access(cpu_gen_t *cpu,m_uint64_t vaddr,m_uint64_t data,
                       m_uint32_t op_size,m_uint32_t op_type);

/* Show the last memory accesses */
void memlog_dump(cpu_gen_t *cpu);

/* Update the data obtained by a read access */
void memlog_update_read(cpu_gen_t *cpu,m_iptr_t raddr);

/* Copy a memory block from VM physical RAM to real host */
void physmem_copy_from_vm(vm_instance_t *vm,void *real_buffer,
                          m_uint64_t paddr,size_t len);

/* Copy a memory block to VM physical RAM from real host */
void physmem_copy_to_vm(vm_instance_t *vm,void *real_buffer,
                        m_uint64_t paddr,size_t len);

/* Copy a 32-bit word from the VM physical RAM to real host */
m_uint32_t physmem_copy_u32_from_vm(vm_instance_t *vm,m_uint64_t paddr);

/* Copy a 32-bit word to the VM physical RAM from real host */
void physmem_copy_u32_to_vm(vm_instance_t *vm,m_uint64_t paddr,m_uint32_t val);

/* Copy a 16-bit word from the VM physical RAM to real host */
m_uint16_t physmem_copy_u16_from_vm(vm_instance_t *vm,m_uint64_t paddr);

/* Copy a 16-bit word to the VM physical RAM from real host */
void physmem_copy_u16_to_vm(vm_instance_t *vm,m_uint64_t paddr,m_uint16_t val);

/* Copy a byte from the VM physical RAM to real host */
m_uint8_t physmem_copy_u8_from_vm(vm_instance_t *vm,m_uint64_t paddr);

/* Copy a 16-bit word to the VM physical RAM from real host */
void physmem_copy_u8_to_vm(vm_instance_t *vm,m_uint64_t paddr,m_uint8_t val);

/* DMA transfer operation */
void physmem_dma_transfer(vm_instance_t *vm,m_uint64_t src,m_uint64_t dst,
                          size_t len);

/* strlen in VM physical memory */
size_t physmem_strlen(vm_instance_t *vm,m_uint64_t paddr);

/* find sequence of bytes in VM cacheable physical memory interval [first,last] */
int physmem_cfind(vm_instance_t *vm,m_uint8_t *bytes,size_t len,
                 m_uint64_t first,m_uint64_t last, m_uint64_t *paddr);

/* Physical memory dump (32-bit words) */
void physmem_dump_vm(vm_instance_t *vm,m_uint64_t paddr,m_uint32_t u32_count);

#endif
