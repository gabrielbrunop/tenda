#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TokenSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl TokenSpan {
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        TokenSpan {
            start,
            end,
            line,
            col,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub span: TokenSpan,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        lexeme: String,
        literal: Option<Literal>,
        span: TokenSpan,
    ) -> Token {
        Token {
            kind,
            lexeme,
            literal,
            span,
        }
    }

    pub fn eoi(span: TokenSpan) -> Token {
        const EOT: i32 = 0x04;

        Token::new(TokenKind::Eof, EOT.to_string(), None, span)
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
