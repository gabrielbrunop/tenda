use interpreter::Interpreter;
use scanner::Scanner;

use crate::parser::Parser;

mod ast;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod value;

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
            Err(errs) => {
                return errs
                    .iter()
                    .map(|e| format!("Erro léxico: {}", e))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        };

        let mut parser = Parser::new(&tokens);

        let ast = match parser.parse() {
            Ok(expr) => expr,
            Err(err) => return format!("Erro sintático: {}", err),
        };

        let result = match self.interpreter.interpret(ast) {
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
