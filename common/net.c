/*  
 * Copyright (c) 2005,2006 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * Network Utility functions.
 */

#include "utils.h"
#include "net.h"
#include "rust_dynamips_c.h"

/* Analyze L4 for an IP packet */
int pkt_ctx_ip_analyze_l4(n_pkt_ctx_t *ctx)
{
   switch(ctx->ip_l4_proto) {
      case N_IP_PROTO_TCP:
         ctx->flags |= N_PKT_CTX_FLAG_L4_TCP;
         break;
      case N_IP_PROTO_UDP:
         ctx->flags |= N_PKT_CTX_FLAG_L4_UDP;
         break;
      case N_IP_PROTO_ICMP:
         ctx->flags |= N_PKT_CTX_FLAG_L4_ICMP;
         break;
   }

   return(TRUE);
}

/* Analyze a packet */
int pkt_ctx_analyze(n_pkt_ctx_t *ctx,m_uint8_t *pkt,size_t pkt_len)
{
   n_eth_dot1q_hdr_t *eth = (n_eth_dot1q_hdr_t *)pkt;
   m_uint16_t eth_type;
   m_uint8_t *p;

   ctx->pkt = pkt;
   ctx->pkt_len = pkt_len;
   ctx->flags = 0;
   ctx->vlan_id = 0;
   ctx->l3.ptr = NULL;
   ctx->l4.ptr = NULL;

   eth_type = ntohs(eth->type_);
   p = PTR_ADJUST(m_uint8_t *,eth,N_ETH_HLEN);

   if (eth_type >= N_ETH_MTU) {
      if (eth_type == N_ETH_PROTO_DOT1Q) {
         ctx->flags |= N_PKT_CTX_FLAG_VLAN;
         ctx->vlan_id = htons(eth->vlan_id);

         /* override the ethernet type */
         eth_type = ntohs(*(m_uint16_t *)(p+2));

         /* skip 802.1Q header info */
         p += sizeof(m_uint32_t);
      }
   }

   if (eth_type < N_ETH_MTU) {
      /* LLC/SNAP: TODO */
      return(TRUE);
   } else {
      ctx->flags |= N_PKT_CTX_FLAG_ETHV2;
   }

   switch(eth_type) {
      case N_ETH_PROTO_IP: {
         n_ip_hdr_t *ip;
         u_int len,offset;

         ctx->flags |= N_PKT_CTX_FLAG_L3_IP;
         ctx->l3.ip = ip = (n_ip_hdr_t *)p;

         /* Check header */
         if (((ip->ihl & 0xF0) != 0x40) || 
             ((len = ip->ihl & 0x0F) < N_IP_MIN_HLEN) ||
             ((len << 2) > ntohs(ip->tot_len)) || 
             !ip_verify_cksum(ctx->l3.ip))
            return(TRUE);

         ctx->flags |= N_PKT_CTX_FLAG_IPH_OK;
         ctx->ip_l4_proto = ip->proto;
         ctx->l4.ptr = PTR_ADJUST(void *,ip,len << 2);

         /* Check if the packet is a fragment */
         offset = ntohs(ip->frag_off);

         if (((offset & N_IP_OFFMASK) != 0) || (offset & N_IP_FLAG_MF))
            ctx->flags |= N_PKT_CTX_FLAG_IP_FRAG;
         break;
      }

      case N_ETH_PROTO_ARP:
         ctx->flags |= N_PKT_CTX_FLAG_L3_ARP;
         ctx->l3.arp = (n_arp_hdr_t *)p;
         return(TRUE);

      default:
         /* other: unknown, stop now */
         return(TRUE);
   }

   return(TRUE);
}

/* Dump packet context */
void pkt_ctx_dump(n_pkt_ctx_t *ctx)
{
   printf("pkt=%p (len=%lu), flags=0x%8.8x, vlan_id=0x%4.4x, l3=%p, l4=%p\n",
          ctx->pkt,(u_long)ctx->pkt_len,ctx->flags,ctx->vlan_id,
          ctx->l3.ptr,ctx->l4.ptr);
}
