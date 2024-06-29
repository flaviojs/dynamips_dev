/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * ATM definitions.
 */

#ifndef __ATM_H__
#define __ATM_H__

#include "rust_dynamips_c.h"

#include <pthread.h>

#include "utils.h"

#define ATMSW_LOCK(t)   pthread_mutex_lock(&(t)->lock)
#define ATMSW_UNLOCK(t) pthread_mutex_unlock(&(t)->lock)

/* Compute HEC field for ATM header */
m_uint8_t atm_compute_hec(m_uint8_t *cell_header);

/* Insert HEC field into an ATM header */
void atm_insert_hec(m_uint8_t *cell_header);

/* Update the CRC on the data block one byte at a time */
m_uint32_t atm_update_crc(m_uint32_t crc_accum,m_uint8_t *ptr,int len);

/* Initialize ATM code (for HEC checksums) */
void atm_init(void);

/* Acquire a reference to an ATM switch (increment reference count) */
atmsw_table_t *atmsw_acquire(char *name);

/* Release an ATM switch (decrement reference count) */
int atmsw_release(char *name);

/* Create a virtual switch table */
atmsw_table_t *atmsw_create_table(char *name);

/* Create a VP switch connection */
int atmsw_create_vpc(atmsw_table_t *t,char *nio_input,u_int vpi_in,
                     char *nio_output,u_int vpi_out);

/* Delete a VP switch connection */
int atmsw_delete_vpc(atmsw_table_t *t,char *nio_input,u_int vpi_in,
                     char *nio_output,u_int vpi_out);

/* Create a VC switch connection */
int atmsw_create_vcc(atmsw_table_t *t,
                     char *input,u_int vpi_in,u_int vci_in,
                     char *output,u_int vpi_out,u_int vci_out);

/* Delete a VC switch connection */
int atmsw_delete_vcc(atmsw_table_t *t,
                     char *nio_input,u_int vpi_in,u_int vci_in,
                     char *nio_output,u_int vpi_out,u_int vci_out);

/* Save the configuration of an ATM switch */
void atmsw_save_config(atmsw_table_t *t,FILE *fd);

/* Save configurations of all ATM switches */
void atmsw_save_config_all(FILE *fd);

/* Delete an ATM switch */
int atmsw_delete(char *name);

/* Delete all ATM switches */
int atmsw_delete_all(void);

/* Start a virtual ATM switch */
int atmsw_start(char *filename);

#endif
