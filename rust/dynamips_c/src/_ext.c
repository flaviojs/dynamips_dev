//! Required C code for the crate dynamips_c.

#include <arpa/inet.h>
#include <errno.h>
#include <stdio.h>
#include <time.h>

#ifdef __CYGWIN__
#define GET_TIMEZONE _timezone
#else
#define GET_TIMEZONE timezone
#endif

int c_errno(void) {
   return(errno);
}

void c_set_errno(int x) {
   errno = x;
}

FILE *c_stderr(void) {
    return(stderr);
}

FILE *c_stdout(void) {
    return(stderr);
}

long c_timezone(void) {
    return(GET_TIMEZONE);
}

socklen_t c_INET6_ADDRSTRLEN(void) {
    return(INET6_ADDRSTRLEN);
}
