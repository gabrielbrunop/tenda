use common::span::SourceSpan;
use macros::Report;
use scanner::token::Token;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Vec<ParserError>>;

#[derive(Error, Debug, PartialEq, Clone, Report)]
pub enum ParserError {
    #[error("fim inesperado de entrada")]
    UnexpectedEoi { span: SourceSpan },

    #[error("esperado ')'")]
    MissingParentheses { span: SourceSpan },

    #[error("esperado ']'")]
    MissingBrackets { span: SourceSpan },

    #[error("esperado '}}'")]
    MissingBraces { span: SourceSpan },

    #[error("esperado ':'")]
    MissingColon { span: SourceSpan },

    #[error("token inesperado: {}", .token.lexeme.escape_default())]
    UnexpectedToken { token: Token, span: SourceSpan },

    #[error("o valor à direita do '=' não é um valor válido para receber atribuições")]
    InvalidAssignmentTarget { token: Token, span: SourceSpan },

    #[error("retorno fora de uma função")]
    IllegalReturn { span: SourceSpan },

    #[error("'pare' fora de uma estrutura de repetição")]
    IllegalBreak { span: SourceSpan },

    #[error("'continue' fora de uma estrutura de repetição")]
    IllegalContinue { span: SourceSpan },

    #[error("parâmetro '{}' duplicado na função", .name)]
    DuplicateParameter { name: String, span: SourceSpan },
}

macro_rules! unexpected_token {
    ($token:expr) => {{
        let token = $token;

        match token.kind {
            TokenKind::Eof => ParserError::UnexpectedEoi {
                span: token.span.clone(),
            },
            _ => ParserError::UnexpectedToken {
                token: token.clone_ref(),
                span: token.span.clone(),
            },
        }
    }};
}

pub(crate) use unexpected_token;
