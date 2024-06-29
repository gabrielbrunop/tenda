use core::fmt;

use crate::{
    ast::{BinaryOp, Expr, UnaryOp},
    value::Value,
};

macro_rules! runtime_error {
    ($kind:expr) => {{
        use RuntimeErrorKind::*;
        RuntimeError {
            kind: $kind,
            message: None,
        }
    }};
    ($kind:expr, $message:expr) => {{
        use RuntimeErrorKind::*;
        RuntimeError {
            kind: $kind,
            message: Some($message.to_string()),
        }
    }};
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn interpret_expr(&self, expr: Expr) -> Result<Value, RuntimeError> {
        use Expr::*;

        match expr {
            Binary { lhs, op, rhs } => self.interpret_binary_op(*lhs, op, *rhs),
            Unary { op, rhs } => self.interpret_unary_op(op, *rhs),
            Grouping { expr } => self.interpret_expr(*expr),
            Literal { value } => Ok(value),
        }
    }

    fn interpret_binary_op(
        &self,
        lhs: Expr,
        op: BinaryOp,
        rhs: Expr,
    ) -> Result<Value, RuntimeError> {
        use crate::ast::BinaryOp::*;
        use Value::*;

        let lhs = self.interpret_expr(lhs)?;
        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Add => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs + rhs),
                (lhs, rhs) => {
                    return Err(runtime_error!(
                        TypeError,
                        format!("cannot add {} to {}", lhs.get_type(), rhs.get_type())
                    ))
                }
            },
            Subtract => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs - rhs),
                (lhs, rhs) => {
                    return Err(runtime_error!(
                        TypeError,
                        format!("cannot subtract {} from {}", rhs.get_type(), lhs.get_type())
                    ))
                }
            },
            Multiply => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs * rhs),
                (lhs, rhs) => {
                    return Err(runtime_error!(
                        TypeError,
                        format!("cannot multiply {} by {}", lhs.get_type(), rhs.get_type())
                    ))
                }
            },
            Divide => match (lhs, rhs) {
                (_, Number(rhs)) if rhs == 0.0 => return Err(runtime_error!(DivisionByZero)),
                (Number(lhs), Number(rhs)) => Number(lhs / rhs),
                (lhs, rhs) => {
                    return Err(runtime_error!(
                        TypeError,
                        format!("cannot divide {} by {}", lhs.get_type(), rhs.get_type())
                    ))
                }
            },
            Exponentiation => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs.powf(rhs)),
                (lhs, rhs) => {
                    return Err(runtime_error!(
                        TypeError,
                        format!(
                            "cannot raise {} to the power of {}",
                            lhs.get_type(),
                            rhs.get_type()
                        )
                    ))
                }
            },
            Modulo => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs % rhs),
                (lhs, rhs) => {
                    return Err(runtime_error!(
                        TypeError,
                        format!("cannot mod {} by {}", lhs.get_type(), rhs.get_type())
                    ))
                }
            },
            Equality => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs == rhs),
                (Boolean(lhs), Boolean(rhs)) => Boolean(lhs == rhs),
                (String(lhs), String(rhs)) => Boolean(lhs == rhs),
                _ => Boolean(false),
            },
            Inequality => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs != rhs),
                (Boolean(lhs), Boolean(rhs)) => Boolean(lhs != rhs),
                (String(lhs), String(rhs)) => Boolean(lhs != rhs),
                _ => Boolean(false),
            },
        };

        match expr {
            Number(value) if value.abs() == f64::INFINITY => Err(runtime_error!(NumberOverflow)),
            _ => Ok(expr),
        }
    }

    fn interpret_unary_op(&self, op: UnaryOp, rhs: Expr) -> Result<Value, RuntimeError> {
        use crate::ast::UnaryOp::*;
        use Value::*;

        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Negative => match rhs {
                Number(rhs) => Number(-rhs),
                _ => {
                    return Err(runtime_error!(
                        TypeError,
                        format!("cannot negate {}", rhs.get_type())
                    ))
                }
            },
        };

        Ok(expr)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
    message: Option<String>,
}

impl RuntimeError {
    pub fn message(&self) -> String {
        use RuntimeErrorKind::*;

        if let Some(message) = &self.message {
            return message.to_string();
        }

        match &self.kind {
            DivisionByZero => "division by zero".to_string(),
            NumberOverflow => "number overflow".to_string(),
            TypeError => "type error".to_string(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

#[derive(Debug)]
pub enum RuntimeErrorKind {
    DivisionByZero,
    NumberOverflow,
    TypeError,
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, scanner::Scanner};

    use super::*;

    fn run_expr<T: ToString>(string: T) -> Result<Value, RuntimeError> {
        let input = string.to_string();

        let mut scanner = Scanner::new(&input);

        let tokens = match scanner.scan() {
            Ok(tokens) => tokens,
            Err(err) => panic!("could not scan input: {:?}", err),
        };

        let mut parser = Parser::new(&tokens);

        let ast = match parser.parse() {
            Ok(expr) => expr,
            Err(err) => panic!("could not parse tokens: {:?}", err),
        };

        let interpreter: Interpreter = Interpreter::new();

        interpreter.interpret_expr(ast)
    }

    #[test]
    fn division_by_zero() {
        run_expr("0/0")
            .is_ok()
            .then(|| panic!("division by zero should error"));
    }

    #[test]
    fn reflexive_zero() {
        assert_eq!(
            run_expr("0").unwrap(),
            Value::Number(0.0),
            "zero evaluates to itself"
        )
    }

    #[test]
    fn sum_of_integers() {
        assert_eq!(
            run_expr("1 + 2").unwrap(),
            Value::Number(3.0),
            "sum of integers"
        )
    }

    #[test]
    fn precedence_of_operations() {
        assert_eq!(
            run_expr("1 + 2 * 3").unwrap(),
            Value::Number(7.0),
            "expression depending on order of precendence of operations"
        )
    }

    #[test]
    fn chain_of_additions() {
        assert_eq!(
            run_expr("1 + 1 + 1 + 1").unwrap(),
            Value::Number(4.0),
            "chain of additions"
        )
    }

    #[test]
    fn chain_of_operations() {
        assert_eq!(
            run_expr("1 - 1 - 1 + 2 * 4 / 2 / 2").unwrap(),
            Value::Number(1.0),
            "chain of basic arithmetical operations"
        )
    }

    #[test]
    fn negative_number() {
        assert_eq!(
            run_expr("-1").unwrap(),
            Value::Number(-1.0),
            "negative number evaluates to itself"
        )
    }

    #[test]
    fn negative_number_with_operation() {
        assert_eq!(
            run_expr("-1 + -1").unwrap(),
            Value::Number(-2.0),
            "addition of negative numbers"
        )
    }

    #[test]
    fn parentheses() {
        assert_eq!(
            run_expr("(1 + 1)").unwrap(),
            Value::Number(2.0),
            "addition of integers within parentheses"
        )
    }

    #[test]
    fn parentheses_with_operation() {
        assert_eq!(
            run_expr("(1 + 1) * 2").unwrap(),
            Value::Number(4.0),
            "multiplication of integer with parentheses"
        )
    }

    #[test]
    fn number_overflow() {
        run_expr("10^1000")
            .is_ok()
            .then(|| panic!("overflow should error"));
    }

    #[test]
    fn reflexive_boolean() {
        assert_eq!(
            run_expr("verdadeiro").unwrap(),
            Value::Boolean(true),
            "`verdadeiro` evaluates to itself"
        );

        assert_eq!(
            run_expr("falso").unwrap(),
            Value::Boolean(false),
            "`falso` evaluates to itself"
        );
    }

    #[test]
    fn reflexive_string() {
        assert_eq!(
            run_expr("\"Hello, world!\"").unwrap(),
            Value::String("Hello, world!".to_string()),
            "string evaluates to itself"
        )
    }

    #[test]
    fn reflexive_nil() {
        assert_eq!(
            run_expr("Nada").unwrap(),
            Value::Nil,
            "nil evaluates to itself"
        )
    }

    #[test]
    fn numeric_equality() {
        assert_eq!(
            run_expr("1 for 1").unwrap(),
            Value::Boolean(true),
            "1 is equal to 1"
        )
    }

    #[test]
    fn numeric_inequality() {
        assert_eq!(
            run_expr("1 for 2").unwrap(),
            Value::Boolean(false),
            "1 is not equal to 2"
        )
    }
}
