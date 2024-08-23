/** @file
 * @brief Master Boot Record
 *
 * Based on http://thestarman.pcministry.com/asm/mbr/PartTables.htm
 */

/*
 * Copyright (c) 2014 Flávio J. Saraiva <flaviojs2005@gmail.com>
 */

#ifndef FS_MBR__
#define FS_MBR__

#include "rust_dynamips_c.h"

#include "dynamips_common.h"

void mbr_get_chs(m_uint8_t chs[3], m_uint16_t *cyl, m_uint8_t *head, m_uint8_t *sect);
void mbr_set_chs(m_uint8_t chs[3], m_uint16_t cyl, m_uint8_t head, m_uint8_t sect);
int  mbr_write_fd(int fd, struct mbr_data *mbr);
int  mbr_read_fd(int fd, struct mbr_data *mbr);

#endif
