//! Required C code for the crate dynamips_c.

#include <stdio.h>

FILE *c_stderr(void) {
    return(stderr);
}
