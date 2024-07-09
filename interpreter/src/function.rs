use std::collections::HashMap;

use parser::stmt::Stmt;

use crate::interpreter::Interpreter;

use super::runtime_error::Result;
use super::value::Value;

#[derive(Debug, PartialEq)]
pub struct Function {
    pub context: FunctionContext,
    pub object: FunctionObject,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<String>,
        body: Option<Box<Stmt>>,
        object: FunctionObject,
    ) -> Self {
        Function {
            context: FunctionContext::new(name, params, body),
            object,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionContext {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Box<Stmt>>,
}

impl FunctionContext {
    pub fn new(name: String, params: Vec<String>, body: Option<Box<Stmt>>) -> Self {
        FunctionContext { name, params, body }
    }
}

type FunctionObject = fn(
    params: HashMap<String, Value>,
    body: Option<Box<Stmt>>,
    interpreter: &mut Interpreter,
) -> Result<Value>;

#[macro_export]
macro_rules! add_native_fn {
    ($stack:ident, $fn:expr) => {{
        let func = $fn;
        let func_name = $fn.context.name.clone();
        let func_object = Value::Function(Rc::new(func));
        $stack.define(func_name, func_object).unwrap();
    }};
}

#[macro_export]
macro_rules! native_fn {
    ($name:literal, $args:expr, $body:expr) => {
        Function::new($name.to_string(), $args, None, $body)
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
