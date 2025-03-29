use common::span::SourceSpan;
use macros::Report;
use thiserror::Error;

use crate::{
    associative_array::AssociativeArrayKey,
    value::{Value, ValueType},
};

pub type Result<T> = std::result::Result<T, Box<RuntimeError>>;

#[derive(Error, Debug, PartialEq, Clone, Report)]
pub enum RuntimeError {
    #[error("divisão por zero não é permitida")]
    DivisionByZero { span: Option<SourceSpan> },

    #[error("números muito grandes não são permitidos")]
    NumberOverflow { span: Option<SourceSpan> },

    #[error("operação inválida para os tipos '{}' e '{}'", .first.to_string(), .second.to_string())]
    TypeMismatch {
        first: ValueType,
        second: ValueType,
        span: Option<SourceSpan>,
        help: Option<String>,
    },

    #[error("esperado valor de tipo '{}', encontrado '{}'", .expected.to_string(), .found.to_string())]
    UnexpectedTypeError {
        expected: ValueType,
        found: ValueType,
        span: Option<SourceSpan>,
        help: Option<String>,
    },

    #[error("a variável identificada por '{}' não está definida neste escopo", .var_name)]
    UndefinedReference {
        var_name: String,
        span: Option<SourceSpan>,
        help: Option<String>,
    },

    #[error("variável identifica com {0} já está declarada neste escopo", .var_name)]
    AlreadyDeclared {
        var_name: String,
        span: Option<SourceSpan>,
    },

    #[error("número de argumentos incorreto: esperado {}, encontrado {}", .expected, .found)]
    WrongNumberOfArguments {
        expected: usize,
        found: usize,
        span: Option<SourceSpan>,
    },

    #[error("índice fora dos limites: índice {}, tamanho {}", .index, .len)]
    IndexOutOfBounds {
        index: usize,
        len: usize,
        span: Option<SourceSpan>,
    },

    #[error("não é possível acessar um valor do tipo '{}'", .value.to_string())]
    WrongIndexType {
        value: ValueType,
        span: Option<SourceSpan>,
    },

    #[error("limites de intervalo precisam ser números inteiros finitos: encontrado '{}'", .bound)]
    InvalidRangeBounds {
        bound: f64,
        span: Option<SourceSpan>,
    },

    #[error("índice de lista precisa ser um número inteiro positivo e finito: encontrado '{}'", .index)]
    InvalidIndex {
        index: f64,
        span: Option<SourceSpan>,
    },

    #[error("chave de dicionário precisa ser número inteiro ou texto: encontrado '{}'", .key)]
    InvalidNumberAssociativeArrayKey { key: f64, span: Option<SourceSpan> },

    #[error("chave de dicionário precisa ser número inteiro ou texto: encontrado '{}'", .key)]
    InvalidTypeAssociativeArrayKey {
        key: ValueType,
        span: Option<SourceSpan>,
    },

    #[error("chave de dicionário não encontrada: '{}'", .key.to_string())]
    AssociativeArrayKeyNotFound {
        key: AssociativeArrayKey,
        span: Option<SourceSpan>,
    },

    #[error("não é possível iterar sobre um valor do tipo '{}'", .value.to_string())]
    NotIterable {
        value: ValueType,
        span: Option<SourceSpan>,
    },

    #[error("o valor do tipo '{}' não é um argumento válido para a função", .value.to_string())]
    InvalidArgument {
        value: Value,
        span: Option<SourceSpan>,
    },

    #[error("textos são imutáveis e não podem ser modificados")]
    ImmutableString { span: Option<SourceSpan> },

    #[error("timestamp inválido: {}", .timestamp.to_string())]
    InvalidTimestamp {
        timestamp: i64,
        span: Option<SourceSpan>,
    },

    #[error("falha ao analisar data ISO: {}", .source)]
    DateIsoParseError {
        source: chrono::ParseError,
        span: Option<SourceSpan>,
    },

    #[error("fuso horário inválido: '{tz_str}'")]
    InvalidTimeZoneString {
        tz_str: String,
        span: Option<SourceSpan>,
    },
}
