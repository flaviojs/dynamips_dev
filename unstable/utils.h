/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __UTILS_H__
#define __UTILS_H__

#include "rust_dynamips_c.h"

#include "dynamips_common.h"

#include <sys/time.h>
#include <netinet/in.h>
#include <signal.h>

/* Forward declarations */
typedef struct cpu_gen cpu_gen_t;
typedef struct vm_instance vm_instance_t;
typedef struct vm_platform vm_platform_t;
typedef struct mips64_jit_tcb mips64_jit_tcb_t;
typedef struct ppc32_jit_tcb ppc32_jit_tcb_t;
typedef struct jit_op jit_op_t;
typedef struct cpu_tb cpu_tb_t;
typedef struct cpu_tc cpu_tc_t;

/* Macros for double linked list */
#define M_LIST_ADD(item,head,prefix) \
   do { \
      (item)->prefix##_next  = (head); \
      (item)->prefix##_pprev = &(head); \
      \
      if ((head) != NULL) \
         (head)->prefix##_pprev = &(item)->prefix##_next; \
      \
      (head) = (item); \
   }while(0)

#define M_LIST_REMOVE(item,prefix) \
   do { \
      if ((item)->prefix##_pprev != NULL) { \
         if ((item)->prefix##_next != NULL) \
            (item)->prefix##_next->prefix##_pprev = (item)->prefix##_pprev; \
         \
         *((item)->prefix##_pprev) = (item)->prefix##_next; \
         \
         (item)->prefix##_pprev = NULL; \
         (item)->prefix##_next  = NULL; \
      } \
   }while(0)

/* Global logfile */
extern FILE *log_file;

/* Add an element to a list */
m_list_t *m_list_add(m_list_t **head,void *data);

/* Dynamic sprintf */
char *dyn_sprintf(const char *fmt,...);

/* Split a string */
int m_strsplit(char *str,char delim,char **array,int max_count);

/* Tokenize a string */
int m_strtok(char *str,char delim,char **array,int max_count);

/* Quote a string */
char *m_strquote(char *buffer,size_t buf_len,char *str);

/* Decode from hex. */
int hex_decode(unsigned char *out,const unsigned char *in,int maxlen);

/* Ugly function that dumps a structure in hexa and ascii. */
void mem_dump(FILE *f_output,u_char *pkt,u_int len);

/* Logging function */
void m_flog(FILE *fd,char *module,char *fmt,va_list ap);

/* Logging function */
void m_log(char *module,char *fmt,...);

/* Write an array of string to a logfile */
void m_flog_str_array(FILE *fd,int count,char *str[]);

/* Returns a line from specified file (remove trailing '\n') */
char *m_fgets(char *buffer,int size,FILE *fd);

/* Read a file and returns it in a buffer */
int m_read_file(const char *filename,u_char **buffer,size_t *length);

/* Allocate aligned memory */
void *m_memalign(size_t boundary,size_t size);

/* Block specified signal for calling thread */
int m_signal_block(int sig);

/* Unblock specified signal for calling thread */
int m_signal_unblock(int sig);

/* Set non-blocking mode on a file descriptor */
int m_fd_set_non_block(int fd);

/* Sync a memory zone */
int memzone_sync(void *addr, size_t len);

/* Sync all mappings of a memory zone */
int memzone_sync_all(void *addr, size_t len);

/* Unmap a memory zone */
int memzone_unmap(void *addr, size_t len);

/* Map a memory zone as an executable area */
u_char *memzone_map_exec_area(size_t len);

/* Map a memory zone from a file */
u_char *memzone_map_file(int fd,size_t len);

/* Map a memory zone from a file, with copy-on-write (COW) */
u_char *memzone_map_cow_file(int fd,size_t len);

/* Create a file to serve as a memory zone */
int memzone_create_file(char *filename,size_t len,u_char **ptr);

/* Open a file to serve as a COW memory zone */
int memzone_open_cow_file(char *filename,size_t len,u_char **ptr);

/* Open a file and map it in memory */
int memzone_open_file(char *filename,u_char **ptr,off_t *fsize);
int memzone_open_file_ro(char *filename,u_char **ptr,off_t *fsize);

/* Compute NVRAM checksum */
m_uint16_t nvram_cksum(m_uint16_t *ptr,size_t count);

/* Byte-swap a memory block */
void mem_bswap32(void *ptr,size_t len);

/* Reverse a byte */
m_uint8_t m_reverse_u8(m_uint8_t val);

/* Generate a pseudo random block of data */
void m_randomize_block(m_uint8_t *buf,size_t len);

/* Free an FD pool */
void fd_pool_free(fd_pool_t *pool);

/* Initialize an empty pool */
void fd_pool_init(fd_pool_t *pool);

/* Get a free slot for a FD in a pool */
int fd_pool_get_free_slot(fd_pool_t *pool,int **slot);

/* Fill a FD set and get the maximum FD in order to use with select */
int fd_pool_set_fds(fd_pool_t *pool,fd_set *fds);

/* Send a buffer to all FDs of a pool */
int fd_pool_send(fd_pool_t *pool,void *buffer,size_t len,int flags);

/* Call a function for each FD having incoming data */
int fd_pool_check_input(fd_pool_t *pool,fd_set *fds,
                        void (*cbk)(int *fd_slot,void *opt),void *opt);

/* Equivalent to fprintf, but for a posix fd */
ssize_t fd_printf(int fd,int flags,char *fmt,...);

#endif
