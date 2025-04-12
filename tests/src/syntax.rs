use rstest::rstest;
use tenda_core::{platform::OSPlatform, runtime::Platform};

use crate::interpret_expr;

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
