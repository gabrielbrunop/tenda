use std::collections::HashMap;

use parser::ast::Stmt;

use crate::environment::Environment;
use crate::interpreter::Interpreter;

use super::runtime_error::Result;
use super::value::Value;

#[derive(Debug, Clone)]
pub struct Function {
    pub context: FunctionContext,
    pub object: FunctionObject,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<String>,
        captured_env: Box<Environment>,
        body: Option<Box<Stmt>>,
        object: FunctionObject,
    ) -> Self {
        Function {
            context: FunctionContext::new(name, params, captured_env, body),
            object,
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub name: String,
    pub params: Vec<String>,
    pub captured_env: Box<Environment>,
    pub body: Option<Box<Stmt>>,
}

impl FunctionContext {
    pub fn new(
        name: String,
        params: Vec<String>,
        captured_env: Box<Environment>,
        body: Option<Box<Stmt>>,
    ) -> Self {
        FunctionContext {
            name,
            params,
            body,
            captured_env,
        }
    }
}

type FunctionObject = fn(
    params: HashMap<String, Value>,
    body: Option<Box<Stmt>>,
    interpreter: &mut Interpreter,
    captured_env: &Box<Environment>,
) -> Result<Value>;

macro_rules! add_native_fn {
    ($stack:ident, $fn:expr) => {{
        let func = $fn;
        let func_name = $fn.context.name.clone();
        let func_object = Value::Function(func);
        $stack
            .define(func_name, StoredValue::Unique(func_object))
            .unwrap();
    }};
}

macro_rules! native_fn {
    ($name:literal, $args:expr, $body:expr) => {
        Function::new(
            $name.to_string(),
            $args,
            Box::new(Environment::new()),
            None,
            $body,
        )
    };
}

macro_rules! param_list {
    ($($kind:expr),*) => {
        {
            vec![$($kind.to_string()),*]
        }
    };
}

pub(crate) use add_native_fn;
pub(crate) use native_fn;
pub(crate) use param_list;
