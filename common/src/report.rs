use crate::span::SourceSpan;

pub trait Report {
    fn to_report(&self) -> ariadne::Report<SourceSpan>;
    fn set_span(&mut self, new_span: &SourceSpan);
    fn get_span(&self) -> Option<SourceSpan>;
    fn get_message(&self) -> Option<String>;
    fn get_labels(&self) -> Vec<SourceSpan>;
    fn get_helps(&self) -> Vec<String>;
    fn get_notes(&self) -> Vec<String>;
    fn build_report_config(&self) -> ReportConfig;
}

#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub span: Option<SourceSpan>,
    pub labels: Vec<SourceSpan>,
    pub helps: Vec<String>,
    pub notes: Vec<String>,
    pub message: String,
    pub stacktrace: Vec<ariadne::StackFrame<SourceSpan>>,
}

impl ReportConfig {
    pub fn new(
        span: Option<SourceSpan>,
        labels: Vec<SourceSpan>,
        helps: Vec<String>,
        notes: Vec<String>,
        message: String,
        stacktrace: Vec<ariadne::StackFrame<SourceSpan>>,
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

    pub fn span(mut self, new_span: Option<SourceSpan>) -> Self {
        self.span = new_span;
        self
    }

    pub fn labels(mut self, new_labels: Vec<SourceSpan>) -> Self {
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

    pub fn stacktrace(mut self, new_stacktrace: Vec<ariadne::StackFrame<SourceSpan>>) -> Self {
        self.stacktrace.extend(new_stacktrace);
        self
    }
}

pub trait HasReportHooks {
    fn hooks() -> &'static [fn(&Self, ReportConfig) -> ReportConfig] {
        &[]
    }
}
