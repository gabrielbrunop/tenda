use delegate::delegate;
use tenda_os_platform::Platform as OSPlatform;
use tenda_runtime::ValueType;

#[derive(Debug)]
pub struct Platform {
    pub inner: OSPlatform,
    pub send: fn(ProtocolMessage),
    pub read_line: fn() -> String,
}

pub enum ProtocolMessage {
    Ready,
    Unlock,
    Output(String),
    Result(ValueType, String),
    Error(String),
}

impl Platform {
    pub fn new(send: fn(ProtocolMessage), read_line: fn() -> String) -> Self {
        Platform {
            inner: OSPlatform,
            send,
            read_line,
        }
    }
}

impl tenda_runtime::Platform for Platform {
    delegate! {
        to self.inner {
            fn rand(&self) -> f64;
            fn read_file(&self, path: &str) -> Result<String, tenda_runtime::FileErrorKind>;
            fn write_file(&self, path: &str, content: &str) -> Result<(), tenda_runtime::FileErrorKind>;
            fn remove_file(&self, path: &str) -> Result<(), tenda_runtime::FileErrorKind>;
            fn list_files(&self, path: &str) -> Result<Vec<String>, tenda_runtime::FileErrorKind>;
            fn create_dir(&self, path: &str) -> Result<(), tenda_runtime::FileErrorKind>;
            fn remove_dir(&self, path: &str) -> Result<(), tenda_runtime::FileErrorKind>;
            fn list_dirs(&self, path: &str) -> Result<Vec<String>, tenda_runtime::FileErrorKind>;
            fn current_dir(&self) -> Result<String, tenda_runtime::FileErrorKind>;
            fn file_append(&self, path: &str, content: &str) -> Result<(), tenda_runtime::FileErrorKind>;
            fn args(&self) -> Vec<String>;
            fn exit(&self, code: i32);
            fn sleep(&self, seconds: f64);
            fn date_now(&self) -> i64;
            fn timezone_offset(&self) -> i32;
        }
    }

    fn println(&self, message: &str) {
        let message = format!("{}\n", message);

        (self.send)(ProtocolMessage::Output(message.to_string()))
    }

    fn print(&self, message: &str) {
        (self.send)(ProtocolMessage::Output(message.to_string()));
    }

    fn write(&self, message: &str) {
        (self.send)(ProtocolMessage::Output(message.to_string()));
    }

    fn read_line(&self) -> String {
        (self.send)(ProtocolMessage::Ready);

        let input = (self.read_line)();

        (self.send)(ProtocolMessage::Unlock);

        input.trim_end_matches('\n').to_string()
    }
}
