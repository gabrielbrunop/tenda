use tenda_common::span::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub span: SourceSpan,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        lexeme: String,
        literal: Option<Literal>,
        span: SourceSpan,
    ) -> Token {
        Token {
            kind,
            lexeme,
            literal,
            span,
        }
    }

    pub fn eoi(span: SourceSpan) -> Token {
        Token::new(TokenKind::Eof, "EOF".to_string(), None, span)
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
    Continue,
    Identifier,
    EqualSign,
    Until,
    ForOrBreak,
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
    pub const POSITIVE_INFINITY_LITERAL: &'static str = "infinito";
    pub const NEGATIVE_INFINITY_LITERAL: &'static str = "-infinito";
    pub const NAN_LITERAL: &'static str = "NaN";
}
