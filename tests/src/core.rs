use rstest::rstest;
use tenda_core::{
    platform::OSPlatform,
    runtime::{Platform, Value},
};

use crate::{interpret_expr, interpret_stmt, interpret_stmt_and_get};

#[rstest]
#[case(OSPlatform)]
fn if_statement(#[case] platform: impl Platform + 'static) {
    let source = r#"
        seja resultado = 0

        se verdadeiro então faça
            resultado = 1
        senão faça
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
        seja soma(a, b) = faça
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
        seja cria_somador(x) = faça
            seja somador(y) = faça
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
        seja cria_multiplicador(x) = faça
            seja multiplicador(y) = faça
                seja multiplicador_interno(z) = faça
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
        seja resultado = (função(x, y) -> x + y)(2, 3)
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
            se i é 5 então faça
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
            se i é 5 então faça
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

        seja incrementa() = faça
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

        seja incrementa() = faça
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

            se i é 3 então faça
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

        seja testa() = faça
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
        seja duplica(x) = faça
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

        se verdadeiro então faça
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
        seja testa(a) = faça
            se a > 5 então faça
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
        seja testa(b) = faça
            se b é 0 então faça
                retorna 999
            senão faça
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

        seja cria() = faça
            seja closure() = faça
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
        seja f() = 0
        seja g() = 1
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
        seja f() = faça
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

            se i é 5 então faça
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
        seja f(a, a) = faça
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
        seja sem_retorno() = faça
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
        seja f() = 0
        f() = 999
    "#;

    interpret_stmt(platform, source);
}

#[rstest]
#[case(OSPlatform)]
fn overshadow_variable_in_function_block(#[case] platform: impl Platform + 'static) {
    let source = r#"
        seja x = 10

        seja testa() = faça
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
        seja soma_duas_vezes(a) = faça
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
        seja checa_flag(flag) = faça
            se flag então faça
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
fn parse_error_incomplete_function_declaration(#[case] platform: impl Platform + 'static) {
    interpret_stmt(platform, r#"seja soma(a, b)"#);
}

#[rstest]
#[case(OSPlatform)]
#[should_panic]
fn parse_error_incomplete_function_declaration_2(#[case] platform: impl Platform + 'static) {
    interpret_stmt(platform, r#"seja soma(a, b) ="#);
}

#[rstest]
#[case(OSPlatform)]
#[should_panic]
fn parse_error_incomplete_if(#[case] platform: impl Platform + 'static) {
    let input = r#"
        se verdadeiro então faça
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

#[rstest]
#[case(OSPlatform)]
#[should_panic]
fn use_function_argument_outside_function(#[case] platform: impl Platform + 'static) {
    let source = r#"
        seja f(x) = x
        f(1)
        x
    "#;

    interpret_stmt(platform, source);
}
