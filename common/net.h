/*
 * Copyright (c) 2006 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * net.h: Protocol Headers and Constants Definitions.
 */

#ifndef __NET_H__
#define __NET_H__  1

#include "rust_dynamips_c.h"

#include "dynamips_common.h"

#include <arpa/inet.h>
#include <netdb.h>
/* TODO missing in MinGW */
#include <sys/socket.h>
#include <netinet/in.h>

/* Initialize IPv6 masks */
void ipv6_init_masks(void);

/* Convert an IPv4 address into a string */
char *n_ip_ntoa(char *buffer,n_ip_addr_t ip_addr);

/* Convert in IPv6 address into a string */
char *n_ipv6_ntoa(char *buffer,n_ipv6_addr_t *ipv6_addr);

/* Convert a string containing an IP address in binary */
int n_ip_aton(n_ip_addr_t *ip_addr,char *ip_str);

/* Convert an IPv6 address from string into binary */
int n_ipv6_aton(n_ipv6_addr_t *ipv6_addr,char *ip_str);

/* Parse an IPv4 CIDR prefix */
int ip_parse_cidr(char *token,n_ip_addr_t *net_addr,n_ip_addr_t *net_mask);

/* Parse an IPv6 CIDR prefix */
int ipv6_parse_cidr(char *token,n_ipv6_addr_t *net_addr,u_int *net_mask);

/* Parse a MAC address */
int parse_mac_addr(n_eth_addr_t *addr,char *str);

/* Parse a board id */
int parse_board_id(m_uint8_t * buf,const char *id,int encode);

/* Convert an Ethernet address into a string */
char *n_eth_ntoa(char *buffer,n_eth_addr_t *addr,int format);

/* Create a new socket to connect to specified host */
int udp_connect(int local_port,char *remote_host,int remote_port);

/* Listen on the specified port */
int ip_listen(char *ip_addr,int port,int sock_type,int max_fd,int fd_array[]);

/* Listen on a TCP/UDP port - port is choosen in the specified rnaage */
int ip_listen_range(char *ip_addr,int port_start,int port_end,int *port,
                    int sock_type);

/* Create a socket UDP listening in a port of specified range */
int udp_listen_range(char *ip_addr,int port_start,int port_end,int *port);

/* Connect an existing socket to connect to specified host */
int ip_connect_fd(int fd,char *remote_host,int remote_port);

/* ISL rewrite */
void cisco_isl_rewrite(m_uint8_t *pkt,m_uint32_t tot_len);

/* Verify checksum of an IP header */
int ip_verify_cksum(n_ip_hdr_t *hdr);

/* Compute an IP checksum */
void ip_compute_cksum(n_ip_hdr_t *hdr);

/* Compute TCP/UDP checksum */
m_uint16_t pkt_ctx_tcp_cksum(n_pkt_ctx_t *ctx,int ph);

/* Analyze L4 for an IP packet */
int pkt_ctx_ip_analyze_l4(n_pkt_ctx_t *ctx);

/* Analyze a packet */
int pkt_ctx_analyze(n_pkt_ctx_t *ctx,m_uint8_t *pkt,size_t pkt_len);

/* Dump packet context */
void pkt_ctx_dump(n_pkt_ctx_t *ctx);

#endif
