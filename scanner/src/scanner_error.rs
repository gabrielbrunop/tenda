use core::fmt;
use std::fmt::Display;

use thiserror::Error;

use crate::token::TokenSpan;

#[derive(Error, Debug)]
pub struct LexicalError {
    pub span: TokenSpan,
    #[source]
    pub source: LexicalErrorKind,
}

impl Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (na posição {})", self.source, self.span.start)
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
