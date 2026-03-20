mod grammar;
use crate::ast::Program;
use grammar::lang;

const KEYWORDS: [&str; 2] = ["let", "fn"];

use crate::ast::err::{Error, Result};

pub fn parse(src: &str) -> Result<Program> {
    lang::program(src).map_err(|e| {
        Error::new("syntax error", src.to_string()).label(
            format!("expected one of {}", e.expected),
            (e.location.offset, e.location.offset + 1),
        )
    })
}
