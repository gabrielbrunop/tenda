use miette::Diagnostic;
use thiserror::Error;

use crate::{
    associative_array::AssociativeArrayKey,
    value::{Value, ValueType},
};

pub type Result<T> = std::result::Result<T, Box<RuntimeError>>;

#[derive(Error, Debug, PartialEq, Clone, Diagnostic)]
pub enum RuntimeError {
    #[error("divisão por zero não é permitida")]
    #[diagnostic(
        code(interpreter::division_by_zero),
        help("verifique se o divisor é diferente de zero antes de realizar a divisão.")
    )]
    DivisionByZero {
        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("a divisão por zero ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("números muito grandes não são permitidos")]
    #[diagnostic(code(interpreter::number_overflow))]
    NumberOverflow {
        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o número muito grande foi encontrado aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("operação inválida para os tipos '{}' e '{}'", .first.to_string(), .second.to_string())]
    #[diagnostic(code(interpreter::type_error))]
    TypeMismatch {
        first: ValueType,
        second: ValueType,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de tipo ocorreu aqui")]
        span: Option<miette::SourceSpan>,

        #[help]
        help: Option<String>,
    },

    #[error("esperado valor de tipo '{}', encontrado '{}'", .expected.to_string(), .found.to_string())]
    #[diagnostic(code(interpreter::type_error))]
    UnexpectedTypeError {
        expected: ValueType,
        found: ValueType,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de tipo ocorreu aqui")]
        span: Option<miette::SourceSpan>,

        #[help]
        help: Option<String>,
    },

    #[error("a variável identificada por '{}' não está definida neste escopo", .var_name)]
    #[diagnostic(code(interpreter::undefined_reference))]
    UndefinedReference {
        var_name: String,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("a referência indefinida ocorreu aqui")]
        span: Option<miette::SourceSpan>,

        #[help]
        help: Option<String>,
    },

    #[error("variável identifica com {0} já está declarada neste escopo", .var_name)]
    #[diagnostic(code(interpreter::variable_already_declared))]
    AlreadyDeclared {
        var_name: String,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("a variável já declarada ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("número de argumentos incorreto: esperado {}, encontrado {}", .expected, .found)]
    #[diagnostic(code(interpreter::wrong_number_of_arguments))]
    WrongNumberOfArguments {
        expected: usize,
        found: usize,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de número de argumentos incorreto ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("índice fora dos limites: índice {}, tamanho {}", .index, .len)]
    #[diagnostic(code(interpreter::index_out_of_bounds))]
    IndexOutOfBounds {
        index: usize,
        len: usize,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de índice fora dos limites ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("não é possível acessar um valor do tipo '{}'", .value.to_string())]
    #[diagnostic(code(interpreter::wrong_index_type))]
    WrongIndexType {
        value: Value,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de tipo de índice incorreto ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("limites de intervalo precisam ser números inteiros finitos: encontrado '{}'", .bound)]
    #[diagnostic(code(interpreter::invalid_range_bounds))]
    InvalidRangeBounds {
        bound: f64,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de limites de intervalo inválidos ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("índice de lista precisa ser um número inteiro positivo e finito: encontrado '{}'", .index)]
    #[diagnostic(code(interpreter::invalid_list_index))]
    InvalidIndex {
        index: f64,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de índice de lista inválido ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("chave de dicionário precisa ser número inteiro ou texto: encontrado '{}'", .key)]
    #[diagnostic(code(interpreter::invalid_associative_array_key))]
    InvalidNumberAssociativeArrayKey {
        key: f64,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("a chave de dicionário inválida ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("chave de dicionário precisa ser número inteiro ou texto: encontrado '{}'", .key)]
    #[diagnostic(code(interpreter::invalid_associative_array_key))]
    InvalidTypeAssociativeArrayKey {
        key: ValueType,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("a chave de dicionário inválida ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("chave de dicionário não encontrada: '{}'", .key.to_string())]
    #[diagnostic(code(interpreter::associative_array_key_not_found))]
    AssociativeArrayKeyNotFound {
        key: AssociativeArrayKey,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("a chave de dicionário não encontrada ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("não é possível iterar sobre um valor do tipo '{}'", .value.to_string())]
    #[diagnostic(code(interpreter::not_iterable))]
    NotIterable {
        value: ValueType,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de não iterável ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("o valor do tipo '{}' não é um argumento válido para a função", .value.to_string())]
    #[diagnostic(code(interpreter::invalid_argument))]
    InvalidArgument {
        value: Value,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de argumento inválido ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("textos são imutáveis e não podem ser modificados")]
    #[diagnostic(code(interpreter::immutable_string))]
    ImmutableString {
        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("a tentativa de modificar um texto ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("timestamp inválido: {}", .timestamp.to_string())]
    #[diagnostic(code(interpreter::invalid_timestamp))]
    InvalidTimestamp {
        timestamp: i64,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de timestamp inválido ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("falha ao analisar data ISO: {}", .source)]
    #[diagnostic(code(interpreter::invalid_date_iso))]
    DateIsoParseError {
        source: chrono::ParseError,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de análise de data ISO ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },

    #[error("fuso horário inválido: '{tz_str}'")]
    #[diagnostic(code(interpreter::invalid_timezone_string))]
    InvalidTimeZoneString {
        tz_str: String,

        #[source_code]
        source_code: Option<miette::NamedSource<String>>,

        #[label("o erro de fuso horário inválido ocorreu aqui")]
        span: Option<miette::SourceSpan>,
    },
}

impl RuntimeError {
    pub fn get_span(&self) -> Option<miette::SourceSpan> {
        match self {
            Self::DivisionByZero { span, .. }
            | Self::NumberOverflow { span, .. }
            | Self::TypeMismatch { span, .. }
            | Self::UnexpectedTypeError { span, .. }
            | Self::UndefinedReference { span, .. }
            | Self::AlreadyDeclared { span, .. }
            | Self::WrongNumberOfArguments { span, .. }
            | Self::IndexOutOfBounds { span, .. }
            | Self::WrongIndexType { span, .. }
            | Self::InvalidRangeBounds { span, .. }
            | Self::InvalidIndex { span, .. }
            | Self::InvalidNumberAssociativeArrayKey { span, .. }
            | Self::InvalidTypeAssociativeArrayKey { span, .. }
            | Self::AssociativeArrayKeyNotFound { span, .. }
            | Self::NotIterable { span, .. }
            | Self::InvalidArgument { span, .. }
            | Self::ImmutableString { span, .. }
            | Self::InvalidTimestamp { span, .. }
            | Self::DateIsoParseError { span, .. }
            | Self::InvalidTimeZoneString { span, .. } => *span,
        }
    }

    pub fn get_src(&self) -> Option<miette::NamedSource<String>> {
        match self {
            Self::DivisionByZero { source_code, .. }
            | Self::NumberOverflow { source_code, .. }
            | Self::TypeMismatch { source_code, .. }
            | Self::UnexpectedTypeError { source_code, .. }
            | Self::UndefinedReference { source_code, .. }
            | Self::AlreadyDeclared { source_code, .. }
            | Self::WrongNumberOfArguments { source_code, .. }
            | Self::IndexOutOfBounds { source_code, .. }
            | Self::WrongIndexType { source_code, .. }
            | Self::InvalidRangeBounds { source_code, .. }
            | Self::InvalidIndex { source_code, .. }
            | Self::InvalidNumberAssociativeArrayKey { source_code, .. }
            | Self::InvalidTypeAssociativeArrayKey { source_code, .. }
            | Self::AssociativeArrayKeyNotFound { source_code, .. }
            | Self::NotIterable { source_code, .. }
            | Self::InvalidArgument { source_code, .. }
            | Self::ImmutableString { source_code, .. }
            | Self::InvalidTimestamp { source_code, .. }
            | Self::DateIsoParseError { source_code, .. }
            | Self::InvalidTimeZoneString { source_code, .. } => source_code.clone(),
        }
    }

    pub fn set_span(&mut self, new_span: &parser::ast::AstSpan) {
        match self {
            Self::DivisionByZero {
                span, source_code, ..
            }
            | Self::NumberOverflow {
                span, source_code, ..
            }
            | Self::TypeMismatch {
                span, source_code, ..
            }
            | Self::UnexpectedTypeError {
                span, source_code, ..
            }
            | Self::UndefinedReference {
                span, source_code, ..
            }
            | Self::AlreadyDeclared {
                span, source_code, ..
            }
            | Self::WrongNumberOfArguments {
                span, source_code, ..
            }
            | Self::IndexOutOfBounds {
                span, source_code, ..
            }
            | Self::WrongIndexType {
                span, source_code, ..
            }
            | Self::InvalidRangeBounds {
                span, source_code, ..
            }
            | Self::InvalidIndex {
                span, source_code, ..
            }
            | Self::InvalidNumberAssociativeArrayKey {
                span, source_code, ..
            }
            | Self::InvalidTypeAssociativeArrayKey {
                span, source_code, ..
            }
            | Self::AssociativeArrayKeyNotFound {
                span, source_code, ..
            }
            | Self::NotIterable {
                span, source_code, ..
            }
            | Self::InvalidArgument {
                span, source_code, ..
            }
            | Self::ImmutableString {
                span, source_code, ..
            }
            | Self::InvalidTimestamp {
                span, source_code, ..
            }
            | Self::DateIsoParseError {
                span, source_code, ..
            }
            | Self::InvalidTimeZoneString {
                span, source_code, ..
            } => {
                *span = Some(new_span.into());
                *source_code = Some(new_span.source.as_ref().clone().into());
            }
        }
    }
}

macro_rules! span_src {
    ($span:expr) => {
        Some($span.source.as_ref().clone().into())
    };
}

macro_rules! err_span {
    ($span:expr) => {
        Some($span.into())
    };
}

pub(crate) use err_span;
pub(crate) use span_src;
