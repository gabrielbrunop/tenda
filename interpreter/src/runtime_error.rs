use core::fmt;
use std::fmt::Display;
use thiserror::Error;

use crate::{
    associative_array::AssociativeArrayKey,
    value::{Value, ValueType},
};

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum RuntimeErrorKind {
    #[error("divisão por zero não é permitida")]
    DivisionByZero,

    #[error("números muito grandes não são permitidos")]
    NumberOverflow,

    #[error("esperado '{}', encontrado '{}'", .expected.to_string(), .found.to_string())]
    TypeError {
        expected: ValueType,
        found: ValueType,
    },

    #[error("a variável identificada por '{0}' não está definida neste escopo")]
    UndefinedReference(String),

    #[error("variável identifica com {0} já está declarada neste escopo")]
    AlreadyDeclared(String),

    #[error("número de argumentos incorreto: esperado {}, encontrado {}", .expected, .found)]
    WrongNumberOfArguments { expected: usize, found: usize },

    #[error("índice fora dos limites: índice {}, tamanho {}", .index, .len)]
    IndexOutOfBounds { index: usize, len: usize },

    #[error("não é possível acessar um valor do tipo '{}'", .value.to_string())]
    WrongIndexType { value: Value },

    #[error("limites de intervalo precisam ser números inteiros finitos: encontrado '{}'", .bound)]
    InvalidRangeBounds { bound: f64 },

    #[error("índice de lista precisa ser um número inteiro positivo e finito: encontrado '{}'", .index)]
    InvalidIndex { index: f64 },

    #[error("chave de dicionário precisa ser número inteiro ou texto: encontrado '{}'", .key)]
    InvalidNumberAssociativeArrayKey { key: f64 },

    #[error("chave de dicionário precisa ser número inteiro ou texto: encontrado '{}'", .key)]
    InvalidTypeAssociativeArrayKey { key: ValueType },

    #[error("chave de dicionário não encontrada: '{}'", .key.to_string())]
    AssociativeArrayKeyNotFound { key: AssociativeArrayKey },

    #[error("não é possível iterar sobre um valor do tipo '{}'", .value.to_string())]
    NotIterable { value: ValueType },

    #[error("o valor do tipo '{}' não é um argumento válido para a função", .value.to_string())]
    InvalidArgument { value: Value },

    #[error("textos são imutáveis e não podem ser modificados")]
    ImmutableString,

    #[error("timestamp inválido: {}", .timestamp.to_string())]
    InvalidTimestamp { timestamp: i64 },

    #[error("falha ao analisar data ISO: {0}")]
    DateIsoParseError(#[from] chrono::ParseError),

    #[error("fuso horário inválido: '{tz_str}'")]
    InvalidTimeZoneString { tz_str: String },
}

#[derive(Error, Debug, PartialEq, Clone)]
pub struct RuntimeError {
    #[source]
    pub source: RuntimeErrorKind,
    pub context: Option<String>,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.context {
            Some(context) => write!(f, "{}", context),
            None => write!(f, "{}", self.source),
        }
    }
}

impl From<RuntimeErrorKind> for RuntimeError {
    fn from(kind: RuntimeErrorKind) -> Self {
        RuntimeError {
            source: kind,
            context: None,
        }
    }
}

macro_rules! runtime_err {
    ($kind:expr) => {{
        use crate::runtime_error::RuntimeError;
        use crate::runtime_error::RuntimeErrorKind::*;

        Err(RuntimeError {
            source: $kind,
            context: None,
        })
    }};
    ($kind:expr, $message:expr) => {{
        use crate::runtime_error::RuntimeErrorKind::*;

        Err(RuntimeError {
            source: $kind,
            context: Some($message),
        })?
    }};
}

macro_rules! type_err {
    ($expected:expr, $found: expr) => {{
        use crate::runtime_error::runtime_err;
        use crate::value::ValueType::{self, *};

        let expected: ValueType = $expected.into();
        let found: ValueType = $found.into();

        runtime_err!(TypeError {
            expected: expected.clone(),
            found: found.clone(),
        })
    }};
    ($message:literal, $expected:expr, $found: expr) => {{
        let expected: ValueType = $expected.into();
        let found: ValueType = $found.into();

        Err(RuntimeError {
            source: RuntimeErrorKind::TypeError {
                expected: expected.clone(),
                found: found.clone(),
            },
            context: Some(format!($message, expected, found)),
        })
    }};
}

pub(crate) use runtime_err;
pub(crate) use type_err;
