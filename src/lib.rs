use interpreter::Interpreter;
use scanner::Scanner;

use crate::parser::Parser;

mod environment;
mod function;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;
mod value;

macro_rules! print_errors {
    ($errs:expr, $msg:literal) => {
        $errs
            .into_iter()
            .map(|e| format!("{}: {}", $msg, e))
            .collect::<Vec<String>>()
            .join("\n")
    };
}

pub struct Tenda {
    interpreter: Interpreter,
}

impl Tenda {
    pub fn new() -> Tenda {
        Tenda {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, string: String) -> String {
        let mut scanner = Scanner::new(&string);

        let tokens = match scanner.scan() {
            Ok(token) => token,
            Err(errs) => return print_errors!(errs, "Erro léxico"),
        };

        let mut parser = Parser::new(&tokens);

        let ast = match parser.parse() {
            Ok(expr) => expr,
            Err(errs) => return print_errors!(errs, "Erro sintático"),
        };

        let result = match self.interpreter.eval(&ast) {
            Ok(val) => val,
            Err(err) => return format!("Erro semântico: {}", err),
        };

        format!("{}", result)
    }
}

impl Default for Tenda {
    fn default() -> Self {
        Self::new()
    }
}
