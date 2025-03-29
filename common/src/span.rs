use crate::source::IdentifiedSource;

pub trait Span: Clone + std::fmt::Debug + PartialEq + ariadne::Span {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn source(&self) -> IdentifiedSource;
    fn extract(&self, source: &str) -> String;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSpan {
    start: usize,
    end: usize,
    source: IdentifiedSource,
    label: Option<String>,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize, source: IdentifiedSource) -> Self {
        SourceSpan {
            start,
            end,
            source,
            label: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}

impl ariadne::Span for SourceSpan {
    type SourceId = IdentifiedSource;

    fn source(&self) -> &Self::SourceId {
        &self.source
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl Span for SourceSpan {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn source(&self) -> IdentifiedSource {
        self.source
    }

    fn extract(&self, source: &str) -> String {
        source[self.start..self.end].to_string()
    }
}
