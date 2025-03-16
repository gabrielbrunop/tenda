use std::{cell::RefCell, rc::Rc};

use parser::ast::{self, Access, DeclVisitor, ExprVisitor, StmtVisitor};

use crate::{
    environment::{Environment, StoredValue},
    function::{add_native_fn, native_fn, param_list, Function},
    runtime_error::{runtime_err, type_err, Result, RuntimeError, RuntimeErrorKind},
    stack::Stack,
    value::{Value, ValueType},
};

pub struct Interpreter {
    stack: Stack,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut stack = Stack::new();

        Self::setup_native_bindings(&mut stack);

        Interpreter { stack }
    }

    pub fn eval(&mut self, ast: &ast::Ast) -> Result<Value> {
        let mut last_value = Value::Nil;

        let ast::Ast(ast) = ast;

        for stmt in ast {
            let value = self.interpret_stmt(stmt)?;

            last_value = value;

            if self.stack.has_return() || self.stack.has_break() || self.stack.has_continue() {
                break;
            }
        }

        Ok(last_value)
    }

    fn interpret_stmt(&mut self, stmt: &ast::Stmt) -> Result<Value> {
        use ast::Stmt::*;

        match stmt {
            Expr(expr) => self.visit_expr(expr),
            Decl(decl) => self.visit_decl(decl),
            Cond(cond) => self.visit_cond(cond),
            Block(block) => self.visit_block(block),
            Return(return_value) => self.visit_return(return_value),
            While(while_stmt) => self.visit_while(while_stmt),
            Break(break_stmt) => self.visit_break(break_stmt),
            Continue(continue_stmt) => self.visit_continue(continue_stmt),
        }
    }
}

impl StmtVisitor<Result<Value>> for Interpreter {
    fn visit_decl(&mut self, decl: &ast::Decl) -> Result<Value> {
        use ast::Decl::*;

        let _ = match decl {
            Local(local) => self.visit_local(local),
            Function(function) => self.visit_function(function),
        };

        Ok(Value::Nil)
    }

    fn visit_expr(&mut self, expr: &ast::Expr) -> Result<Value> {
        use ast::Expr::*;

        match expr {
            Binary(binary) => self.visit_binary(binary),
            Unary(unary) => self.visit_unary(unary),
            Grouping(grouping) => self.visit_grouping(grouping),
            List(list) => self.visit_list(list),
            Literal(literal) => self.visit_literal(literal),
            Call(call) => self.visit_call(call),
            Assign(assign) => self.visit_assign(assign),
            Access(indexing) => self.visit_access(indexing),
            Variable(variable) => self.visit_variable(variable),
        }
    }

    fn visit_block(&mut self, block: &ast::Block) -> Result<Value> {
        let ast::Block(ast) = block;

        self.stack.push(Environment::new());

        self.eval(ast)?;

        self.stack.pop();

        Ok(Value::Nil)
    }

    fn visit_return(&mut self, return_stmt: &ast::Return) -> Result<Value> {
        let ast::Return(value) = return_stmt;

        if let Some(expr) = value {
            let value = self.visit_expr(expr)?;
            self.stack.set_return(StoredValue::new_unique(value));
        }

        Ok(Value::Nil)
    }

    fn visit_cond(&mut self, cond: &ast::Cond) -> Result<Value> {
        let ast::Cond {
            cond,
            then,
            or_else,
        } = cond;

        if self.visit_expr(cond)?.to_bool() {
            self.interpret_stmt(then)?;
        } else if let Some(or_else) = or_else {
            self.interpret_stmt(or_else)?;
        };

        Ok(Value::Nil)
    }

    fn visit_while(&mut self, while_stmt: &ast::While) -> Result<Value> {
        let ast::While { cond, body } = while_stmt;

        while self.visit_expr(cond)?.to_bool() && !self.stack.has_break() {
            self.interpret_stmt(body)?;

            self.stack.set_continue(false);
        }

        self.stack.set_break(false);

        Ok(Value::Nil)
    }

    fn visit_break(&mut self, _break_stmt: &ast::Break) -> Result<Value> {
        self.stack.set_break(true);

        Ok(Value::Nil)
    }

    fn visit_continue(&mut self, _continue_stmt: &ast::Continue) -> Result<Value> {
        self.stack.set_continue(true);

        Ok(Value::Nil)
    }
}

impl DeclVisitor<Result<Value>> for Interpreter {
    fn visit_local(&mut self, local: &ast::LocalDecl) -> Result<Value> {
        let ast::LocalDecl { name, value, .. } = local;

        if self.stack.local_exists(name) {
            let name = name.to_string();
            Err(RuntimeErrorKind::AlreadyDeclared(name))?;
        }

        let value = self.visit_expr(value)?;
        let value = match local.is_captured_var {
            true => StoredValue::new_shared(value),
            false => StoredValue::new_unique(value),
        };

        let _ = self.stack.define(name.clone(), value);

        Ok(Value::Nil)
    }

    fn visit_function(&mut self, function: &ast::FunctionDecl) -> Result<Value> {
        let ast::FunctionDecl {
            name,
            params,
            body,
            captured_vars,
            ..
        } = function;

        let mut env = Environment::new();

        for var_name in captured_vars {
            if let Some(value_in_parent) = self.stack.find(var_name) {
                env.set(var_name.clone(), value_in_parent.clone());
            }
        }

        for param in params {
            env.set(param.clone(), StoredValue::new_shared(Value::Nil));
        }

        let func = Function::new(
            name.to_string(),
            params.clone(),
            Box::new(env),
            Some(body.clone()),
            |params, body, interpreter, captured_env| {
                interpreter.stack.push(*captured_env.clone());

                for (param_name, arg_value) in params.into_iter() {
                    let _ = interpreter
                        .stack
                        .set(param_name.clone(), StoredValue::new_unique(arg_value));
                }

                if let Some(body) = body {
                    interpreter.interpret_stmt(&body)?;
                }

                let value = interpreter
                    .stack
                    .consume_return()
                    .map(|v| v.clone_value())
                    .unwrap_or(Value::Nil);

                interpreter.stack.pop();

                Ok(value)
            },
        );

        let _ = self
            .stack
            .define(name.clone(), StoredValue::new_unique(Value::Function(func)));

        Ok(Value::Nil)
    }
}

impl ExprVisitor<Result<Value>> for Interpreter {
    fn visit_binary(&mut self, binary: &ast::BinaryOp) -> Result<Value> {
        let ast::BinaryOp { lhs, op, rhs } = binary;

        use ast::BinaryOperator::*;
        use Value::*;

        let lhs = self.visit_expr(lhs)?;
        let rhs = self.visit_expr(rhs)?;

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
                (Number(_), Number(0.0)) => Err(RuntimeErrorKind::DivisionByZero)?,
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
            ast::BinaryOperator::Range => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Value::Range(lhs, rhs),
                (lhs, rhs) => type_err!(
                    "não é possível criar um intervalo entre '{}' e '{}'",
                    lhs,
                    rhs
                ),
            },
        };

        match expr {
            Number(value) if value.abs() == f64::INFINITY => Err(RuntimeErrorKind::NumberOverflow)?,
            _ => Ok(expr),
        }
    }

    fn visit_unary(&mut self, unary: &ast::UnaryOp) -> Result<Value> {
        let ast::UnaryOp { op, rhs } = unary;

        use ast::UnaryOperator::*;
        use Value::*;

        let rhs = self.visit_expr(rhs)?;

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

    fn visit_call(&mut self, call: &ast::Call) -> Result<Value> {
        let ast::Call { callee, args } = call;

        let callee = self.visit_expr(callee)?;

        let args = args
            .iter()
            .map(|arg| self.visit_expr(arg))
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
                &func.context.captured_env,
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

    fn visit_access(&mut self, index: &ast::Access) -> Result<Value> {
        let Access { subscripted, index } = index;

        let subscripted = match self.visit_expr(subscripted)? {
            Value::List(list) => list,
            val => type_err!(
                "não é possível indexar '{}'; esperado {}",
                val.kind(),
                ValueType::List
            ),
        };

        let index = match self.visit_expr(index)? {
            Value::Number(num) => num as usize,
            val => type_err!(
                "{} não é um tipo válido para indexação; esperado {}",
                val,
                ValueType::Number
            ),
        };

        let list = subscripted.borrow_mut();

        if index >= list.len() {
            Err(RuntimeErrorKind::IndexOutOfBounds {
                index,
                len: list.len(),
            })?;
        }

        Ok(list[index].clone())
    }

    fn visit_list(&mut self, list: &ast::List) -> Result<Value> {
        let mut elements = Vec::with_capacity(list.elements.len());

        for e in &list.elements {
            elements.push(self.visit_expr(e)?);
        }

        Ok(Value::List(Rc::new(RefCell::new(elements))))
    }

    fn visit_grouping(&mut self, grouping: &ast::Grouping) -> Result<Value> {
        let ast::Grouping { expr } = grouping;
        self.visit_expr(expr)
    }

    fn visit_literal(&mut self, literal: &ast::Literal) -> Result<Value> {
        let ast::Literal { value } = literal;
        Ok(value.clone().into())
    }

    fn visit_variable(&mut self, variable: &ast::Variable) -> Result<Value> {
        let ast::Variable { name, .. } = variable;

        self.stack
            .find(name)
            .map(|v| v.clone_value())
            .ok_or(RuntimeErrorKind::UndefinedReference(name.clone()))
            .map_err(|e| e.into())
    }

    fn visit_assign(&mut self, assign: &ast::Assign) -> Result<Value> {
        let ast::Assign {
            name: variable,
            value,
        } = assign;

        match &**variable {
            ast::Expr::Variable(ast::Variable { name, .. }) => {
                let value = self.visit_expr(value)?;

                let result = self
                    .stack
                    .set(name.clone(), StoredValue::new_unique(value.clone()));

                match result {
                    Ok(_) => Ok(value),
                    Err(_) => runtime_err!(
                        RuntimeErrorKind::UndefinedReference(name.clone()),
                        format!(
                            "a variável identificada por '{}' precisa ser definida com `seja`",
                            name
                        )
                    ),
                }
            }
            ast::Expr::Access(ast::Access { index, subscripted }) => {
                let index = self.visit_expr(index)?;
                let subscripted = self.visit_expr(subscripted)?;
                let value = self.visit_expr(value)?;

                let subscripted = match subscripted {
                    Value::List(list) => list,
                    val => type_err!(
                        "não é possível indexar '{}'; esperado {}",
                        val.kind(),
                        ValueType::List
                    ),
                };

                let index = match index {
                    Value::Number(num) => num as usize,
                    val => type_err!(
                        "{} não é um tipo válido para indexação; esperado {}",
                        val,
                        ValueType::Number
                    ),
                };

                let mut list = subscripted.borrow_mut();

                if index >= list.len() {
                    Err(RuntimeErrorKind::IndexOutOfBounds {
                        index,
                        len: list.len(),
                    })?;
                }

                list[index] = value.clone();

                Ok(value)
            }
            _ => unreachable!(),
        }
    }
}

impl Interpreter {
    fn setup_native_bindings(stack: &mut Stack) {
        add_native_fn!(
            stack,
            native_fn!("exiba", param_list!["texto"], |args, _, _, _| {
                let text = match &args[0].1 {
                    Value::String(value) => value.to_string(),
                    value => format!("{}", value),
                };

                println!("{}", text);

                Ok(Value::Nil)
            })
        );
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
