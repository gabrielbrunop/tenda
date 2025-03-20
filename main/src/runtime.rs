use interpreter::interpreter::Interpreter;
use parser::parser::Parser;
use scanner::scanner::Scanner;

macro_rules! print_errors {
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
    pub fn new() -> Runtime {
        Runtime {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, string: String) -> Result<String, String> {
        let mut scanner = Scanner::new(&string);

        let tokens = match scanner.scan() {
            Ok(token) => token,
            Err(errs) => return Err(print_errors!(errs, "Erro léxico")),
        };

        let mut parser = Parser::new(&tokens);

        let ast = match parser.parse() {
            Ok(expr) => expr,
            Err(errs) => return Err(print_errors!(errs, "Erro sintático")),
        };

        let result = match self.interpreter.eval(&ast) {
            Ok(val) => val,
            Err(err) => return Err(format!("Erro semântico: {}", err)),
        };

        Ok(format!("{}", result))
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
