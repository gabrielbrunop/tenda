use crate::token;
use crate::token::{Token, TokenKind};
use crate::value::Value;
use core::fmt;
use std::char;
use std::fmt::Display;
use std::iter::Peekable;
use std::str::Chars;

macro_rules! lexical_error {
    ($kind:expr, $line:expr) => {{
        use LexicalErrorKind::*;
        LexicalError {
            kind: $kind,
            line: $line,
        }
    }};
}

pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner<'a> {
        Scanner {
            source: source.chars().peekable(),
            line: 0,
        }
    }

    pub fn scan(&mut self) -> Result<Vec<Token>, Vec<LexicalError>> {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        let mut had_error = false;

        macro_rules! ignore_char {
            () => {
                Ok(None)
            };
        }

        while let Some(c) = self.source.next() {
            let token: Result<Option<Token>, LexicalError> = match c {
                c if c.is_whitespace() => Ok(None),
                '(' => token!(LeftParen, ")", self.line).into(),
                ')' => token!(RightParen, ")", self.line).into(),
                '+' => token!(Plus, "+", self.line).into(),
                '-' => token!(Minus, "-", self.line).into(),
                '*' => token!(Star, "*", self.line).into(),
                '^' => token!(Caret, "^", self.line).into(),
                '%' => token!(Percent, "%", self.line).into(),
                '"' => self.consume_string(c).map(Some),
                c if c.is_ascii_digit() => self.consume_number(c).map(Some),
                c if c.is_alphabetic() || c == '_' => self.consume_identifier(c).map(Some),
                '/' => match self.source.peek() {
                    Some('/') => {
                        self.consume_comment();
                        ignore_char!()
                    }
                    Some('*') => {
                        self.consume_multiline_comment();
                        ignore_char!()
                    }
                    _ => token!(Slash, "/", self.line).into(),
                },
                '\n' => {
                    self.line += 1;
                    ignore_char!()
                }
                _ => lexical_error!(UnexpectedChar(c), self.line).into(),
            };

            match token {
                Ok(Some(value)) => {
                    had_error = false;
                    tokens.push(value)
                }
                Err(err) if !had_error => {
                    had_error = true;
                    errors.push(err);
                }
                _ => (),
            };
        }

        tokens.push(Token::eoi(self.line));

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn consume_string(&mut self, char: char) -> Result<Token, LexicalError> {
        let mut string = String::new();
        string.push(char);

        while let Some(&peeked) = self.source.peek() {
            match peeked {
                '"' => {
                    self.source.next();
                    break;
                }
                '\n' => {
                    return lexical_error!(UnexpectedStringEol, self.line).into();
                }
                _ => {
                    string.push(peeked);
                    self.source.next();
                }
            }
        }

        let string = string[1..].to_string();
        let token = token!(String, &string, self.line, Value::String(string));

        Ok(token)
    }

    fn consume_number(&mut self, char: char) -> Result<Token, LexicalError> {
        let mut number = String::new();
        let mut matched_dot = false;

        number.push(char);

        while let Some(&peeked) = self.source.peek() {
            let is_unexpected = |c: char| c == '.' && matched_dot || c.is_alphabetic();

            match peeked {
                c if is_unexpected(c) => {
                    return lexical_error!(UnexpectedChar(c), self.line).into();
                }
                c if c.is_numeric() || c == '.' => {
                    if c == '.' {
                        matched_dot = true;
                    }

                    number.push(peeked);
                    self.source.next();
                }
                _ => break,
            }
        }

        let illegal_leading_zero =
            number.starts_with('0') && !number.starts_with("0.") && number != "0";

        if illegal_leading_zero {
            return lexical_error!(LeadingZeroNumberLiterals, self.line).into();
        }

        let number: f64 = number.parse().unwrap();
        let token = token!(Number, &number, self.line, Value::Number(number));

        Ok(token)
    }

    fn consume_identifier(&mut self, char: char) -> Result<Token, LexicalError> {
        let mut identifier = String::new();

        identifier.push(char);

        while let Some(&peeked) = self.source.peek() {
            if peeked.is_alphanumeric() || peeked == '_' {
                identifier.push(peeked);
                self.source.next();
            } else {
                break;
            }
        }

        let token = match identifier.as_str() {
            Value::TRUE_LITERAL => {
                token!(True, Value::TRUE_LITERAL, self.line, Value::Boolean(true))
            }
            Value::FALSE_LITERAL => token!(
                False,
                Value::FALSE_LITERAL,
                self.line,
                Value::Boolean(false)
            ),
            Value::NIL_LITERAL => token!(Nil, Value::NIL_LITERAL, self.line, Value::Nil),
            "for" => token!(Equals, "for", self.line),
            _ => return lexical_error!(UnexpectedChar(char), self.line).into(),
        };

        Ok(token)
    }

    fn consume_comment(&mut self) {
        while let Some(&peeked) = self.source.peek() {
            if peeked == '\n' {
                break;
            }

            self.source.next();
        }
    }

    fn consume_multiline_comment(&mut self) {
        while let Some(_) = self.source.next() {
            if self.peek_match("*/") {
                break;
            }
        }
    }

    fn peek_match(&mut self, expected: &str) -> bool {
        for c in expected.chars() {
            if let Some(&peeked) = self.source.peek() {
                if peeked != c {
                    return false;
                }

                self.source.next();
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct LexicalError {
    pub line: usize,
    pub kind: LexicalErrorKind,
}

impl LexicalError {
    pub fn message(&self) -> String {
        use LexicalErrorKind::*;

        match self.kind {
            LeadingZeroNumberLiterals => {
                "leading zeros in number literals are not permitted".to_string()
            }
            UnexpectedChar(c) => format!("unexpected character: {}", c),
            UnexpectedStringEol => "unexpected end of line in single-line string".to_string(),
        }
    }
}

impl<T> From<LexicalError> for Result<T, LexicalError> {
    fn from(val: LexicalError) -> Self {
        Err(val)
    }
}

impl Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at line {})", self.message(), self.line)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LexicalErrorKind {
    LeadingZeroNumberLiterals,
    UnexpectedStringEol,
    UnexpectedChar(char),
}

#[cfg(test)]
mod tests {
    use crate::token_list;

    use super::*;

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
                .eq(token_list![Number, Plus, Number, Eof]),
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
                .eq(token_list![Number, Eof]),
            "legal leading zero"
        )
    }

    #[test]
    fn boolean_lexemes() {
        assert!(
            scan_to_token_list("verdadeiro")
                .unwrap()
                .iter()
                .eq(token_list![True, Eof]),
            "`verdadeiro` is a lexeme"
        );

        assert!(
            scan_to_token_list("falso")
                .unwrap()
                .iter()
                .eq(token_list![False, Eof]),
            "`falso` is a lexeme"
        );
    }

    #[test]
    fn string_literals() {
        assert!(
            scan_to_token_list("\"Hello, world!\"")
                .unwrap()
                .iter()
                .eq(token_list![String, Eof]),
            "\"Hello, world!\" is a string literal lexeme"
        )
    }

    #[test]
    fn nil_literal() {
        assert!(
            scan_to_token_list("Nada")
                .unwrap()
                .iter()
                .eq(token_list![Nil, Eof]),
            "Nada is a lexeme"
        )
    }

    #[test]
    fn identifier_equals() {
        assert!(
            scan_to_token_list("for")
                .unwrap()
                .iter()
                .eq(token_list![Equals, Eof]),
            "`for` is a identifier"
        )
    }
}
