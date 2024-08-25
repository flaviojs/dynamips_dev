/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 *
 * Network I/O Layer.
 */

#ifndef __NET_IO_H__
#define __NET_IO_H__

#include "rust_dynamips_c.h"

#include <sys/types.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <pthread.h>

#include "utils.h"

#ifdef GEN_ETH
#include "gen_eth.h"
#endif

/* Get NETIO type given a description */
int netio_get_type(char *type);

/* Show the NETIO types */
void netio_show_types(void);

/* Create a new NetIO descriptor */
netio_desc_t *netio_desc_create_unix(char *nio_name,char *local,char *remote);

/* Create a new NetIO descriptor with VDE method */
netio_desc_t *netio_desc_create_vde(char *nio_name,char *control,char *local);

/* Create a new NetIO descriptor with TAP method */
netio_desc_t *netio_desc_create_tap(char *nio_name,char *tap_name);

/* Create a new NetIO descriptor with TCP_CLI method */
netio_desc_t *netio_desc_create_tcp_cli(char *nio_name,char *addr,char *port);

/* Create a new NetIO descriptor with TCP_SER method */
netio_desc_t *netio_desc_create_tcp_ser(char *nio_name,char *port);

/* Create a new NetIO descriptor with UDP method */
netio_desc_t *netio_desc_create_udp(char *nio_name,int local_port,
                                    char *remote_host,int remote_port);

/* Get local port */
int netio_udp_auto_get_local_port(netio_desc_t *nio);

/* Connect to a remote host/port */
int netio_udp_auto_connect(netio_desc_t *nio,char *host,int port);

/* Create a new NetIO descriptor with auto UDP method */
netio_desc_t *netio_desc_create_udp_auto(char *nio_name,char *local_addr,
                                         int port_start,int port_end);

#ifdef LINUX_ETH
/* Create a new NetIO descriptor with raw Ethernet method */
netio_desc_t *netio_desc_create_lnxeth(char *nio_name,char *dev_name);
#endif

#ifdef GEN_ETH
/* Create a new NetIO descriptor with generic raw Ethernet method */
netio_desc_t *netio_desc_create_geneth(char *nio_name,char *dev_name);
#endif

/* Establish a cross-connect between two FIFO NetIO */
int netio_fifo_crossconnect(netio_desc_t *a,netio_desc_t *b);

/* Create a new NetIO descriptor with FIFO method */
netio_desc_t *netio_desc_create_fifo(char *nio_name);

/* Create a new NetIO descriptor with NULL method */
netio_desc_t *netio_desc_create_null(char *nio_name);

/* Acquire a reference to NIO from registry (increment reference count) */
netio_desc_t *netio_acquire(char *name);

/* Release an NIO (decrement reference count) */
int netio_release(char *name);

/* Delete a NetIO descriptor */
int netio_delete(char *name);

/* Delete all NetIO descriptors */
int netio_delete_all(void);

/* Save the configuration of a NetIO descriptor */
void netio_save_config(netio_desc_t *nio,FILE *fd);

/* Save configurations of all NetIO descriptors */
void netio_save_config_all(FILE *fd);

/* Send a packet through a NetIO descriptor */
ssize_t netio_send(netio_desc_t *nio,void *pkt,size_t len);

/* Receive a packet through a NetIO descriptor */
ssize_t netio_recv(netio_desc_t *nio,void *pkt,size_t max_len);

/* Get a NetIO FD */
int netio_get_fd(netio_desc_t *nio);

/* Reset NIO statistics */
void netio_reset_stats(netio_desc_t *nio);

/* Indicate if a NetIO can transmit a packet */
int netio_can_transmit(netio_desc_t *nio);

/* Update bandwidth counter */
void netio_update_bw_stat(netio_desc_t *nio,m_uint64_t bytes);

/* Reset NIO bandwidth counter */
void netio_clear_bw_stat(netio_desc_t *nio);

/* Set the bandwidth constraint */
void netio_set_bandwidth(netio_desc_t *nio,u_int bandwidth);

/* Enable a RX listener */
int netio_rxl_enable(netio_desc_t *nio);

/* Add an RX listener in the listener list */
int netio_rxl_add(netio_desc_t *nio,netio_rx_handler_t rx_handler,
                  void *arg1,void *arg2);

/* Remove a NIO from the listener list */
int netio_rxl_remove(netio_desc_t *nio);

/* Initialize the RXL thread */
int netio_rxl_init(void);

#endif
