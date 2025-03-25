use core::fmt;
use scanner::token::{Token, TokenSpan};
use std::fmt::Display;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Vec<ParserError>>;

#[derive(Debug)]
pub struct ParserErrorSpan {
    pub start: usize,
    pub end: usize,
}

impl From<TokenSpan> for ParserErrorSpan {
    fn from(span: TokenSpan) -> Self {
        ParserErrorSpan {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<AstSpan> for ParserErrorSpan {
    fn from(span: AstSpan) -> Self {
        ParserErrorSpan {
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Error, Debug)]
pub struct ParserError {
    pub span: ParserErrorSpan,
    pub context: Option<String>,
    #[source]
    pub source: ParserErrorKind,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.context {
            Some(context) => write!(f, "{}", context),
            None => write!(f, "{}", self.source),
        }
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ParserErrorKind {
    #[error("fim inesperado de entrada")]
    UnexpectedEoi,

    #[error("esperado ')'")]
    MissingParentheses,

    #[error("esperado ']'")]
    MissingBrackets,

    #[error("esperado '}}'")]
    MissingBraces,

    #[error("esperado ':'")]
    MissingColon,

    #[error("token inesperado: {}", .0.lexeme)]
    UnexpectedToken(Token),

    #[error("o valor à direita do '=' não é um valor válido para receber atribuições")]
    InvalidAssignmentTarget(Token),

    #[error("retorno fora de uma função")]
    IllegalReturn,

    #[error("'pare' fora de uma estrutura de repetição")]
    IllegalBreak,

    #[error("'continue' fora de uma estrutura de repetição")]
    IllegalContinue,

    #[error("parâmetro '{0}' duplicado na função")]
    DuplicateParameter(String),
}

#[macro_export]
macro_rules! parser_err {
    ($kind:expr, $span:expr) => {{
        use ParserErrorKind::*;
        ParserError {
            source: $kind,
            span: $span.into(),
            context: None,
        }
    }};
    ($kind:expr, $span:expr, $context:expr) => {{
        use ParserErrorKind::*;
        ParserError {
            source: $kind,
            span: $span.into(),
            context: Some($context),
        }
    }};
}

macro_rules! unexpected_token {
    ($token:expr) => {{
        let token = $token;
        parser_err!(UnexpectedToken(token.clone_ref()), token.span)
    }};
}

pub(crate) use unexpected_token;

use crate::ast::AstSpan;
