use tenda_common::span::SourceSpan;
use tenda_reporting_derive::Diagnostic;
use tenda_scanner::Token;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Vec<ParserError>>;

#[derive(Error, Debug, PartialEq, Clone, Diagnostic)]
#[report("erro sintático")]
pub enum ParserError {
    #[error("fim inesperado de entrada")]
    UnexpectedEoi {
        #[span]
        span: SourceSpan,
    },

    #[error("esperado ')'")]
    MissingParentheses {
        #[span]
        span: SourceSpan,
    },

    #[error("esperado ']'")]
    MissingBrackets {
        #[span]
        span: SourceSpan,
    },

    #[error("esperado '}}'")]
    MissingBraces {
        #[span]
        span: SourceSpan,
    },

    #[error("esperado ':'")]
    MissingColon {
        #[span]
        span: SourceSpan,
    },

    #[error("token inesperado: {}", .token.lexeme.escape_debug())]
    UnexpectedToken {
        token: Token,

        #[span]
        span: SourceSpan,
    },

    #[error("o valor à direita do '=' não é um valor válido para receber atribuições")]
    InvalidAssignmentTarget {
        token: Token,

        #[span]
        span: SourceSpan,
    },

    #[error("retorno fora de uma função")]
    IllegalReturn {
        #[span]
        span: SourceSpan,
    },

    #[error("'pare' fora de uma estrutura de repetição")]
    IllegalBreak {
        #[span]
        span: SourceSpan,
    },

    #[error("'continue' fora de uma estrutura de repetição")]
    IllegalContinue {
        #[span]
        span: SourceSpan,
    },

    #[error("parâmetro '{}' duplicado na função", .name)]
    DuplicateParameter {
        name: String,

        #[span]
        span: SourceSpan,
    },

    #[error("o operador '{}' não pode ser encadeado", .op.lexeme)]
    InvalidChaining {
        op: Token,

        #[span]
        span: SourceSpan,

        #[message]
        message: Option<String>,

        #[help]
        help: Option<String>,
    },
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
