use tenda_core::common::source::IdentifiedSource;
use tenda_core::parser::ast::Ast;
use tenda_core::parser::Parser;
use tenda_core::prelude::setup_runtime_prelude;
use tenda_core::runtime::{Platform, Runtime, Value};
use tenda_core::scanner::Scanner;

mod bin_ops;

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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use tenda_core::platform::OSPlatform;

    #[rstest]
    #[case(OSPlatform)]
    fn if_statement(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = 0

            se verdadeiro então
                resultado = 1
            senão
                resultado = 2
            fim
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(1.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn for_loop_with_list(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = 0
            seja lista = [1, 2, 3, 4, 5]

            para cada i em lista faça
                resultado = resultado + i
            fim
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(15.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn for_loop_with_range(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = 0

            para cada i em 1 até 5 faça
                resultado = resultado + i
            fim
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(15.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn while_loop(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = 0
            seja i = 1

            enquanto i <= 5 faça
                resultado = resultado + i
                i = i + 1
            fim
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(15.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn function_definition_and_call(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função soma(a, b)
                retorna a + b
            fim

            seja resultado = soma(2, 3)
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(5.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn closures(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função cria_somador(x)
                função somador(y)
                    retorna x + y
                fim

                retorna somador
            fim

            seja somador = cria_somador(2)
            seja resultado = somador(3)
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(5.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn nested_closures(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função cria_multiplicador(x)
                função multiplicador(y)
                    função multiplicador_interno(z)
                        retorna x * y * z
                    fim

                    retorna multiplicador_interno
                fim

                retorna multiplicador
            fim

            seja multiplicador = cria_multiplicador(2)
            seja resultado = multiplicador(3)(4)
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(24.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn anonymous_function(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = (função(x, y)
                retorna x + y
            fim)(2, 3)
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(5.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn list_access(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja lista = [1, 2, 3]
            seja resultado = lista[1]
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(2.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn list_mutation(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja lista = [1, 2, 3]

            lista[0] = 10

            seja resultado = lista[0]
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(10.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn associative_array_access(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja dicionário = { "chave1": 1, "chave2": 2 }
            seja resultado = dicionário["chave1"] + dicionário.chave2
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(3.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn associative_array_mutation(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja dicionário = { "chave1": 1, "chave2": 2 }

            dicionário["chave1"] = 10
            dicionário.chave2 = 20

            seja resultado = dicionário["chave1"] + dicionário.chave2
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(30.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn for_loop_break(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = 0

            para cada i em 1 até 10 faça
                se i é 5 então
                    para
                fim

                resultado = resultado + i
            fim
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(10.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    fn for_loop_continue(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = 0

            para cada i em 1 até 10 faça
                se i é 5 então
                    continua
                fim

                resultado = resultado + i
            fim
        "#;

        assert_eq!(
            interpret_stmt_and_get(platform, source, "resultado"),
            Value::Number(50.0)
        );
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn undefined_reference(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "resultado");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn already_declared(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = 0
            seja resultado = 1
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn wrong_number_of_arguments(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função soma(a, b)
                retorna a + b
            fim

            seja resultado = soma(1)
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn list_out_of_bounds(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja lista = [1, 2, 3]
            seja resultado = lista[3]
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn invalid_index_type(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja lista = [1, 2, 3]
            seja resultado = lista["chave"]
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn invalid_range_bounds(#[case] platform: impl Platform + 'static) {
        interpret_stmt(platform, "1.1 até 2");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn invalid_associative_array_key_value(#[case] platform: impl Platform + 'static) {
        interpret_stmt(platform, "{ 1.5: 2 }");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn associative_array_key_not_found(#[case] platform: impl Platform + 'static) {
        let source = r#"
                seja dicionário = { "chave1": 1, "chave2": 2 }
                seja resultado = dicionário["chave3"]
            "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn not_iterable(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja resultado = 0

            para cada i em 1.5 faça
                resultado = resultado + i
            fim
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn immutable_strings(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja texto = "Olá, mundo!"
            texto[0] = "o"
        "#;

        interpret_stmt(platform, source);
    }
}
