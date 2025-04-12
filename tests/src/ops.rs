use std::{cell::RefCell, rc::Rc};
use tenda_core::{
    platform::OSPlatform,
    runtime::{self, AssociativeArrayKey, Platform, Value},
};

use crate::interpret_expr_with_prelude;

macro_rules! expr_tests {
    ($($name:ident: $expr:expr => $variant:ident $parens:tt),+ $(,)?) => {
        $(
            #[rstest::rstest]
            #[case(OSPlatform)]
            fn $name(#[case] platform: impl Platform + 'static) {
                use Value::*;

                assert_eq!(
                    interpret_expr_with_prelude(platform, $expr),
                    $variant $parens
                );
            }
        )*
    };
}

macro_rules! expr_tests_should_panic {
    ($($name:ident: $expr:expr),+ $(,)?) => {
        $(
            #[rstest::rstest]
            #[case(OSPlatform)]
            #[should_panic]
            fn $name(#[case] platform: impl Platform + 'static) {
                interpret_expr_with_prelude(platform, $expr);
            }
        )*
    };
}

expr_tests_should_panic!(
    chained_comparison_expr: "1 < 2 < 3",
    chained_comparison2_expr: "1 > 2 > 3",
    chained_comparison3_expr: "1 <= 2 <= 3",
    chained_comparison4_expr: "1 >= 2 >= 3",
    chained_comparison_mixed_expr: "1 < 2 > 3",
    chained_comparison_mixed2_expr: "1 > 2 < 3",
    chained_comparison_mixed3_expr: "1 <= 2 >= 3",
    chained_comparison_mixed4_expr: "1 >= 2 <= 3",
);

expr_tests_should_panic!(
    func_call_invalid_type_expr: "verdadeiro()",
    func_call_invalid_type2_expr: "[1, 2, 3]()",
    func_call_invalid_type3_expr: "{ 1: 2 }()",
    func_call_invalid_type4_expr: "\"olá\"()",
);

expr_tests!(
    num_expr: "1" => Number(1.0),
    num_sum_expr: "(1 + 2) + (3 + 4) + 5" => Number(15.0),
    num_mult_expr: "(1 * 2) * (3 * 4) * 5" => Number(120.0),
    num_sub_expr: "(1 - 2) - (3 - 4) - 5" => Number(-5.0),
    num_div_expr: "10 / 2" => Number(5.0),
    num_exp_expr: "2 ^ 3" => Number(8.0),
    num_mod_expr: "10 % 3" => Number(1.0),
    num_greater_expr: "5 > 3" => Boolean(true),
    num_greater_equality_expr: "5 >= 5" => Boolean(true),
    num_less_expr: "3 < 5" => Boolean(true),
    num_less_equality_expr: "3 <= 3" => Boolean(true),
    num_equality_num_expr: "3 é 3" => Boolean(true),
    num_nequality_expr: "3 não é 4" => Boolean(true),
    num_neg_exp_expr: "-2 ^ 3" => Number(-8.0),
    num_nequality_str_expr: "3 não é \"3\"" => Boolean(true),
    num_mod_neg_expr: "-10 % 3" => Number(-1.0),
    num_neg_zero_literal: "(-0)" => Number(0.0),
    num_sum_with_neg_operand: "1 + -2" => Number(-1.0),
    num_decimal_sum_then_mult: "(1.5 + 2.5) * 2" => Number(8.0),
    num_div_by_fraction: "2 / 0.5" => Number(4.0),
    num_div_result_fraction: "10 / 4" => Number(2.5),
    num_neg_exponentiation_square: "(-5) ^ 2" => Number(25.0),
    num_mod_operation_positive: "5 % 2" => Number(1.0),
    num_neg_exponentiation_reciprocal: "(-2) ^ -2" => Number(0.25),
    num_mixed_precedence_exponentiation: "(2 * 2) ^ 3" => Number(64.0),
    num_operator_precedence_linear: "1 + 2 * 3 / 6" => Number(2.0),
    num_small_number_scaling: "0.0001 * 10000" => Number(1.0),
    num_equality_ge_test_equal: "1 >= 1" => Boolean(true),
    num_inequality_test_equal_fail: "1 > 1" => Boolean(false),
    num_large_precision_sum: "1000000 + 0.000001" => Number(1000000.000001),
    num_neg_mult: "(-2) * (-3)" => Number(6.0),
    num_mixed_add_and_mult: "2 * 3 + 4 * 5" => Number(26.0),
    num_decimal_imprecision_sum: "0.1 + 0.2" => Number(0.30000000000000004),
    num_large_integers_sum: "123456789 + 987654321" => Number(1111111110.0),
    num_sub_groupings: "(1 - 2) + (3 - 4)" => Number(-2.0),
    num_neg_mod_with_negs: "(-10) % (-3)" => Number(-1.0),
);

expr_tests_should_panic!(
    num_div_by_zero: "0 / 0",
);

expr_tests!(
    bool_expr: "verdadeiro" => Boolean(true),
    bool_expr2: "falso" => Boolean(false),
    bool_equality_expr: "verdadeiro é falso" => Boolean(false),
    bool_and_expr: "verdadeiro e falso" => Boolean(false),
    bool_or_expr: "verdadeiro ou falso" => Boolean(true),
    bool_all_true_and_chain: "verdadeiro e verdadeiro e verdadeiro" => Boolean(true),
    bool_and_chain_includes_false: "verdadeiro e verdadeiro e falso" => Boolean(false),
    bool_or_chain_ends_true: "falso ou falso ou verdadeiro" => Boolean(true),
    bool_chained_equality_false: "falso é falso é falso" => Boolean(false),
    bool_chained_equality_all_true: "verdadeiro é verdadeiro é verdadeiro" => Boolean(true),
    bool_and_with_comparison_true: "verdadeiro e (1 < 2)" => Boolean(true),
    bool_or_with_comparison_false: "falso ou (2 > 3)" => Boolean(false),
    bool_equality_comparison_in_and: "(1 é 1) e (2 é 3)" => Boolean(false),
    bool_multiple_less_than_comparisons_true: "(1 < 2) e (3 < 4) e (5 < 6)" => Boolean(true),
    bool_or_with_greater_equal_true: "(10 >= 10) ou (1 > 2)" => Boolean(true),
    bool_equality_of_false_values: "(5 não é 5) ou (6 não é 6)" => Boolean(false),
    bool_or_of_false_and_true_combo: "verdadeiro ou falso e verdadeiro" => Boolean(true),
    bool_nested_triple_equality_true: "((1 < 2) é (3 < 4)) é verdadeiro" => Boolean(true),
    bool_complex_and_with_false: "((1 > 2) e verdadeiro) ou falso" => Boolean(false),
    bool_not_true_expr: "não verdadeiro" => Boolean(false),
    bool_not_false_expr: "não falso" => Boolean(true),
    bool_or_with_and_precedence_expr: "verdadeiro ou verdadeiro e falso" => Boolean(true),
    bool_equality_comparison_with_range: "(1 < 2) é (1 < 2)" => Boolean(true),
    bool_multiple_chained_and_comparisons: "(1 >= 1) e (1 <= 1)" => Boolean(true),
);

expr_tests!(
    str_expr: "\"abc\"" => String("abc".to_string()),
    str_concat_expr: "\"Olá, \" + \"mundo!\"" => String("Olá, mundo!".to_string()),
    str_equality_expr: "\"abc\" é \"abc\"" => Boolean(true),
    str_nequality_expr: "\"abc\" não é \"def\"" => Boolean(true),
    str_num_concat_expr: "\"abc\" + 123" => String("abc123".to_string()),
    str_bool_concat_expr: "\"abc\" + verdadeiro" => String("abcverdadeiro".to_string()),
    str_list_concat_expr: "\"abc\" + [1, 2]" => String("abc[1, 2]".to_string()),
    str_assoc_array_concat_expr: "\"abc\" + { 1: 2 }" => String("abc{ 1: 2 }".to_string()),
    str_range_concat_expr: "\"abc\" + (1 até 5)" => String("abc1 até 5".to_string()),
    str_date_concat_expr: "\"abc\" + Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        String("abc2025-04-10T00:45:26.580-03:00".to_string()),
    str_nil_concat_expr: "\"abc\" + Nada" => String("abcNada".to_string()),
    str_nequality_num_expr: "\"123\" não é 123" => Boolean(true),
    str_unicode_chinese: "\"unicode: 你好\"" => String("unicode: 你好".to_string()),
    str_unicode_emoji_snake: "\"unicode: 🐍\"" => String("unicode: 🐍".to_string()),
    str_multiple_spaces: "\"Espaços  múltiplos\"" => String("Espaços  múltiplos".to_string()),
    str_with_newline_escape: "\"Com\\nQuebra\"" => String("Com\nQuebra".to_string()),
    str_concat_with_numeric_computation: "\"2 + 2 = \" + (2 + 2)" => String("2 + 2 = 4".to_string()),
    str_concat_empty_strings: "\"\" + \"\"" => String("".to_string()),
    str_hello_space_world: "\"Olá\" + \" \" + \"Mundo\"" => String("Olá Mundo".to_string()),
    str_concat_with_negative_number: "\"foo\" + (-5)" => String("foo-5".to_string()),
    str_concatenate_two_literals: "\"AB\" + \"CD\"" => String("ABCD".to_string()),
    str_emoji_concatenation: "\"😃\" + \"😡\"" => String("😃😡".to_string()),
    str_with_tab_escape: "\"Tab:\\t\"" => String("Tab:\t".to_string()),
    str_with_linefeed_escape: "\"Linha:\\n\"" => String("Linha:\n".to_string()),
    str_with_backslash_escape: "\"Backslash: \\\\\"" => String("Backslash: \\".to_string()),
    str_concat_with_trailing_spaces: "\"Margem \" + \"    \"" => String("Margem     ".to_string()),
    str_trim_test_unchanged: "\"  trim  \"" => String("  trim  ".to_string()),
    str_mixed_case_literal: "\"MixedCase\"" => String("MixedCase".to_string()),
    str_multiple_concat_sequence: "\"algo \" + \"coisas\" + \" aqui\"" => String("algo coisas aqui".to_string()),
    str_concat_boolean_comparison_false: "\"abc\" + (verdadeiro é falso)" => String("abcfalso".to_string()),
    str_concat_comparison_numeric_true: "\"abc\" + (10 > 9)" => String("abcverdadeiro".to_string()),
    str_concat_with_nil: "\"xyz\" + Nada" => String("xyzNada".to_string()),
    str_concat_with_list_representation: "\"Combinar\" + [1, 2]" => String("Combinar[1, 2]".to_string()),
    str_nested_concatenation_with_space: "\"Olá\" + (\" \" + \"amigo\")" => String("Olá amigo".to_string())
);

expr_tests!(
    date_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Date(runtime::Date::from_iso_string("2025-04-10T00:45:26.580-03:00").unwrap()),
    date_equality_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") é Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_nequality_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") não é Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(false),
    date_sum_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") + 1" =>
        Date(runtime::Date::from_iso_string("2025-04-10T00:45:26.581-03:00").unwrap()),
    date_sub: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") - 1" =>
        Date(runtime::Date::from_iso_string("2025-04-10T00:45:26.579-03:00").unwrap()),
    date_greater_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") > Data.de_iso(\"2025-04-09T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_greater_equality_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") >= Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_less_expr: "Data.de_iso(\"2025-04-09T00:45:26.580-03:00\") < Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_less_equality_expr: "Data.de_iso(\"2025-04-10T00:45:26.580-03:00\") <= Data.de_iso(\"2025-04-10T00:45:26.580-03:00\")" =>
        Boolean(true),
    date_iso_jan_first_2023: "Data.de_iso(\"2023-01-01T00:00:00Z\")" =>
        Date(runtime::Date::from_iso_string("2023-01-01T00:00:00Z").unwrap()),
    date_iso_leap_feb29_2024: "Data.de_iso(\"2024-02-29T12:34:56Z\")" =>
        Date(runtime::Date::from_iso_string("2024-02-29T12:34:56Z").unwrap()),
    date_end_of_year_increment_msec: "Data.de_iso(\"2030-12-31T23:59:59Z\") + 1" =>
        Date(runtime::Date::from_iso_string("2030-12-31T23:59:59.001Z").unwrap()),
    date_end_of_year_decrement_sec: "Data.de_iso(\"2030-12-31T23:59:59Z\") - 1000" =>
        Date(runtime::Date::from_iso_string("2030-12-31T23:59:58Z").unwrap()),
    date_comparison_later_time_true: "Data.de_iso(\"2023-05-10T10:00:00Z\") > Data.de_iso(\"2023-05-10T09:59:59Z\")" =>
        Boolean(true),
    date_comparison_earlier_time_true: "Data.de_iso(\"2025-12-31T23:59:59Z\") < Data.de_iso(\"2026-01-01T00:00:00Z\")" =>
        Boolean(true),
    date_exact_equality_expr: "Data.de_iso(\"2023-01-01T00:00:00Z\") é Data.de_iso(\"2023-01-01T00:00:00Z\")" =>
        Boolean(true),
    date_exact_inequality_test_seconds_diff: "Data.de_iso(\"2023-01-01T00:00:00Z\") não é Data.de_iso(\"2023-01-01T00:00:01Z\")" =>
        Boolean(true),
    date_with_positive_timezone_offset: "Data.de_iso(\"2022-01-01T00:00:00+02:00\")" =>
        Date(runtime::Date::from_iso_string("2022-01-01T00:00:00+02:00").unwrap()),
    date_leap_year_exact_equality: "Data.de_iso(\"2000-02-29T12:00:00Z\") é Data.de_iso(\"2000-02-29T12:00:00Z\")" =>
        Boolean(true),
    date_different_offset_inequality: "Data.de_iso(\"2000-02-29T12:00:00Z\") não é Data.de_iso(\"2000-02-29T12:00:00-01:00\")" =>
        Boolean(true),
    date_2038_boundary_plus_msec: "Data.de_iso(\"2038-01-19T03:14:07Z\") + 1" =>
        Date(runtime::Date::from_iso_string("2038-01-19T03:14:07.001Z").unwrap()),
    date_epoch_minus_one_msec: "Data.de_iso(\"1970-01-01T00:00:00Z\") - 1" =>
        Date(runtime::Date::from_iso_string("1969-12-31T23:59:59.999Z").unwrap()),
    date_2038_comparison_true: "Data.de_iso(\"2038-01-19T03:14:08Z\") > Data.de_iso(\"2038-01-19T03:14:07Z\")" =>
        Boolean(true),
    date_chronological_false_expr: "Data.de_iso(\"2000-01-01T00:00:00Z\") < Data.de_iso(\"1999-12-31T23:59:59Z\")" =>
        Boolean(false),
    date_equality_or_greater_expr: "Data.de_iso(\"2000-01-01T00:00:00Z\") >= Data.de_iso(\"2000-01-01T00:00:00Z\")" =>
        Boolean(true),
    date_comparison_false_for_less: "Data.de_iso(\"2038-01-19T03:14:07Z\") <= Data.de_iso(\"2038-01-19T03:14:06Z\")" =>
        Boolean(false),
    date_new_year_from_ending_msec: "Data.de_iso(\"2021-12-31T23:59:59.999Z\") + 1" =>
        Date(runtime::Date::from_iso_string("2022-01-01T00:00:00Z").unwrap()),
    date_equal_with_z_suffix: "Data.de_iso(\"2022-01-01T00:00:00Z\") é Data.de_iso(\"2022-01-01T00:00:00+00:00\")" =>
        Boolean(true),
    date_different_timezone_false: "Data.de_iso(\"2022-01-01T00:00:00-03:00\") não é Data.de_iso(\"2022-01-01T00:00:00Z\")" =>
        Boolean(true),
    date_post_leap_comparison_true: "Data.de_iso(\"2000-03-01T00:00:00Z\") > Data.de_iso(\"2000-02-29T23:59:59Z\")" =>
        Boolean(true),
    date_epoch_transition_comparison_true: "Data.de_iso(\"1999-12-31T23:59:59Z\") < Data.de_iso(\"2000-01-01T00:00:00Z\")" =>
        Boolean(true)
);

expr_tests_should_panic!(
    date_mult_error: "Data.de_iso(\"2025-04-10T00:00:00Z\") * 2",
);

expr_tests!(
    list_expr: "[1, 2, 3]" =>
        List(Rc::new(RefCell::new(vec![Number(1.0), Number(2.0), Number(3.0)]))),
    list_concat_expr: "[1, 2] + [3, 4]" =>
        List(Rc::new(RefCell::new(vec![Number(1.0), Number(2.0), Number(3.0), Number(4.0)]))),
    list_equality_expr: "[1, 2] é [1, 2]" => Boolean(true),
    list_equality2_expr: "[1, 2] é [1, 2, 3]" => Boolean(false),
    list_has_expr: "[1, 2, 3] tem 2" => Boolean(true),
    list_lacks_expr: "[1, 2, 3] não tem 4" => Boolean(true),
    empty_list_expr: "[]" =>
        List(Rc::new(RefCell::new(vec![]))),
    nested_list_of_numbers_expr: "[[1, 2], [3, 4]]" =>
        List(Rc::new(RefCell::new(vec![
            List(Rc::new(RefCell::new(vec![Number(1.0), Number(2.0)]))),
            List(Rc::new(RefCell::new(vec![Number(3.0), Number(4.0)]))),
        ]))),
    list_with_nested_list_mixed_expr: "[1, [2, 3], 4]" =>
        List(Rc::new(RefCell::new(vec![
            Number(1.0),
            List(Rc::new(RefCell::new(vec![Number(2.0), Number(3.0)]))),
            Number(4.0),
        ]))),
    list_of_mixed_types_expr: "[\"olá\", verdadeiro, 123, Nada]" =>
        List(Rc::new(RefCell::new(vec![
            String("olá".to_string()),
            Boolean(true),
            Number(123.0),
            Nil,
        ]))),
    list_including_range_expr: "[(1 até 3), \"coisas\", 42]" =>
        List(Rc::new(RefCell::new(vec![
            Range(1, 3),
            String("coisas".to_string()),
            Number(42.0),
        ]))),
    list_arithmetic_expressions_expr: "[1 + 2, 3 * 4]" =>
        List(Rc::new(RefCell::new(vec![Number(3.0), Number(12.0)]))),
    list_chained_concatenation_expr: "([1, 2] + [3, 4]) + [5]" =>
        List(Rc::new(RefCell::new(vec![
            Number(1.0), Number(2.0), Number(3.0), Number(4.0), Number(5.0)
        ]))),
    list_deeply_nested_empty_expr: "[[[]]]" =>
        List(Rc::new(RefCell::new(vec![
            List(Rc::new(RefCell::new(vec![
                List(Rc::new(RefCell::new(vec![]))),
            ]))),
        ]))),
    list_nested_equality_true_expr: "[[1], [2]] é [[1], [2]]" => Boolean(true),
    list_string_elements_equality_false_expr: "[\"abc\", \"def\"] é [\"abc\", \"xyz\"]" => Boolean(false),
    list_inequality_due_to_order_expr: "[1, 2] não é [2, 1]" => Boolean(true),
    list_multiple_concatenation_expr: "[1, 2] + [3] + [4, 5]" =>
        List(Rc::new(RefCell::new(vec![
            Number(1.0), Number(2.0), Number(3.0), Number(4.0), Number(5.0)
        ]))),
    list_empty_concatenation_expr: "[] + []" =>
        List(Rc::new(RefCell::new(vec![]))),
    list_membership_found_numeric_expr: "[1, 2, 3] tem 1" => Boolean(true),
    list_membership_not_found_wrong_type_expr: "[1, 2, 3] tem \"1\"" => Boolean(false),
    list_membership_boolean_nil_expr: "[Nada, verdadeiro, falso] tem verdadeiro" => Boolean(true),
    list_membership_arithmetic_expr: "[1, 2, 3] tem (1 + 1)" => Boolean(true),
    list_membership_negative_expr: "[1, 2, 3] não tem 2" => Boolean(false),
    list_equality_chained_comparison_true_expr: "([1, 2] é [1, 2]) é ([1, 2] é [1, 2])" => Boolean(true),
    list_inequality_different_length_expr: "[1, 2, 3] não é [1, 2, 3, 4]" => Boolean(true),
    list_mixed_elements_structure_expr: "[Nada, Nada] é [Nada, Nada]" => Boolean(true)
);

expr_tests!(
    assoc_array_expr: "{ 1: 2, 3: 4 }" => AssociativeArray(Rc::new(RefCell::new(
        runtime::AssociativeArray::from([
            (AssociativeArrayKey::Number(1), Number(2.0)),
            (AssociativeArrayKey::Number(3), Number(4.0)),
        ])
    ))),
    assoc_array_has: "{ 1: 2, 3: 4 } tem 1" => Boolean(true),
    assoc_array_lack: "{ 1: 2, 3: 4 } não tem 5" => Boolean(true),
    assoc_array_equality_expr: "{ 1: 2, 3: 4 } é { 1: 2, 3: 4 }" => Boolean(true),
    assoc_array_equality2_expr: "{ 1: 2, 3: 4 } é { 1: 2, 3: 5 }" => Boolean(false),
    assoc_array_empty_expr: "{ }" =>
        AssociativeArray(Rc::new(RefCell::new(
            runtime::AssociativeArray::from([])
        ))),
    assoc_array_single_string_key_expr: "{ \"foo\": \"bar\" }" =>
        AssociativeArray(Rc::new(RefCell::new(
            runtime::AssociativeArray::from([
                (AssociativeArrayKey::String("foo".to_string()),
                    String("bar".to_string()))
            ])
        ))),
    assoc_array_nested_list_and_map_expr: "{ \"aninhado\": [1, 2], \"dicionário\": { \"interior\": 42 } }" =>
        AssociativeArray(Rc::new(RefCell::new(
            runtime::AssociativeArray::from([
                (AssociativeArrayKey::String("aninhado".to_string()),
                List(Rc::new(RefCell::new(vec![Number(1.0), Number(2.0)])))),
                (AssociativeArrayKey::String("dicionário".to_string()),
                AssociativeArray(Rc::new(RefCell::new(
                    runtime::AssociativeArray::from([
                        (AssociativeArrayKey::String("interior".to_string()),
                        Number(42.0))
                    ])
                ))))
            ])
        ))),
    assoc_array_numeric_keys_with_list_expr: "{ 1: [2, 3], 2: \"algo\" }" =>
        AssociativeArray(Rc::new(RefCell::new(
            runtime::AssociativeArray::from([
                (AssociativeArrayKey::Number(1),
                List(Rc::new(RefCell::new(vec![Number(2.0), Number(3.0)])))),
                (AssociativeArrayKey::Number(2),
                    String("algo".to_string()))
            ])
        ))),
    assoc_array_mixed_numeric_values_expr: "{ \"a\": 1.5, \"b\": -2 }" =>
        AssociativeArray(Rc::new(RefCell::new(
            runtime::AssociativeArray::from([
                (AssociativeArrayKey::String("a".to_string()), Number(1.5)),
                (AssociativeArrayKey::String("b".to_string()), Number(-2.0))
            ])
        ))),
    assoc_array_equality_true_expr: "{ \"a\": \"b\" } é { \"a\": \"b\" }" => Boolean(true),
    assoc_array_equality_false_expr: "{ \"a\": 1 } é { \"a\": 2 }" => Boolean(false),
    assoc_array_membership_found_numeric_key_in_list_expr: "{ 1: [\"x\"] } tem 1" => Boolean(true),
    assoc_array_membership_wrong_type_expr: "{ 1: [\"x\"] } tem \"1\"" => Boolean(false),
    assoc_array_membership_found_string_key_expr: "{ \"abc\": 123, \"def\": 456 } tem \"abc\"" => Boolean(true),
    assoc_array_membership_key_not_found_expr: "{ \"abc\": 123 } não tem \"xyz\"" => Boolean(true),
    assoc_array_nested_assoc_in_value_expr: "{ \"x\": [1,2], \"y\": {\"aninhado\": verdadeiro} } tem \"y\"" => Boolean(true),
    assoc_array_equality_nested_maps_true_expr: "{ \"x\": {1: 2}, \"y\": {1: 2} } é { \"x\": {1: 2}, \"y\": {1: 2} }" => Boolean(true),
    assoc_array_inequality_nested_maps_diff_expr: "{ \"x\": {1: 2}, \"y\": {1: 2} } é { \"x\": {1: 2}, \"y\": {1: 3} }" => Boolean(false),
    assoc_array_empty_inner_assoc_expr: "{ \"vazio\": { } }" =>
        AssociativeArray(Rc::new(RefCell::new(
            runtime::AssociativeArray::from([
                (AssociativeArrayKey::String("vazio".to_string()),
                    AssociativeArray(Rc::new(RefCell::new(
                        runtime::AssociativeArray::from([])
                    ))))
            ])
        ))),
    assoc_array_membership_numeric_key_true_expr: "{ 10: 20, 11: 21 } tem 10" => Boolean(true),
    assoc_array_membership_numeric_key_false_expr: "{ 10: 20, 11: 21 } tem 12" => Boolean(false),
    assoc_array_deeply_nested_assoc_expr: "{ 1: {2: {3: 4}} }" =>
        AssociativeArray(Rc::new(RefCell::new(
            runtime::AssociativeArray::from([
                (AssociativeArrayKey::Number(1),
                    AssociativeArray(Rc::new(RefCell::new(
                        runtime::AssociativeArray::from([
                            (AssociativeArrayKey::Number(2),
                            AssociativeArray(Rc::new(RefCell::new(
                                runtime::AssociativeArray::from([
                                    (AssociativeArrayKey::Number(3),
                                    Number(4.0))
                                ])
                            ))))
                        ])
                    ))))
            ])
        ))),
    assoc_array_numeric_keys_equality_expr: "{ 1: {2: 3} } é { 1: {2: 3} }" => Boolean(true),
    assoc_array_complex_nested_inequality_expr:
        "{ \"abc\": 1, \"def\": [1,2], \"ghi\": { \"x\": 9} } não é { \"abc\": 1, \"def\": [1,2], \"ghi\": { \"x\": 10} }"
            => Boolean(true),
    assoc_array_unordered_keys_equality_expr:
        "{ \"k1\": \"v1\", \"k2\": \"v2\" } é { \"k2\": \"v2\", \"k1\": \"v1\" }" => Boolean(true),
    assoc_array_mixed_elements_membership_expr: "{ \"arr\": [1,2,3], \"flag\": verdadeiro }" =>
        AssociativeArray(Rc::new(RefCell::new(
            runtime::AssociativeArray::from([
                (tenda_core::runtime::AssociativeArrayKey::String("arr".to_string()),
                List(Rc::new(RefCell::new(vec![
                    Number(1.0), Number(2.0), Number(3.0)
                ])))),
                (tenda_core::runtime::AssociativeArrayKey::String("flag".to_string()),
                Boolean(true))
            ])
        )))
);

expr_tests!(
    range_expr: "1 até 5" => Range(1, 5),
    range_equality_expr: "1 até 5 é 1 até 5" => Boolean(true),
    range_descending_expr: "5 até 1" => Range(5, 1),
    range_zero_expr: "0 até 0" => Range(0, 0),
    range_single_value_expr: "5 até 5" => Range(5, 5),
    range_normal_increasing_expr: "1 até 10" => Range(1, 10),
    range_small_descending_expr: "2 até 1" => Range(2, 1),
    range_large_descending_expr: "10 até 2" => Range(10, 2),
    range_with_arithmetic_expression_expr: "(1 + 2) até (3 + 4)" => Range(3, 7),
    range_minimal_increasing_expr: "0 até 1" => Range(0, 1),
    range_identical_single_expr: "3 até 3" => Range(3, 3),
    range_equality_true_expr: "1 até 2 é 1 até 2" => Boolean(true),
    range_inequality_due_to_shift_expr: "1 até 2 não é 2 até 3" => Boolean(true),
    range_equality_single_value_expr: "1 até 1 é 1 até 1" => Boolean(true),
    range_inequality_false_expr: "5 até 4 não é 5 até 4" => Boolean(false),
    range_length_difference_expr: "1 até 5 não é 1 até 4" => Boolean(true),
    range_expression_calculation_expr: "(2 + 3) até (2 * 3)" => Range(5, 6),
    range_chained_equality_with_boolean_true_expr: "((1 até 2) é (1 até 2)) é verdadeiro" => Boolean(true),
    range_chained_equality_failure_expr: "((1 até 1) é (1 até 2))" => Boolean(false),
    range_comparison_with_nil_false_expr: "((1 até 2) é Nada)" => Boolean(false),
    range_comparison_with_nil_true_expr: "((1 até 2) não é Nada)" => Boolean(true),
    range_nested_equality_comparison_expr: "((1 até 2) é (1 até 2)) é ((1 até 3) é (1 até 3))" => Boolean(true),
    range_singleton_expr: "1 até 1" => Range(1, 1),
    range_equality_of_same_values_expr: "(2 até 2) é (2 até 2)" => Boolean(true)
);

expr_tests_should_panic!(
    range_plus_num_error: "(1 até 5) + 1",
);
