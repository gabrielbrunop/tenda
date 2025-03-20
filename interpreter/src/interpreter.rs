use std::{cell::RefCell, rc::Rc};

use parser::ast::{self, Access, DeclVisitor, ExprVisitor, StmtVisitor};

use crate::{
    builtins,
    environment::{Environment, StoredValue},
    function::Function,
    runtime_error::{runtime_err, type_err, Result, RuntimeError, RuntimeErrorKind},
    stack::Stack,
    value::{AssociativeArrayKey, Value, ValueType},
};

#[derive(Debug)]
pub struct Interpreter {
    stack: Stack,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut stack = Stack::new();

        builtins::setup_native_bindings(&mut stack);

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
            ForEach(for_each) => self.visit_for_each(for_each),
            Break(break_stmt) => self.visit_break(break_stmt),
            Continue(continue_stmt) => self.visit_continue(continue_stmt),
        }
    }
}

impl StmtVisitor<Result<Value>> for Interpreter {
    fn visit_decl(&mut self, decl: &ast::Decl) -> Result<Value> {
        use ast::Decl::*;

        match decl {
            Local(local) => self.visit_local(local)?,
            Function(function) => self.visit_function(function)?,
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
            AssociativeArray(associative_array) => self.visit_associative_array(associative_array),
            AnonymousFunction(anonymous_function) => {
                self.visit_anonymous_function(anonymous_function)
            }
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
            self.stack.set_return(StoredValue::new(value));
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

    fn visit_for_each(&mut self, for_each: &ast::ForEach) -> Result<Value> {
        let ast::ForEach {
            item,
            iterable,
            body,
            ..
        } = for_each;

        let iterable = self.visit_expr(iterable)?;

        if !iterable.is_iterable() {
            return runtime_err!(NotIterable {
                value: iterable.kind(),
            });
        }

        for value in iterable {
            let mut env = Environment::new();
            let stored_value = if item.captured {
                StoredValue::new_shared(value.clone())
            } else {
                StoredValue::new(value.clone())
            };

            env.set(item.name.clone(), stored_value);

            self.stack.push(env);
            self.interpret_stmt(body)?;

            if self.stack.has_break() {
                break;
            }

            self.stack.set_continue(false);
            self.stack.pop();
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

            return runtime_err!(AlreadyDeclared(name));
        }

        let value = self.visit_expr(value)?;

        let value = match local.captured {
            true => StoredValue::new_shared(value),
            false => StoredValue::new(value),
        };

        let _ = self.stack.define(name.clone(), value);

        Ok(Value::Nil)
    }

    fn visit_function(&mut self, function: &ast::FunctionDecl) -> Result<Value> {
        let ast::FunctionDecl {
            name, params, body, ..
        } = function;

        let func = self.create_function(params, body.clone());

        let _ = self
            .stack
            .define(name.clone(), StoredValue::new(Value::Function(func)));

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
                (List(lhs), List(rhs)) => {
                    let mut list = lhs.borrow().clone();
                    list.extend_from_slice(&rhs.borrow());

                    List(Rc::new(RefCell::new(list)))
                }
                (lhs, rhs) => return type_err!("não é possível somar '{}' e '{}'", lhs, rhs),
            },
            Subtract => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs - rhs),
                (lhs, rhs) => return type_err!("não é possível subtrair '{}' de '{}'", rhs, lhs),
            },
            Multiply => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs * rhs),
                (lhs, rhs) => {
                    return type_err!("não é possível multiplicar '{}' por '{}'", lhs, rhs)
                }
            },
            Divide => match (lhs, rhs) {
                (Number(_), Number(0.0)) => return runtime_err!(DivisionByZero),
                (Number(lhs), Number(rhs)) => Number(lhs / rhs),
                (lhs, rhs) => return type_err!("não é possível dividir '{}' por '{}'", lhs, rhs),
            },
            Exponentiation => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs.powf(rhs)),
                (lhs, rhs) => {
                    return type_err!("não é possível elevar '{}' à potência de '{}'", lhs, rhs)
                }
            },
            Modulo => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs % rhs),
                (lhs, rhs) => {
                    return type_err!(
                        "não é possível encontrar o resto da divisão de '{}' por '{}'",
                        lhs,
                        rhs
                    )
                }
            },
            Equality => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs == rhs),
                (Boolean(lhs), Boolean(rhs)) => Boolean(lhs == rhs),
                (String(lhs), String(rhs)) => Boolean(lhs == rhs),
                (List(lhs), List(rhs)) => Boolean(lhs == rhs),
                (Value::Range(lhs_start1, lhs_end1), Value::Range(rhs_start2, rhs_end2)) => {
                    Boolean(lhs_start1 == rhs_start2 && lhs_end1 == rhs_end2)
                }
                (AssociativeArray(lhs), AssociativeArray(rhs)) => Boolean(lhs == rhs),
                (Nil, Nil) => Boolean(true),
                (Function(lhs), Function(rhs)) => Boolean(lhs == rhs),
                _ => Boolean(false),
            },
            Inequality => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs != rhs),
                (Boolean(lhs), Boolean(rhs)) => Boolean(lhs != rhs),
                (String(lhs), String(rhs)) => Boolean(lhs != rhs),
                (List(lhs), List(rhs)) => Boolean(lhs != rhs),
                (Value::Range(lhs_start1, lhs_end1), Value::Range(rhs_start2, rhs_end2)) => {
                    Boolean(lhs_start1 != rhs_start2 || lhs_end1 != rhs_end2)
                }
                (AssociativeArray(lhs), AssociativeArray(rhs)) => Boolean(lhs != rhs),
                (Nil, Nil) => Boolean(false),
                (Function(lhs), Function(rhs)) => Boolean(lhs != rhs),
                _ => Boolean(true),
            },
            Greater => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs > rhs),
                (String(lhs), String(rhs)) => Boolean(lhs > rhs),
                (lhs, rhs) => {
                    return type_err!(
                        "não é possível aplicar a operação de 'maior que' para '{}' e '{}'",
                        lhs,
                        rhs
                    )
                }
            },
            GreaterOrEqual => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs >= rhs),
                (String(lhs), String(rhs)) => Boolean(lhs >= rhs),
                (lhs, rhs) => {
                    return type_err!(
                        "não é possível aplicar a operação de 'maior ou igual' para '{}' e '{}'",
                        lhs,
                        rhs
                    )
                }
            },
            Less => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs < rhs),
                (String(lhs), String(rhs)) => Boolean(lhs < rhs),
                (lhs, rhs) => {
                    return type_err!(
                        "não é possível aplicar a operação de 'menor que' para '{}' e '{}'",
                        lhs,
                        rhs
                    )
                }
            },
            LessOrEqual => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs <= rhs),
                (String(lhs), String(rhs)) => Boolean(lhs <= rhs),
                (lhs, rhs) => {
                    return type_err!(
                        "não é possível aplicar a operação de 'menor ou igual a' para '{}' e '{}'",
                        lhs,
                        rhs
                    )
                }
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
                (Number(lhs), Number(_)) if lhs != lhs.trunc() || !lhs.is_finite() => {
                    return runtime_err!(InvalidRangeBounds { bound: lhs });
                }
                (Number(_), Number(rhs)) if rhs != rhs.trunc() || !rhs.is_finite() => {
                    return runtime_err!(InvalidRangeBounds { bound: rhs });
                }
                (Number(lhs), Number(rhs)) => Value::Range(lhs as usize, rhs as usize),
                (lhs, rhs) => {
                    return type_err!(
                        "não é possível criar um intervalo entre '{}' e '{}'",
                        lhs,
                        rhs
                    )
                }
            },
        };

        match expr {
            Number(value) if value.abs() == f64::INFINITY => runtime_err!(NumberOverflow),
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
                _ => {
                    return type_err!(
                        "não é possível negar valor de tipo '{1}'; esperado '{0}'",
                        ValueType::Number,
                        rhs
                    )
                }
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
            Value::Function(func) if args.len() != func.get_params().len() => {
                runtime_err!(WrongNumberOfArguments {
                    expected: func.get_params().len(),
                    found: args.len(),
                })
            }
            Value::Function(func) => func.call(
                func.get_params()
                    .iter()
                    .zip(args)
                    .map(|(a, b)| (a.clone(), b))
                    .collect(),
                self,
            ),
            _ => runtime_err!(
                TypeError {
                    expected: ValueType::Function,
                    found: callee.kind()
                },
                format!("não é possível chamar '{}' como função", callee.kind())
            ),
        }
    }

    fn visit_access(&mut self, index: &ast::Access) -> Result<Value> {
        let Access { subscripted, index } = index;

        let subscripted = self.visit_expr(subscripted)?;

        match subscripted {
            Value::List(list) => self.visit_list_access(&list.borrow(), index),
            Value::AssociativeArray(associative_array) => {
                self.visit_associative_array_access(associative_array.borrow().clone(), index)
            }
            value => runtime_err!(WrongIndexType { value }),
        }
    }

    fn visit_list(&mut self, list: &ast::List) -> Result<Value> {
        let mut elements = Vec::with_capacity(list.elements.len());

        for e in &list.elements {
            let value = self.visit_expr(e)?;
            elements.push(value);
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
                    .set(name.clone(), StoredValue::new(value.clone()));

                match result {
                    Ok(_) => Ok(value),
                    Err(_) => runtime_err!(
                        UndefinedReference(name.clone()),
                        format!(
                            "a variável identificada por '{}' precisa ser definida com `seja`",
                            name
                        )
                    ),
                }
            }
            ast::Expr::Access(ast::Access { index, subscripted }) => {
                let subscripted = self.visit_expr(subscripted)?;

                match subscripted {
                    Value::List(list) => {
                        let mut list = list.borrow_mut();
                        let value = self.visit_expr(value)?;
                        let index = self.visit_expr(index)?;

                        self.visit_list_assign(&mut list, index, value)
                    }
                    Value::AssociativeArray(associative_array) => {
                        let mut associative_array = associative_array.borrow_mut();
                        let value = self.visit_expr(value)?;
                        let index = self.visit_expr(index)?;

                        self.visit_associative_array_assign(&mut associative_array, index, value)
                    }
                    value => runtime_err!(WrongIndexType { value }),
                }
            }
            _ => unreachable!(),
        }
    }

    fn visit_associative_array(
        &mut self,
        associative_array: &ast::AssociativeArray,
    ) -> Result<Value> {
        let ast::AssociativeArray { elements } = associative_array;

        let mut map = indexmap::IndexMap::new();

        for (key, value) in elements {
            let key = self.visit_literal(key)?;
            let key = self.visit_associative_array_key(key)?;

            let value = self.visit_expr(value)?;

            map.insert(key, value);
        }

        Ok(Value::AssociativeArray(Rc::new(RefCell::new(map))))
    }

    fn visit_anonymous_function(
        &mut self,
        anonymous_function: &ast::AnonymousFunction,
    ) -> Result<Value> {
        let ast::AnonymousFunction { params, body, .. } = anonymous_function;

        let func = self.create_function(params, body.clone());

        Ok(Value::Function(func))
    }
}

impl Interpreter {
    fn visit_list_access(&mut self, list: &[Value], index: &ast::Expr) -> Result<Value> {
        let index = match self.visit_expr(index)? {
            Value::Number(num) if !num.is_finite() || num.trunc() != num || num < 0.0 => {
                return runtime_err!(InvalidIndex { index: num });
            }
            Value::Number(num) => num as usize,
            val => {
                return type_err!(
                    "{} não é um tipo válido para indexação; esperado {}",
                    val,
                    ValueType::Number
                )
            }
        };

        if index >= list.len() {
            return runtime_err!(IndexOutOfBounds {
                index,
                len: list.len(),
            });
        }

        Ok(list[index].clone())
    }

    fn visit_associative_array_access(
        &mut self,
        associative_array: indexmap::IndexMap<AssociativeArrayKey, Value>,
        index: &ast::Expr,
    ) -> Result<Value> {
        let index = self.visit_expr(index)?;
        let index = self.visit_associative_array_key(index)?;

        match associative_array.get(&index) {
            Some(value) => Ok(value.clone()),
            None => runtime_err!(AssociativeArrayKeyNotFound { key: index }),
        }
    }

    fn visit_associative_array_key(&mut self, key: Value) -> Result<AssociativeArrayKey> {
        match key {
            Value::String(value) => Ok(AssociativeArrayKey::String(value)),
            Value::Number(value) if !value.is_finite() || value.trunc() != value => {
                runtime_err!(InvalidNumberAssociativeArrayKey { key: value })
            }
            Value::Number(value) => Ok(AssociativeArrayKey::Number(value as i64)),
            val => runtime_err!(InvalidTypeAssociativeArrayKey { key: val.kind() }),
        }
    }

    fn visit_list_assign(
        &mut self,
        list: &mut [Value],
        index: Value,
        value: Value,
    ) -> Result<Value> {
        let index = match index {
            Value::Number(num) => num as usize,
            val => {
                return type_err!(
                    "{} não é um tipo válido para indexação; esperado {}",
                    val,
                    ValueType::Number
                )
            }
        };

        if index >= list.len() {
            return runtime_err!(IndexOutOfBounds {
                index,
                len: list.len()
            });
        }

        list[index] = value.clone();

        Ok(value)
    }

    fn visit_associative_array_assign(
        &mut self,
        associative_array: &mut indexmap::IndexMap<AssociativeArrayKey, Value>,
        index: Value,
        value: Value,
    ) -> Result<Value> {
        let index = self.visit_associative_array_key(index)?;

        associative_array.insert(index, value.clone());

        Ok(value)
    }

    fn create_function(&self, params: &[ast::FunctionParam], body: Box<ast::Stmt>) -> Function {
        let mut context = Environment::new();

        for env in self.stack.into_iter() {
            for (name, value) in env {
                if params.iter().any(|param| param.name == *name) {
                    continue;
                }

                if let StoredValue::Shared(value) = value {
                    context.set(name.clone(), StoredValue::Shared(value.clone()));
                }
            }
        }

        Function::new(
            params.iter().map(|p| p.clone().into()).collect(),
            Some(Box::new(context)),
            Some(body),
            |params, body, interpreter, context| {
                if let Some(context) = context {
                    interpreter.stack.push(*context.clone());
                }

                for (param, arg_value) in params.into_iter() {
                    let stored_value = if param.is_captured {
                        StoredValue::new_shared(arg_value)
                    } else {
                        StoredValue::new(arg_value)
                    };

                    interpreter
                        .stack
                        .define(param.name.clone(), stored_value)
                        .unwrap();
                }

                if let Some(body) = body {
                    interpreter.interpret_stmt(&body)?;
                }

                let value = interpreter
                    .stack
                    .consume_return()
                    .map(|v| v.clone_value())
                    .unwrap_or(Value::Nil);

                if context.is_some() {
                    interpreter.stack.pop();
                }

                Ok(value)
            },
        )
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
