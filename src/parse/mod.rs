mod grammar;
use crate::ast::Program;
use grammar::lang;

const KEYWORDS: [&str; 4] = ["let", "fn", "if", "else"];

use crate::ast::{Error, Result};

pub fn program(src: &str) -> Result<Program> {
    lang::program(src).map_err(|e| {
        Error::new("syntax error").label(
            format!("expected one of {}", e.expected),
            (e.location.offset, e.location.offset + 1),
        )
    })
}
