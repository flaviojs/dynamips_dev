//! Mini-parser.

use crate::prelude::*;

pub type parser_token_t = parser_token;

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

#[no_mangle]
pub extern "C" fn _export(_: *mut parser_token_t) {}
