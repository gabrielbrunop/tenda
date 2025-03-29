use common::{source::IdentifiedSource, span::SourceSpan};
use std::iter::Peekable;
use std::str::Chars;

use crate::token::{Literal, Token, TokenKind};

pub struct SourceIter<'a> {
    iter: Peekable<Chars<'a>>,
    source_id: IdentifiedSource,
    start_position: usize,
    end_position: usize,
}

impl<'a> SourceIter<'a> {
    pub fn new(input: &'a str, source_id: IdentifiedSource) -> Self {
        Self {
            iter: input.chars().peekable(),
            source_id,
            start_position: 0,
            end_position: 0,
        }
    }

    pub fn consume_token(&mut self, kind: TokenKind, lexeme: &str) -> Token {
        Token::new(kind, lexeme.to_string(), None, self.consume_span())
    }

    pub fn consume_token_with_literal(
        &mut self,
        kind: TokenKind,
        lexeme: String,
        literal: Literal,
    ) -> Token {
        Token::new(kind, lexeme, Some(literal), self.consume_span())
    }

    pub fn consume_eof(&mut self) -> Token {
        Token::eoi(self.consume_span())
    }

    pub fn consume_span(&mut self) -> SourceSpan {
        let span = SourceSpan::new(self.start_position, self.end_position, self.source_id);

        self.start_position = self.end_position;

        span
    }

    pub fn ignore_char(&mut self) {
        self.start_position = self.end_position;
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }
}

impl Iterator for SourceIter<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.iter.next() {
            self.end_position += 1;

            Some(c)
        } else {
            None
        }
    }
}
