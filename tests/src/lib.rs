use tenda_core::common::source::IdentifiedSource;
use tenda_core::parser::ast::Ast;
use tenda_core::parser::Parser;
use tenda_core::prelude::setup_runtime_prelude;
use tenda_core::runtime::{Platform, Runtime, Value};
use tenda_core::scanner::Scanner;

mod ops;

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

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn negative_list_index(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja lista = [10, 20, 30]
            seja x = lista[-1]
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    fn short_circuit_and(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja x = 0

            função incrementa()
                x = x + 1
                retorna verdadeiro
            fim

            falso e incrementa()
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime.get_global_env().get("x").unwrap().extract();

        assert_eq!(value, Value::Number(0.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn short_circuit_or(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja x = 0

            função incrementa()
                x = x + 1
                retorna verdadeiro
            fim

            verdadeiro ou incrementa()
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime.get_global_env().get("x").unwrap().extract();

        assert_eq!(value, Value::Number(0.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn nested_loops_break(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja soma_exterior = 0

            para cada i em [1,2] faça
                seja soma_interior = 0

                enquanto verdadeiro faça
                    soma_interior = soma_interior + 1
                    para
                fim

                soma_exterior = soma_exterior + soma_interior
            fim
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime
            .get_global_env()
            .get("soma_exterior")
            .unwrap()
            .extract();

        assert_eq!(value, Value::Number(2.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn nested_loops_break_outer(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja soma_exterior = 0

            para cada i em [1, 2, 3, 4, 5] faça
                seja j = 0

                enquanto j < 3 faça
                    j = j + 1
                fim

                soma_exterior = soma_exterior + i

                se i é 3 então
                    para
                fim
            fim
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime
            .get_global_env()
            .get("soma_exterior")
            .unwrap()
            .extract();

        assert_eq!(value, Value::Number(6.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn overshadow_local_in_nested_block(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja x = 1

            função testa()
                seja x = 2
            fim

            testa()
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime.get_global_env().get("x").unwrap().extract();

        assert_eq!(value, Value::Number(1.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn overshadow_function_param(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função duplica(x)
                seja x = x * 2
                retorna x
            fim

            seja resultado = duplica(3)
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime.get_global_env().get("resultado").unwrap().extract();

        assert_eq!(value, Value::Number(6.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn empty_block_in_if(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja x = 0

            se verdadeiro então
            fim
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime.get_global_env().get("x").unwrap().extract();

        assert_eq!(value, Value::Number(0.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn if_return(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função testa(a)
                se a > 5 então
                    retorna 100
                fim

                retorna 0
            fim

            seja resultado1 = testa(10)
            seja resultado2 = testa(2)
        "#;

        let runtime = interpret_stmt(platform, source);
        let r1 = runtime
            .get_global_env()
            .get("resultado1")
            .unwrap()
            .extract();

        let r2 = runtime
            .get_global_env()
            .get("resultado2")
            .unwrap()
            .extract();

        assert_eq!(r1, Value::Number(100.0));
        assert_eq!(r2, Value::Number(0.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn else_return(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função testa(b)
                se b é 0 então
                    retorna 999
                senão
                    retorna -1
                fim
            fim

            seja a = testa(0)
            seja c = testa(42)
        "#;

        let runtime = interpret_stmt(platform, source);
        let a_val = runtime.get_global_env().get("a").unwrap().extract();
        let c_val = runtime.get_global_env().get("c").unwrap().extract();

        assert_eq!(a_val, Value::Number(999.0));
        assert_eq!(c_val, Value::Number(-1.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn closure_sees_updated_var(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja x = 1

            função cria()
                função closure()
                    retorna x
                fim

                retorna closure
            fim

            seja c = cria()

            x = 999

            seja resultado = c()
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime.get_global_env().get("resultado").unwrap().extract();

        assert_eq!(value, Value::Number(999.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn function_reference_equality(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função f()
                retorna 0
            fim

            função g()
                retorna 1
            fim

            seja cmp = f é g
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime.get_global_env().get("cmp").unwrap().extract();

        assert_eq!(value, Value::Boolean(false));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn same_function_reference_equality(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função f()
                retorna 0
            fim

            seja cmp = f é f
        "#;

        let runtime = interpret_stmt(platform, source);
        let value = runtime.get_global_env().get("cmp").unwrap().extract();

        assert_eq!(value, Value::Boolean(true));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn while_break_in_nested_if(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja i = 0
            seja total = 0

            enquanto i < 10 faça
                i = i + 1

                se i é 5 então
                    para
                fim

                total = total + i
            fim
        "#;

        let runtime = interpret_stmt(platform, source);
        let val = runtime.get_global_env().get("total").unwrap().extract();

        assert_eq!(val, Value::Number(10.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn break_outside_loop(#[case] platform: impl Platform + 'static) {
        interpret_stmt(platform, "para");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn continue_outside_loop(#[case] platform: impl Platform + 'static) {
        interpret_stmt(platform, "continua");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn repeated_function_param(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função f(a, a)
                retorna a
            fim
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn block_scope_after_while(#[case] platform: impl Platform + 'static) {
        let source = r#"
            enquanto falso faça
                seja x = 1
            fim

            seja y = x
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    fn function_returns_no_explicit_value(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função sem_retorno()
                seja x = 123
            fim

            seja resultado = sem_retorno()
        "#;

        let runtime = interpret_stmt(platform, source);
        let val = runtime.get_global_env().get("resultado").unwrap().extract();

        assert_eq!(val, Value::Nil);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn chained_comparison(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "1 < 2 < 3");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn negative_list_index_assignment(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja lista = [100, 200]
            lista[-1] = 999
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn function_in_lhs_of_assignment(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função f() retorna 0 fim
            f() = 999
        "#;

        interpret_stmt(platform, source);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn call_non_function_boolean(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "verdadeiro()");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn call_non_function_list(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "[1, 2, 3]()");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn call_non_function_assoc_array(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "{ 1: 2 }()");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn call_non_function_string(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "\"hello\"()");
    }

    #[rstest]
    #[case(OSPlatform)]
    fn overshadow_variable_in_function_block(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja x = 10

            função testa()
                seja x = 999

                retorna x
            fim

            seja resultado = testa()
        "#;

        let runtime = interpret_stmt(platform, source);
        let outer_x = runtime.get_global_env().get("x").unwrap().extract();
        let result = runtime.get_global_env().get("resultado").unwrap().extract();

        assert_eq!(outer_x, Value::Number(10.0));
        assert_eq!(result, Value::Number(999.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn overshadow_parameter_in_function_block(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função soma_duas_vezes(a)
                seja a = a + a
                retorna a
            fim

            seja resultado = soma_duas_vezes(5)
        "#;

        let runtime = interpret_stmt(platform, source);
        let val = runtime.get_global_env().get("resultado").unwrap().extract();

        assert_eq!(val, Value::Number(10.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    fn partial_return_in_if(#[case] platform: impl Platform + 'static) {
        let source = r#"
            função checa_flag(flag)
                se flag então
                    retorna 111
                fim

                retorna 222
            fim

            seja r1 = checa_flag(verdadeiro)
            seja r2 = checa_flag(falso)
        "#;

        let runtime = interpret_stmt(platform, source);
        let r1 = runtime.get_global_env().get("r1").unwrap().extract();
        let r2 = runtime.get_global_env().get("r2").unwrap().extract();

        assert_eq!(r1, Value::Number(111.0));
        assert_eq!(r2, Value::Number(222.0));
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_unclosed_paren(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "(1 + 2");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_unclosed_bracket(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "[1, 2");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_unclosed_brace(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "{ 1 : 2");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_missing_colon_in_assoc(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "{ 1 2 }");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_invalid_operator(#[case] platform: impl Platform + 'static) {
        interpret_expr(platform, "1 $ 2");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_incomplete_function_declaration(#[case] platform: impl Platform + 'static) {
        interpret_stmt(platform, r#"função soma(a, b)"#);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_incomplete_if(#[case] platform: impl Platform + 'static) {
        let input = r#"
            se verdadeiro então
                seja x = 1
        "#;

        interpret_stmt(platform, input);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_incomplete_while(#[case] platform: impl Platform + 'static) {
        let input = r#"
            enquanto verdadeiro faça
                seja x = 1
        "#;

        interpret_stmt(platform, input);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_incomplete_for(#[case] platform: impl Platform + 'static) {
        let input = r#"
            para cada i em [1, 2]
                seja x = i
        "#;

        interpret_stmt(platform, input);
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_return_outside_function(#[case] platform: impl Platform + 'static) {
        interpret_stmt(platform, "retorna 5");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_break_outside_loop(#[case] platform: impl Platform + 'static) {
        interpret_stmt(platform, "para");
    }

    #[rstest]
    #[case(OSPlatform)]
    #[should_panic]
    fn parse_error_continue_outside_loop(#[case] platform: impl Platform + 'static) {
        interpret_stmt(platform, "continua");
    }

    #[rstest]
    #[case(OSPlatform)]
    fn zero_or_negative_range_iteration(#[case] platform: impl Platform + 'static) {
        let source = r#"
            seja soma = 0

            para cada i em 5 até 1 faça
                soma = soma + i
            fim
        "#;

        let runtime = interpret_stmt(platform, source);
        let val = runtime.get_global_env().get("soma").unwrap().extract();

        assert_eq!(val, Value::Number(0.0));
    }
}
