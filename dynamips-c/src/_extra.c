//! Extra C symbols.

#include <stdio.h>

FILE *c_stderr(void) {
    return stderr;
}

FILE *c_stdout(void) {
    return stdout;
}
