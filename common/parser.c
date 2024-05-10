/*
 * Cisco router simulation platform.
 * Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
 *
 * Mini-parser.
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <signal.h>
#include <fcntl.h>
#include <errno.h>
#include <assert.h>
#include <stdarg.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <arpa/inet.h>

#include "utils.h"
#include "parser.h"

/* Parser tests */
static char *parser_test_str[] = {
   "c7200 show_hardware R1",
   "c7200 show_hardware \"R1\"",
   "   c7200    show_hardware   \"R1\"    ",
   "\"c7200\" \"show_hardware\" \"R1\"",
   "hypervisor set_working_dir \"C:\\Program Files\\Dynamips Test\"",
   "hypervisor # This is a comment set_working_dir \"C:\\Program Files\"",
   "\"c7200\" \"show_hardware\" \"R1",
   NULL,
};

void parser_run_tests(void)
{
   parser_context_t ctx;
   int i,res;

   for(i=0;parser_test_str[i];i++) {
      parser_context_init(&ctx);

      res = parser_scan_buffer(&ctx,parser_test_str[i],
                               strlen(parser_test_str[i])+1);

      printf("\n%d: Test string: [%s] => res=%d, state=%d\n",
             i,parser_test_str[i],res,ctx.state);
      
      if ((res != 0) && (ctx.error == 0)) {
         if (ctx.tok_head) {
            printf("Tokens: ");
            parser_dump_tokens(&ctx);
            printf("\n");
         }
      }

      parser_context_free(&ctx);
   }
}
