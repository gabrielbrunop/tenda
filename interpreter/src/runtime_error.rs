use core::fmt;
use std::fmt::Display;
use thiserror::Error;

use crate::value::{AssociativeArrayKey, Value, ValueType};

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

macro_rules! type_err {
    ($message:literal, $expected:expr, $found: expr) => {{
        let expected: ValueType = $expected.into();
        let found: ValueType = $found.into();
        Err(RuntimeError {
            source: RuntimeErrorKind::TypeError {
                expected: expected.clone(),
                found: found.clone(),
            },
            context: Some(format!($message, expected, found)),
        })?
    }};
}

macro_rules! runtime_err {
    ($kind:expr, $message:expr) => {{
        Err(RuntimeError {
            source: $kind,
            context: Some($message),
        })?
    }};
}

pub(crate) use runtime_err;
pub(crate) use type_err;
