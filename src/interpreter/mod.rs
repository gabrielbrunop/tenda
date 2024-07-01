use core::fmt;
use std::collections::HashMap;

use crate::{
    ast::{BinaryOp, Decl, Expr, Stmt, UnaryOp},
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
    ($kind:expr, $message:expr, $($params:expr),*) => {{
        use RuntimeErrorKind::*;
        RuntimeError {
            kind: $kind,
            message: Some(format!($message, $($params),*)),
        }
    }};
}

macro_rules! type_error {
    ($message:expr, $($params:expr),*) => {
        runtime_error!(
            TypeError,
            $message,
            $($params.get_type()),*
        )
    };
}

pub struct Interpreter {
    state: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            state: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, stmt_list: Vec<Stmt>) -> Result<Value, RuntimeError> {
        use Stmt::*;

        let stmt_iter = stmt_list.iter();
        let mut last_value = Value::Nil;

        for stmt in stmt_iter {
            let value = match stmt {
                Expr(expr) => self.interpret_expr(expr)?,
                Decl(decl) => self.interpret_decl(decl)?,
            };

            last_value = value;
        }

        Ok(last_value)
    }

    pub fn interpret_decl(&mut self, decl: &Decl) -> Result<Value, RuntimeError> {
        match decl {
            Decl::Local { name, value } => {
                if self.state.contains_key(name) {
                    return Err(runtime_error!(
                        AlreadyDeclared,
                        "a variável identificada por '{}' já foi declarada neste escopo",
                        name
                    ));
                }

                let value = self.interpret_expr(value)?;

                self.state.insert(name.clone(), value);
            }
        };

        Ok(Value::Nil)
    }

    pub fn interpret_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        use Expr::*;

        match expr {
            Binary { lhs, op, rhs } => self.interpret_binary_op(lhs, *op, rhs),
            Unary { op, rhs } => self.interpret_unary_op(*op, rhs),
            Grouping { expr } => self.interpret_expr(expr),
            Literal { value } => Ok(value.clone()),
            Variable { name } => self
                .state
                .get(name)
                .ok_or(runtime_error!(
                    UndefinedReference,
                    "a variável identificada por '{}' não está definida neste escopo",
                    name
                ))
                .cloned(),
        }
    }

    fn interpret_binary_op(
        &mut self,
        lhs: &Expr,
        op: BinaryOp,
        rhs: &Expr,
    ) -> Result<Value, RuntimeError> {
        use crate::ast::BinaryOp::*;
        use Value::*;

        let lhs = self.interpret_expr(lhs)?;
        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Add => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs + rhs),
                (String(lhs), String(rhs)) => String(format!("{}{}", lhs, rhs)),
                (String(lhs), rhs) => String(format!("{}{}", lhs, rhs)),
                (lhs, String(rhs)) => String(format!("{}{}", lhs, rhs)),
                (lhs, rhs) => {
                    return Err(type_error!("não é possível somar '{}' e '{}'", lhs, rhs))
                }
            },
            Subtract => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs - rhs),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível subtrair '{}' de '{}'",
                        rhs,
                        lhs
                    ))
                }
            },
            Multiply => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs * rhs),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível multiplicar '{}' por '{}'",
                        lhs,
                        rhs
                    ))
                }
            },
            Divide => match (lhs, rhs) {
                (Number(_), Number(rhs)) if rhs == 0.0 => {
                    return Err(runtime_error!(DivisionByZero))
                }
                (Number(lhs), Number(rhs)) => Number(lhs / rhs),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível dividir '{}' por '{}'",
                        lhs,
                        rhs
                    ))
                }
            },
            Exponentiation => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs.powf(rhs)),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível elevar '{}' à potência de '{}'",
                        lhs,
                        rhs
                    ))
                }
            },
            Modulo => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs % rhs),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível encontrar o resto da divisão de '{}' por '{}'",
                        lhs,
                        rhs
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
            Greater => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs > rhs),
                (String(lhs), String(rhs)) => Boolean(lhs > rhs),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível aplicar a operação de 'maior que' para '{}' e '{}'",
                        lhs,
                        rhs
                    ))
                }
            },
            GreaterOrEqual => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs >= rhs),
                (String(lhs), String(rhs)) => Boolean(lhs >= rhs),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível aplicar a operação de 'maior ou igual' para '{}' e '{}'",
                        lhs,
                        rhs
                    ))
                }
            },
            Less => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs < rhs),
                (String(lhs), String(rhs)) => Boolean(lhs < rhs),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível aplicar a operação de 'menor que' para '{}' e '{}'",
                        lhs,
                        rhs
                    ))
                }
            },
            LessOrEqual => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs <= rhs),
                (String(lhs), String(rhs)) => Boolean(lhs <= rhs),
                (lhs, rhs) => {
                    return Err(type_error!(
                        "não é possível aplicar a operação de 'menor ou igual a' para '{}' e '{}'",
                        lhs,
                        rhs
                    ))
                }
            },
            Assignment => match (lhs, rhs) {
                (String(lhs), rhs) => {
                    if !self.state.contains_key(&lhs) {
                        return Err(runtime_error!(
                            AlreadyDeclared,
                            "a variável identificada por '{}' precisa ser definida com `seja`",
                            lhs
                        ));
                    }

                    self.state.insert(lhs.clone(), rhs.clone());
                    rhs
                }
                _ => unreachable!(),
            },
        };

        match expr {
            Number(value) if value.abs() == f64::INFINITY => Err(runtime_error!(NumberOverflow)),
            _ => Ok(expr),
        }
    }

    fn interpret_unary_op(&mut self, op: UnaryOp, rhs: &Expr) -> Result<Value, RuntimeError> {
        use crate::ast::UnaryOp::*;
        use Value::*;

        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Negative => match rhs {
                Number(rhs) => Number(-rhs),
                _ => return Err(type_error!("não é possível negar '{}'", rhs)),
            },
            LogicalNot => match rhs {
                Number(rhs) if rhs == 0.0 => Boolean(false),
                Number(_) => Boolean(true),
                Boolean(rhs) => Boolean(!rhs),
                Nil => Boolean(false),
                _ => {
                    return Err(type_error!(
                        "a negação lógica não é uma operação válida para '{}'",
                        rhs
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
            DivisionByZero => "divisão por zero não é permitida".to_string(),
            NumberOverflow => "números muito grandes não são permitidos".to_string(),
            TypeError => "erro de tipo".to_string(),
            UndefinedReference => "referência não encontrada".to_string(),
            AlreadyDeclared => "variável já declarada neste escopo".to_string(),
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
    UndefinedReference,
    AlreadyDeclared,
}

#[cfg(test)]
mod tests;
