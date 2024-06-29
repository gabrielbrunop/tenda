use crate::ast::{BinaryOp, Expr};
use crate::token::{Token, TokenKind};
use crate::token_list;
use peekmore::{PeekMore, PeekMoreIterator};
use std::fmt;
use std::slice::Iter;

macro_rules! parser_error {
    ($kind:expr, $line:expr) => {{
        use ParserErrorKind::*;
        ParserError {
            kind: $kind,
            line: $line,
        }
    }};
}

pub struct Parser<'a> {
    tokens: PeekMoreIterator<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [Token]) -> Parser<'a> {
        Parser {
            tokens: source.iter().peekmore(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, ParserError> {
        self.program()
    }

    fn program(&mut self) -> Result<Expr, ParserError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        loop {
            let op: Option<BinaryOp> = {
                if let Some(token) = self.match_tokens(token_list![Equals]) {
                    Some(token.into())
                } else if self.matches_sequence(token_list![Not, Equals]) {
                    Some(BinaryOp::Inequality)
                } else if let Some(token) = self.match_tokens(token_list![Not]) {
                    return parser_error!(UnexpectedToken(token.clone()), token.line).into();
                } else {
                    None
                }
            };

            if let Some(op) = op {
                let lhs = expr;
                let rhs = self.comparison()?;
                expr = Expr::make_binary(lhs, op, rhs);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;

        while let Some(op) =
            self.match_tokens(token_list![Greater, GreaterOrEqual, Less, LessOrEqual])
        {
            let lhs = expr;
            let rhs = self.term()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while let Some(op) = self.match_tokens(token_list![Plus, Minus]) {
            let lhs = expr;
            let rhs = self.factor()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.exponent()?;

        while let Some(op) = self.match_tokens(token_list![Star, Slash, Percent]) {
            let lhs = expr;
            let rhs = self.exponent()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn exponent(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while let Some(op) = self.match_tokens(token_list![Caret]) {
            let lhs = expr;
            let rhs = self.unary()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if let Some(op) = self.match_tokens(token_list![Minus]) {
            let rhs = self.unary()?;
            let expr = Expr::make_unary(op.into(), rhs);

            Ok(expr)
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        use TokenKind::*;

        let token = match self.tokens.next() {
            Some(token) => token,
            _ => unreachable!(),
        };

        match token.kind {
            Number | True | False | String | Nil => {
                Ok(Expr::make_literal(token.literal.clone().unwrap()))
            }
            LeftParen => {
                let expr = self.expression()?;

                if self.match_tokens(token_list![RightParen]).is_none() {
                    return parser_error!(MissingParentheses, token.line).into();
                }

                Ok(Expr::make_grouping(expr))
            }
            Eof => parser_error!(UnexpectedEoi, token.line).into(),
            _ => parser_error!(UnexpectedToken(token.clone()), token.line).into(),
        }
    }
}

impl<'a> Parser<'a> {
    fn match_tokens(&mut self, token_types: Iter<TokenKind>) -> Option<Token> {
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

    fn matches_sequence(&mut self, token_types: Iter<TokenKind>) -> bool {
        assert_eq!(self.tokens.cursor(), 0, "cursor is already in use");

        for token_type in token_types {
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
}

#[derive(Debug)]
pub struct ParserError {
    line: usize,
    kind: ParserErrorKind,
}

impl ParserError {
    pub fn message(&self) -> String {
        use ParserErrorKind::*;

        match &self.kind {
            UnexpectedEoi => "unexpected end of input".to_string(),
            MissingParentheses => "expected ')' after expression.".to_string(),
            UnexpectedToken(token) => format!("unexpected token: {}", token.lexeme),
        }
    }
}

impl<T> From<ParserError> for Result<T, ParserError> {
    fn from(val: ParserError) -> Self {
        Err(val)
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at line {})", self.message(), self.line)
    }
}

#[derive(Debug)]
pub enum ParserErrorKind {
    UnexpectedEoi,
    UnexpectedToken(Token),
    MissingParentheses,
}
