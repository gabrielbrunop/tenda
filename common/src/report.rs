use crate::span::SourceSpan;

pub trait Report {
    fn to_report(&self) -> ariadne::Report<SourceSpan>;
    fn set_span(&mut self, new_span: &SourceSpan);
    fn get_span(&self) -> Option<SourceSpan>;
    fn get_message(&self) -> Option<String>;
    fn get_labels(&self) -> Vec<SourceSpan>;
    fn get_helps(&self) -> Vec<String>;
    fn get_notes(&self) -> Vec<String>;
}
