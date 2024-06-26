/*  
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot.  All rights reserved.
 *
 * Utility functions.
 */

#include "dynamips_common.h"

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <unistd.h>
#include <time.h>
#include <signal.h>
#include <sys/time.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <fcntl.h>
#include <errno.h>
#include <assert.h>
#ifdef __CYGWIN__
#include <malloc.h>
#endif

#include "utils.h"

/* Global logfile */
FILE *log_file = NULL;

/* Add an element to a list */
m_list_t *m_list_add(m_list_t **head,void *data)
{
   m_list_t *item;

   if ((item = malloc(sizeof(*item))) != NULL) {
      item->data = data;
      item->next = *head;
      *head = item;
   }

   return item;
}

/* Dynamic sprintf */
char *dyn_sprintf(const char *fmt,...)
{
   int n,size = 512;
   va_list ap;
   char *p,*p2;

   if ((p = malloc(size)) == NULL) {
      perror("dyn_sprintf: malloc");
      return NULL;
   }

   for(;;)
   {
      /* Try to print in the allocated space */
      va_start(ap,fmt);
      n = vsnprintf(p,size,fmt,ap);
      va_end(ap);

      /* If that worked, return the string */
      if ((n > -1) && (n < size))
         return p;

      /* Else try again with more space. */
      if (n > -1)
         size = n + 1;
      else
         size *= 2;

      if ((p2 = realloc(p,size)) == NULL) {
         perror("dyn_sprintf: realloc");
         free(p);
         return NULL;
      }

      p = p2;
   }
}

/* Quote a string */
char *m_strquote(char *buffer,size_t buf_len,char *str)
{
   char *p;
   
   if (!(p = strpbrk(str," \t\"'")))
      return str;

   snprintf(buffer,buf_len,"\"%s\"",str);
   return buffer;
}

/* 
 * Decode from hex.
 *
 * hex to raw bytes, returning count of bytes.
 * maxlen limits output buffer size.
 */
int hex_decode(unsigned char *out,const unsigned char *in,int maxlen)
{
#define BAD -1
   static const char hexval[] = {
      BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
      BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
      BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
        0,  1,  2,  3,   4,  5,  6,  7,   8,  9,BAD,BAD, BAD,BAD,BAD,BAD,
      BAD, 10, 11, 12,  13, 14, 15,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
      BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD,
      BAD, 10, 11, 12,  13, 14, 15,BAD, BAD,BAD,BAD,BAD, BAD,BAD,BAD,BAD
   };
   int len = 0;
   int empty = TRUE;

   for(;len < maxlen;) {
      if (*in >= sizeof(hexval) || hexval[*in] == BAD)
         break;

      if (empty) {
         *out = (unsigned char)hexval[*in] << 4;
      }
      else {
         *out |= (unsigned char)hexval[*in];
         ++out;
         ++len;
      }
      ++in;
      empty = !empty;
   }

   return(len);
}

/* Logging function */
void m_flog(FILE *fd,char *module,char *fmt,va_list ap)
{
   struct timespec spec;
   struct tm tmn;

   if (fd != NULL) {
      clock_gettime(CLOCK_REALTIME, &spec);
      gmtime_r(&spec.tv_sec, &tmn);

      // NOTE never use strftime for timestamps, it is crashy
      fprintf(fd,"%d-%02d-%02dT%02d:%02d:%02d.%03dZ %s: ",tmn.tm_year+1900,tmn.tm_mon+1,tmn.tm_mday,tmn.tm_hour,tmn.tm_min,tmn.tm_sec,(int)(spec.tv_nsec/1000000),module);
      vfprintf(fd,fmt,ap);
      fflush(fd);
   }
}

/* Logging function */
void m_log(char *module,char *fmt,...)
{
   va_list ap;

   va_start(ap,fmt);
   m_flog(log_file,module,fmt,ap);
   va_end(ap);
}

/* Write an array of string to a logfile */
void m_flog_str_array(FILE *fd,int count,char *str[])
{
   int i;

   for(i=0;i<count;i++)
      fprintf(fd,"%s ",str[i]);

   fprintf(fd,"\n");
   fflush(fd);
}

/* Read a file and returns it in a buffer */
int m_read_file(const char *filename,u_char **buffer,size_t *length)
{
   FILE *fd;
   long len;

   // open
   fd = fopen(filename,"rb");
   if (fd == NULL) {
      return(-1);
   }

   // len
   fseek(fd, 0, SEEK_END);
   len = ftell(fd);
   fseek(fd, 0, SEEK_SET);
   if (len < 0 || ferror(fd)) {
      fclose(fd);
      return(-1);
   }

   if (length) {
      *length = (size_t)len;
   }

    // data
   if (buffer) {
      *buffer = (u_char *)malloc((size_t)len);
      if (*buffer == NULL || fread(*buffer, (size_t)len, 1, fd) != 1) {
         free(*buffer);
         *buffer = NULL;
         fclose(fd);
         return(-1);
      }
   }

   // close
   fclose(fd);
   return(0);
}

/* Allocate aligned memory */
void *m_memalign(size_t boundary,size_t size)
{
   void *p;

#if defined(__linux__) || HAS_POSIX_MEMALIGN
   if (posix_memalign((void *)&p,boundary,size))
#else
#if defined(__CYGWIN__) || defined(SUNOS)
   if (!(p = memalign(boundary,size)))
#else
   if (!(p = malloc(size)))    
#endif
#endif
      return NULL;

   assert(((m_iptr_t)p & (boundary-1)) == 0);
   return p;
}

/* Unblock specified signal for calling thread */
int m_signal_unblock(int sig)
{
   sigset_t sig_mask;
   sigemptyset(&sig_mask);
   sigaddset(&sig_mask,sig);
   return(pthread_sigmask(SIG_UNBLOCK,&sig_mask,NULL));
}

/* Sync a memory zone */
int memzone_sync(void *addr, size_t len)
{
   return(msync(addr, len, MS_SYNC));
}

/* Sync all mappings of a memory zone */
int memzone_sync_all(void *addr, size_t len)
{
   return(msync(addr, len, MS_SYNC | MS_INVALIDATE));
}

/* Unmap a memory zone */
int memzone_unmap(void *addr, size_t len)
{
   return(munmap(addr, len));
}

/* Return a memory zone or NULL on error */
static void *mmap_or_null(void *addr, size_t length, int prot, int flags, int fd, off_t offset)
{
   void *ptr;

   if ((ptr = mmap(addr, length, prot, flags, fd, offset)) == MAP_FAILED)
      return(NULL);

   return(ptr);
}

/* Map a memory zone as an executable area */
u_char *memzone_map_exec_area(size_t len)
{
   return(mmap_or_null(NULL,len,PROT_EXEC|PROT_READ|PROT_WRITE,MAP_SHARED|MAP_ANONYMOUS,-1,(off_t)0));
}

/* Map a memory zone from a file */
u_char *memzone_map_file(int fd,size_t len)
{
   return(mmap_or_null(NULL,len,PROT_READ|PROT_WRITE,MAP_SHARED,fd,(off_t)0));
}

/* Map a memory zone from a ro file */
u_char *memzone_map_file_ro(int fd,size_t len)
{
   return(mmap_or_null(NULL,len,PROT_READ,MAP_PRIVATE,fd,(off_t)0));
}

/* Map a memory zone from a file, with copy-on-write (COW) */
u_char *memzone_map_cow_file(int fd,size_t len)
{
   return(mmap_or_null(NULL,len,PROT_READ|PROT_WRITE,MAP_PRIVATE,fd,(off_t)0));
}

/* Create a file to serve as a memory zone */
int memzone_create_file(char *filename,size_t len,u_char **ptr)
{
   int fd;

   if ((fd = open(filename,O_CREAT|O_RDWR,S_IRWXU)) == -1) {
      perror("memzone_create_file: open");
      return(-1);
   }

   if (ftruncate(fd,len) == -1) {
      perror("memzone_create_file: ftruncate");
      close(fd);
      return(-1);
   }

   *ptr = memzone_map_file(fd,len);

   if (!*ptr) {
      close(fd);
      fd = -1;
   }

   return(fd);
}

/* Open a file to serve as a COW memory zone */
int memzone_open_cow_file(char *filename,size_t len,u_char **ptr)
{
   int fd;

   if ((fd = open(filename,O_RDONLY,S_IRWXU)) == -1) {
      perror("memzone_open_file: open");
      return(-1);
   }

   *ptr = memzone_map_cow_file(fd,len);

   if (!*ptr) {
      close(fd);
      fd = -1;
   }

   return(fd);
}

/* Open a file and map it in memory */
int memzone_open_file(char *filename,u_char **ptr,off_t *fsize)
{
   struct stat fprop;
   int fd;

   if ((fd = open(filename,O_RDWR,S_IRWXU)) == -1)
      return(-1);

   if (fstat(fd,&fprop) == -1)
      goto err_fstat;

   *fsize = fprop.st_size;
   if (!(*ptr = memzone_map_file(fd,*fsize)))
      goto err_mmap;
   
   return(fd);

 err_mmap:   
 err_fstat:
   close(fd);
   return(-1);
}

int memzone_open_file_ro(char *filename,u_char **ptr,off_t *fsize)
{
   struct stat fprop;
   int fd;

   if ((fd = open(filename,O_RDONLY,S_IRWXU)) == -1)
      return(-1);

   if (fstat(fd,&fprop) == -1)
      goto err_fstat;

   *fsize = fprop.st_size;
   if (!(*ptr = memzone_map_file_ro(fd,*fsize)))
      goto err_mmap;
   
   return(fd);

 err_mmap:   
 err_fstat:
   close(fd);
   return(-1);
}

/* Compute NVRAM checksum */
m_uint16_t nvram_cksum(m_uint16_t *ptr,size_t count) 
{
   m_uint32_t sum = 0;

   while(count > 1) {
      sum = sum + ntohs(*ptr);
      ptr++;
      count -= sizeof(m_uint16_t);
   }

   if (count > 0) 
      sum = sum + ((ntohs(*ptr) & 0xFF) << 8); 

   while(sum>>16)
      sum = (sum & 0xffff) + (sum >> 16);

   return(~sum);
}

/* Byte-swap a memory block */
void mem_bswap32(void *ptr,size_t len)
{
   m_uint32_t *p _not_aligned = ptr;
   size_t count = len >> 2;
   int i;

   for(i=0;i<count;i++,p++)
      *p = swap32(*p);
}

/* Reverse a byte */
m_uint8_t m_reverse_u8(m_uint8_t val)
{
   m_uint8_t res = 0;
   int i;

   for(i=0;i<8;i++)
      if (val & (1 << i))
         res |= 1 << (7 - i);
   
   return(res);
}

/* Equivalent to fprintf, but for a posix fd */
ssize_t fd_printf(int fd,int flags,char *fmt,...)
{
   char buffer[2048];
   va_list ap;
    
   va_start(ap,fmt);
   vsnprintf(buffer,sizeof(buffer),fmt,ap);
   va_end(ap);
   
   return(send(fd,buffer,strlen(buffer),flags));
}
