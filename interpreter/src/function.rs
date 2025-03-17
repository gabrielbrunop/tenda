use std::sync::atomic::{AtomicUsize, Ordering};

use parser::ast::{self, Stmt};

use crate::environment::Environment;
use crate::interpreter::Interpreter;

use super::runtime_error::Result;
use super::value::Value;

static FUNCTION_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone)]
pub struct Function {
    pub id: usize,
    pub context: FunctionContext,
    pub object: FunctionObject,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<FunctionParam>,
        context: Box<Environment>,
        body: Option<Box<Stmt>>,
        object: FunctionObject,
    ) -> Self {
        let unique_id = FUNCTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Function {
            id: unique_id,
            context: FunctionContext::new(name, params, context, body),
            object,
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub env: Box<Environment>,
    pub body: Option<Box<Stmt>>,
}

impl FunctionContext {
    pub fn new(
        name: String,
        params: Vec<FunctionParam>,
        context: Box<Environment>,
        body: Option<Box<Stmt>>,
    ) -> Self {
        FunctionContext {
            name,
            params,
            body,
            env: context,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub is_captured: bool,
}

impl From<ast::FunctionParam> for FunctionParam {
    fn from(param: ast::FunctionParam) -> Self {
        FunctionParam {
            name: param.name,
            is_captured: param.captured,
        }
    }
}

type FunctionObject = fn(
    params: Vec<(FunctionParam, Value)>,
    body: Option<Box<Stmt>>,
    interpreter: &mut Interpreter,
    context: &Box<Environment>,
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
            use crate::function::FunctionParam;
            vec![$($kind.to_string()),*].into_iter().map(|name| FunctionParam {
                name,
                is_captured: false,
            }).collect()
        }
    };
}

pub(crate) use add_native_fn;
pub(crate) use native_fn;
pub(crate) use param_list;
