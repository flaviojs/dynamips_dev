//! C code that is available in rust.

#include <arpa/inet.h>
#include <errno.h>
#include <stdio.h>
#include <time.h>
#include <unistd.h>

/// errno is a C macro or variable and is not available in libc.
void c_errno_set(int x) {
   errno = x;
}

/// errno is a C macro or variable and is not available in libc.
int c_errno(void) {
   return(errno);
}

/// optarg is a C macro or variable and is not available in libc.
char *c_optarg(void) {
   return(optarg);
}

/// opterr is a C macro or variable and is not available in libc.
void c_opterr_set(int x) {
    opterr = x;
}

/// opterr is a C macro or variable and is not available in libc.
int c_opterr(void) {
   return(opterr);
}

/// optind is a C macro or variable and is not available in libc.
int c_optind(void) {
   return(optind);
}

/// INET6_ADDRSTRLEN is a C macro and is not available in libc.
socklen_t c_INET6_ADDRSTRLEN(void) {
    return(INET6_ADDRSTRLEN);
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
