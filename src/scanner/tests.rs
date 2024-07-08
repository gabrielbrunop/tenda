use crate::{scanner::*, token_iter};

fn scan<T: ToString>(string: T) -> Result<Vec<Token>, Vec<LexicalError>> {
    let input = string.to_string();

    let mut scanner = Scanner::new(&input);

    scanner.scan()
}

fn scan_to_token_list<T: ToString>(string: T) -> Result<Vec<TokenKind>, Vec<LexicalError>> {
    let result = scan(string)?
        .into_iter()
        .map(|t| t.kind)
        .collect::<Vec<TokenKind>>();

    Ok(result)
}

#[test]
fn extra_spacing() {
    assert!(
        scan_to_token_list(" 1 + 2 ")
            .unwrap()
            .iter()
            .eq(token_iter![Number, Plus, Number, Eof]),
        "sum of integers with additional spacing between characters"
    )
}

#[test]
fn illegal_leading_zero() {
    assert!(scan_to_token_list("01").is_err(), "illegal leading zero")
}

#[test]
fn legal_leading_zero() {
    assert!(
        scan_to_token_list("0.1")
            .unwrap()
            .iter()
            .eq(token_iter![Number, Eof]),
        "legal leading zero"
    )
}

#[test]
fn boolean_lexemes() {
    assert!(
        scan_to_token_list("verdadeiro")
            .unwrap()
            .iter()
            .eq(token_iter![True, Eof]),
        "`verdadeiro` is a lexeme"
    );

    assert!(
        scan_to_token_list("falso")
            .unwrap()
            .iter()
            .eq(token_iter![False, Eof]),
        "`falso` is a lexeme"
    );
}

#[test]
fn string_literals() {
    assert!(
        scan_to_token_list("\"Hello, world!\"")
            .unwrap()
            .iter()
            .eq(token_iter![String, Eof]),
        "\"Hello, world!\" is a string literal lexeme"
    )
}

#[test]
fn nil_literal() {
    assert!(
        scan_to_token_list("Nada")
            .unwrap()
            .iter()
            .eq(token_iter![Nil, Eof]),
        "Nada is a lexeme"
    )
}

#[test]
fn identifier_equals() {
    assert!(
        scan_to_token_list("for")
            .unwrap()
            .iter()
            .eq(token_iter![Equals, Eof]),
        "`for` is a identifier"
    )
}

#[test]
fn identifier_not() {
    assert!(
        scan_to_token_list("não")
            .unwrap()
            .iter()
            .eq(token_iter![Not, Eof]),
        "`não` is a identifier"
    )
}
