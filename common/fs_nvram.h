/** @file
 * @brief Cisco NVRAM filesystem.
 *
 * Format was inferred by analysing the NVRAM data after changing/erasing stuff.
 * All data is big endian.
 *
 * Based on the platforms c1700/c2600/c2692/c3600/c3725/c3745/c7200/c6msfc1.
 */

/*
 * Copyright (c) 2013 Flávio J. Saraiva <flaviojs2005@gmail.com>
 */

#ifndef FS_NVRAM_H__
#define FS_NVRAM_H__

#include "rust_dynamips_c.h"

#include "utils.h"


typedef struct fs_nvram fs_nvram_t;


/* Functions */
fs_nvram_t *fs_nvram_open(u_char *base, size_t len, m_uint32_t addr, u_int flags);
void fs_nvram_close(fs_nvram_t *fs);
int fs_nvram_read_config(fs_nvram_t *fs, u_char **startup_config, size_t *startup_len, u_char **private_config, size_t *private_len);
int fs_nvram_write_config(fs_nvram_t *fs, const u_char *startup_config, size_t startup_len, const u_char *private_config, size_t private_len);
size_t fs_nvram_num_sectors(fs_nvram_t *fs);
// TODO read/write file sectors
int fs_nvram_verify(fs_nvram_t *fs, u_int what);

#endif
