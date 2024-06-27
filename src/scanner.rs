use crate::token;
use crate::token::{Token, TokenKind};
use crate::value::Value;
use core::fmt;
use std::char;
use std::fmt::Display;
use std::iter::Peekable;
use std::str::Chars;

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

        while let Some(c) = self.source.next() {
            let token: Result<Option<Token>, LexicalError> = match c {
                c if c.is_whitespace() => Ok(None),
                '(' => Ok(Some(token!(LeftParen, ")", self.line))),
                ')' => Ok(Some(token!(RightParen, ")", self.line))),
                '+' => Ok(Some(token!(Plus, "+", self.line))),
                '-' => Ok(Some(token!(Minus, "-", self.line))),
                '*' => Ok(Some(token!(Star, "*", self.line))),
                '^' => Ok(Some(token!(Caret, "^", self.line))),
                '%' => Ok(Some(token!(Percent, "%", self.line))),
                c if c.is_ascii_digit() => match self.consume_number(c) {
                    Ok(token) => Ok(Some(token)),
                    Err(err) => Err(err),
                },
                '/' => match self.source.peek() {
                    Some('/') => {
                        self.consume_comment();
                        Ok(None)
                    }
                    Some('*') => {
                        self.consume_multiline_comment();
                        Ok(None)
                    }
                    _ => Ok(Some(token!(Slash, "/", self.line))),
                },
                '\n' => {
                    self.line += 1;
                    Ok(None)
                }
                _ => Err(LexicalError {
                    line: 0,
                    message: format!("unexpected character: {}", c),
                }),
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

        let eof_token = Token::new(TokenKind::Eof, "".to_string(), None, self.line);

        tokens.push(eof_token);

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
            match peeked {
                c if c == '.' && matched_dot => {
                    return LexicalError {
                        line: self.line,
                        message: "unexpected character: '.'".to_string(),
                    }
                    .into();
                }
                c if c.is_numeric() || c == '.' => {
                    if c == '.' {
                        matched_dot = true;
                    }

                    number.push(peeked);
                    self.source.next();
                }
                c if c.is_alphabetic() => {
                    return LexicalError {
                        line: self.line,
                        message: format!("unexpected character: {}", c),
                    }
                    .into();
                }
                _ => break,
            }
        }

        let illegal_leading_zero = number.starts_with('0') && !number.starts_with("0.");

        if illegal_leading_zero {
            return Err(LexicalError {
                message: "leading zeros in number literals are not permitted".to_string(),
                line: self.line,
            });
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
    pub message: String,
}

impl<T> From<LexicalError> for Result<T, LexicalError> {
    fn from(val: LexicalError) -> Self {
        Err(val)
    }
}

impl Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at line {})", self.message, self.line)
    }
}
