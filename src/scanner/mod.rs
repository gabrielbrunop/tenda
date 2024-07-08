use token::{Literal, Token, TokenKind};

use crate::token as t;
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
            line: 1,
        }
    }

    pub fn scan(&mut self) -> Result<Vec<Token>, Vec<LexicalError>> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors = Vec::new();
        let mut had_error = false;

        macro_rules! ignore_char {
            () => {
                Ok(None)
            };
        }

        while let Some(c) = self.source.next() {
            let token: Result<Option<Token>, LexicalError> = match c {
                '\n' => {
                    self.line += 1;
                    match tokens.last() {
                        Some(token) if token.kind != TokenKind::Newline => {
                            t!(Newline, "\n", self.line - 1).into()
                        }
                        _ => ignore_char!(),
                    }
                }
                c if c.is_whitespace() => Ok(None),
                '(' => t!(LeftParen, ")", self.line).into(),
                ')' => t!(RightParen, ")", self.line).into(),
                '+' => t!(Plus, "+", self.line).into(),
                '-' => t!(Minus, "-", self.line).into(),
                '*' => t!(Star, "*", self.line).into(),
                '^' => t!(Caret, "^", self.line).into(),
                '%' => t!(Percent, "%", self.line).into(),
                '=' => t!(EqualSign, "=", self.line).into(),
                '"' => self.consume_string(c).map(Some),
                ',' => t!(Comma, ",", self.line).into(),
                '>' => match self.source.peek() {
                    Some('=') => {
                        self.source.next();
                        t!(GreaterOrEqual, ">", self.line).into()
                    }
                    _ => t!(Greater, ">", self.line).into(),
                },
                '<' => match self.source.peek() {
                    Some('=') => {
                        self.source.next();
                        t!(LessOrEqual, ">", self.line).into()
                    }
                    _ => t!(Less, ">", self.line).into(),
                },
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
                    _ => t!(Slash, "/", self.line).into(),
                },
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
        let token = t!(String, &string, self.line, Literal::String(string));

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
        let token = t!(Number, &number, self.line, Literal::Number(number));

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
            Literal::TRUE_LITERAL => {
                t!(
                    True,
                    Literal::TRUE_LITERAL,
                    self.line,
                    Literal::Boolean(true)
                )
            }
            Literal::FALSE_LITERAL => t!(
                False,
                Literal::FALSE_LITERAL,
                self.line,
                Literal::Boolean(false)
            ),
            Literal::NIL_LITERAL => t!(Nil, Literal::NIL_LITERAL, self.line, Literal::Nil),
            "função" => t!(Function, "função", self.line),
            "não" => t!(Not, "não", self.line),
            "for" => t!(Equals, "for", self.line),
            "seja" => t!(Let, "seja", self.line),
            "se" => t!(If, "se", self.line),
            "então" => t!(Then, "então", self.line),
            "retorne" => t!(Return, "retorne", self.line),
            "senão" => t!(Else, "senão", self.line),
            "fim" => t!(BlockEnd, "fim", self.line),
            "ou" => t!(Or, "ou", self.line),
            "e" => t!(And, "e", self.line),
            identifier => t!(
                Identifier,
                identifier,
                self.line,
                Literal::String(identifier.to_string())
            ),
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
                "zeros à esquerda em literais numéricos não são permitidos".to_string()
            }
            UnexpectedChar(c) => format!("caractere inesperado: {}", c),
            UnexpectedStringEol => "fim de linha inesperado em texto".to_string(),
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
        write!(f, "{} (na linha {})", self.message(), self.line)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LexicalErrorKind {
    LeadingZeroNumberLiterals,
    UnexpectedStringEol,
    UnexpectedChar(char),
}

#[cfg(test)]
mod tests;
pub mod token;
