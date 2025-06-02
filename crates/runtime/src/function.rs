use std::sync::atomic::{AtomicUsize, Ordering};
use tenda_common::span::SourceSpan;
use tenda_parser::ast;

use crate::environment::Environment;
use crate::runtime::Runtime;

use super::runtime_error::Result;
use super::value::Value;

static FUNCTION_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone)]
pub struct Function {
    pub id: usize,
    pub object: FunctionObject,
    pub metadata: Option<FunctionRuntimeMetadata>,
}

impl Function {
    pub fn new(
        params: Vec<FunctionParam>,
        captured_env: Environment,
        body: Box<ast::Stmt>,
    ) -> Self {
        let unique_id = FUNCTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Function {
            id: unique_id,
            object: FunctionObject::new(params, Box::new(captured_env), body),
            metadata: None,
        }
    }

    pub fn new_builtin(params: Vec<FunctionParam>, func_ptr: BuiltinFunctionPointer) -> Self {
        let unique_id = FUNCTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Function {
            id: unique_id,
            object: FunctionObject::new_builtin(params, Box::default(), func_ptr),
            metadata: None,
        }
    }

    pub fn get_params(&self) -> Vec<FunctionParam> {
        match &self.object {
            FunctionObject::UserDefined { params, .. } => params.clone(),
            FunctionObject::Builtin { params, .. } => params.clone(),
        }
    }

    pub fn get_env(&self) -> &Environment {
        match &self.object {
            FunctionObject::UserDefined { env, .. } => env,
            FunctionObject::Builtin { env, .. } => env,
        }
    }

    pub fn get_env_mut(&mut self) -> &mut Box<Environment> {
        match &mut self.object {
            FunctionObject::UserDefined { env, .. } => env,
            FunctionObject::Builtin { env, .. } => env,
        }
    }

    pub fn set_metadata(&mut self, metadata: FunctionRuntimeMetadata) {
        self.metadata = Some(metadata);
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
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

type BuiltinFunctionPointer = fn(
    params: Vec<(FunctionParam, Value)>,
    runtime: &mut Runtime,
    context: Box<Environment>,
) -> Result<Value>;

#[derive(Debug, Clone)]
pub enum FunctionObject {
    UserDefined {
        params: Vec<FunctionParam>,
        env: Box<Environment>,
        body: Box<ast::Stmt>,
    },
    Builtin {
        params: Vec<FunctionParam>,
        env: Box<Environment>,
        func_ptr: BuiltinFunctionPointer,
    },
}

impl FunctionObject {
    pub fn new(
        params: Vec<FunctionParam>,
        context: Box<Environment>,
        body: Box<ast::Stmt>,
    ) -> Self {
        FunctionObject::UserDefined {
            params,
            body,
            env: context,
        }
    }

    pub fn new_builtin(
        params: Vec<FunctionParam>,
        env: Box<Environment>,
        func_ptr: BuiltinFunctionPointer,
    ) -> Self {
        FunctionObject::Builtin {
            params,
            env,
            func_ptr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionRuntimeMetadata {
    span: Option<Box<SourceSpan>>,
    name: Option<String>,
}

impl FunctionRuntimeMetadata {
    pub fn new(span: Option<SourceSpan>, name: Option<String>) -> Self {
        FunctionRuntimeMetadata {
            span: span.map(Box::new),
            name,
        }
    }
}

impl FunctionRuntimeMetadata {
    pub fn get_span(&self) -> Option<Box<SourceSpan>> {
        self.span.clone()
    }

    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }
}

#[macro_export]
macro_rules! params {
    ($($kind:expr),*) => {
        {
            use $crate::FunctionParam;
            vec![$($kind.to_string()),*].into_iter().map(|name| FunctionParam {
                name,
                is_captured: false,
            }).collect()
        }
    };
}
