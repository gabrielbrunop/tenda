use peekmore::{PeekMore, PeekMoreIterator};
use std::{cell::RefCell, rc::Rc, slice::Iter};

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
    Identifier,
    EqualSign,
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
    Comma,
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

pub struct TokenIterator<'a> {
    tokens: PeekMoreIterator<Iter<'a, Token>>,
    ignoring_newline_counter: Rc<RefCell<usize>>,
    last_line: usize,
}

impl TokenIterator<'_> {
    pub fn peek(&mut self) -> Option<&&Token> {
        self.tokens.peek()
    }

    pub fn set_ignoring_newline(&mut self) -> NewlineGuard {
        NewlineGuard::new(self.ignoring_newline_counter.clone())
    }

    pub fn match_tokens(&mut self, token_types: Iter<TokenKind>) -> Option<Token> {
        self.ignore_newline();

        let next = self.tokens.peek();

        for t in token_types {
            match next {
                Some(token) if token.kind == *t => {
                    return Some(self.tokens.next().unwrap().clone())
                }
                _ => (),
            }
        }

        None
    }

    pub fn matches_sequence(&mut self, token_types: Iter<TokenKind>) -> bool {
        assert_eq!(self.tokens.cursor(), 0, "cursor is already in use");

        for token_type in token_types {
            if self.ignore_newline().is_some() {
                continue;
            }

            let matched_sequence =
                matches!(self.tokens.peek(), Some(token) if *token_type == token.kind);

            if !matched_sequence {
                self.tokens.reset_cursor();
                return false;
            }

            self.tokens.advance_cursor();
        }

        for _ in 0..self.tokens.cursor() {
            self.tokens.next();
        }

        true
    }

    pub fn get_last_line(&mut self) -> usize {
        self.last_line
    }

    fn ignore_newline(&mut self) -> Option<&Token> {
        if *self.ignoring_newline_counter.borrow() == 0 {
            None
        } else if matches!(self.tokens.peek(), Some(token) if token.kind == TokenKind::Newline) {
            self.tokens.next()
        } else {
            None
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
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

pub struct NewlineGuard(Rc<RefCell<usize>>);

impl NewlineGuard {
    pub fn new(counter: Rc<RefCell<usize>>) -> Self {
        *counter.borrow_mut() += 1;
        NewlineGuard(counter)
    }
}

impl Drop for NewlineGuard {
    fn drop(&mut self) {
        *self.0.borrow_mut() -= 1;
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
