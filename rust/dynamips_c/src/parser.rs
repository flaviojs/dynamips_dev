//! Mini-parser.

use crate::prelude::*;

// Parser Errors // TODO enmm
pub const PARSER_ERROR_NOMEM: c_int = 1;
/// Unexpected quote in a word
pub const PARSER_ERROR_UNEXP_QUOTE: c_int = 2;
/// Unexpected end of line
pub const PARSER_ERROR_UNEXP_EOL: c_int = 3;
