//! Required C code for the crate dynamips_c.

#include <stdio.h>
#include <time.h>

#ifdef __CYGWIN__
#define GET_TIMEZONE _timezone
#else
#define GET_TIMEZONE timezone
#endif

FILE *c_stderr(void) {
    return(stderr);
}

long c_timezone(void) {
    return(GET_TIMEZONE);
}
