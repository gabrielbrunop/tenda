use std::io::Write;

#[derive(Debug)]
pub struct Platform;

fn map_file_error_kind(kind: std::io::ErrorKind) -> interpreter::platform::FileErrorKind {
    use interpreter::platform::FileErrorKind;

    match kind {
        std::io::ErrorKind::NotFound => FileErrorKind::NotFound,
        std::io::ErrorKind::PermissionDenied => FileErrorKind::PermissionDenied,
        std::io::ErrorKind::AlreadyExists => FileErrorKind::AlreadyExists,
        std::io::ErrorKind::Other => FileErrorKind::Other,
        _ => FileErrorKind::Other,
    }
}

impl interpreter::platform::Platform for Platform {
    fn println(&self, message: &str) {
        println!("{}", message);
    }

    fn write(&self, message: &str) {
        print!("{}", message);
    }

    fn rand(&self) -> f64 {
        rand::random()
    }

    fn read_file(&self, path: &str) -> Result<String, interpreter::platform::FileErrorKind> {
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn write_file(
        &self,
        path: &str,
        content: &str,
    ) -> Result<(), interpreter::platform::FileErrorKind> {
        match std::fs::write(path, content) {
            Ok(_) => Ok(()),
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn remove_file(&self, path: &str) -> Result<(), interpreter::platform::FileErrorKind> {
        match std::fs::remove_file(path) {
            Ok(_) => Ok(()),
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn list_files(&self, path: &str) -> Result<Vec<String>, interpreter::platform::FileErrorKind> {
        match std::fs::read_dir(path) {
            Ok(entries) => Ok(entries
                .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
                .map(|name| name.to_string_lossy().to_string())
                .collect()),
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn create_dir(&self, path: &str) -> Result<(), interpreter::platform::FileErrorKind> {
        match std::fs::create_dir(path) {
            Ok(_) => Ok(()),
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn remove_dir(&self, path: &str) -> Result<(), interpreter::platform::FileErrorKind> {
        match std::fs::remove_dir(path) {
            Ok(_) => Ok(()),
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn list_dirs(&self, path: &str) -> Result<Vec<String>, interpreter::platform::FileErrorKind> {
        match std::fs::read_dir(path) {
            Ok(entries) => Ok(entries
                .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
                .map(|name| name.to_string_lossy().to_string())
                .collect()),
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn current_dir(&self) -> Result<String, interpreter::platform::FileErrorKind> {
        match std::env::current_dir() {
            Ok(path) => Ok(path.to_string_lossy().to_string()),
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn file_append(
        &self,
        path: &str,
        content: &str,
    ) -> Result<(), interpreter::platform::FileErrorKind> {
        match std::fs::OpenOptions::new().append(true).open(path) {
            Ok(mut file) => match file.write_all(content.as_bytes()) {
                Ok(_) => Ok(()),
                Err(error) => Err(map_file_error_kind(error.kind())),
            },
            Err(error) => Err(map_file_error_kind(error.kind())),
        }
    }

    fn args(&self) -> Vec<String> {
        std::env::args().collect()
    }

    fn exit(&self, code: i32) {
        std::process::exit(code);
    }

    fn sleep(&self, seconds: f64) {
        std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
    }
}
