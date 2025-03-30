use common::span::SourceSpan;
use macros::Report;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone, Report)]
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
}
