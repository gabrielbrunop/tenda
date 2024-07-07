use core::fmt;
use std::rc::Rc;

use crate::{
    add_native_fn,
    environment::Stack,
    function::Function,
    native_fn, param_list,
    stmt::{BinaryOp, Block, Cond, Decl, Expr, Stmt, UnaryOp},
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
    ($kind:expr, $message:literal, $($params:expr),*) => {{
        use RuntimeErrorKind::*;
        RuntimeError {
            kind: $kind,
            message: Some(format!($message, $($params),*)),
        }
    }};
}

macro_rules! type_error {
    ($message:literal, $($params:expr),*) => {
        runtime_error!(
            TypeError,
            $message,
            $($params.get_type()),*
        )
    };
}

pub struct Interpreter {
    stack: Stack,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut stack = Stack::new();

        add_native_fn!(
            stack,
            native_fn!("exiba", param_list!["texto"], |args, _, _| {
                let text = match &args["texto"] {
                    Value::String(value) => value.to_string(),
                    value => format!("{}", value),
                };

                println!("{}", text);

                Ok(Value::Nil)
            })
        );

        Interpreter { stack }
    }

    pub fn eval(&mut self, stmt_list: &[Stmt]) -> Result<Value, RuntimeError> {
        self.interpret(stmt_list)
    }

    fn interpret(&mut self, stmt_list: &[Stmt]) -> Result<Value, RuntimeError> {
        let stmt_iter = stmt_list.iter();
        let mut last_value = Value::Nil;

        for stmt in stmt_iter {
            let value = self.interpret_stmt(stmt)?;

            last_value = value;
        }

        Ok(last_value)
    }

    fn interpret_stmt(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        use Stmt::*;

        match stmt {
            Expr(expr) => self.interpret_expr(expr),
            Decl(decl) => self.interpret_decl(decl),
            Cond(cond) => self.interpret_if(cond),
            Block(block) => self.interpret_block(block),
            Return(return_value) => self.interpret_return(return_value),
        }
    }

    fn interpret_decl(&mut self, decl: &Decl) -> Result<Value, RuntimeError> {
        match decl {
            Decl::Local { name, value } => {
                if self.stack.local_exists(name) {
                    return Err(runtime_error!(
                        AlreadyDeclared,
                        "a variável identificada por '{}' já foi declarada neste escopo",
                        name
                    ));
                }

                let value = self.interpret_expr(value)?;

                let _ = self.stack.define(name.clone(), value);
            }
            Decl::Function { name, params, body } => {
                let body = (*body).clone();

                let func = Function::new(
                    name.to_string(),
                    params.clone(),
                    Some(body),
                    |params, body, interpreter| {
                        interpreter.stack.allocate();

                        for (param, arg) in params.iter() {
                            let _ = interpreter.stack.define(param.clone(), arg.clone());
                        }

                        if let Some(body) = body {
                            interpreter.interpret_stmt(&body)?;
                        }

                        let value = interpreter.stack.consume_return().unwrap_or(Value::Nil);

                        interpreter.stack.pop();

                        Ok(value)
                    },
                );

                let _ = self
                    .stack
                    .define(name.clone(), Value::Function(Rc::new(func)));
            }
        };

        Ok(Value::Nil)
    }

    fn interpret_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        use Expr::*;

        match expr {
            Binary { lhs, op, rhs } => self.interpret_binary_op(lhs, *op, rhs),
            Unary { op, rhs } => self.interpret_unary_op(*op, rhs),
            Grouping { expr } => self.interpret_expr(expr),
            Literal { value } => Ok(value.clone()),
            Call { callee, args } => self.interpret_call(callee, args),
            Variable { name } => self
                .stack
                .find(name)
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
        use crate::stmt::BinaryOp::*;
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
                    self.stack.set(lhs.clone(), rhs.clone()).map_err(|_| {
                        runtime_error!(
                            AlreadyDeclared,
                            "a variável identificada por '{}' precisa ser definida com `seja`",
                            lhs
                        )
                    })?;

                    rhs
                }
                _ => unreachable!(),
            },
            LogicalAnd => {
                if lhs.to_bool() {
                    rhs
                } else {
                    lhs
                }
            }
            LogicalOr => {
                if lhs.to_bool() {
                    lhs
                } else {
                    rhs
                }
            }
        };

        match expr {
            Number(value) if value.abs() == f64::INFINITY => Err(runtime_error!(NumberOverflow)),
            _ => Ok(expr),
        }
    }

    fn interpret_unary_op(&mut self, op: UnaryOp, rhs: &Expr) -> Result<Value, RuntimeError> {
        use crate::stmt::UnaryOp::*;
        use Value::*;

        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Negative => match rhs {
                Number(rhs) => Number(-rhs),
                _ => return Err(type_error!("não é possível negar '{}'", rhs)),
            },
            LogicalNot => Value::Boolean(!rhs.to_bool()),
        };

        Ok(expr)
    }

    fn interpret_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, RuntimeError> {
        let callee = self.interpret_expr(callee)?;

        let args = args
            .iter()
            .map(|arg| self.interpret_expr(arg))
            .collect::<Result<Vec<_>, _>>()?;

        match callee {
            Value::Function(func) if args.len() != func.context.params.len() => {
                Err(runtime_error!(
                    TypeError,
                    "esperado {} argumento(s) mas recebido {}",
                    func.context.params.len(),
                    args.len()
                ))
            }
            Value::Function(func) => (func.object)(
                func.context
                    .params
                    .iter()
                    .zip(args)
                    .map(|(a, b)| (a.clone(), b))
                    .collect(),
                func.context.body.clone(),
                self,
            ),
            _ => Err(type_error!(
                "não é possível chamar '{}' como função",
                callee
            )),
        }
    }

    fn interpret_block(&mut self, block: &Block) -> Result<Value, RuntimeError> {
        self.stack.allocate();

        self.interpret(block)?;

        self.stack.pop();

        Ok(Value::Nil)
    }

    fn interpret_return(&mut self, return_value: &Option<Expr>) -> Result<Value, RuntimeError> {
        if let Some(expr) = return_value {
            let value = self.interpret_expr(expr)?;
            self.stack.set_return(value);
        }

        Ok(Value::Nil)
    }

    fn interpret_if(&mut self, cond: &Cond) -> Result<Value, RuntimeError> {
        let Cond {
            cond,
            then,
            or_else,
        } = cond;

        if self.interpret_expr(cond)?.to_bool() {
            self.interpret_stmt(then)?;
        } else if let Some(or_else) = or_else {
            self.interpret_stmt(or_else)?;
        };

        Ok(Value::Nil)
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
