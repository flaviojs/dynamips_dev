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

#[no_mangle]
pub extern "C" fn _export(_: *mut parser_token_t, _: *mut parser_context_t) {}
