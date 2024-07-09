use core::fmt;
use std::fmt::Display;

use thiserror::Error;

#[derive(Error, Debug)]
pub struct LexicalError {
    pub line: usize,
    #[source]
    pub source: LexicalErrorKind,
}

impl Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (na linha {})", self.source, self.line)
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum LexicalErrorKind {
    #[error("zeros à esquerda em literais numéricos não são permitidos")]
    LeadingZeroNumberLiterals,
    #[error("fim de linha inesperado em texto")]
    UnexpectedStringEol,
    #[error("caractere inesperado: {0}")]
    UnexpectedChar(char),
}

#[macro_export]
macro_rules! lexical_error {
    ($kind:expr, $line:expr) => {{
        use LexicalErrorKind::*;
        LexicalError {
            source: $kind,
            line: $line,
        }
    }};
}
