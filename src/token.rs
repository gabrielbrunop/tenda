use crate::value::Value;
use std::fmt::Display;

#[macro_export]
macro_rules! token_list {
    ($($kind:expr),*) => {
        {
            use TokenKind::*;
            vec![$($kind),*].iter()
        }
    };
}

#[macro_export]
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

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: Option<Value>,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, literal: Option<Value>, line: usize) -> Token {
        Token {
            kind,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {{ Token {:?}, Literal {:?}, Line {} }}",
            self.lexeme, self.kind, self.literal, self.line
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LeftParen,
    RightParen,
    Eof,
}
