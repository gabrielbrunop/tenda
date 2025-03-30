use common::span::SourceSpan;
use macros::Report;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone, Report)]
#[report("erro léxico")]
pub enum LexicalError {
    #[error("zeros à esquerda em literais numéricos não são permitidos")]
    LeadingZeroNumberLiterals { span: SourceSpan },

    #[error("fim de linha inesperado em texto")]
    UnexpectedStringEol { span: SourceSpan },

    #[error("caractere inesperado: {}", .character)]
    UnexpectedChar { character: char, span: SourceSpan },

    #[error("fim inesperado de entrada")]
    UnexpectedEoi { span: SourceSpan },
}
