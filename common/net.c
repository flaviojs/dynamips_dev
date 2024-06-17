/*  
 * Copyright (c) 2005,2006 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * Network Utility functions.
 */

#include "utils.h"
#include "net.h"
#include "rust_dynamips_c.h"

#if HAS_RFC2553
/* Set port in an address info structure */
static int ip_socket_set_port(struct sockaddr *addr,int port)
{
   if (!addr)
      return(-1);
   
   switch(addr->sa_family) {
      case AF_INET:
         ((struct sockaddr_in *)addr)->sin_port = htons(port);
         return(0);
                  
      case AF_INET6:
         ((struct sockaddr_in6 *)addr)->sin6_port = htons(port);
         return(0);
         
      default:
         fprintf(stderr,"ip_socket_set_port: unknown address family %d\n",
                 addr->sa_family);
         return(-1);
   }
}

/* Try to create a socket and bind to the specified address info */
static int ip_socket_bind(struct addrinfo *addr)
{
   int fd,off=0;
   
   if ((fd = socket(addr->ai_family,addr->ai_socktype,addr->ai_protocol)) < 0)
      return(-1);
      
#ifdef IPV6_V6ONLY
   if (addr->ai_family == AF_INET6) {
      // if supported, allow packets to/from IPv4-mapped IPv6 addresses
      (void)setsockopt(fd,IPPROTO_IPV6,IPV6_V6ONLY,&off,sizeof(off));
   }
#endif

   if ( (bind(fd,addr->ai_addr,addr->ai_addrlen) < 0) ||
        ((addr->ai_socktype == SOCK_STREAM) && (listen(fd,5) < 0)) )
   {
      close(fd);
      return(-1);
   }

   return(fd);
}

/* Listen on a TCP/UDP port - port is choosen in the specified range */
int ip_listen_range(char *ip_addr,int port_start,int port_end,int *port,
                    int sock_type)
{
   struct addrinfo hints,*res,*res0;
   struct sockaddr_storage st;
   socklen_t st_len;
   char port_str[20],*addr;
   int error,i,fd = -1;

   memset(&hints,0,sizeof(hints));
   hints.ai_family   = PF_UNSPEC;
   hints.ai_socktype = sock_type;
   hints.ai_flags    = AI_PASSIVE;

   snprintf(port_str,sizeof(port_str),"%d",port_start);
   addr = (ip_addr && strlen(ip_addr)) ? ip_addr : NULL;

   if ((error = getaddrinfo(addr,port_str,&hints,&res0)) != 0) {
      fprintf(stderr,"ip_listen_range: %s", gai_strerror(error));
      return(-1);
   }

   for(i=port_start;i<=port_end;i++) {
      for(res=res0;res!=NULL;res=res->ai_next) {
         ip_socket_set_port(res->ai_addr,i);
         
         if ((fd = ip_socket_bind(res)) >= 0) {
            st_len = sizeof(st);
            if (getsockname(fd,(struct sockaddr *)&st,&st_len) != 0) {
               close(fd);
               continue;
            }
            *port = ip_socket_get_port((struct sockaddr *)&st);
            goto done;
         }
      }
   }
   
 done:
   freeaddrinfo(res0);
   return(fd);
}

/* Connect an existing socket to connect to specified host */
int ip_connect_fd(int fd,char *remote_host,int remote_port)
{
   struct addrinfo hints,*res,*res0;
   char port_str[20];
   int error;

   memset(&hints,0,sizeof(hints));
   hints.ai_family = PF_UNSPEC;

   snprintf(port_str,sizeof(port_str),"%d",remote_port);

   if ((error = getaddrinfo(remote_host,port_str,&hints,&res0)) != 0) {
      fprintf(stderr,"%s\n",gai_strerror(error));
      return(-1);
   }

   for(res=res0;res;res=res->ai_next) {
      if ((res->ai_family != PF_INET) && (res->ai_family != PF_INET6))
         continue;

      if (!connect(fd,res->ai_addr,res->ai_addrlen))
         break;
   }

   freeaddrinfo(res0);
   return(0);
}
#else
/* Try to create a socket and bind to the specified address info */
static int ip_socket_bind(struct sockaddr_in *sin,int sock_type)
{
   int fd;
   
   if ((fd = socket(sin->sin_family,sock_type,0)) < 0)
      return(-1);
   
   if ( (bind(fd,(struct sockaddr *)sin,sizeof(*sin)) < 0) ||
        ((sock_type == SOCK_STREAM) && (listen(fd,5) < 0)) )
   {
      close(fd);
      return(-1);
   }
   
   return(fd);
}

/* Listen on a TCP/UDP port - port is choosen in the specified range */
int ip_listen_range(char *ip_addr,int port_start,int port_end,int *port,
                    int sock_type)
{
   struct hostent *hp;
   struct sockaddr_in sin;
   socklen_t len;
   int i,fd;

   memset(&sin,0,sizeof(sin));
   sin.sin_family = PF_INET;
   
   if (ip_addr && strlen(ip_addr)) {
      if (!(hp = gethostbyname(ip_addr))) {
         fprintf(stderr,"ip_listen_range: unable to resolve '%s'\n",ip_addr);
         return(-1);
      }
   
      memcpy(&sin.sin_addr,hp->h_addr_list[0],sizeof(struct in_addr));
   }
      
   for(i=port_start;i<=port_end;i++) {
      sin.sin_port = htons(i);
      
      if ((fd = ip_socket_bind(&sin,sock_type)) >= 0) {
         len = sizeof(sin);
         if (getsockname(fd,(struct sockaddr *)&sin,&len) != 0) {
            close(fd);
            continue;
         }
         *port = ntohs(sin.sin_port);
         return(fd);
      }
   }
   
   return(-1);
}

/* Connect an existing socket to connect to specified host */
int ip_connect_fd(int fd,char *remote_host,int remote_port)
{
   struct sockaddr_in sin;
   struct hostent *hp;
 
   if (!(hp = gethostbyname(remote_host))) {
      fprintf(stderr,"ip_connect_fd: unable to resolve '%s'\n",remote_host);
      return(-1);
   }
   
   /* try to connect to remote host */
   memset(&sin,0,sizeof(sin));
   memcpy(&sin.sin_addr,hp->h_addr_list[0],sizeof(struct in_addr));
   sin.sin_family = PF_INET;
   sin.sin_port   = htons(remote_port);

   return(connect(fd,(struct sockaddr *)&sin,sizeof(sin)));
}
#endif

/* Create a socket UDP listening in a port of specified range */
int udp_listen_range(char *ip_addr,int port_start,int port_end,int *port)
{
   return(ip_listen_range(ip_addr,port_start,port_end,port,SOCK_DGRAM));
}


/* 
 * ISL rewrite.
 *
 * See: http://www.cisco.com/en/US/tech/tk389/tk390/technologies_tech_note09186a0080094665.shtml
 */
void cisco_isl_rewrite(m_uint8_t *pkt,m_uint32_t tot_len)
{
   static m_uint8_t isl_xaddr[N_ETH_ALEN] = { 0x01,0x00,0x0c,0x00,0x10,0x00 };
   u_int real_offset,real_len;
   n_eth_hdr_t *hdr;
   m_uint32_t ifcs;

   hdr = (n_eth_hdr_t *)pkt;
   if (!memcmp(&hdr->daddr,isl_xaddr,N_ETH_ALEN)) {
      real_offset = N_ETH_HLEN + N_ISL_HDR_SIZE;
      real_len    = ntohs(hdr->type);
      real_len    -= (N_ISL_HDR_SIZE + 4);
   
      if ((real_offset+real_len) > tot_len)
         return;
   
      /* Rewrite the destination MAC address */
      hdr->daddr.eth_addr_byte[4] = 0x00;

      /* Compute the internal FCS on the encapsulated packet */
      ifcs = crc32_compute(0xFFFFFFFF,pkt+real_offset,real_len);
      pkt[tot_len-4] = ifcs & 0xff;
      pkt[tot_len-3] = (ifcs >> 8) & 0xff;
      pkt[tot_len-2] = (ifcs >> 16) & 0xff;
      pkt[tot_len-1] = ifcs >> 24;
   }
}

/* Verify checksum of an IP header */
int ip_verify_cksum(n_ip_hdr_t *hdr)
{
   m_uint8_t *p = (m_uint8_t *)hdr;
   m_uint32_t sum = 0;
   u_int len;

   len = (hdr->ihl & 0x0F) << 1;
   while(len-- > 0) {
      sum += ((m_uint16_t)p[0] << 8) | p[1];
      p += sizeof(m_uint16_t);
   }

   while(sum >> 16)
      sum = (sum & 0xFFFF) + (sum >> 16);

   return(sum == 0xFFFF);
}

/* Compute an IP checksum */
void ip_compute_cksum(n_ip_hdr_t *hdr)
{  
   m_uint8_t *p = (m_uint8_t *)hdr;
   m_uint32_t sum = 0;
   u_int len;

   hdr->cksum = 0;

   len = (hdr->ihl & 0x0F) << 1;
   while(len-- > 0) {
      sum += ((m_uint16_t)p[0] << 8) | p[1];
      p += sizeof(m_uint16_t);      
   }

   while(sum >> 16)
      sum = (sum & 0xFFFF) + (sum >> 16);

   hdr->cksum = htons(~sum);
}

/* Partial checksum (for UDP/TCP) */
static inline m_uint32_t ip_cksum_partial(m_uint8_t *buf,int len)
{
   m_uint32_t sum = 0;

   while(len > 1) {
      sum += ((m_uint16_t)buf[0] << 8) | buf[1];
      buf += sizeof(m_uint16_t);
      len -= sizeof(m_uint16_t);
   }

   if (len == 1)
      sum += (m_uint16_t)(*buf) << 8;
   
   return(sum);
}

/* Partial checksum test */
int ip_cksum_partial_test(void)
{
#define N_BUF  4
   m_uint8_t buffer[N_BUF][512];
   m_uint16_t psum[N_BUF];
   m_uint32_t tmp,sum,gsum;
   int i;

   for(i=0;i<N_BUF;i++) {
      m_randomize_block(buffer[i],sizeof(buffer[i]));
      //mem_dump(stdout,buffer[i],sizeof(buffer[i]));

      sum = ip_cksum_partial(buffer[i],sizeof(buffer[i]));

      while(sum >> 16)
         sum = (sum & 0xFFFF) + (sum >> 16);

      psum[i] = ~sum;
   }

   /* partial sums + accumulator */
   for(i=0,tmp=0;i<N_BUF;i++) {
      //printf("psum[%d] = 0x%4.4x\n",i,psum[i]);
      tmp += (m_uint16_t)(~psum[i]);
   }

   /* global sum */
   sum = ip_cksum_partial((m_uint8_t *)buffer,sizeof(buffer));

   while(sum >> 16)
      sum = (sum & 0xFFFF) + (sum >> 16);

   gsum = sum;

   /* accumulator */
   while(tmp >> 16)
      tmp = (tmp & 0xFFFF) + (tmp >> 16);

   //printf("gsum = 0x%4.4x, tmp = 0x%4.4x : %s\n",
   //       gsum,tmp,(gsum == tmp) ? "OK" : "FAILURE");

   return(tmp == gsum);
#undef N_BUF
}

/* Compute TCP/UDP checksum */
m_uint16_t pkt_ctx_tcp_cksum(n_pkt_ctx_t *ctx,int ph)
{
   m_uint32_t sum;
   m_uint16_t old_cksum = 0;
   u_int len;

   /* replace the actual checksum value with 0 to recompute it */
   if (!(ctx->flags & N_PKT_CTX_FLAG_IP_FRAG)) {
      switch(ctx->ip_l4_proto) {
         case N_IP_PROTO_TCP:
            old_cksum = ctx->tcp->cksum;
            ctx->tcp->cksum = 0;
            break;
         case N_IP_PROTO_UDP:
            old_cksum = ctx->udp->cksum;
            ctx->udp->cksum = 0;
            break;
      }
   }

   len = ntohs(ctx->ip->tot_len) - ((ctx->ip->ihl & 0x0F) << 2);
   sum = ip_cksum_partial(ctx->l4,len);
   
   /* include pseudo-header */
   if (ph) {
      sum += ip_cksum_partial((m_uint8_t *)&ctx->ip->saddr,8);
      sum += ctx->ip_l4_proto + len;
   }

   while(sum >> 16)
      sum = (sum & 0xFFFF) + (sum >> 16);

   /* restore the old value */
   if (!(ctx->flags & N_PKT_CTX_FLAG_IP_FRAG)) {
      switch(ctx->ip_l4_proto) {
         case N_IP_PROTO_TCP:
            ctx->tcp->cksum = old_cksum;
            break;
         case N_IP_PROTO_UDP:
            ctx->udp->cksum = old_cksum;
            break;
      }
   }

   return(~sum);
}

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
   ctx->l3 = NULL;
   ctx->l4 = NULL;

   eth_type = ntohs(eth->type);
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
         ctx->ip = ip = (n_ip_hdr_t *)p;

         /* Check header */
         if (((ip->ihl & 0xF0) != 0x40) || 
             ((len = ip->ihl & 0x0F) < N_IP_MIN_HLEN) ||
             ((len << 2) > ntohs(ip->tot_len)) || 
             !ip_verify_cksum(ctx->ip))
            return(TRUE);

         ctx->flags |= N_PKT_CTX_FLAG_IPH_OK;
         ctx->ip_l4_proto = ip->proto;
         ctx->l4 = PTR_ADJUST(void *,ip,len << 2);

         /* Check if the packet is a fragment */
         offset = ntohs(ip->frag_off);

         if (((offset & N_IP_OFFMASK) != 0) || (offset & N_IP_FLAG_MF))
            ctx->flags |= N_PKT_CTX_FLAG_IP_FRAG;
         break;
      }

      case N_ETH_PROTO_ARP:
         ctx->flags |= N_PKT_CTX_FLAG_L3_ARP;
         ctx->arp = (n_arp_hdr_t *)p;
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
          ctx->l3,ctx->l4);
}
