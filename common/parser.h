/*
 * Cisco router simulation platform.
 * Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
 */

#ifndef __PARSER_H__
#define __PARSER_H__

#include "rust_dynamips_c.h"

#include <sys/types.h>

/* Get a description given an error code */
char *parser_strerror(parser_context_t *ctx);

/* Dump a token list */
void parser_dump_tokens(parser_context_t *ctx);

/* Map a token list to an array */
char **parser_map_array(parser_context_t *ctx);

/* Initialize parser context */
void parser_context_init(parser_context_t *ctx);

/* Free memory used by a parser context */
void parser_context_free(parser_context_t *ctx);

/* Send a buffer to the tokenizer */
int parser_scan_buffer(parser_context_t *ctx,char *buf,size_t buf_size);

/* Tokenize a string */
int parser_tokenize(char *str,struct parser_token **tokens,int *tok_count);

#endif
