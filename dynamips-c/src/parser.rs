//! Cisco router simulation platform.
//! Copyright (c) 2005,2006 Christophe Fillot (cf@utc.fr)
//! Copyright (c) 2006 Christophe Fillot (cf@utc.fr)
//!
//! Mini-parser.

use crate::_extra::*;
use libc::size_t;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ptr::null_mut;

// Parser Errors
pub type _PARSER_ERROR_ENUM = c_int; // TODO enum
pub const PARSER_ERROR_NOMEM: _PARSER_ERROR_ENUM = 1;
pub const PARSER_ERROR_UNEXP_QUOTE: _PARSER_ERROR_ENUM = 2; // Unexpected quote in a word
pub const PARSER_ERROR_UNEXP_EOL: _PARSER_ERROR_ENUM = 3; // Unexpected end of line

// Parser states
pub type _PARSER_STATE_ENUM = c_int; // TODO enum
pub const PARSER_STATE_DONE: _PARSER_STATE_ENUM = 0;
pub const PARSER_STATE_SKIP: _PARSER_STATE_ENUM = 1;
pub const PARSER_STATE_BLANK: _PARSER_STATE_ENUM = 2;
pub const PARSER_STATE_STRING: _PARSER_STATE_ENUM = 3;
pub const PARSER_STATE_QUOTED_STRING: _PARSER_STATE_ENUM = 4;

// Token
pub type parser_token_t = parser_token;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parser_token {
    pub value: *mut c_char,
    pub next: *mut parser_token,
}

// Parser context
pub type parser_context_t = parser_context;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parser_context {
    // Token list
    pub tok_head: *mut parser_token_t,
    pub tok_last: *mut parser_token_t,
    pub tok_count: c_int,

    // Temporary token
    pub tmp_tok: *mut c_char,
    pub tmp_tot_len: size_t,
    pub tmp_cur_len: size_t,

    // Parser state and error
    pub state: c_int,
    pub error: c_int,

    // Number of consumed chars
    pub consumed_len: size_t,
}

const TOKEN_MAX_SIZE: usize = 512;

// Character types
pub type _PARSER_CHAR_ENUM = c_int; // TODO enum
pub const PARSER_CHAR_BLANK: _PARSER_CHAR_ENUM = 0;
pub const PARSER_CHAR_NEWLINE: _PARSER_CHAR_ENUM = 1;
pub const PARSER_CHAR_COMMENT: _PARSER_CHAR_ENUM = 2;
pub const PARSER_CHAR_QUOTE: _PARSER_CHAR_ENUM = 3;
pub const PARSER_CHAR_OTHER: _PARSER_CHAR_ENUM = 4;

// Get a description given an error code
#[no_mangle]
pub unsafe extern "C" fn parser_strerror(ctx: *mut parser_context_t) -> *mut c_char {
    libc::printf(c"error = %d\n".as_ptr(), (*ctx).error);

    match (*ctx).error {
        0 => c"no error".as_ptr().cast_mut(),
        PARSER_ERROR_NOMEM => c"insufficient memory".as_ptr().cast_mut(),
        PARSER_ERROR_UNEXP_QUOTE => c"unexpected quote".as_ptr().cast_mut(),
        PARSER_ERROR_UNEXP_EOL => c"unexpected end of line".as_ptr().cast_mut(),
        _ => c"unknown error".as_ptr().cast_mut(),
    }
}

// Dump a token list
#[no_mangle]
pub unsafe extern "C" fn parser_dump_tokens(ctx: *mut parser_context_t) {
    let mut tok: *mut parser_token_t;

    tok = (*ctx).tok_head;
    while !tok.is_null() {
        libc::printf(c"\"%s\" ".as_ptr(), (*tok).value);
        tok = (*tok).next
    }
}

// Map a token list to an array
#[no_mangle]
pub unsafe extern "C" fn parser_map_array(ctx: *mut parser_context_t) -> *mut *mut c_char {
    let mut tok: *mut parser_token_t;
    let mut i: c_int;

    if (*ctx).tok_count <= 0 {
        return null_mut();
    }

    let map: *mut *mut c_char = libc::calloc((*ctx).tok_count as size_t, size_of::<*mut c_char>()).cast::<_>();
    if map.is_null() {
        return null_mut();
    }

    i = 0;
    tok = (*ctx).tok_head;
    while (i < (*ctx).tok_count) && !tok.is_null() {
        *map.add(i as usize) = (*tok).value;
        i += 1;
        tok = (*tok).next;
    }

    map
}

// Add a character to temporary token (resize if necessary)
unsafe fn tmp_token_add_char(ctx: *mut parser_context_t, c: c_char) -> c_int {
    let new_size: size_t;
    let new_str: *mut c_char;

    if (*ctx).tmp_tok.is_null() || ((*ctx).tmp_cur_len == ((*ctx).tmp_tot_len - 1)) {
        new_size = (*ctx).tmp_tot_len + TOKEN_MAX_SIZE;
        new_str = libc::realloc((*ctx).tmp_tok.cast::<_>(), new_size).cast::<_>();

        if new_str.is_null() {
            return -1;
        }

        (*ctx).tmp_tok = new_str;
        (*ctx).tmp_tot_len = new_size;
    }

    *(*ctx).tmp_tok.add((*ctx).tmp_cur_len) = c;
    (*ctx).tmp_cur_len += 1;
    *(*ctx).tmp_tok.add((*ctx).tmp_cur_len) = 0;
    0
}

// Move current token to the active token list
unsafe fn parser_move_tmp_token(ctx: *mut parser_context_t) -> c_int {
    // no token ...
    if (*ctx).tmp_tok.is_null() {
        return 0;
    }

    let tok: *mut parser_token_t = libc::malloc(size_of::<parser_token_t>()).cast::<_>();
    if tok.is_null() {
        return -1;
    }

    (*tok).value = (*ctx).tmp_tok;
    (*tok).next = null_mut();

    // add it to the token list
    if !(*ctx).tok_last.is_null() {
        (*(*ctx).tok_last).next = tok;
    } else {
        (*ctx).tok_head = tok;
    }

    (*ctx).tok_last = tok;
    (*ctx).tok_count += 1;

    // start a new token
    (*ctx).tmp_tok = null_mut();
    (*ctx).tmp_tot_len = 0;
    (*ctx).tmp_cur_len = 0;
    0
}

// Initialize parser context
#[no_mangle]
pub unsafe extern "C" fn parser_context_init(ctx: *mut parser_context_t) {
    (*ctx).tok_head = null_mut();
    (*ctx).tok_last = null_mut();
    (*ctx).tok_count = 0;

    (*ctx).tmp_tok = null_mut();
    (*ctx).tmp_tot_len = 0;
    (*ctx).tmp_cur_len = 0;

    (*ctx).state = PARSER_STATE_BLANK;
    (*ctx).error = 0;

    (*ctx).consumed_len = 0;
}

// Free a token list
#[no_mangle]
pub unsafe extern "C" fn parser_free_tokens(tok_list: *mut parser_token_t) {
    let mut t: *mut parser_token_t;
    let mut next: *mut parser_token_t;

    t = tok_list;
    while !t.is_null() {
        next = (*t).next;
        libc::free((*t).value.cast::<_>());
        libc::free(t.cast::<_>());
        t = next;
    }
}

// Free memory used by a parser context
#[no_mangle]
pub unsafe extern "C" fn parser_context_free(ctx: *mut parser_context_t) {
    parser_free_tokens((*ctx).tok_head);

    if !(*ctx).tmp_tok.is_null() {
        libc::free((*ctx).tmp_tok.cast::<_>());
    }

    parser_context_init(ctx);
}

// Determine the type of the input character
unsafe fn parser_get_char_type(c: u_char) -> c_int {
    match c {
      b'\n' | b'\r' | 0 => PARSER_CHAR_NEWLINE,
      b'\t' /* | b'\r' */ | b' ' => PARSER_CHAR_BLANK,
      b'!' | b'#' => PARSER_CHAR_COMMENT,
      b'"' => PARSER_CHAR_QUOTE,
      _ => PARSER_CHAR_OTHER,
   }
}

// Send a buffer to the tokenizer
#[no_mangle]
pub unsafe extern "C" fn parser_scan_buffer(ctx: *mut parser_context_t, buf: *mut c_char, buf_size: size_t) -> c_int {
    let mut type_: c_int;
    let mut c: u_char;

    for i in 0..buf_size as c_int {
        if (*ctx).state == PARSER_STATE_DONE {
            break;
        }
        (*ctx).consumed_len += 1;
        c = *buf.add(i as usize) as u_char;

        // Determine character type
        type_ = parser_get_char_type(c);

        // Basic finite state machine
        match (*ctx).state {
            PARSER_STATE_SKIP => {
                if type_ == PARSER_CHAR_NEWLINE {
                    (*ctx).state = PARSER_STATE_DONE;
                }

                // Simply ignore character until we reach end of line
            }

            PARSER_STATE_BLANK => {
                match type_ {
                    PARSER_CHAR_BLANK => {
                        // Eat space
                    }

                    PARSER_CHAR_COMMENT => {
                        (*ctx).state = PARSER_STATE_SKIP;
                    }

                    PARSER_CHAR_NEWLINE => {
                        (*ctx).state = PARSER_STATE_DONE;
                    }

                    PARSER_CHAR_QUOTE => {
                        (*ctx).state = PARSER_STATE_QUOTED_STRING;
                    }

                    _ => {
                        // Begin a new string
                        if 0 == tmp_token_add_char(ctx, c as c_char) {
                            (*ctx).state = PARSER_STATE_STRING;
                        } else {
                            (*ctx).state = PARSER_STATE_SKIP;
                            (*ctx).error = PARSER_ERROR_NOMEM;
                        }
                    }
                }
            }

            PARSER_STATE_STRING => {
                match type_ {
                    PARSER_CHAR_BLANK => {
                        if 0 == parser_move_tmp_token(ctx) {
                            (*ctx).state = PARSER_STATE_BLANK;
                        } else {
                            (*ctx).state = PARSER_STATE_SKIP;
                            (*ctx).error = PARSER_ERROR_NOMEM;
                        }
                    }

                    PARSER_CHAR_NEWLINE => {
                        if parser_move_tmp_token(ctx) == -1 {
                            (*ctx).error = PARSER_ERROR_NOMEM;
                        }

                        (*ctx).state = PARSER_STATE_DONE;
                    }

                    PARSER_CHAR_COMMENT => {
                        if parser_move_tmp_token(ctx) == -1 {
                            (*ctx).error = PARSER_ERROR_NOMEM;
                        }

                        (*ctx).state = PARSER_STATE_SKIP;
                    }

                    PARSER_CHAR_QUOTE => {
                        (*ctx).error = PARSER_ERROR_UNEXP_QUOTE;
                        (*ctx).state = PARSER_STATE_SKIP;
                    }

                    _ => {
                        // Add the character to the buffer
                        if tmp_token_add_char(ctx, c as c_char) == -1 {
                            (*ctx).state = PARSER_STATE_SKIP;
                            (*ctx).error = PARSER_ERROR_NOMEM;
                        }
                    }
                }
            }

            PARSER_STATE_QUOTED_STRING => {
                match type_ {
                    PARSER_CHAR_NEWLINE => {
                        // Unterminated string!
                        (*ctx).error = PARSER_ERROR_UNEXP_EOL;
                        (*ctx).state = PARSER_STATE_DONE;
                    }

                    PARSER_CHAR_QUOTE => {
                        if 0 == parser_move_tmp_token(ctx) {
                            (*ctx).state = PARSER_STATE_BLANK;
                        } else {
                            (*ctx).state = PARSER_STATE_SKIP;
                            (*ctx).error = PARSER_ERROR_NOMEM;
                        }
                    }

                    _ => {
                        // Add the character to the buffer
                        if tmp_token_add_char(ctx, c as c_char) == -1 {
                            (*ctx).state = PARSER_STATE_SKIP;
                            (*ctx).error = PARSER_ERROR_NOMEM;
                        }
                    }
                }
            }

            _ => unreachable!(),
        }
    }

    ((*ctx).state == PARSER_STATE_DONE) as _
}
