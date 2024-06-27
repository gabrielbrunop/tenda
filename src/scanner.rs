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
                c if c.is_ascii_digit() => self.consume_number(c).map(Some),
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

#[derive(Debug)]
pub enum LexicalErrorKind {
    LeadingZeroNumberLiterals,
    UnexpectedChar(char),
}
