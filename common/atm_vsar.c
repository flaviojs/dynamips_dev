/*
 * Cisco router simulation platform.
 * Copyright (c) 2007 Christophe Fillot (cf@utc.fr)
 *
 * ATM Virtual Segmentation & Reassembly Engine.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <errno.h>
#include <sys/select.h>
#include <sys/time.h>
#include <sys/types.h>
#include <sys/uio.h>

#include "utils.h"
#include "rust_dynamips_c.h"
#include "atm_vsar.h"

/* Send a packet through a rfc1483 bridge encap */
int atm_aal5_send_rfc1483b(netio_desc_t *nio,u_int vpi,u_int vci,
                           void *pkt,size_t len)
{
   struct iovec vec[2];

   vec[0].iov_base = (void *)atm_rfc1483b_header;
   vec[0].iov_len  = ATM_RFC1483B_HLEN;
   vec[1].iov_base = pkt;
   vec[1].iov_len  = len;

   return(atm_aal5_send(nio,vpi,vci,vec,2));
}
