pub use ariadne::*;

pub trait Diagnostic<S: crate::Span> {
    fn to_report(&self) -> crate::Report<S>;
    fn set_span(&mut self, new_span: &S);
    fn get_span(&self) -> Option<S>;
    fn get_message(&self) -> Option<String>;
    fn get_labels(&self) -> Vec<S>;
    fn get_helps(&self) -> Vec<String>;
    fn get_notes(&self) -> Vec<String>;
    fn build_report_config(&self) -> DiagnosticConfig<S>;
}

#[derive(Debug, Clone)]
pub struct DiagnosticConfig<S: crate::Span> {
    pub span: Option<S>,
    pub labels: Vec<S>,
    pub helps: Vec<String>,
    pub notes: Vec<String>,
    pub message: String,
    pub stacktrace: Vec<crate::StackFrame<S>>,
}

impl<S: crate::Span> DiagnosticConfig<S> {
    pub fn new(
        span: Option<S>,
        labels: Vec<S>,
        helps: Vec<String>,
        notes: Vec<String>,
        message: String,
        stacktrace: Vec<crate::StackFrame<S>>,
    ) -> Self {
        Self {
            span,
            labels,
            helps,
            notes,
            message,
            stacktrace,
        }
    }

    pub fn span(mut self, new_span: Option<S>) -> Self {
        self.span = new_span;
        self
    }

    pub fn labels(mut self, new_labels: Vec<S>) -> Self {
        self.labels.extend(new_labels);
        self
    }

    pub fn helps(mut self, new_helps: Vec<String>) -> Self {
        self.helps.extend(new_helps);
        self
    }

    pub fn notes(mut self, new_notes: Vec<String>) -> Self {
        self.notes.extend(new_notes);
        self
    }

    pub fn message(mut self, new_message: String) -> Self {
        self.message = new_message;
        self
    }

    pub fn stacktrace(mut self, new_stacktrace: Vec<crate::StackFrame<S>>) -> Self {
        self.stacktrace.extend(new_stacktrace);
        self
    }
}

type DiagnosticConfigFn<T, S> = fn(&T, DiagnosticConfig<S>) -> DiagnosticConfig<S>;

pub trait HasDiagnosticHooks<S: crate::Span> {
    fn hooks() -> &'static [DiagnosticConfigFn<Self, S>] {
        &[]
    }
}
