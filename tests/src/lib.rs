use tenda_core::common::source::IdentifiedSource;
use tenda_core::parser::ast::Ast;
use tenda_core::parser::Parser;
use tenda_core::prelude::setup_runtime_prelude;
use tenda_core::runtime::{Platform, Runtime, Value};
use tenda_core::scanner::Scanner;

#[cfg(test)]
mod ops;

#[cfg(test)]
mod core;

#[cfg(test)]
mod syntax;

pub fn src_to_ast(source: &str) -> Ast {
    let source_id = IdentifiedSource::dummy();
    let tokens = Scanner::new(source, source_id).scan().unwrap();

    Parser::new(&tokens, source_id).parse().unwrap()
}

pub fn interpret_expr<P: Platform + 'static>(platform: P, source: &str) -> Value {
    let ast = src_to_ast(source);

    Runtime::new(platform).eval(&ast).unwrap()
}

pub fn interpret_expr_with_prelude<P: Platform + 'static>(platform: P, source: &str) -> Value {
    let ast = src_to_ast(source);
    let mut runtime = Runtime::new(platform);

    setup_runtime_prelude(runtime.get_global_env_mut());

    runtime.eval(&ast).unwrap()
}

pub fn interpret_stmt<P: Platform + 'static>(platform: P, source: &str) -> Runtime {
    let ast = src_to_ast(source);
    let mut runtime = Runtime::new(platform);

    runtime.eval(&ast).unwrap();
    runtime
}

pub fn interpret_stmt_and_get<P: Platform + 'static>(
    platform: P,
    source: &str,
    name: &str,
) -> Value {
    let runtime = interpret_stmt(platform, source);

    runtime.get_global_env().get(name).unwrap().extract()
}
