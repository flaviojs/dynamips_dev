//! Extra C symbols.

#include <errno.h>
#include <stdio.h>

void c_errno_set(int x) {
    errno = x;
}
int c_errno(void) {
    return errno;
}

FILE *c_stderr(void) {
    return stderr;
}

FILE *c_stdout(void) {
    return stdout;
}
