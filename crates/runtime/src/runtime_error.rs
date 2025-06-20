use tenda_common::span::SourceSpan;
use tenda_reporting::{Diagnostic, DiagnosticConfig, HasDiagnosticHooks};
use tenda_reporting_derive::Diagnostic;
use thiserror::Error;

use crate::{
    associative_array::AssociativeArrayKey,
    value::{Value, ValueType},
};

pub type Result<T> = std::result::Result<T, Box<RuntimeError>>;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionName {
    Anonymous,
    TopLevel,
    Named(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackFrame {
    pub function_name: FunctionName,
    pub location: Option<SourceSpan>,
}

impl StackFrame {
    pub fn new(function_name: FunctionName, location: Option<SourceSpan>) -> Self {
        Self {
            function_name,
            location,
        }
    }
}

impl From<StackFrame> for tenda_reporting::StackFrame<SourceSpan> {
    fn from(val: StackFrame) -> Self {
        let function_name = match val.function_name {
            FunctionName::Anonymous => "<anônimo>".to_string(),
            FunctionName::TopLevel => "<raiz>".to_string(),
            FunctionName::Named(name) => name,
        };

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

    #[error("valores do tipo '{}' são imutáveis e não podem ser modificados", .value.to_string())]
    ImmutableValueType {
        value: ValueType,

        #[span]
        span: Option<SourceSpan>,

        #[help]
        help: Option<String>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("não é possível reatribuir um módulo ou seus membros")]
    ReassignModule {
        #[span]
        span: Option<SourceSpan>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("variável '{}' não pode ser reatribuída", .name)]
    UnreassignableVariable {
        name: String,

        #[span]
        span: Option<SourceSpan>,

        #[help]
        help: Option<String>,

        #[metadata]
        stacktrace: Vec<StackFrame>,
    },

    #[error("valor está congelado e não pode ser modificado")]
    FrozenValue {
        #[span]
        span: Option<SourceSpan>,

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

/// Builds a Vec<StackFrame> from a RuntimeError’s stacktrace:
/// - Returns `None` if there’s no stacktrace or it’s empty.
/// - Otherwise, returns `Some(frames)` where `frames.len() == original.len() + 1`:
///     1. **First frame**: the first entry’s function name paired with the error span.
///     2. **Middle frames**: for each adjacent pair in the original stacktrace,
///        the later function name is paired with the previous call-site location.
///     3. **Last frame**: a “top-level” placeholder (`FunctionName::TopLevel`)
///        paired with the last call-site location.
///
/// Note: at the moment in the runtime we’re still associating function names with
/// their call sites rather than their actual in-function error sites.
/// This is a temporary workaround until we can implement a better solution.
fn build_stack_frames(runtime_error: &RuntimeError) -> Option<Vec<StackFrame>> {
    let st = runtime_error.get_stacktrace()?;

    if st.is_empty() {
        return None;
    }

    let mut frames = Vec::with_capacity(st.len() + 1);

    frames.push(StackFrame::new(
        st[0].function_name.clone(),
        runtime_error.get_span().clone(),
    ));

    for window in st.windows(2) {
        let (prev, curr) = (&window[0], &window[1]);

        frames.push(StackFrame::new(
            curr.function_name.clone(),
            prev.location.clone(),
        ));
    }

    frames.push(StackFrame::new(
        FunctionName::TopLevel,
        st.last().unwrap().location.clone(),
    ));

    Some(frames)
}

fn add_stacktrace(
    runtime_error: &RuntimeError,
    config: DiagnosticConfig<SourceSpan>,
) -> DiagnosticConfig<SourceSpan> {
    match build_stack_frames(runtime_error) {
        Some(frames) => config.stacktrace(frames.into_iter().map(Into::into).collect()),
        None => config,
    }
}

macro_rules! attach_span_if_missing {
    ($err:expr, $span:expr) => {{
        if $err.get_span().is_none() {
            $err.set_span($span);
        }

        $err
    }};
}

pub(crate) use attach_span_if_missing;
