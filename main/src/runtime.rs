use interpreter::interpreter::Interpreter;
use miette::GraphicalReportHandler;
use parser::{parser::Parser, parser_error::ParserError};
use scanner::{scanner::Scanner, scanner_error::LexicalError};
use std::{
    io::{self},
    rc::Rc,
};

macro_rules! format_error_messages {
    ($errs:expr, $msg:literal) => {
        $errs
            .into_iter()
            .map(|e| format!("{}: {}", $msg, e))
            .collect::<Vec<String>>()
            .join("\n")
    };
}

pub struct Runtime {
    interpreter: Interpreter,
}

impl Runtime {
    pub fn new(platform: impl interpreter::platform::Platform + 'static) -> Self {
        Runtime {
            interpreter: Interpreter::new(platform),
        }
    }

    pub fn run(&mut self, source: String, name: String) -> Result<String, RuntimeError> {
        let mut scanner = Scanner::new(&source);

        let tokens = match scanner.scan() {
            Ok(token) => token,
            Err(errs) => return Err(errs.into()),
        };

        let named_source = Rc::new(parser::ast::NamedSource::new(name, source));
        let mut parser = Parser::new(&tokens, Some(named_source));

        let ast = match parser.parse() {
            Ok(expr) => expr,
            Err(errs) => return Err(errs.into()),
        };

        let result = match self.interpreter.eval(&ast) {
            Ok(val) => val,
            Err(err) => return Err((*err).into()),
        };

        Ok(format!("{}", result))
    }
}

pub enum RuntimeError {
    Scanner(Vec<LexicalError>),
    Parser(Vec<ParserError>),
    Runtime(interpreter::runtime_error::RuntimeError),
}

impl From<Vec<LexicalError>> for RuntimeError {
    fn from(err: Vec<LexicalError>) -> Self {
        RuntimeError::Scanner(err)
    }
}

impl From<Vec<ParserError>> for RuntimeError {
    fn from(err: Vec<ParserError>) -> Self {
        RuntimeError::Parser(err)
    }
}

impl From<interpreter::runtime_error::RuntimeError> for RuntimeError {
    fn from(err: interpreter::runtime_error::RuntimeError) -> Self {
        RuntimeError::Runtime(err)
    }
}

impl RuntimeError {
    pub fn print_to_stderr(&self) {
        match self {
            RuntimeError::Scanner(err) => eprint!("{}", format_error_messages!(err, "Erro léxico")),
            RuntimeError::Parser(err) => {
                eprint!("{}", format_error_messages!(err, "Erro sintático"))
            }
            RuntimeError::Runtime(err) => {
                let mut output_string = String::new();
                let handler = GraphicalReportHandler::new();

                handler
                    .render_report(&mut output_string, err)
                    .map_err(|_fmt_err| io::Error::new(io::ErrorKind::Other, "formatting error"))
                    .unwrap();

                eprint!("{}", output_string);
            }
        }
    }
}
