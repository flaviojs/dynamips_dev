/** @file
 * @brief FAT filesystem.
 *
 * Based on http://www.win.tue.nl/~aeb/linux/fs/fat/fat-1.html
 */

/*
 * Copyright (c) 2014 Flávio J. Saraiva <flaviojs2005@gmail.com>
 */

#include "rust_dynamips_c.h"

#include "fs_fat.h"

/*
 */

struct fat16_data {
   const char *volume_name;
   m_uint32_t  volume_sectors;
   m_uint16_t  reserved_sectors;
   m_uint16_t  root_entry_count;
   m_uint16_t  fat_sectors;
   m_uint16_t  sects_per_track;
   m_uint16_t  heads;
   m_uint8_t   sects_per_cluster;
   m_uint8_t   nr_fats;
};

struct sec_per_clus_table
{
    m_uint32_t  sectors;
    m_uint8_t   sectors_per_cluster;
};

static struct {
   m_uint32_t  sectors;
   m_uint8_t   sects_per_cluster;
} cluster_size_table16[] = {
   { 32680, 2},    /* 16MB - 1K */
   { 262144, 4},   /* 128MB - 2K */
   { 524288, 8},   /* 256MB - 4K */
   { 1048576, 16}, /* 512MB - 8K */
   { 2097152, 32}, /* 1GB - 16K */
   { 4194304, 64}, /* 2GB - 32K */
   { 8388608, 128},/* 2GB - 64K (not supported on some systems) */
   { 0 , 0 }       /* done */
};

static inline void set_u32(m_uint8_t *p, size_t i, m_uint32_t v) {
   p[i+0] = (m_uint8_t)((v>>0)&0xFF);
   p[i+1] = (m_uint8_t)((v>>8)&0xFF);
   p[i+2] = (m_uint8_t)((v>>16)&0xFF);
   p[i+3] = (m_uint8_t)((v>>24)&0xFF);
}

static inline void set_u16(m_uint8_t *p, size_t i, m_uint16_t v) {
   p[i+0] = (m_uint8_t)((v>>0)&0xFF);
   p[i+1] = (m_uint8_t)((v>>8)&0xFF);
}

static void boot16(m_uint8_t *sector, struct fat16_data *fat16)
{
   int i;

   memset(sector, 0x00, FS_FAT_SECTOR_SIZE);

   /* start of boot program */
   sector[0x0] = 0xEB;// jmp 0x3E
   sector[0x1] = 0x3C;
   sector[0x2] = 0x90;// nop

   /* OEM string */
   sector[0x3] = 'D';
   sector[0x4] = 'Y';
   sector[0x5] = 'N';
   sector[0x6] = 'A';
   sector[0x7] = 'M';
   sector[0x8] = 'I';
   sector[0x9] = 'P';
   sector[0xA] = 'S';

   // Bytes per sector
   set_u16(sector,0xB,FS_FAT_SECTOR_SIZE);

   // Sectors per cluster
   sector[0xD] = fat16->sects_per_cluster;

   // Reserved Sectors
   set_u16(sector,0xE,fat16->reserved_sectors);

   // Number of FATS
   sector[0x10] = fat16->nr_fats;

   // Max entries in root dir (FAT16 only)
   set_u16(sector,0x11,fat16->root_entry_count);

   // [FAT16] Total sectors (use FAT32 count instead)
   set_u16(sector,0x13,0x0000);

   // Media type (Fixed Disk)
   sector[0x15] = 0xF8;

   // FAT16 Bootstrap Details

   // Count of sectors used by the FAT table (FAT16 only)
   set_u16(sector,0x16,fat16->fat_sectors);

   // Sectors per track
   set_u16(sector,0x18,fat16->sects_per_track);

   // Heads
   set_u16(sector,0x1A,fat16->heads);

   // Hidden sectors
   set_u16(sector,0x1C,0x0000);

   // Total sectors for this volume
   set_u32(sector,0x20,fat16->volume_sectors);

   // Drive number (1st Hard Disk)
   sector[0x24] = 0x80;

   // Reserved
   sector[0x25] = 0x00;

   // Boot signature
   sector[0x26] = 0x29;

   // Volume ID
   sector[0x27] = (rand()&0xFF);
   sector[0x28] = (rand()&0xFF);
   sector[0x29] = (rand()&0xFF);
   sector[0x2A] = (rand()&0xFF);

   // Volume name
   for (i = 0; i < 11 && fat16->volume_name[i]; i++) {
      sector[i+0x2B] = fat16->volume_name[i];
   }
   for (; i < 11; i++) {
      sector[i+0x2B] = ' ';
   }

   // File sys type
   sector[0x36] = 'F';
   sector[0x37] = 'A';
   sector[0x38] = 'T';
   sector[0x39] = '1';
   sector[0x3A] = '6';
   sector[0x3B] = ' ';
   sector[0x3C] = ' ';
   sector[0x3D] = ' ';
   
   /* boot program (empty) */

   /* Signature */
   sector[0x1FE] = 0x55;
   sector[0x1FF] = 0xAA;
}

static void fat16_first(m_uint8_t *sector, struct fat16_data *fat16)
{
   memset(sector, 0x00, FS_FAT_SECTOR_SIZE);

   // Initialise default allocate / reserved clusters
   set_u16(sector,0x0,0xFFF8);
   set_u16(sector,0x2,0xFFFF);
}

static void fat16_empty(m_uint8_t *sector, struct fat16_data *fat16)
{
   memset(sector, 0x00, FS_FAT_SECTOR_SIZE);
}

static int write_sector(int fd, m_uint32_t lba, m_uint8_t *sector)
{
   off_t offset;

   errno = 0;
   offset = (off_t)lba * FS_FAT_SECTOR_SIZE;
   if (lseek(fd,offset,SEEK_SET) != offset) {
      perror("write_sector(fs_fat): lseek");
      return(-1);
   }

   if (write(fd, sector, FS_FAT_SECTOR_SIZE) != FS_FAT_SECTOR_SIZE) {
      perror("write_sector(fs_fat): write");
      return(-1);
   }

   return(0);
}
