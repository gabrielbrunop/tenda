use std::collections::HashMap;

use crate::{
    interpreter::{Interpreter, RuntimeError},
    stmt::Stmt,
    value::Value,
};

#[derive(Debug, PartialEq)]
pub struct Function {
    pub context: FunctionContext,
    pub object: FunctionObject,
}

impl Function {
    pub fn new(params: Vec<String>, body: Option<Box<Stmt>>, object: FunctionObject) -> Self {
        Function {
            context: FunctionContext::new(params, body),
            object,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionContext {
    pub params: Vec<String>,
    pub body: Option<Box<Stmt>>,
}

impl FunctionContext {
    pub fn new(params: Vec<String>, body: Option<Box<Stmt>>) -> Self {
        FunctionContext { params, body }
    }
}

type FunctionObject = fn(
    params: HashMap<String, Value>,
    body: Option<Box<Stmt>>,
    interpreter: &mut Interpreter,
) -> Result<Value, RuntimeError>;

#[macro_export]
macro_rules! add_native_fn {
    ($stack:ident, $name:literal, $fn:expr) => {{
        $stack.define($name.to_string(), $fn).unwrap();
    }};
}

#[macro_export]
macro_rules! native_fn {
    ($args:expr, $body:expr) => {
        Value::Function(Rc::new(Function::new($args, None, $body)))
    };
}

#[macro_export]
macro_rules! param_list {
    ($($kind:expr),*) => {
        {
            vec![$($kind.to_string()),*]
        }
    };
}
