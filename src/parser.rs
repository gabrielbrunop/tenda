use crate::ast::Expr;
use crate::token::{Token, TokenKind};
use crate::token_list;
use std::fmt;
use std::iter::Peekable;
use std::slice::Iter;

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [Token]) -> Parser<'a> {
        Parser {
            tokens: source.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, ParserError> {
        self.program()
    }

    fn program(&mut self) -> Result<Expr, ParserError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.term()
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
        let token = match self.tokens.next() {
            Some(token) => token,
            _ => unreachable!(),
        };

        match token.kind {
            TokenKind::Number => Ok(Expr::make_literal(token.literal.clone().unwrap())),
            TokenKind::LeftParen => {
                let expr = self.expression()?;

                if self.match_tokens(token_list![RightParen]).is_none() {
                    return Err(ParserError::MissingParentheses);
                }

                Ok(Expr::make_grouping(expr))
            }
            TokenKind::Eof => Err(ParserError::UnexpectedEoi),
            _ => Err(ParserError::UnexpectedToken(token.clone())),
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
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedEoi,
    UnexpectedToken(Token),
    MissingParentheses,
}

impl ParserError {
    pub fn message(&self) -> String {
        use ParserError::*;

        match self {
            UnexpectedEoi => "unexpected end of input".to_string(),
            MissingParentheses => "expected ')' after expression.".to_string(),
            UnexpectedToken(token) => format!("unexpected token: {}", token.lexeme),
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
