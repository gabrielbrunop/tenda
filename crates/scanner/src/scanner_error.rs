use tenda_common::span::SourceSpan;
use tenda_reporting_derive::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone, Diagnostic)]
#[report("erro léxico")]
pub enum LexicalError {
    #[error("zeros à esquerda em literais numéricos não são permitidos")]
    LeadingZeroNumberLiterals {
        #[span]
        span: SourceSpan,
    },

    #[error("fim de linha inesperado em texto")]
    UnexpectedStringEol {
        #[span]
        span: SourceSpan,
    },

    #[error("caractere inesperado: {}", .character)]
    UnexpectedChar {
        character: char,
        #[span]
        span: SourceSpan,
    },

    #[error("fim inesperado de entrada")]
    UnexpectedEoi {
        #[span]
        span: SourceSpan,
    },

    #[error("escape hexadecimal inválido")]
    InvalidHexEscape {
        #[span]
        span: SourceSpan,
    },

    #[error("escape octal inválido")]
    InvalidOctalEscape {
        #[span]
        span: SourceSpan,
    },

    #[error("escape unicode inválido")]
    InvalidUnicodeEscape {
        #[span]
        span: SourceSpan,
    },

    #[error("escape não reconhecido: {}", .found)]
    UnknownEscape {
        #[span]
        span: SourceSpan,
        found: char,
    },
}
