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

/* Receive an ATM cell and process reassembly */
int atm_aal5_recv(struct atm_reas_context *arc,m_uint8_t *cell)
{
   m_uint32_t atm_hdr;

   /* Check buffer boundary */
   if ((arc->buf_pos + ATM_PAYLOAD_SIZE) > ATM_REAS_MAX_SIZE) {
      atm_aal5_recv_reset(arc);
      return(-1);
   }

   /* Get the PTI field: we cannot handle "network" traffic */
   atm_hdr = m_ntoh32(cell);

   if (atm_hdr & ATM_PTI_NETWORK)
      return(2);
   
   /* Copy the payload */
   memcpy(&arc->buffer[arc->buf_pos],&cell[ATM_HDR_SIZE],ATM_PAYLOAD_SIZE);
   arc->buf_pos += ATM_PAYLOAD_SIZE;

   /* 
    * If this is the last cell of the packet, get the real length (the
    * trailer is at the end).
    */
   if (atm_hdr & ATM_PTI_EOP) {
      arc->len = m_ntoh16(&cell[ATM_AAL5_TRAILER_POS+2]);
      return((arc->len <= arc->buf_pos) ? 1 : -2);
   }

   return(0);
}

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
