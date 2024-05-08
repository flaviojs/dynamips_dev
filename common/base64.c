/*
 * base64.c -- base-64 conversion routines.
 * Copyright (C)2002 by Eric S. Raymond.
 *
 * For license terms, see the file COPYING in this directory.
 *
 * This base 64 encoding is defined in RFC2045 section 6.8,
 * "Base64 Content-Transfer-Encoding", but lines must not be broken in the
 * scheme used here.
 */

#include "rust_dynamips_c.h"

#include <ctype.h>
