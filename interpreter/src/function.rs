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
        context: Box<Environment>,
        body: Box<Stmt>,
        object: UserDefinedFunctionObject,
    ) -> Self {
        let unique_id = FUNCTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Function {
            id: unique_id,
            context: FunctionContext::new(params, context, body),
            object: FunctionObject::UserDefined(object),
        }
    }

    pub fn new_builtin(
        params: Vec<FunctionParam>,
        object: BuiltinFunctionObject,
        context: Option<Box<Environment>>,
    ) -> Self {
        let unique_id = FUNCTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Function {
            id: unique_id,
            context: FunctionContext::new_builtin(
                params,
                context.unwrap_or(Box::new(Environment::new())),
            ),
            object: FunctionObject::Builtin(object),
        }
    }

    pub fn call(
        &self,
        params: Vec<(FunctionParam, Value)>,
        interpreter: &mut Interpreter,
    ) -> Result<Value> {
        let env = match &self.context {
            FunctionContext::UserDefined { env, .. } => env,
            FunctionContext::Builtin { env, .. } => env,
        };

        match self.object {
            FunctionObject::UserDefined(f) => {
                let body = match &self.context {
                    FunctionContext::UserDefined { body, .. } => body.clone(),
                    _ => unreachable!(),
                };

                f(params, body.clone(), interpreter, env)
            }
            FunctionObject::Builtin(f) => f(params, interpreter, env.clone()),
        }
    }

    pub fn get_fn_ptr(&self) -> usize {
        match self.object {
            FunctionObject::UserDefined(f) => f as *const () as usize,
            FunctionObject::Builtin(f) => f as *const () as usize,
        }
    }

    pub fn get_params(&self) -> Vec<FunctionParam> {
        match &self.context {
            FunctionContext::UserDefined { params, .. } => params.clone(),
            FunctionContext::Builtin { params, .. } => params.clone(),
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
pub enum FunctionContext {
    UserDefined {
        params: Vec<FunctionParam>,
        env: Box<Environment>,
        body: Box<Stmt>,
    },
    Builtin {
        params: Vec<FunctionParam>,
        env: Box<Environment>,
    },
}

impl FunctionContext {
    pub fn new(params: Vec<FunctionParam>, context: Box<Environment>, body: Box<Stmt>) -> Self {
        FunctionContext::UserDefined {
            params,
            body,
            env: context,
        }
    }

    pub fn new_builtin(params: Vec<FunctionParam>, env: Box<Environment>) -> Self {
        FunctionContext::Builtin { params, env }
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

type UserDefinedFunctionObject = fn(
    params: Vec<(FunctionParam, Value)>,
    body: Box<Stmt>,
    interpreter: &mut Interpreter,
    context: &Box<Environment>,
) -> Result<Value>;

type BuiltinFunctionObject = fn(
    params: Vec<(FunctionParam, Value)>,
    interpreter: &mut Interpreter,
    context: Box<Environment>,
) -> Result<Value>;

#[derive(Debug, Clone)]
pub enum FunctionObject {
    UserDefined(UserDefinedFunctionObject),
    Builtin(BuiltinFunctionObject),
}

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
