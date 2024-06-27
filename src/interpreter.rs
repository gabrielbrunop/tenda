use core::fmt;

use crate::{
    ast::{BinaryOp, Expr, UnaryOp},
    value::Value,
};

macro_rules! runtime_error {
    ($kind:expr) => {{
        use RuntimeErrorKind::*;
        RuntimeError { kind: $kind }
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

        let lhs = self.interpret_expr(lhs)?;
        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Add => lhs + rhs,
            Subtract => lhs - rhs,
            Multiply => lhs * rhs,
            Divide => {
                if rhs == Value::Number(0.0) {
                    return runtime_error!(DivisionByZero).into();
                }

                lhs / rhs
            }
            Exponentiation => lhs.to_number().powf(rhs.into()).into(),
            Modulo => lhs % rhs,
        };

        match expr {
            Value::Number(number) if number.abs() == f64::INFINITY => {
                runtime_error!(NumberOverflow).into()
            }
            _ => Ok(expr),
        }
    }

    fn interpret_unary_op(&self, op: UnaryOp, rhs: Expr) -> Result<Value, RuntimeError> {
        use crate::ast::UnaryOp::*;

        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Negative => -rhs,
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
}

impl RuntimeError {
    pub fn message(&self) -> String {
        use RuntimeErrorKind::*;

        match &self.kind {
            DivisionByZero => "division by zero".to_string(),
            NumberOverflow => "number overflow".to_string(),
        }
    }
}

impl<T> From<RuntimeError> for Result<T, RuntimeError> {
    fn from(val: RuntimeError) -> Self {
        Err(val)
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
    fn reflexive_property() {
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
}
