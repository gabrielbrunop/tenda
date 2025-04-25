use tenda_common::source::IdentifiedSource;

use crate::scanner_error::LexicalError;
use crate::source_iter::SourceIter;
use crate::token::{Literal, Token, TokenKind};
use std::char;

pub struct Scanner<'a> {
    source: SourceIter<'a>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str, source_id: IdentifiedSource) -> Scanner<'a> {
        Scanner {
            source: SourceIter::new(source, source_id),
        }
    }

    pub fn scan(&mut self) -> Result<Vec<Token>, Vec<LexicalError>> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors = Vec::new();
        let mut had_error = false;

        while let Some(c) = self.source.next() {
            let token = self.consume_token(c, tokens.last());

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

        tokens.push(self.source.consume_eof());

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn consume_token(
        &mut self,
        char: char,
        previous_token: Option<&Token>,
    ) -> Result<Option<Token>, LexicalError> {
        match char {
            '\n' => match previous_token {
                Some(token) if token.kind != TokenKind::Newline => {
                    self.source.consume_token(TokenKind::Newline, "\n").into()
                }
                _ => {
                    self.source.ignore_char();
                    Ok(None)
                }
            },
            c if c.is_whitespace() => {
                self.source.ignore_char();
                Ok(None)
            }
            '(' => self.source.consume_token(TokenKind::LeftParen, "(").into(),
            ')' => self.source.consume_token(TokenKind::RightParen, ")").into(),
            '[' => self
                .source
                .consume_token(TokenKind::LeftBracket, "[")
                .into(),
            ']' => self
                .source
                .consume_token(TokenKind::RightBracket, "]")
                .into(),
            '{' => self.source.consume_token(TokenKind::LeftBrace, "{").into(),
            '}' => self.source.consume_token(TokenKind::RightBrace, "}").into(),
            ':' => self.source.consume_token(TokenKind::Colon, ":").into(),
            '+' => self.source.consume_token(TokenKind::Plus, "+").into(),
            '-' => self.source.consume_token(TokenKind::Minus, "-").into(),
            '*' => self.source.consume_token(TokenKind::Star, "*").into(),
            '^' => self.source.consume_token(TokenKind::Caret, "^").into(),
            '%' => self.source.consume_token(TokenKind::Percent, "%").into(),
            '=' => self.source.consume_token(TokenKind::EqualSign, "=").into(),
            '"' => self.consume_string(char).map(Some),
            ',' => self.source.consume_token(TokenKind::Comma, ",").into(),
            '.' => self.source.consume_token(TokenKind::Dot, ".").into(),
            '>' => match self.source.peek() {
                Some('=') => {
                    self.source.next();
                    self.source
                        .consume_token(TokenKind::GreaterOrEqual, ">")
                        .into()
                }
                _ => self.source.consume_token(TokenKind::Greater, ">").into(),
            },
            '<' => match self.source.peek() {
                Some('=') => {
                    self.source.next();
                    self.source
                        .consume_token(TokenKind::LessOrEqual, "<")
                        .into()
                }
                _ => self.source.consume_token(TokenKind::Less, "<").into(),
            },
            c if c.is_ascii_digit() => self.consume_number(c).map(Some),
            c if c.is_alphabetic() || c == '_' => self.consume_identifier(c).map(Some),
            '/' => match self.source.peek() {
                Some('/') => {
                    self.consume_comment();
                    Ok(None)
                }
                Some('*') => {
                    self.consume_multiline_comment();
                    Ok(None)
                }
                _ => self.source.consume_token(TokenKind::Slash, "/").into(),
            },
            _ => Err(LexicalError::UnexpectedChar {
                character: char,
                span: self.source.consume_span(),
            }),
        }
    }

    fn consume_string(&mut self, first_quote: char) -> Result<Token, LexicalError> {
        let mut buf = String::new();
        let mut closed = false;

        buf.push(first_quote);

        while let Some(&ch) = self.source.peek() {
            match ch {
                '"' => {
                    self.source.next();
                    closed = true;
                    break;
                }
                '\n' => {
                    return Err(LexicalError::UnexpectedStringEol {
                        span: self.source.consume_span(),
                    });
                }
                '\\' => {
                    self.source.next();

                    let esc = self
                        .source
                        .next()
                        .ok_or(LexicalError::UnexpectedStringEol {
                            span: self.source.consume_span(),
                        })?;

                    let resolved = match esc {
                        '0' => Some('\0'),
                        'a' => Some('\x07'),
                        'b' => Some('\x08'),
                        'e' => Some('\x1B'),
                        'f' => Some('\x0C'),
                        'n' => Some('\n'),
                        'r' => Some('\r'),
                        't' => Some('\t'),
                        'v' => Some('\x0B'),
                        '\\' => Some('\\'),
                        '\'' => Some('\''),
                        '"' => Some('"'),
                        'x' => {
                            let hi = self.read_hex_digit()?;
                            let lo = self.read_hex_digit()?;
                            Some(char::from(
                                u8::from_str_radix(&format!("{hi}{lo}"), 16).unwrap(),
                            ))
                        }
                        'u' => {
                            let code = self.read_n_hex(4)?;
                            char::from_u32(code)
                        }
                        'U' => {
                            let code = self.read_n_hex(8)?;
                            char::from_u32(code)
                        }
                        d @ '1'..='7' => {
                            let d2 = self.read_octal_digit()?;
                            let d3 = self.read_octal_digit()?;
                            let val = u8::from_str_radix(&format!("{d}{d2}{d3}"), 8).unwrap();
                            Some(char::from(val))
                        }
                        _ => {
                            return Err(LexicalError::UnknownEscape {
                                span: self.source.consume_span(),
                                found: esc,
                            })
                        }
                    };

                    if let Some(c) = resolved {
                        buf.push(c);
                    } else {
                        return Err(LexicalError::InvalidUnicodeEscape {
                            span: self.source.consume_span(),
                        });
                    }
                }
                _ => {
                    buf.push(ch);
                    self.source.next();
                }
            }
        }

        if !closed {
            return Err(LexicalError::UnexpectedStringEol {
                span: self.source.consume_span(),
            });
        }

        let literal = buf[1..].to_owned();

        Ok(self.source.consume_token_with_literal(
            TokenKind::String,
            literal.clone(),
            Literal::String(literal),
        ))
    }

    fn consume_number(&mut self, char: char) -> Result<Token, LexicalError> {
        let mut number = String::new();
        let mut matched_dot = false;

        number.push(char);

        while let Some(&peeked) = self.source.peek() {
            let is_unexpected = |c: char| c == '.' && matched_dot || c.is_alphabetic();

            match peeked {
                c if is_unexpected(c) => {
                    return Err(LexicalError::UnexpectedChar {
                        character: c,
                        span: self.source.consume_span(),
                    });
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
            return Err(LexicalError::LeadingZeroNumberLiterals {
                span: self.source.consume_span(),
            });
        }

        let number: f64 = number.parse().unwrap();

        let token = self.source.consume_token_with_literal(
            TokenKind::Number,
            number.to_string(),
            Literal::Number(number),
        );

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
            Literal::TRUE_LITERAL => self.source.consume_token_with_literal(
                TokenKind::True,
                Literal::TRUE_LITERAL.to_string(),
                Literal::Boolean(true),
            ),
            Literal::FALSE_LITERAL => self.source.consume_token_with_literal(
                TokenKind::False,
                Literal::FALSE_LITERAL.to_string(),
                Literal::Boolean(false),
            ),
            Literal::NIL_LITERAL => self.source.consume_token_with_literal(
                TokenKind::Nil,
                Literal::NIL_LITERAL.to_string(),
                Literal::Nil,
            ),
            "função" => self.source.consume_token(TokenKind::Function, "função"),
            "não" => self.source.consume_token(TokenKind::Not, "não"),
            "é" => self.source.consume_token(TokenKind::Equals, "é"),
            "seja" => self.source.consume_token(TokenKind::Let, "seja"),
            "se" => self.source.consume_token(TokenKind::If, "se"),
            "então" => self.source.consume_token(TokenKind::Then, "então"),
            "retorna" => self.source.consume_token(TokenKind::Return, "retorna"),
            "senão" => self.source.consume_token(TokenKind::Else, "senão"),
            "fim" => self.source.consume_token(TokenKind::BlockEnd, "fim"),
            "ou" => self.source.consume_token(TokenKind::Or, "ou"),
            "e" => self.source.consume_token(TokenKind::And, "e"),
            "até" => self.source.consume_token(TokenKind::Until, "até"),
            "para" => self.source.consume_token(TokenKind::ForOrBreak, "para"),
            "cada" => self.source.consume_token(TokenKind::Each, "cada"),
            "em" => self.source.consume_token(TokenKind::In, "em"),
            "tem" => self.source.consume_token(TokenKind::Has, "tem"),
            "enquanto" => self.source.consume_token(TokenKind::While, "enquanto"),
            "faça" => self.source.consume_token(TokenKind::Do, "faça"),
            "continua" => self.source.consume_token(TokenKind::Continue, "continua"),
            identifier => self.source.consume_token_with_literal(
                TokenKind::Identifier,
                identifier.to_string(),
                Literal::String(identifier.to_string()),
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

        self.source.ignore_char();
    }

    fn consume_multiline_comment(&mut self) {
        while let Some(_) = self.source.next() {
            if self.peek_match("*/") {
                break;
            }
        }

        self.source.ignore_char();
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

    fn read_hex_digit(&mut self) -> Result<char, LexicalError> {
        self.source
            .next()
            .filter(|c| c.is_ascii_hexdigit())
            .ok_or(LexicalError::InvalidHexEscape {
                span: self.source.consume_span(),
            })
    }

    fn read_n_hex(&mut self, n: usize) -> Result<u32, LexicalError> {
        let mut s = String::new();
        for _ in 0..n {
            s.push(self.read_hex_digit()?);
        }
        u32::from_str_radix(&s, 16).map_err(|_| LexicalError::InvalidHexEscape {
            span: self.source.consume_span(),
        })
    }

    fn read_octal_digit(&mut self) -> Result<char, LexicalError> {
        self.source
            .next()
            .filter(|c| ('0'..='7').contains(c))
            .ok_or(LexicalError::InvalidOctalEscape {
                span: self.source.consume_span(),
            })
    }
}
