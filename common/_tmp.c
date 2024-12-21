/* XXX stuff that was rustified but still needs a copy in C */

#include "dynamips_c.h"

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
