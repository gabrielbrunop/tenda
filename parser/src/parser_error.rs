use core::fmt;
use std::fmt::Display;

use scanner::token::Token;
use thiserror::Error;

#[derive(Error, Debug)]
pub struct ParserError {
    pub line: usize,
    pub context: Option<String>,
    #[source]
    pub source: ParserErrorKind,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.context {
            Some(context) => write!(f, "{} (na linha {})", context, self.line),
            None => write!(f, "{} (na linha {})", self.source, self.line),
        }
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ParserErrorKind {
    #[error("fim inesperado de input")]
    UnexpectedEoi,
    #[error("esperado ')'")]
    MissingParentheses,
    #[error("token inesperado: {}", .0.lexeme)]
    UnexpectedToken(Token),
    #[error("o valor à direita do '=' não é um valor válido para receber atribuições")]
    InvalidAssignmentTarget(Token),
    #[error("retorno fora de uma função")]
    IllegalReturn,
    #[error("parâmetro '{0}' duplicado na função")]
    DuplicateParameter(String),
}

#[macro_export]
macro_rules! parser_err {
    ($kind:expr, $line:expr) => {{
        use ParserErrorKind::*;
        ParserError {
            source: $kind,
            line: $line,
            context: None,
        }
    }};
    ($kind:expr, $line:expr, $context:expr) => {{
        use ParserErrorKind::*;
        ParserError {
            source: $kind,
            line: $line,
            context: Some($context),
        }
    }};
}

macro_rules! unexpected_token {
    ($token:expr) => {{
        let token = $token;
        parser_err!(UnexpectedToken(token.clone_ref()), token.line)
    }};
}

macro_rules! unexpected_eoi {
    ($self:ident) => {
        parser_err!(UnexpectedEoi, $self.tokens.get_last_line())
    };
}

pub(crate) use unexpected_eoi;
pub(crate) use unexpected_token;
