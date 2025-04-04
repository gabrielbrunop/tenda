use tenda_common::span::SourceSpan;
use tenda_reporting::{DiagnosticConfig, HasDiagnosticHooks};
use tenda_reporting_derive::Diagnostic;
use thiserror::Error;

use crate::{
    associative_array::AssociativeArrayKey,
    value::{Value, ValueType},
};

pub type Result<T> = std::result::Result<T, Box<RuntimeError>>;

#[derive(Debug, Clone, PartialEq)]
pub struct StackFrame {
    pub function_name: Option<String>,
    pub location: Option<SourceSpan>,
}

impl StackFrame {
    pub fn new(function_name: Option<String>, location: Option<SourceSpan>) -> Self {
        Self {
            function_name,
            location,
        }
    }
}

impl From<StackFrame> for tenda_reporting::StackFrame<SourceSpan> {
    fn from(val: StackFrame) -> Self {
        let function_name = val.function_name.unwrap_or_else(|| "<anônimo>".to_string());

        tenda_reporting::StackFrame::new(function_name, val.location)
    }
}

#[derive(Error, Debug, PartialEq, Clone, Diagnostic)]
#[accept_hooks]
#[report("erro de execução")]
pub enum RuntimeError {
    #[error("divisão por zero não é permitida")]
    DivisionByZero {
        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("operação inválida para os tipos '{}' e '{}'", .first.to_string(), .second.to_string())]
    TypeMismatch {
        first: ValueType,
        second: ValueType,

        #[span]
        span: Option<SourceSpan>,

        #[message]
        message: Option<String>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("esperado valor de tipo '{}', encontrado '{}'", .expected.to_string(), .found.to_string())]
    UnexpectedTypeError {
        expected: ValueType,
        found: ValueType,

        #[message]
        message: Option<String>,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("a variável identificada por '{}' não está definida neste escopo", .var_name)]
    UndefinedReference {
        var_name: String,

        #[span]
        span: Option<SourceSpan>,

        #[help]
        help: Option<String>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("variável identificada por {0} já está declarada neste escopo", .var_name)]
    AlreadyDeclared {
        var_name: String,

        #[span]
        span: Option<SourceSpan>,

        #[help]
        help: Option<String>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("número de argumentos incorreto: esperado {}, encontrado {}", .expected, .found)]
    WrongNumberOfArguments {
        expected: usize,
        found: usize,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("índice fora dos limites: índice {}, tamanho {}", .index, .len)]
    IndexOutOfBounds {
        index: usize,
        len: usize,

        #[span]
        span: Option<SourceSpan>,

        #[help]
        help: Vec<String>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("não é possível acessar um valor do tipo '{}'", .value.to_string())]
    WrongIndexType {
        value: ValueType,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("limites de intervalo precisam ser números inteiros finitos: encontrado '{}'", .bound)]
    InvalidRangeBounds {
        bound: f64,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("índice de lista precisa ser um número inteiro positivo e finito: encontrado '{}'", .index)]
    InvalidIndex {
        index: f64,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("chave de dicionário precisa ser número inteiro ou texto: encontrado '{}'", .key)]
    InvalidNumberAssociativeArrayKey {
        key: f64,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("chave de dicionário precisa ser número inteiro ou texto: encontrado '{}'", .key)]
    InvalidTypeAssociativeArrayKey {
        key: ValueType,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("chave de dicionário não encontrada: '{}'", .key.to_string())]
    AssociativeArrayKeyNotFound {
        key: AssociativeArrayKey,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("não é possível iterar sobre um valor do tipo '{}'", .value.to_string())]
    NotIterable {
        value: ValueType,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("o valor do tipo '{}' não é um argumento válido para a função", .value.to_string())]
    InvalidArgument {
        value: Value,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("textos são imutáveis e não podem ser modificados")]
    ImmutableString {
        #[span]
        span: Option<SourceSpan>,

        #[help]
        help: Option<String>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("timestamp inválido: {}", .timestamp.to_string())]
    InvalidTimestamp {
        timestamp: i64,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("falha ao analisar data ISO: {}", .source)]
    DateIsoParseError {
        source: chrono::ParseError,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("fuso horário inválido: '{tz_str}'")]
    InvalidTimeZoneString {
        tz_str: String,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("valor inválido para conversão para tipo '{}'", .value.to_string())]
    InvalidValueForConversion {
        value: Value,

        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },
}

impl HasDiagnosticHooks<SourceSpan> for RuntimeError {
    fn hooks() -> &'static [fn(&Self, DiagnosticConfig<SourceSpan>) -> DiagnosticConfig<SourceSpan>]
    {
        &[add_stacktrace]
    }
}

fn add_stacktrace(
    runtime_error: &RuntimeError,
    config: DiagnosticConfig<SourceSpan>,
) -> DiagnosticConfig<SourceSpan> {
    let stacktrace = match runtime_error.get_stacktrace().cloned() {
        Some(stacktrace) => stacktrace,
        None => return config,
    };

    config.stacktrace(stacktrace.into_iter().map(|c| c.into()).collect())
}
