use crate::{interpreter::*, parser::Parser, scanner::Scanner};
use runtime_error::Result;

fn run<T: ToString>(string: T) -> Result<Value> {
    let input = string.to_string();

    let mut scanner = Scanner::new(&input);

    let tokens = match scanner.scan() {
        Ok(tokens) => tokens,
        Err(err) => panic!("could not scan input: {:?}", err),
    };

    let mut parser = Parser::new(&tokens);

    let ast = match parser.parse() {
        Ok(expr) => expr,
        Err(err) => panic!("could not parse tokens: {:?}", err),
    };

    let mut interpreter: Interpreter = Interpreter::new();

    interpreter.interpret(&ast)
}

#[test]
fn division_by_zero() {
    run("0/0")
        .is_ok()
        .then(|| panic!("division by zero should error"));
}

#[test]
fn reflexive_zero() {
    assert_eq!(
        run("0").unwrap(),
        Value::Number(0.0),
        "zero evaluates to itself"
    )
}

#[test]
fn sum_of_integers() {
    assert_eq!(run("1 + 2").unwrap(), Value::Number(3.0), "sum of integers")
}

#[test]
fn precedence_of_operations() {
    assert_eq!(
        run("1 + 2 * 3").unwrap(),
        Value::Number(7.0),
        "expression depending on order of precendence of operations"
    )
}

#[test]
fn chain_of_additions() {
    assert_eq!(
        run("1 + 1 + 1 + 1").unwrap(),
        Value::Number(4.0),
        "chain of additions"
    )
}

#[test]
fn chain_of_operations() {
    assert_eq!(
        run("1 - 1 - 1 + 2 * 4 / 2 / 2").unwrap(),
        Value::Number(1.0),
        "chain of basic arithmetical operations"
    )
}

#[test]
fn negative_number() {
    assert_eq!(
        run("-1").unwrap(),
        Value::Number(-1.0),
        "negative number evaluates to itself"
    )
}

#[test]
fn negative_number_with_operation() {
    assert_eq!(
        run("-1 + -1").unwrap(),
        Value::Number(-2.0),
        "addition of negative numbers"
    )
}

#[test]
fn parentheses() {
    assert_eq!(
        run("(1 + 1)").unwrap(),
        Value::Number(2.0),
        "addition of integers within parentheses"
    )
}

#[test]
fn parentheses_with_operation() {
    assert_eq!(
        run("(1 + 1) * 2").unwrap(),
        Value::Number(4.0),
        "multiplication of integer with parentheses"
    )
}

#[test]
fn number_overflow() {
    run("10^1000")
        .is_ok()
        .then(|| panic!("overflow should error"));
}

#[test]
fn reflexive_boolean() {
    assert_eq!(
        run("verdadeiro").unwrap(),
        Value::Boolean(true),
        "`verdadeiro` evaluates to itself"
    );

    assert_eq!(
        run("falso").unwrap(),
        Value::Boolean(false),
        "`falso` evaluates to itself"
    );
}

#[test]
fn reflexive_string() {
    assert_eq!(
        run("\"Hello, world!\"").unwrap(),
        Value::String("Hello, world!".to_string()),
        "string evaluates to itself"
    )
}

#[test]
fn reflexive_nil() {
    assert_eq!(run("Nada").unwrap(), Value::Nil, "nil evaluates to itself")
}

#[test]
fn numeric_equality() {
    assert_eq!(
        run("1 for 1").unwrap(),
        Value::Boolean(true),
        "1 is equal to 1"
    )
}

#[test]
fn numeric_inequality() {
    assert_eq!(
        run("1 for 2").unwrap(),
        Value::Boolean(false),
        "1 is not equal to 2"
    )
}

#[test]
fn numeric_greater() {
    assert_eq!(
        run("1 > 2").unwrap(),
        Value::Boolean(false),
        "1 is not greater than 2"
    )
}

#[test]
fn numeric_greater_than() {
    assert_eq!(
        run("1 < 2").unwrap(),
        Value::Boolean(true),
        "1 is less than 2"
    )
}

#[test]
fn numeric_less() {
    assert_eq!(
        run("1 >= 1").unwrap(),
        Value::Boolean(true),
        "1 is greater than or equal to itself"
    )
}

#[test]
fn numeric_less_than() {
    assert_eq!(
        run("1 >= 2").unwrap(),
        Value::Boolean(false),
        "1 is not greater than or equal to 2"
    )
}

#[test]
fn concatenation() {
    assert_eq!(
        run("\"Hello, \" + \"world!\"").unwrap(),
        Value::String("Hello, world!".to_string()),
        "string concatenation"
    )
}

#[test]
fn logical_not() {
    assert_eq!(
        run("não 0 for não Nada for não verdadeiro for não não falso").unwrap(),
        Value::Boolean(true),
        "logical not"
    )
}
