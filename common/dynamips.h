/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 * Patched by Jeremy Grossmann for the GNS3 project (www.gns3.net)
 */

#ifndef __DYNAMIPS_H__
#define __DYNAMIPS_H__

#include "rust_dynamips_c.h"

#include <libelf.h>

#include "utils.h"

/* Global log file */
extern FILE *log_file;

/* Operating system name */
extern const char *os_name;

/* Software version */
extern const char *sw_version;

/* Software version specific tag */
extern const char *sw_version_tag;

/* Global binding address */
extern char *binding_addr;

/* Global console (vtty tcp) binding address */
extern char *console_binding_addr;

/* Delete all objects */
void dynamips_reset(void);

#endif
