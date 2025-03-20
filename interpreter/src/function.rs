use parser::ast::{self, Stmt};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::environment::Environment;
use crate::interpreter::Interpreter;

use super::runtime_error::Result;
use super::value::Value;

static FUNCTION_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone)]
pub struct Function {
    id: usize,
    context: FunctionContext,
    object: FunctionObject,
}

impl Function {
    pub fn new(
        params: Vec<FunctionParam>,
        context: Option<Box<Environment>>,
        body: Option<Box<Stmt>>,
        object: FunctionObject,
    ) -> Self {
        let unique_id = FUNCTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Function {
            id: unique_id,
            context: FunctionContext::new(params, context, body),
            object,
        }
    }

    pub fn call(
        &self,
        params: Vec<(FunctionParam, Value)>,
        interpreter: &mut Interpreter,
    ) -> Result<Value> {
        (self.object)(
            params,
            self.context.body.clone(),
            interpreter,
            self.context.env.as_ref(),
        )
    }

    pub fn get_fn_ptr(&self) -> usize {
        self.object as *const () as usize
    }

    pub fn get_params(&self) -> Vec<FunctionParam> {
        self.context.params.clone()
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub params: Vec<FunctionParam>,
    pub env: Option<Box<Environment>>,
    pub body: Option<Box<Stmt>>,
}

impl FunctionContext {
    pub fn new(
        params: Vec<FunctionParam>,
        context: Option<Box<Environment>>,
        body: Option<Box<Stmt>>,
    ) -> Self {
        FunctionContext {
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
    context: Option<&Box<Environment>>,
) -> Result<Value>;

macro_rules! params {
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

pub(crate) use params;
