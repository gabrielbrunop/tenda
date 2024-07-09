use std::rc::Rc;

use parser::ast::{Ast, BinaryOp, Block, Cond, Decl, Expr, Stmt, UnaryOp};

use crate::{
    add_native_fn,
    function::Function,
    native_fn, param_list, runtime_err,
    runtime_error::{Result, RuntimeError, RuntimeErrorKind},
    stack::Stack,
    type_err,
    value::{Value, ValueType},
};

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

    pub fn eval(&mut self, ast: Ast) -> Result<Value> {
        self.interpret(ast)
    }

    fn interpret(&mut self, ast: Ast) -> Result<Value> {
        let mut last_value = Value::Nil;

        for stmt in ast {
            let value = self.interpret_stmt(stmt)?;

            last_value = value;
        }

        Ok(last_value)
    }

    fn interpret_stmt(&mut self, stmt: Stmt) -> Result<Value> {
        use Stmt::*;

        match stmt {
            Expr(expr) => self.interpret_expr(expr),
            Decl(decl) => self.interpret_decl(decl),
            Cond(cond) => self.interpret_if(cond),
            Block(block) => self.interpret_block(block),
            Return(return_value) => self.interpret_return(return_value),
        }
    }

    fn interpret_decl(&mut self, decl: Decl) -> Result<Value> {
        match decl {
            Decl::Local { name, value } => {
                if self.stack.local_exists(&name) {
                    let name = name.to_string();
                    Err(RuntimeErrorKind::AlreadyDeclared(name))?;
                }

                let value = self.interpret_expr(*value)?;

                let _ = self.stack.define(name, value);
            }
            Decl::Function { name, params, body } => {
                let func = Function::new(
                    name.to_string(),
                    params,
                    Some(body),
                    |params, body, interpreter| {
                        interpreter.stack.allocate();

                        for (param, arg) in params.into_iter() {
                            let _ = interpreter.stack.define(param, arg);
                        }

                        if let Some(body) = body {
                            interpreter.interpret_stmt(*body)?;
                        }

                        let value = interpreter.stack.consume_return().unwrap_or(Value::Nil);

                        interpreter.stack.pop();

                        Ok(value)
                    },
                );

                let _ = self.stack.define(name, Value::Function(Rc::new(func)));
            }
        };

        Ok(Value::Nil)
    }

    fn interpret_expr(&mut self, expr: Expr) -> Result<Value> {
        use Expr::*;

        match expr {
            Binary { lhs, op, rhs } => self.interpret_binary_op(*lhs, op, *rhs),
            Unary { op, rhs } => self.interpret_unary_op(op, *rhs),
            Grouping { expr } => self.interpret_expr(*expr),
            Literal { value } => Ok(value.into()),
            Call { callee, args } => self.interpret_call(*callee, args),
            Variable { name } => self
                .stack
                .find(&name)
                .ok_or(RuntimeErrorKind::UndefinedReference(name))
                .map_err(|e| e.into())
                .cloned(),
        }
    }

    fn interpret_binary_op(&mut self, lhs: Expr, op: BinaryOp, rhs: Expr) -> Result<Value> {
        use BinaryOp::*;
        use Value::*;

        let lhs = self.interpret_expr(lhs)?;
        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Add => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs + rhs),
                (String(lhs), String(rhs)) => String(format!("{}{}", lhs, rhs)),
                (String(lhs), rhs) => String(format!("{}{}", lhs, rhs)),
                (lhs, String(rhs)) => String(format!("{}{}", lhs, rhs)),
                (lhs, rhs) => type_err!("não é possível somar '{}' e '{}'", lhs, rhs),
            },
            Subtract => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs - rhs),
                (lhs, rhs) => type_err!("não é possível subtrair '{}' de '{}'", rhs, lhs),
            },
            Multiply => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs * rhs),
                (lhs, rhs) => {
                    type_err!("não é possível multiplicar '{}' por '{}'", lhs, rhs)
                }
            },
            Divide => match (lhs, rhs) {
                (Number(_), Number(rhs)) if rhs == 0.0 => Err(RuntimeErrorKind::DivisionByZero)?,
                (Number(lhs), Number(rhs)) => Number(lhs / rhs),
                (lhs, rhs) => type_err!("não é possível dividir '{}' por '{}'", lhs, rhs),
            },
            Exponentiation => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs.powf(rhs)),
                (lhs, rhs) => {
                    type_err!("não é possível elevar '{}' à potência de '{}'", lhs, rhs)
                }
            },
            Modulo => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs % rhs),
                (lhs, rhs) => type_err!(
                    "não é possível encontrar o resto da divisão de '{}' por '{}'",
                    lhs,
                    rhs
                ),
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
                (lhs, rhs) => type_err!(
                    "não é possível aplicar a operação de 'maior que' para '{}' e '{}'",
                    lhs,
                    rhs
                ),
            },
            GreaterOrEqual => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs >= rhs),
                (String(lhs), String(rhs)) => Boolean(lhs >= rhs),
                (lhs, rhs) => type_err!(
                    "não é possível aplicar a operação de 'maior ou igual' para '{}' e '{}'",
                    lhs,
                    rhs
                ),
            },
            Less => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs < rhs),
                (String(lhs), String(rhs)) => Boolean(lhs < rhs),
                (lhs, rhs) => type_err!(
                    "não é possível aplicar a operação de 'menor que' para '{}' e '{}'",
                    lhs,
                    rhs
                ),
            },
            LessOrEqual => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs <= rhs),
                (String(lhs), String(rhs)) => Boolean(lhs <= rhs),
                (lhs, rhs) => type_err!(
                    "não é possível aplicar a operação de 'menor ou igual a' para '{}' e '{}'",
                    lhs,
                    rhs
                ),
            },
            Assignment => match (lhs, rhs) {
                (String(lhs), rhs) => {
                    if self.stack.set(lhs.clone(), rhs.clone()).is_err() {
                        runtime_err!(
                            RuntimeErrorKind::AlreadyDeclared(lhs.clone()),
                            format!(
                                "a variável identificada por '{}' precisa ser definida com `seja`",
                                lhs
                            )
                        )
                    }

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
            Number(value) if value.abs() == f64::INFINITY => Err(RuntimeErrorKind::NumberOverflow)?,
            _ => Ok(expr),
        }
    }

    fn interpret_unary_op(&mut self, op: UnaryOp, rhs: Expr) -> Result<Value> {
        use UnaryOp::*;
        use Value::*;

        let rhs = self.interpret_expr(rhs)?;

        let expr = match op {
            Negative => match rhs {
                Number(rhs) => Number(-rhs),
                _ => type_err!(
                    "não é possível negar valor de tipo '{1}'; esperado '{0}'",
                    ValueType::Number,
                    rhs
                ),
            },
            LogicalNot => Value::Boolean(!rhs.to_bool()),
        };

        Ok(expr)
    }

    fn interpret_call(&mut self, callee: Expr, args: Vec<Expr>) -> Result<Value> {
        let callee = self.interpret_expr(callee)?;

        let args = args
            .into_iter()
            .map(|arg| self.interpret_expr(arg))
            .collect::<Result<Vec<_>>>()?;

        match callee {
            Value::Function(func) if args.len() != func.context.params.len() => {
                Err(RuntimeErrorKind::WrongNumberOfArguments {
                    expected: func.context.params.len(),
                    found: args.len(),
                })?
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
            _ => runtime_err!(
                RuntimeErrorKind::TypeError {
                    expected: ValueType::Function,
                    found: callee.kind()
                },
                format!("não é possível chamar '{}' como função", callee.kind())
            ),
        }
    }

    fn interpret_block(&mut self, block: Block) -> Result<Value> {
        self.stack.allocate();

        self.interpret(block)?;

        self.stack.pop();

        Ok(Value::Nil)
    }

    fn interpret_return(&mut self, return_value: Option<Expr>) -> Result<Value> {
        if let Some(expr) = return_value {
            let value = self.interpret_expr(expr)?;
            self.stack.set_return(value);
        }

        Ok(Value::Nil)
    }

    fn interpret_if(&mut self, cond: Cond) -> Result<Value> {
        let Cond {
            cond,
            then,
            or_else,
        } = cond;

        if self.interpret_expr(*cond)?.to_bool() {
            self.interpret_stmt(*then)?;
        } else if let Some(or_else) = or_else {
            self.interpret_stmt(*or_else)?;
        };

        Ok(Value::Nil)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
