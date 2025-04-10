macro_rules! expr_tests {
    ($($name:ident: $expr:expr => $variant:ident $parens:tt),+ $(,)?) => {
        $(
            #[rstest::rstest]
            #[case(tenda_core::platform::OSPlatform)]
            fn $name(#[case] platform: impl tenda_core::runtime::Platform + 'static) {
                use tenda_core::runtime::Value::*;

                assert_eq!(
                    crate::interpret_expr_with_prelude(platform, $expr),
                    $variant $parens
                );
            }
        )*
    };
}

expr_tests!(
    number_expr: "1" => Number(1.0),
    number_sum_expr: "(1 + 2) + (3 + 4) + 5" => Number(15.0),
    number_mult_expr: "(1 * 2) * (3 * 4) * 5" => Number(120.0),
    number_sub_expr: "(1 - 2) - (3 - 4) - 5" => Number(-5.0),
    number_div_expr: "10 / 2" => Number(5.0),
    number_exp_expr: "2 ^ 3" => Number(8.0),
    number_mod_expr: "10 % 3" => Number(1.0),
    number_greater_expr: "5 > 3" => Boolean(true),
    number_greater_eq_expr: "5 >= 5" => Boolean(true),
    number_less_expr: "3 < 5" => Boolean(true),
    number_less_eq_expr: "3 <= 3" => Boolean(true),
    number_eq_num_expr: "3 é 3" => Boolean(true),
    number_neq_expr: "3 não é 4" => Boolean(true),

    bool_expr: "verdadeiro" => Boolean(true),
    bool_expr2: "falso" => Boolean(false),
    bool_eq_expr: "verdadeiro é falso" => Boolean(false),
    bool_and_expr: "verdadeiro e falso" => Boolean(false),
    bool_or_expr: "verdadeiro ou falso" => Boolean(true),

    str_expr: "\"abc\"" => String("abc".to_string()),
    str_concat_expr: "\"Olá, \" + \"mundo!\"" => String("Olá, mundo!".to_string()),
    str_eq_expr: "\"abc\" é \"abc\"" => Boolean(true),
    str_neq_expr: "\"abc\" não é \"def\"" => Boolean(true),
    str_number_concat_expr: "\"abc\" + 123" => String("abc123".to_string()),
    str_bool_concat_expr: "\"abc\" + verdadeiro" => String("abcverdadeiro".to_string()),
    str_list_concat_expr: "\"abc\" + [1, 2]" => String("abc[1, 2]".to_string()),
    str_assoc_array_concat_expr: "\"abc\" + { 1: 2 }" => String("abc{ 1: 2 }".to_string()),
    str_range_concat_expr: "\"abc\" + (1 até 5)" => String("abc1 até 5".to_string()),
    str_date_concat_expr: "\"abc\" + Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        String("abc2025-04-10T00:45:26.580-03:00".to_string()),
    str_nil_concat_expr: "\"abc\" + Nada" => String("abcNada".to_string()),

    date_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Date(tenda_core::runtime::Date::from_iso_string("2025-04-10T00:45:26.580-03:00").unwrap()),
    date_eq_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") é Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_neq_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") não é Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(false),
    date_sum_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") + 1" =>
        Date(tenda_core::runtime::Date::from_iso_string("2025-04-10T00:45:26.581-03:00").unwrap()),
    date_sub: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") - 1" =>
        Date(tenda_core::runtime::Date::from_iso_string("2025-04-10T00:45:26.579-03:00").unwrap()),
    date_greater_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") > Data.de_iso(\"2025-04-09T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_greater_eq_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") >= Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_less_expr: "Data.de_iso(\"2025-04-09T00:45:26.580-03:00\") < Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_less_eq_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") <= Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(true),

    list_expr: "[1, 2, 3]" =>
        List(std::rc::Rc::new(std::cell::RefCell::new(vec![Number(1.0), Number(2.0), Number(3.0)]))),
    list_concat_expr: "[1, 2] + [3, 4]" =>
        List(std::rc::Rc::new(std::cell::RefCell::new(vec![Number(1.0), Number(2.0), Number(3.0), Number(4.0)]))),
    list_eq_expr: "[1, 2] é [1, 2]" => Boolean(true),
    list_has_expr: "[1, 2, 3] tem 2" => Boolean(true),
    list_lacks_expr: "[1, 2, 3] não tem 4" => Boolean(true),

    assoc_array_expr: "{ 1: 2, 3: 4 }" => AssociativeArray(std::rc::Rc::new(std::cell::RefCell::new(
        tenda_core::runtime::AssociativeArray::from([
            (tenda_core::runtime::AssociativeArrayKey::Number(1), Number(2.0)),
            (tenda_core::runtime::AssociativeArrayKey::Number(3), Number(4.0)),
        ])
    ))),
    assoc_array_has: "{ 1: 2, 3: 4 } tem 1" => Boolean(true),
    assoc_array_lack: "{ 1: 2, 3: 4 } não tem 5" => Boolean(true),

    range_expr: "1 até 5" => Range(1, 5),
    range_eq_expr: "1 até 5 é 1 até 5" => Boolean(true),
);

#[rstest::rstest]
#[should_panic]
#[case(tenda_core::platform::OSPlatform)]
fn division_by_zero_expr(#[case] platform: impl tenda_core::runtime::Platform + 'static) {
    crate::interpret_expr(platform, "0 / 0");
}
