use crate::lexical_error;
use crate::scanner_error::{LexicalError, LexicalErrorKind};
use crate::token::{token, Literal, Token, TokenKind};
use std::char;
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
                            token!(Newline, "\n", self.line - 1).into()
                        }
                        _ => ignore_char!(),
                    }
                }
                c if c.is_whitespace() => Ok(None),
                '(' => token!(LeftParen, "(", self.line).into(),
                ')' => token!(RightParen, ")", self.line).into(),
                '[' => token!(LeftBracket, "[", self.line).into(),
                ']' => token!(RightBracket, "]", self.line).into(),
                '{' => token!(LeftBrace, "{", self.line).into(),
                '}' => token!(RightBrace, "}", self.line).into(),
                ':' => token!(Colon, ":", self.line).into(),
                '+' => token!(Plus, "+", self.line).into(),
                '-' => token!(Minus, "-", self.line).into(),
                '*' => token!(Star, "*", self.line).into(),
                '^' => token!(Caret, "^", self.line).into(),
                '%' => token!(Percent, "%", self.line).into(),
                '=' => token!(EqualSign, "=", self.line).into(),
                '"' => self.consume_string(c).map(Some),
                ',' => token!(Comma, ",", self.line).into(),
                '.' => token!(Dot, ".", self.line).into(),
                '>' => match self.source.peek() {
                    Some('=') => {
                        self.source.next();
                        token!(GreaterOrEqual, ">", self.line).into()
                    }
                    _ => token!(Greater, ">", self.line).into(),
                },
                '<' => match self.source.peek() {
                    Some('=') => {
                        self.source.next();
                        token!(LessOrEqual, ">", self.line).into()
                    }
                    _ => token!(Less, ">", self.line).into(),
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
                    _ => token!(Slash, "/", self.line).into(),
                },
                _ => Err(lexical_error!(UnexpectedChar(c), self.line)),
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
                    return Err(lexical_error!(UnexpectedStringEol, self.line));
                }
                _ => {
                    string.push(peeked);
                    self.source.next();
                }
            }
        }

        let string = string[1..].to_string();
        let token = token!(String, &string, self.line, Literal::String(string));

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
                    return Err(lexical_error!(UnexpectedChar(c), self.line));
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
            return Err(lexical_error!(LeadingZeroNumberLiterals, self.line));
        }

        let number: f64 = number.parse().unwrap();
        let token = token!(Number, &number, self.line, Literal::Number(number));

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
                token!(
                    True,
                    Literal::TRUE_LITERAL,
                    self.line,
                    Literal::Boolean(true)
                )
            }
            Literal::FALSE_LITERAL => token!(
                False,
                Literal::FALSE_LITERAL,
                self.line,
                Literal::Boolean(false)
            ),
            Literal::NIL_LITERAL => token!(Nil, Literal::NIL_LITERAL, self.line, Literal::Nil),
            "função" => token!(Function, "função", self.line),
            "não" => token!(Not, "não", self.line),
            "é" => token!(Equals, "é", self.line),
            "seja" => token!(Let, "seja", self.line),
            "se" => token!(If, "se", self.line),
            "então" => token!(Then, "então", self.line),
            "retorna" => token!(Return, "retorna", self.line),
            "senão" => token!(Else, "senão", self.line),
            "fim" => token!(BlockEnd, "fim", self.line),
            "ou" => token!(Or, "ou", self.line),
            "e" => token!(And, "e", self.line),
            "até" => token!(Until, "até", self.line),
            "para" => token!(For, "para", self.line),
            "cada" => token!(Each, "cada", self.line),
            "em" => token!(In, "em", self.line),
            "tem" => token!(Has, "tem", self.line),
            "enquanto" => token!(While, "enquanto", self.line),
            "faça" => token!(Do, "faça", self.line),
            "pare" => token!(Break, "pare", self.line),
            "continue" => token!(Continue, "continue", self.line),
            identifier => token!(
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
}

impl Scanner<'_> {
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
