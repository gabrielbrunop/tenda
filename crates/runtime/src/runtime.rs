use std::{cell::RefCell, fmt::Debug, rc::Rc};
use tenda_common::span::SourceSpan;
use tenda_parser::{self, ast};
use tenda_reporting::Diagnostic;

use crate::{
    associative_array::{AssociativeArray, AssociativeArrayKey},
    environment::{Environment, ValueCell},
    frame::Frame,
    function::{Function, FunctionObject},
    platform::{self},
    runtime_error::{Result, RuntimeError},
    stack::{Stack, StackError},
    value::{Value, ValueType},
    FunctionRuntimeMetadata, StackFrame,
};

#[derive(Debug)]
pub struct Runtime {
    stack: Stack,
    platform: Box<dyn platform::Platform>,
}

impl Runtime {
    pub fn new(platform: impl platform::Platform + 'static) -> Self {
        Runtime {
            stack: Stack::new(),
            platform: Box::new(platform),
        }
    }

    pub fn eval(&mut self, ast: &ast::Ast) -> Result<Value> {
        self.interpret_ast(ast)
    }

    pub fn get_global_env(&self) -> &Environment {
        self.stack.global().get_env()
    }

    pub fn get_global_env_mut(&mut self) -> &mut Environment {
        self.stack.global_mut().get_env_mut()
    }

    pub fn get_platform(&self) -> &dyn platform::Platform {
        self.platform.as_ref()
    }

    fn interpret_ast(&mut self, ast: &ast::Ast) -> Result<Value> {
        let mut last_value = Value::Nil;

        let ast::Ast { inner: ast, .. } = ast;

        for stmt in ast {
            let value = self.interpret_stmt(stmt)?;

            last_value = value;

            if self.stack.has_return_value()
                || self.stack.has_loop_break_flag()
                || self.stack.has_loop_continue_flag()
            {
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
        .map_err(|err| {
            let mut err = err;

            if err.get_span().is_none() {
                err.set_span(stmt.get_span());
            }

            err
        })
    }
}

impl Runtime {
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
        let ast::Block { inner, .. } = block;

        self.stack.push(Frame::new());

        self.interpret_ast(inner)?;

        self.stack.pop();

        Ok(Value::Nil)
    }

    fn visit_return(&mut self, return_stmt: &ast::Return) -> Result<Value> {
        let ast::Return { value, .. } = return_stmt;

        if let Some(expr) = value {
            let value = self.visit_expr(expr)?;
            self.stack.set_return_value(ValueCell::new(value));
        }

        Ok(Value::Nil)
    }

    fn visit_cond(&mut self, cond: &ast::Cond) -> Result<Value> {
        let ast::Cond {
            cond,
            then,
            or_else,
            ..
        } = cond;

        if self.visit_expr(cond)?.to_bool() {
            self.interpret_stmt(then)?;
        } else if let Some(or_else) = or_else {
            self.interpret_stmt(or_else)?;
        };

        Ok(Value::Nil)
    }

    fn visit_while(&mut self, while_stmt: &ast::While) -> Result<Value> {
        let ast::While { cond, body, .. } = while_stmt;

        while self.visit_expr(cond)?.to_bool() && !self.stack.has_loop_break_flag() {
            self.interpret_stmt(body)?;

            self.stack.set_loop_continue_flag(false);
        }

        self.stack.set_loop_break_flag(false);

        Ok(Value::Nil)
    }

    fn visit_for_each(&mut self, for_each: &ast::ForEach) -> Result<Value> {
        let ast::ForEach {
            item,
            iterable,
            body,
            span,
        } = for_each;

        let iterable = self.visit_expr(iterable)?;

        if !iterable.is_iterable() {
            return Err(Box::new(RuntimeError::NotIterable {
                value: iterable.kind(),
                span: Some(span.clone()),
                stacktrace: vec![],
            }));
        }

        for value in iterable {
            let mut frame = Frame::new();

            let stored_value = if item.captured {
                ValueCell::new_shared(value.clone())
            } else {
                ValueCell::new(value.clone())
            };

            frame.get_env_mut().set(item.name.clone(), stored_value);

            self.stack.push(frame);
            self.interpret_stmt(body)?;

            if self.stack.has_loop_break_flag() {
                break;
            }

            self.stack.set_loop_continue_flag(false);
            self.stack.pop();
        }

        self.stack.set_loop_break_flag(false);

        Ok(Value::Nil)
    }

    fn visit_break(&mut self, _break_stmt: &ast::Break) -> Result<Value> {
        self.stack.set_loop_break_flag(true);

        Ok(Value::Nil)
    }

    fn visit_continue(&mut self, _continue_stmt: &ast::Continue) -> Result<Value> {
        self.stack.set_loop_continue_flag(true);

        Ok(Value::Nil)
    }
}

impl Runtime {
    fn visit_local(&mut self, local: &ast::LocalDecl) -> Result<Value> {
        let ast::LocalDecl {
            name, value, span, ..
        } = local;

        let value = self.visit_expr(value)?;

        let value = match local.captured {
            true => ValueCell::new_shared(value),
            false => ValueCell::new(value),
        };

        match self.stack.define(name.clone(), value) {
            Ok(_) => Ok(Value::Nil),
            Err(err) => match err {
                StackError::AlreadyDeclared => Err(Box::new(RuntimeError::AlreadyDeclared {
                    var_name: name.to_string(),
                    span: Some(span.clone()),
                    help: Some("declare a variável com outro nome ou use `=` para atribuir um novo valor a ela".to_string()),
                    stacktrace: vec![],
                })),
                _ => unreachable!(),
            },
        }
    }

    fn visit_function(&mut self, function: &ast::FunctionDecl) -> Result<Value> {
        let ast::FunctionDecl {
            name, params, body, ..
        } = function;

        let metadata =
            FunctionRuntimeMetadata::new(Some(function.span.clone()), Some(name.clone()));
        let func = self.create_function(params, body.clone(), Some(metadata));

        match self
            .stack
            .define(name.clone(), ValueCell::new(Value::Function(func)))
        {
            Ok(_) => Ok(Value::Nil),
            Err(err) => match err {
                StackError::AlreadyDeclared => Err(Box::new(RuntimeError::AlreadyDeclared {
                    var_name: name.to_string(),
                    span: Some(function.span.clone()),
                    help: Some("declare a função com outro nome".to_string()),
                    stacktrace: vec![],
                })),
                _ => unreachable!(),
            },
        }
    }
}

impl Runtime {
    fn visit_binary(&mut self, binary: &ast::BinaryOp) -> Result<Value> {
        let ast::BinaryOp {
            lhs, op, rhs, span, ..
        } = binary;

        use ast::BinaryOperator::*;
        use Value::*;

        let lhs = self.visit_expr(lhs)?;

        match op {
            LogicalAnd => {
                if lhs.to_bool() {
                    return self.visit_expr(rhs);
                } else {
                    return Ok(lhs);
                }
            }
            LogicalOr => {
                if lhs.to_bool() {
                    return Ok(lhs);
                } else {
                    return self.visit_expr(rhs);
                }
            }
            _ => {}
        };

        let rhs = self.visit_expr(rhs)?;

        let value = match op {
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
                (Date(rhs), Number(millis)) => Value::Date(rhs + millis as i64),
                (Number(millis), Date(rhs)) => Value::Date(rhs + millis as i64),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!("não é possível somar '{}' e '{}'", lhs, rhs)),
                        stacktrace: vec![],
                    }));
                }
            },
            Subtract => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs - rhs),
                (Date(rhs), Number(millis)) => Value::Date(rhs - millis as i64),
                (Number(millis), Date(rhs)) => Value::Date(rhs - millis as i64),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!("não é possível subtrair '{}' de '{}'", rhs, lhs)),
                        stacktrace: vec![],
                    }));
                }
            },
            Multiply => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs * rhs),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível multiplicar '{}' por '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            Divide => match (lhs, rhs) {
                (Number(_), Number(0.0)) => {
                    return Err(Box::new(RuntimeError::DivisionByZero {
                        span: Some(span.clone()),
                        stacktrace: vec![],
                    }));
                }
                (Number(lhs), Number(rhs)) => Number(lhs / rhs),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!("não é possível dividir '{}' por '{}'", lhs, rhs)),
                        stacktrace: vec![],
                    }));
                }
            },
            Exponentiation => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs.powf(rhs)),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível elevar '{}' à potência de '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            Modulo => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Number(lhs % rhs),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível encontrar o resto da divisão de '{}' por '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            Equality => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs == rhs),
                (Boolean(lhs), Boolean(rhs)) => Boolean(lhs == rhs),
                (String(lhs), String(rhs)) => Boolean(lhs == rhs),
                (List(lhs), List(rhs)) => Boolean(lhs == rhs),
                (Value::Range(lhs_start, lhs_end), Value::Range(rhs_start, rhs_end)) => {
                    Boolean(lhs_start == rhs_start && lhs_end == rhs_end)
                }
                (AssociativeArray(lhs), AssociativeArray(rhs)) => Boolean(lhs == rhs),
                (Nil, Nil) => Boolean(true),
                (Function(lhs), Function(rhs)) => Boolean(lhs == rhs),
                (Date(lhs), Date(rhs)) => Boolean(lhs == rhs),
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
                (Date(lhs), Date(rhs)) => Boolean(lhs != rhs),
                _ => Boolean(true),
            },
            Greater => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs > rhs),
                (String(lhs), String(rhs)) => Boolean(lhs > rhs),
                (Date(lhs), Date(rhs)) => Boolean(lhs > rhs),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível aplicar a operação de 'maior que' para '{}' e '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            GreaterOrEqual => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs >= rhs),
                (String(lhs), String(rhs)) => Boolean(lhs >= rhs),
                (Date(lhs), Date(rhs)) => Boolean(lhs >= rhs),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível aplicar a operação de 'maior ou igual' para '{}' e '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            Less => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs < rhs),
                (String(lhs), String(rhs)) => Boolean(lhs < rhs),
                (Date(lhs), Date(rhs)) => Boolean(lhs < rhs),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível aplicar a operação de 'menor que' para '{}' e '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            LessOrEqual => match (lhs, rhs) {
                (Number(lhs), Number(rhs)) => Boolean(lhs <= rhs),
                (String(lhs), String(rhs)) => Boolean(lhs <= rhs),
                (Date(lhs), Date(rhs)) => Boolean(lhs <= rhs),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível aplicar a operação de 'menor ou igual a' para '{}' e '{}'",
                            lhs,
                            rhs,
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            ast::BinaryOperator::Range => match (lhs, rhs) {
                (Number(lhs), Number(_)) if lhs != lhs.trunc() || !lhs.is_finite() => {
                    return Err(Box::new(RuntimeError::InvalidRangeBounds {
                        bound: lhs,
                        span: Some(span.clone()),
                        stacktrace: vec![],
                    }));
                }
                (Number(_), Number(rhs)) if rhs != rhs.trunc() || !rhs.is_finite() => {
                    return Err(Box::new(RuntimeError::InvalidRangeBounds {
                        bound: rhs,
                        span: Some(span.clone()),
                        stacktrace: vec![],
                    }));
                }
                (Number(lhs), Number(rhs)) => Value::Range(lhs as usize, rhs as usize),
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível criar um intervalo entre '{}' e '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            Has => match (lhs, rhs) {
                (List(list), value) => Boolean(list.borrow().contains(&value)),
                (AssociativeArray(associative_array), key) => {
                    let key = self.resolve_associative_array_key(key).map_err(|mut src| {
                        src.set_span(span);
                        src
                    })?;

                    Boolean(associative_array.borrow().contains_key(&key))
                }
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível verificar se '{}' contém '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            Lacks => match (lhs, rhs) {
                (List(list), value) => Boolean(!list.borrow().contains(&value)),
                (AssociativeArray(associative_array), key) => {
                    let key = self.resolve_associative_array_key(key).map_err(|mut src| {
                        src.set_span(span);
                        src
                    })?;

                    Boolean(!associative_array.borrow().contains_key(&key))
                }
                (lhs, rhs) => {
                    return Err(Box::new(RuntimeError::TypeMismatch {
                        first: lhs.kind(),
                        second: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível verificar se '{}' não contém '{}'",
                            lhs, rhs
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            LogicalAnd => unreachable!(),
            LogicalOr => unreachable!(),
        };

        Ok(value)
    }

    fn visit_unary(&mut self, unary: &ast::UnaryOp) -> Result<Value> {
        let ast::UnaryOp { op, rhs, span } = unary;

        use ast::UnaryOperator::*;
        use Value::*;

        let rhs = self.visit_expr(rhs)?;

        let expr = match op {
            Negative => match rhs {
                Number(rhs) => Number(-rhs),
                _ => {
                    return Err(Box::new(RuntimeError::UnexpectedTypeError {
                        expected: ValueType::Number,
                        found: rhs.kind(),
                        span: Some(span.clone()),
                        message: Some(format!(
                            "não é possível negar valor de tipo '{}'; esperado '{}'",
                            rhs.kind(),
                            ValueType::Number
                        )),
                        stacktrace: vec![],
                    }));
                }
            },
            LogicalNot => Value::Boolean(!rhs.to_bool()),
        };

        Ok(expr)
    }

    fn visit_call(&mut self, call: &ast::Call) -> Result<Value> {
        let ast::Call { callee, args, span } = call;

        let callee = self.visit_expr(callee)?;

        let args = args
            .iter()
            .map(|arg| self.visit_expr(arg))
            .collect::<Result<Vec<_>>>()?;

        match callee {
            Value::Function(func) if args.len() != func.get_params().len() => {
                Err(Box::new(RuntimeError::WrongNumberOfArguments {
                    expected: func.get_params().len(),
                    found: args.len(),
                    span: Some(span.clone()),
                    stacktrace: vec![],
                }))
            }
            Value::Function(func) => self.call_function(func, args, Some(span.clone())),
            _ => Err(Box::new(RuntimeError::UnexpectedTypeError {
                expected: ValueType::Function,
                found: callee.kind(),
                span: Some(span.clone()),
                message: Some(format!(
                    "não é possível chamar um valor de tipo '{}' como função",
                    callee.kind()
                )),
                stacktrace: vec![],
            })),
        }
    }

    fn visit_access(&mut self, index: &ast::Access) -> Result<Value> {
        let ast::Access {
            subscripted,
            index,
            span,
        } = index;

        let subscripted = self.visit_expr(subscripted)?;

        match subscripted {
            Value::List(list) => self.visit_list_access(&list.borrow(), index),
            Value::String(string) => self.visit_string_access(&string, index),
            Value::AssociativeArray(associative_array) => {
                self.visit_associative_array_access(associative_array.borrow().clone(), index)
            }
            value => Err(Box::new(RuntimeError::WrongIndexType {
                value: value.kind(),
                span: Some(span.clone()),
                stacktrace: vec![],
            })),
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
        let ast::Grouping { expr, .. } = grouping;

        self.visit_expr(expr)
    }

    fn visit_literal(&mut self, literal: &ast::Literal) -> Result<Value> {
        let ast::Literal { value, .. } = literal;

        Ok(value.clone().into())
    }

    fn visit_variable(&mut self, variable: &ast::Variable) -> Result<Value> {
        let ast::Variable { name, span, .. } = variable;

        self.stack.lookup(name).map(|v| v.extract()).ok_or(Box::new(
            RuntimeError::UndefinedReference {
                var_name: name.clone(),
                span: Some(span.clone()),
                help: Some(format!(
                    "você precisa definir a variável '{}' antes de usá-la: `seja {} = ...`",
                    name, name
                )),
                stacktrace: vec![],
            },
        ))
    }

    fn visit_assign(&mut self, assign: &ast::Assign) -> Result<Value> {
        let ast::Assign {
            name: variable,
            value,
            span,
        } = assign;

        match &**variable {
            ast::Expr::Variable(ast::Variable { name, .. }) => {
                let value = self.visit_expr(value)?;

                let result = self
                    .stack
                    .assign(name.clone(), ValueCell::new(value.clone()));

                match result {
                    Ok(_) => Ok(value),
                    Err(err) => match err {
                        StackError::AssignToUndefined(name) => {
                            Err(Box::new(RuntimeError::UndefinedReference {
                                var_name: name.clone(),
                                span: Some(span.clone()),
                                help: Some(format!(
                                    "talvez você queria definir a variável '{}': `seja {} = ...`",
                                    name, name
                                )),
                                stacktrace: vec![],
                            }))
                        }
                        _ => unreachable!(),
                    },
                }
            }
            ast::Expr::Access(ast::Access {
                index,
                subscripted,
                span: lvalue_span,
            }) => {
                let subscripted = self.visit_expr(subscripted)?;

                match subscripted {
                    Value::List(list) => {
                        self.visit_list_assign(list, index, value)
                    }
                    Value::AssociativeArray(associative_array) => {
                        self.visit_associative_array_assign(associative_array, index, value)
                    }
                    Value::String(_) => Err(Box::new(RuntimeError::ImmutableString {
                        span: Some(span.clone()),
                        help: Some(
                            concat!(
                                "em vez de tentar modificar o texto, você pode criar um novo texto\n",
                                "concatenando o texto original com o novo texto: `texto = texto + ...`\n",
                                "ou usando funções como `Texto.substitua(...)`\n",
                                "veja as funções disponíveis em `Texto` para mais possibilidades"
                            )
                            .to_string(),
                        ),
                        stacktrace: vec![],
                    })),
                    value => Err(Box::new(RuntimeError::WrongIndexType {
                        value: value.kind(),
                        span: Some(lvalue_span.clone()),
                        stacktrace: vec![],
                    })),
                }
            }
            _ => unreachable!(),
        }
    }

    fn visit_associative_array(
        &mut self,
        associative_array: &ast::AssociativeArray,
    ) -> Result<Value> {
        let ast::AssociativeArray { elements, span } = associative_array;

        let mut map = indexmap::IndexMap::new();

        for (key, value) in elements {
            let key = self.visit_literal(key)?;
            let key = self
                .resolve_associative_array_key(key)
                .map_err(|mut source| {
                    source.set_span(span);
                    source
                })?;

            let value = self.visit_expr(value)?;

            map.insert(key, value);
        }

        Ok(Value::AssociativeArray(Rc::new(RefCell::new(map))))
    }

    fn visit_anonymous_function(
        &mut self,
        anonymous_function: &ast::AnonymousFunction,
    ) -> Result<Value> {
        let ast::AnonymousFunction {
            params, body, span, ..
        } = anonymous_function;

        let metadata = FunctionRuntimeMetadata::new(Some(span.clone()), None);
        let func = self.create_function(params, body.clone(), Some(metadata));

        Ok(Value::Function(func))
    }
}

impl Runtime {
    fn visit_list_access(&mut self, list: &[Value], index: &ast::Expr) -> Result<Value> {
        let span = index.get_span();
        let index = self.resolve_index(index)?;

        if index >= list.len() {
            return Err(Box::new(RuntimeError::IndexOutOfBounds {
                index,
                len: list.len(),
                span: Some(span.clone()),
                help: vec!["verifique se o índice está dentro dos limites da lista antes de tentar acessá-lo".to_string()],
                stacktrace: vec![],
            }));
        }

        Ok(list[index].clone())
    }

    fn visit_string_access(&mut self, string: &str, index: &ast::Expr) -> Result<Value> {
        let span = index.get_span();
        let index = self.resolve_index(index)?;

        if let Some(char) = string.chars().nth(index) {
            Ok(Value::String(char.to_string()))
        } else {
            Err(Box::new(RuntimeError::IndexOutOfBounds {
                index,
                len: string.len(),
                span: Some(span.clone()),
                help: vec![
                    "verifique o tamanho do texto antes de tentar acessar uma posição nele"
                        .to_string(),
                ],
                stacktrace: vec![],
            }))
        }
    }

    fn visit_associative_array_access(
        &mut self,
        associative_array: AssociativeArray,
        index: &ast::Expr,
    ) -> Result<Value> {
        let span = index.get_span();
        let index = self.visit_expr(index)?;
        let index = self
            .resolve_associative_array_key(index)
            .map_err(|mut source| {
                source.set_span(span);
                source
            })?;

        match associative_array.get(&index) {
            Some(value) => Ok(value.clone()),
            None => Err(Box::new(RuntimeError::AssociativeArrayKeyNotFound {
                key: index,
                span: Some(span.clone()),
                stacktrace: vec![],
            })),
        }
    }

    fn visit_list_assign(
        &mut self,
        list: Rc<RefCell<Vec<Value>>>,
        index: &ast::Expr,
        value: &ast::Expr,
    ) -> Result<Value> {
        let index_span = index.get_span();

        let value = self.visit_expr(value)?;
        let index = self.resolve_index(index)?;

        let mut list = list.borrow_mut();

        if index >= list.len() {
            return Err(Box::new(RuntimeError::IndexOutOfBounds {
                index,
                len: list.len(),
                span: Some(index_span.clone()),
                help: vec![
                    "verifique se o índice está dentro dos limites da lista antes de tentar acessá-lo".to_string(),
                    "se a sua intenção era adicionar um novo elemento à lista, use `Lista.insira`".to_string()
                ],
                stacktrace: vec![],
            }));
        }

        list[index] = value.clone();

        Ok(value)
    }

    fn visit_associative_array_assign(
        &mut self,
        associative_array: Rc<RefCell<indexmap::IndexMap<AssociativeArrayKey, Value>>>,
        index: &ast::Expr,
        value: &ast::Expr,
    ) -> Result<Value> {
        let span = index.get_span();

        let value = self.visit_expr(value)?;
        let index = self.visit_expr(index)?;
        let index = self
            .resolve_associative_array_key(index)
            .map_err(|mut source| {
                source.set_span(span);
                source
            })?;

        let mut associative_array = associative_array.borrow_mut();

        associative_array.insert(index, value.clone());

        Ok(value)
    }
}

impl Runtime {
    pub fn call_function(
        &mut self,
        func: Function,
        args: Vec<Value>,
        span: Option<SourceSpan>,
    ) -> Result<Value> {
        let args = func
            .get_params()
            .iter()
            .zip(args)
            .map(|(a, b)| (a.clone(), b))
            .collect();

        let context_frame = Frame::from_env(func.get_env().clone());

        self.stack.push(context_frame);

        let result = match func.object {
            FunctionObject::Builtin { func_ptr, env, .. } => func_ptr(args, self, env),
            FunctionObject::UserDefined { body, .. } => {
                for (param, arg_value) in args.into_iter() {
                    let stored_value = if param.is_captured {
                        ValueCell::new_shared(arg_value)
                    } else {
                        ValueCell::new(arg_value)
                    };

                    self.stack.define(param.name.clone(), stored_value).unwrap();
                }

                match self.interpret_stmt(&body) {
                    Ok(_) => {
                        let value = self
                            .stack
                            .consume_return_value()
                            .map(|v| v.extract())
                            .unwrap_or(Value::Nil);

                        Ok(value)
                    }
                    Err(err) => Err(err),
                }
            }
        };

        self.stack.pop();

        match result {
            Ok(value) => Ok(value),
            Err(mut err) => {
                let fn_name = func.metadata.as_ref().and_then(|m| m.get_name().clone());

                if let Some(stacktrace) = err.get_mut_stacktrace() {
                    stacktrace.push(StackFrame::new(fn_name.clone(), span.clone()));
                } else {
                    let err_site = err.get_span().clone();
                    err.set_stacktrace(vec![StackFrame::new(fn_name.clone(), err_site)]);
                }

                Err(err)
            }
        }
    }

    fn create_function(
        &self,
        params: &[ast::FunctionParam],
        body: Box<ast::Stmt>,
        metadata: Option<FunctionRuntimeMetadata>,
    ) -> Function {
        let mut context = Environment::new();

        for frame in self.stack.into_iter() {
            for (name, value) in frame.get_env() {
                if params.iter().any(|param| param.name == *name) {
                    continue;
                }

                if let ValueCell::Shared(value) = value {
                    context.set(name.clone(), ValueCell::Shared(value.clone()));
                }
            }
        }

        let mut func = Function::new(
            params.iter().map(|p| p.clone().into()).collect(),
            context,
            body,
        );

        if let Some(metadata) = metadata {
            func.set_metadata(metadata);
        }

        func
    }

    fn resolve_associative_array_key(
        &mut self,
        key: Value,
    ) -> std::result::Result<AssociativeArrayKey, Box<RuntimeError>> {
        match key {
            Value::String(value) => Ok(AssociativeArrayKey::String(value)),
            Value::Number(value) if !value.is_finite() || value.trunc() != value => {
                Err(Box::new(RuntimeError::InvalidNumberAssociativeArrayKey {
                    key: value,
                    span: None,
                    stacktrace: vec![],
                }))
            }
            Value::Number(value) => Ok(AssociativeArrayKey::Number(value as i64)),
            val => Err(Box::new(RuntimeError::InvalidTypeAssociativeArrayKey {
                key: val.kind(),
                span: None,
                stacktrace: vec![],
            })),
        }
    }

    fn resolve_index(&mut self, index: &ast::Expr) -> Result<usize> {
        let span = index.get_span();

        match self.visit_expr(index)? {
            Value::Number(num) if !num.is_finite() || num.trunc() != num || num < 0.0 => {
                Err(Box::new(RuntimeError::InvalidIndex {
                    index: num,
                    span: Some(span.clone()),
                    stacktrace: vec![],
                }))
            }
            Value::Number(num) => Ok(num as usize),
            val => Err(Box::new(RuntimeError::UnexpectedTypeError {
                expected: ValueType::Number,
                found: val.kind(),
                span: Some(span.clone()),
                message: Some(format!(
                    "não é possível indexar com '{}'; esperado '{}'",
                    val.kind(),
                    ValueType::Number
                )),
                stacktrace: vec![],
            })),
        }
    }
}
