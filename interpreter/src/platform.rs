use std::fmt::Debug;

#[non_exhaustive]
pub enum FileErrorKind {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    Other,
}

pub trait Platform: Debug {
    fn println(&self, message: &str);
    fn write(&self, message: &str);
    fn rand(&self) -> f64;
    fn read_file(&self, path: &str) -> Result<String, FileErrorKind>;
    fn write_file(&self, path: &str, content: &str) -> Result<(), FileErrorKind>;
    fn remove_file(&self, path: &str) -> Result<(), FileErrorKind>;
    fn list_files(&self, path: &str) -> Result<Vec<String>, FileErrorKind>;
    fn create_dir(&self, path: &str) -> Result<(), FileErrorKind>;
    fn remove_dir(&self, path: &str) -> Result<(), FileErrorKind>;
    fn list_dirs(&self, path: &str) -> Result<Vec<String>, FileErrorKind>;
    fn current_dir(&self) -> Result<String, FileErrorKind>;
    fn args(&self) -> Vec<String>;
    fn exit(&self, code: i32);
    fn sleep(&self, seconds: f64);
}
