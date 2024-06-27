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
            Err(err) => return format!("LexicalError: {:?}", err),
        };

        let mut parser = Parser::new(&tokens);

        let ast = match parser.parse() {
            Ok(expr) => expr,
            Err(err) => return format!("ParserError: {}", err),
        };

        let result = self.interpreter.interpret_expr(ast);

        format!("{}", result)
    }
}

impl Default for Tenda {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reflexive_property() {
        let mut tenda = Tenda::new();

        let result = tenda.run("1".to_string());
        let expected = "1";

        assert_eq!(result, expected, "{} evaluates to itself", expected);
    }

    #[test]
    fn sum_of_integers() {
        let mut tenda = Tenda::new();

        let result = tenda.run("1 + 2".to_string());
        let expected = "3";

        assert_eq!(result, expected, "sum of integers");
    }

    #[test]
    fn extra_spacing() {
        let mut tenda = Tenda::new();

        let result = tenda.run(" 1  +  2 ".to_string());
        let expected = "3";

        assert_eq!(
            result, expected,
            "sum of integers with additional spacing between characters"
        );
    }

    #[test]
    fn precedence_of_operations() {
        let mut tenda = Tenda::new();

        let result = tenda.run("1 + 2 * 3".to_string());
        let expected = "7";

        assert_eq!(
            result, expected,
            "expression depending on precendence of operations"
        );
    }

    #[test]
    fn chain_of_additions() {
        let mut tenda = Tenda::new();

        let result = tenda.run("1 + 1 + 1 + 1".to_string());
        let expected = "4";

        assert_eq!(result, expected, "chain of additions");
    }

    #[test]
    fn chain_of_operations() {
        let mut tenda = Tenda::new();

        let result = tenda.run("1 - 1 - 1 + 2 * 4 / 2 / 2".to_string());
        let expected = "1";

        assert_eq!(result, expected, "chain of basic arithmetical operations");
    }

    #[test]
    fn negative_number() {
        let mut tenda = Tenda::new();

        let result = tenda.run("-1".to_string());
        let expected = "-1";

        assert_eq!(result, expected, "negative number evaluates to itself");
    }

    #[test]
    fn negative_number_with_operation() {
        let mut tenda = Tenda::new();

        let result = tenda.run("-1 + -1".to_string());
        let expected = "-2";

        assert_eq!(result, expected, "addition of negative numbers");
    }

    #[test]
    fn parentheses() {
        let mut tenda = Tenda::new();

        let result = tenda.run("(1 + 1)".to_string());
        let expected = "2";

        assert_eq!(result, expected, "addition of integers within parentheses");
    }

    #[test]
    fn parentheses_with_operation() {
        let mut tenda = Tenda::new();

        let result = tenda.run("(1 + 1) * 2".to_string());
        let expected = "4";

        assert_eq!(
            result, expected,
            "multiplication of integer with parentheses"
        );
    }

    #[test]
    fn illegal_leading_zero() {
        let mut tenda = Tenda::new();

        let result = tenda.run("01".to_string());
        let expected = "1";

        assert_ne!(result, expected, "illegal leading zeroes");
    }

    #[test]
    fn legal_leading_zero() {
        let mut tenda = Tenda::new();

        let result = tenda.run("0.1".to_string());
        let expected = "0.1";

        assert_eq!(result, expected, "legal leading zeroes");
    }

    #[test]
    fn reflexive_zero() {
        let mut tenda = Tenda::new();

        let result = tenda.run("0".to_string());
        let expected = "0";

        assert_eq!(result, expected, "zero evaluates to itself");
    }
}
