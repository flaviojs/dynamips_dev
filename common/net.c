/*  
 * Copyright (c) 2005,2006 Christophe Fillot.
 * E-mail: cf@utc.fr
 *
 * Network Utility functions.
 */

#include "utils.h"
#include "net.h"
#include "rust_dynamips_c.h"

/* Dump packet context */
void pkt_ctx_dump(n_pkt_ctx_t *ctx)
{
   printf("pkt=%p (len=%lu), flags=0x%8.8x, vlan_id=0x%4.4x, l3=%p, l4=%p\n",
          ctx->pkt,(u_long)ctx->pkt_len,ctx->flags,ctx->vlan_id,
          ctx->l3.ptr,ctx->l4.ptr);
}
