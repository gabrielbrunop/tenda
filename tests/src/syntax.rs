use tenda_core::{platform::OSPlatform, runtime::Platform};

use crate::{expr_tests, expr_tests_should_panic};

expr_tests_should_panic!(
    parse_error_unclosed_paren: "(1 + 2",
    parse_error_unclosed_bracket: "[1, 2",
    parse_error_unclosed_brace: "{ 1 : 2",
    parse_error_missing_colon_in_assoc: "{ 1 2 }",
    parse_error_invalid_operator: "1 $ 2",
);

expr_tests!(
    escape_null_literal: "\"\\0\"" => String("\0".to_string()),
    escape_bell_literal: "\"Sino:\\a\"" => String("Sino:\x07".to_string()),
    escape_backspace_literal: "\"BS:\\b\"" => String("BS:\x08".to_string()),
    escape_formfeed_literal: "\"FF:\\f\"" => String("FF:\x0C".to_string()),
    escape_vertical_tab_literal: "\"VT:\\v\"" => String("VT:\x0B".to_string()),
    escape_escape_literal: "\"ESC:\\e\"" => String("ESC:\x1B".to_string()),
    escape_hex_uppercase_a: "\"hex:\\x41\"" => String("hex:A".to_string()),
    escape_unicode_16bit_a: "\"uni16:\\u0041\"" => String("uni16:A".to_string()),
    escape_unicode_32bit_a: "\"uni32:\\U00000041\"" => String("uni32:A".to_string()),
    escape_octal_literal_a: "\"oct:\\101\"" => String("oct:A".to_string()),
    escape_all_combined: "\"\\0\\a\\b\\e\\f\\n\\r\\t\\v\\\\\\\"\"" =>
        String("\0\x07\x08\x1B\x0C\n\r\t\x0B\\\"".to_string())
);

expr_tests_should_panic!(
    escape_invalid_hex_digit: "\"\\xG1\"",
    escape_incomplete_hex_escape: "\"\\x1\"",
    escape_invalid_unicode_16: "\"\\uZZZZ\"",
    escape_unicode32_out_of_range: "\"\\U0FFFFFFF\"",
    escape_octal_value_too_large: "\"\\400\"",
    escape_octal_too_short: "\"\\12\"",
    escape_unknown_escape: "\"\\q\""
);

expr_tests!(
    number_zero: "0" => Number(0.0),
    number_plain: "123" => Number(123.0),
    number_underscores: "1_000_000" => Number(1_000_000.0),
    number_leading_zero: "0123" => Number(123.0),
    number_bin: "0b1010" => Number(10.0),
    number_bin_underscores: "0b1010_0101" => Number(0b1010_0101 as f64),
    number_bin_uppercase: "0B1101" => Number(13.0),
    number_oct: "0o755" => Number(0o755 as f64),
    number_oct_uppercase: "0O644" => Number(0o644 as f64),
    number_hex: "0xdead_beef" => Number(0xDEAD_BEEFu32 as f64),
    number_hex_uppercase: "0XCAFE" => Number(0xCAFE as f64),
    number_float: "0.123" => Number(0.123),
    number_float_trailing: "1." => Number(1.0),
    number_exp: "1e3" => Number(1e3),
    number_uppercase_exp: "1E3" => Number(1e3),
    number_exp_signed: "2.5e-2" => Number(2.5e-2),
    number_exp_plus: "2.5E+2" => Number(2.5e+2),
    number_exp_underscores: "1_2e3_0" => Number(12e30),
);

expr_tests_should_panic!(
    number_bad_bin_digit: "0b102",
    number_bad_oct_digit: "0o9",
    number_bad_hex_digit: "0xG1",
    number_missing_bin_digits: "0b",
    number_missing_oct_digits: "0o",
    number_missing_hex_digits: "0x",
    number_multi_dot: "1.2.3",
    number_multi_exp: "1e2e3",
    number_missing_exp_digits: "1e",
    number_invalid_suffix: "123abc",
);
