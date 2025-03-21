use peekmore::{PeekMore, PeekMoreIterator};
use std::{cell::RefCell, ops::Neg, rc::Rc, slice::Iter};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, literal: Option<Literal>, line: usize) -> Token {
        Token {
            kind,
            lexeme,
            literal,
            line,
        }
    }

    pub fn eoi(line: usize) -> Token {
        const EOT: i32 = 0x04;
        token!(Eof, EOT, line)
    }

    pub fn clone_ref(&self) -> Token {
        (*self).clone()
    }
}

impl<T> From<Token> for Result<Option<Token>, T> {
    fn from(val: Token) -> Self {
        Ok(Some(val))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Number,
    String,
    True,
    False,
    Nil,
    Equals,
    Not,
    Or,
    And,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Let,
    If,
    Function,
    Then,
    Else,
    Return,
    BlockEnd,
    While,
    Do,
    Break,
    Continue,
    Identifier,
    EqualSign,
    Until,
    For,
    Each,
    In,
    Has,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Literal {
    pub const TRUE_LITERAL: &'static str = "verdadeiro";
    pub const FALSE_LITERAL: &'static str = "falso";
    pub const NIL_LITERAL: &'static str = "Nada";
}

#[derive(Debug)]
pub struct TokenIterator<'a> {
    tokens: PeekMoreIterator<Iter<'a, Token>>,
    ignoring_newline_counter: Rc<RefCell<isize>>,
    last_line: usize,
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

    pub fn consume_matching_tokens(&mut self, token_types: Iter<TokenKind>) -> Option<Token> {
        self.tokens.reset_cursor();

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::Newline && *self.ignoring_newline_counter.borrow() > 0 {
                self.tokens.advance_cursor();
            } else {
                break;
            }
        }

        let skipped = self.tokens.cursor();

        if let Some(token) = self.tokens.peek() {
            let expected: Vec<TokenKind> = token_types.cloned().collect();

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

    pub fn matches_sequence(&mut self, token_types: Iter<TokenKind>) -> bool {
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

        let count = self.tokens.cursor();

        for _ in 0..count {
            self.tokens.next();
        }

        true
    }

    pub fn get_last_line(&mut self) -> usize {
        self.last_line
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
            last_line: value.last().map(|t| t.line).unwrap_or(0),
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
macro_rules! token_iter {
    ($($kind:expr),*) => {
        {
            use $crate::token::TokenKind::*;
            vec![$($kind),*].iter()
        }
    };
}

#[macro_export]
macro_rules! token_vec {
    ($($kind:expr),*) => {
        {
            use $crate::token::TokenKind::*;
            vec![$($kind),*]
        }
    };
}

macro_rules! token {
    ($kind:expr, $lexeme:expr, $line:expr) => {{
        use TokenKind::*;
        Token::new($kind, $lexeme.to_string(), None, $line)
    }};
    ($kind:expr, $lexeme:expr, $line:expr, $literal:expr) => {{
        use TokenKind::*;
        Token::new($kind, $lexeme.to_string(), Some($literal), $line)
    }};
}

pub(crate) use token;
