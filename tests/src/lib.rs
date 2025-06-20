use tenda_core::common::source::IdentifiedSource;
use tenda_core::parser::ast::Ast;
use tenda_core::parser::Parser;
use tenda_core::prelude::get_runtime_prelude;
use tenda_core::runtime::{Platform, Runtime, Unit, Value};
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

    Parser::new(&tokens, source_id).parse().unwrap().ast
}

pub fn interpret_expr<P: Platform + 'static>(platform: P, source: &str) -> Value {
    let ast = src_to_ast(source);
    let unit = Unit::new(ast, vec![]);

    Runtime::new(platform).eval(unit).unwrap()
}

pub fn interpret_expr_with_prelude<P: Platform + 'static>(platform: P, source: &str) -> Value {
    let ast = src_to_ast(source);
    let mut runtime = Runtime::with_builtins(platform, get_runtime_prelude());
    let unit = Unit::new(ast, vec![]);

    runtime.eval(unit).unwrap()
}

pub fn interpret_stmt<P: Platform + 'static>(platform: P, source: &str) -> Runtime {
    let ast = src_to_ast(source);
    let mut runtime = Runtime::new(platform);
    let unit = Unit::new(ast, vec![]);

    runtime.eval(unit).unwrap();
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

#[macro_export]
macro_rules! expr_tests {
    ($($name:ident: $expr:expr => $variant:ident $parens:tt),+ $(,)?) => {
        $(
            #[rstest::rstest]
            #[case(OSPlatform)]
            fn $name(#[case] platform: impl Platform + 'static) {
                use $crate::Value::*;

                assert_eq!(
                    $crate::interpret_expr_with_prelude(platform, $expr),
                    $variant $parens
                );
            }
        )*
    };
}

#[macro_export]
macro_rules! expr_tests_should_panic {
    ($($name:ident: $expr:expr),+ $(,)?) => {
        $(
            #[rstest::rstest]
            #[case(OSPlatform)]
            #[should_panic]
            fn $name(#[case] platform: impl Platform + 'static) {
                $crate::interpret_expr_with_prelude(platform, $expr);
            }
        )*
    };
}
