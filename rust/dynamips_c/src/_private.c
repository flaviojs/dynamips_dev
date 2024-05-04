//! C code that is available in rust.

#include <arpa/inet.h>
#include <errno.h>
#include <stdio.h>
#include <time.h>

/// errno is a C macro or variable and is not available in libc.
int c_errno(void) {
   return(errno);
}

/// INET6_ADDRSTRLEN is a C macro and is not available in libc.
socklen_t c_INET6_ADDRSTRLEN(void) {
    return(INET6_ADDRSTRLEN);
}

/// errno is a C macro or variable and is not available in libc.
void c_set_errno(int x) {
   errno = x;
}

/// stderr is a C macro and is not available in libc.
FILE *c_stderr(void) {
    return(stderr);
}

/// stdour is a C macro and is not available in libc.
FILE *c_stdout(void) {
    return(stderr);
}

/// timezone is a C extern variable that is not available in libc. 
long c_timezone(void) {
#ifdef __CYGWIN__
#define GET_TIMEZONE _timezone
#else
#define GET_TIMEZONE timezone
#endif
    return(GET_TIMEZONE);
}
