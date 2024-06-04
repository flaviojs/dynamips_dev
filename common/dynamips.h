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

/* Symbol */
struct symbol {
   m_uint64_t addr;
   char name[0];
};

/* ROM identification tag */
#define ROM_ID  0x1e94b3df

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

/* Command Line long options */
#define OPT_DISK0_SIZE  0x100
#define OPT_DISK1_SIZE  0x101
#define OPT_EXEC_AREA   0x102
#define OPT_IDLE_PC     0x103
#define OPT_TIMER_ITV   0x104
#define OPT_VM_DEBUG    0x105
#define OPT_IOMEM_SIZE  0x106
#define OPT_SPARSE_MEM  0x107
#define OPT_NOCTRL      0x120
#define OPT_NOTELMSG    0x121
#define OPT_FILEPID     0x122
#define OPT_STARTUP_CONFIG_FILE  0x140
#define OPT_PRIVATE_CONFIG_FILE  0x141
#define OPT_CONSOLE_BINDING_ADDR 0x150

/* Delete all objects */
void dynamips_reset(void);

#endif
