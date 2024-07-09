use crate::{parser::*, scanner::Scanner};

fn parse<T: ToString>(string: T) -> Result<Vec<Stmt>, Vec<ParserError>> {
    let input = string.to_string();

    let mut scanner = Scanner::new(&input);

    let tokens = match scanner.scan() {
        Ok(tokens) => tokens,
        Err(err) => panic!("could not scan input: {:?}", err),
    };

    let mut parser = Parser::new(&tokens);

    parser.parse()
}

#[test]
fn inequality() {
    assert_eq!(
        *parse("1 não for 1").unwrap().first().unwrap(),
        Stmt::Expr(Expr::make_binary(
            Expr::make_literal(Literal::Number(1.0)),
            BinaryOp::Inequality,
            Expr::make_literal(Literal::Number(1.0))
        )),
        "parse inequality"
    )
}

#[test]
fn multiple_primaries_unexpected_token() {
    assert!(matches!(
        parse("1 2 3").unwrap_err().first().unwrap().source,
        ParserErrorKind::UnexpectedToken(_)
    ))
}

#[test]
fn binary_op_sum_eoi() {
    assert!(matches!(
        parse("1 +").unwrap_err().first().unwrap().source,
        ParserErrorKind::UnexpectedEoi
    ))
}

#[test]
fn binary_op_sum_missing_parentheses() {
    assert!(matches!(
        parse("(1 + 1").unwrap_err().first().unwrap().source,
        ParserErrorKind::MissingParentheses
    ))
}
