use peekmore::{PeekMore, PeekMoreIterator};
use std::{cell::RefCell, ops::Neg, rc::Rc, slice::Iter};
use tenda_scanner::{Token, TokenKind};

#[derive(Debug)]
pub struct TokenIterator<'a> {
    tokens: PeekMoreIterator<Iter<'a, Token>>,
    ignoring_newline_counter: Rc<RefCell<isize>>,
    last_token: &'a Token,
}

impl TokenIterator<'_> {
    pub fn peek(&mut self) -> Option<&&Token> {
        self.skip_ignored_newlines();
        self.tokens.peek()
    }

    pub fn set_ignoring_newline(&mut self) -> NewlineGuard {
        NewlineGuard::new(self.ignoring_newline_counter.clone(), None)
    }

    pub fn halt_ignoring_newline(&mut self) -> NewlineGuard {
        let size = self.ignoring_newline_counter.borrow().neg();

        NewlineGuard::new(self.ignoring_newline_counter.clone(), Some(size))
    }

    pub fn consume_one_of(&mut self, token_types: Iter<TokenKind>) -> Option<Token> {
        self.tokens.reset_cursor();

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::Newline && *self.ignoring_newline_counter.borrow() > 0 {
                self.tokens.advance_cursor();
            } else {
                break;
            }
        }

        let skipped = self.tokens.cursor();
        let expected: Vec<TokenKind> = token_types.cloned().collect();

        if let Some(token) = self.tokens.peek() {
            if expected.iter().any(|t| *t == token.kind) {
                for _ in 0..skipped {
                    self.tokens.next();
                }

                return self.tokens.next().cloned();
            }
        }

        self.tokens.reset_cursor();

        None
    }

    pub fn consume_sequence(&mut self, token_types: Iter<TokenKind>) -> Option<Vec<Token>> {
        self.tokens.reset_cursor();

        for token_type in token_types {
            while let Some(token) = self.tokens.peek() {
                if token.kind == TokenKind::Newline && *self.ignoring_newline_counter.borrow() > 0 {
                    self.tokens.advance_cursor();
                } else {
                    break;
                }
            }

            match self.tokens.peek() {
                Some(token) if token.kind == *token_type => {
                    self.tokens.advance_cursor();
                }
                _ => {
                    self.tokens.reset_cursor();

                    return None;
                }
            }
        }

        let count = self.tokens.cursor();
        let mut tokens = Vec::with_capacity(count);

        for _ in 0..count {
            tokens.push(self.tokens.next().cloned().unwrap());
        }

        Some(tokens)
    }

    pub fn check_sequence(&mut self, token_types: Iter<TokenKind>) -> bool {
        self.tokens.reset_cursor();

        for token_type in token_types {
            while let Some(token) = self.tokens.peek() {
                if token.kind == TokenKind::Newline && *self.ignoring_newline_counter.borrow() > 0 {
                    self.tokens.advance_cursor();
                } else {
                    break;
                }
            }

            match self.tokens.peek() {
                Some(token) if token.kind == *token_type => {
                    self.tokens.advance_cursor();
                }
                _ => {
                    self.tokens.reset_cursor();

                    return false;
                }
            }
        }

        self.tokens.reset_cursor();

        true
    }

    pub fn is_next_token(&mut self, token_type: TokenKind) -> bool {
        self.tokens.reset_cursor();

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::Newline && *self.ignoring_newline_counter.borrow() > 0 {
                self.tokens.advance_cursor();
            } else {
                break;
            }
        }

        if let Some(token) = self.tokens.peek() {
            if token.kind == token_type {
                return true;
            }
        }

        self.tokens.reset_cursor();

        false
    }

    pub fn is_next_eof(&mut self) -> bool {
        self.tokens.reset_cursor();

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::Newline {
                self.tokens.advance_cursor();
            } else {
                break;
            }
        }

        let is_eof = matches!(
            self.tokens.peek(),
            None | Some(Token {
                kind: TokenKind::Eof,
                ..
            })
        );

        self.tokens.reset_cursor();

        is_eof
    }

    pub fn is_next_valid(&mut self) -> bool {
        self.tokens.reset_cursor();

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::Newline && *self.ignoring_newline_counter.borrow() > 0 {
                self.tokens.advance_cursor();
            } else {
                break;
            }
        }

        let is_next_valid = !matches!(
            self.tokens.peek(),
            None | Some(Token {
                kind: TokenKind::Eof,
                ..
            })
        );

        self.tokens.reset_cursor();

        is_next_valid
    }

    pub fn advance_while(&mut self, expected_types: Iter<TokenKind>) {
        let token_types: Vec<TokenKind> = expected_types.cloned().collect();

        while let Some(token) = self.tokens.peek() {
            if token_types.iter().any(|t| *t == token.kind) {
                self.tokens.next();
            } else {
                break;
            }
        }
    }

    pub fn last_token(&self) -> &Token {
        self.last_token
    }

    fn skip_ignored_newlines(&mut self) {
        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::Newline && *self.ignoring_newline_counter.borrow() > 0 {
                self.tokens.next();
            } else {
                break;
            }
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ignored_newlines();
        self.tokens.next()
    }
}

impl<'a> From<&'a [Token]> for TokenIterator<'a> {
    fn from(value: &'a [Token]) -> Self {
        TokenIterator {
            tokens: value.iter().peekmore(),
            ignoring_newline_counter: Rc::new(RefCell::new(0)),
            last_token: value.last().expect("token list should not be empty"),
        }
    }
}

pub struct NewlineGuard {
    counter: Rc<RefCell<isize>>,
    size: isize,
}

impl NewlineGuard {
    pub fn new(counter: Rc<RefCell<isize>>, custom_size: Option<isize>) -> Self {
        let size = custom_size.unwrap_or(1);

        *counter.borrow_mut() += size;

        NewlineGuard { counter, size }
    }
}

impl Drop for NewlineGuard {
    fn drop(&mut self) {
        *self.counter.borrow_mut() -= self.size;
    }
}

#[macro_export]
macro_rules! token_stream {
    ($($kind:expr),*) => {
        {
            use tenda_scanner::TokenKind::*;
            vec![$($kind),*].iter()
        }
    };
}

#[macro_export]
macro_rules! token_vec {
    ($($kind:expr),*) => {
        {
            use tenda_scanner::TokenKind::*;
            vec![$($kind),*]
        }
    };
}
