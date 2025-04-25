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
