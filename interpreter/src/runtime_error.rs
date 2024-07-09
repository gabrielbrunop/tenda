use core::fmt;
use std::fmt::Display;

use thiserror::Error;

use crate::value::ValueType;

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
    #[error("variável identifica com {0} já estpa declarada neste escopo")]
    AlreadyDeclared(String),
    #[error("número de argumentos incorreto: esperado {}, encontrado {}", .expected, .found)]
    WrongNumberOfArguments { expected: usize, found: usize },
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

#[macro_export]
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

#[macro_export]
macro_rules! runtime_err {
    ($kind:expr, $message:expr) => {{
        Err(RuntimeError {
            source: $kind,
            context: Some($message),
        })?
    }};
}
