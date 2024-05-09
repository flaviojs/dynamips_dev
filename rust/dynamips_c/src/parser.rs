//! Mini-parser.

use crate::prelude::*;

pub type parser_token_t = parser_token;
pub type parser_context_t = parser_context;

// Parser Errors // TODO enmm
pub const PARSER_ERROR_NOMEM: c_int = 1;
/// Unexpected quote in a word
pub const PARSER_ERROR_UNEXP_QUOTE: c_int = 2;
/// Unexpected end of line
pub const PARSER_ERROR_UNEXP_EOL: c_int = 3;

// Parser states // TODO enum
pub const PARSER_STATE_DONE: c_int = 0;
pub const PARSER_STATE_SKIP: c_int = 1;
pub const PARSER_STATE_BLANK: c_int = 2;
pub const PARSER_STATE_STRING: c_int = 3;
pub const PARSER_STATE_QUOTED_STRING: c_int = 4;

/// Token
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parser_token {
    pub value: *mut c_char,
    pub next: *mut parser_token_t,
}

/// Parser context
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parser_context {
    /// Token list
    pub tok_head: *mut parser_token_t,
    pub tok_last: *mut parser_token_t,
    pub tok_count: c_int,

    /// Temporary token
    pub tmp_tok: *mut c_char,
    pub tmp_tot_len: size_t,
    pub tmp_cur_len: size_t,

    /// Parser state and error
    pub state: c_int,
    pub error: c_int,

    /// Number of consumed chars
    pub consumed_len: size_t,
}

/// Get a description given an error code
#[no_mangle]
pub unsafe extern "C" fn parser_strerror(ctx: *mut parser_context_t) -> *mut c_char {
    libc::printf(cstr!("error = %d\n"), (*ctx).error);

    match (*ctx).error {
        0 => cstr!("no error"),
        PARSER_ERROR_NOMEM => cstr!("insufficient memory"),
        PARSER_ERROR_UNEXP_QUOTE => cstr!("unexpected quote"),
        PARSER_ERROR_UNEXP_EOL => cstr!("unexpected end of line"),
        _ => cstr!("unknown error"),
    }
}

/// Dump a token list
#[no_mangle]
pub unsafe extern "C" fn parser_dump_tokens(ctx: *mut parser_context_t) {
    let mut tok: *mut parser_token_t = (*ctx).tok_head;
    while !tok.is_null() {
        libc::printf(cstr!("\"%s\" "), (*tok).value);
        tok = (*tok).next;
    }
}

/// Map a token list to an array
#[no_mangle]
pub unsafe extern "C" fn parser_map_array(ctx: *mut parser_context_t) -> *mut *mut c_char {
    if (*ctx).tok_count <= 0 {
        return null_mut();
    }

    let map: *mut *mut c_char = libc::calloc((*ctx).tok_count as usize, size_of::<*mut c_char>()).cast::<_>();
    if map.is_null() {
        return null_mut();
    }

    let mut i: c_int = 0;
    let mut tok: *mut parser_token_t = (*ctx).tok_head;
    while i < (*ctx).tok_count && !tok.is_null() {
        *map.offset(i as isize) = (*tok).value;
        i += 1;
        tok = (*tok).next;
    }

    map
}

/// Initialize parser context
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

/// Free a token list
unsafe fn parser_free_tokens(tok_list: *mut parser_token_t) {
    let mut t: *mut parser_token_t = tok_list;
    while !t.is_null() {
        let next: *mut parser_token_t = (*t).next;
        libc::free((*t).value.cast::<_>());
        libc::free(t.cast::<_>());
        t = next;
    }
}

/// Free memory used by a parser context
#[no_mangle]
pub unsafe extern "C" fn parser_context_free(ctx: *mut parser_context_t) {
    parser_free_tokens((*ctx).tok_head);

    if !(*ctx).tmp_tok.is_null() {
        libc::free((*ctx).tmp_tok.cast::<_>());
    }

    parser_context_init(ctx);
}

#[no_mangle]
pub extern "C" fn _export(_: *mut parser_token_t, _: *mut parser_context_t) {}
