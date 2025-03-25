use std::iter::Peekable;
use std::str::Chars;

use crate::{
    scanner_error::{LexicalError, LexicalErrorKind},
    token::{Literal, Token, TokenKind, TokenSpan},
};

pub struct SourceIter<'a> {
    iter: Peekable<Chars<'a>>,
    start_position: usize,
    end_position: usize,
    line: usize,
    col: usize,
}

impl<'a> SourceIter<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            iter: input.chars().peekable(),
            start_position: 0,
            end_position: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn consume_token(&mut self, kind: TokenKind, lexeme: &str) -> Token {
        let span = TokenSpan::new(self.start_position, self.end_position, self.line, self.col);

        self.start_position = self.end_position;

        Token::new(kind, lexeme.to_string(), None, span)
    }

    pub fn consume_token_with_literal(
        &mut self,
        kind: TokenKind,
        lexeme: String,
        literal: Literal,
    ) -> Token {
        let span = TokenSpan::new(self.start_position, self.end_position, self.line, self.col);

        self.start_position = self.end_position;

        Token::new(kind, lexeme, Some(literal), span)
    }

    pub fn eof(&self) -> Token {
        let span = TokenSpan::new(self.start_position, self.end_position, self.line, self.col);

        Token::eoi(span)
    }

    pub fn lexical_error(&mut self, kind: LexicalErrorKind) -> LexicalError {
        let span = TokenSpan::new(self.start_position, self.end_position, self.line, self.col);

        self.start_position = self.end_position;

        LexicalError { span, source: kind }
    }

    pub fn ignore_char(&mut self) {
        self.start_position = self.end_position;
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }
}

impl<'a> Iterator for SourceIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.iter.next() {
            self.end_position += c.len_utf8();

            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }

            Some(c)
        } else {
            None
        }
    }
}
