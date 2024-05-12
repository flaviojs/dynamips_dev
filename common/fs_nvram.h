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

#include "utils.h"

#include "rust_dynamips_c.h"


///////////////////////////////////////////////////////////


/** Header of the NVRAM filesystem.
 * When empty, only this magic and the checksum are filled.
 * @see nvram_header_startup_config
 * @see nvram_header_private_config
 */
struct fs_nvram_header {
   /** Padding. */
   u_char   padding[6];

   /** Magic value 0xF0A5. */
   m_uint16_t   magic;
   // Following data:
   //  - nvram_header_startup_config
   //  - startup-config data
   //  - padding to align the next header to a multiple of 4
   //  - nvram_header_private_config
   //  - private-config data
   //  - padding till end of sector
   //  - the next 2 sectors are reserved for expansion of config files
   //  - the rest of sectors are for normal files
} __attribute__((__packed__));


/** Header of special file startup-config.
 * @see nvram_header
 */
struct fs_nvram_header_startup_config {
   /** Magic value 0xABCD. */
   m_uint16_t   magic;

   /** Format of the data.
    * 0x0001 - raw data;
    * 0x0002 - .Z compressed (12 bits);
    */
   m_uint16_t   format;

   /** Checksum of filesystem data. (all data after the filesystem magic) */
   m_uint16_t   checksum;

   /** 0x0C04 - maybe maximum amount of free space that will be reserved? */
   m_uint16_t   unk1;

   /** Address of the data. */
   m_uint32_t   start;

   /** Address right after the data. */
   m_uint32_t   end;

   /** Length of block.  */
   m_uint32_t   len;

   /** 0x00000000 */
   m_uint32_t   unk2;

   /** 0x00000000 if raw data, 0x00000001 if compressed */
   m_uint32_t   unk3;

   /** 0x0000 if raw data, 0x0001 if compressed */
   m_uint16_t   unk4;

   /** 0x0000 */
   m_uint16_t   unk5;

   /** Length of uncompressed data, 0 if raw data. */
   m_uint32_t   uncompressed_len;

   // startup-config data comes after this header
} __attribute__((__packed__));


/** Header of special file private-config.
 * @see nvram_header
 */
struct fs_nvram_header_private_config {
   /** Magic value 0xFEDC. */
   m_uint16_t   magic;

   /** Format of the file.
    * 0x0001 - raw data;
    */
   m_uint16_t   format;

   /** Address of the data. */
   m_uint32_t   start;

   /** Address right after the data. */
   m_uint32_t   end;

   /** Length of block.  */
   m_uint32_t   len;

   // private-config data comes after this header
} __attribute__((__packed__));


/** Sector containing file data. */
struct fs_nvram_file_sector {
   /** Magic value 0xDCBA */
   m_uint16_t   magic;

   /** Next sector with data, 0 by default */
   m_uint16_t   next_sector;

   /** Flags.
    * @see FS_NVRAM_FLAG_FILE_START
    * @see FS_NVRAM_FLAG_FILE_END
    * @see FS_NVRAM_FLAG_FILE_NO_RW
    */
   m_uint16_t   flags;

   /** Amount of data in this sector. */
   m_uint16_t   length;

   /** File name, always NUL-terminated. */
   char         filename[24];

   /** File data. */
   u_char       data[992];
} __attribute__((__packed__));


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
