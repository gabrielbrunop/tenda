use crate::ast::Expr;
use crate::token::{Token, TokenKind};
use crate::token_list;
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

    pub fn parse(&mut self) -> Expr {
        self.program()
    }

    fn program(&mut self) -> Expr {
        self.expression()
    }

    fn expression(&mut self) -> Expr {
        self.term()
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while let Some(op) = self.match_tokens(token_list![Plus, Minus]) {
            let lhs = expr;
            let rhs = self.factor();
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.exponent();

        while let Some(op) = self.match_tokens(token_list![Star, Slash, Percent]) {
            let lhs = expr;
            let rhs = self.exponent();
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        expr
    }

    fn exponent(&mut self) -> Expr {
        let mut expr = self.unary();

        while let Some(op) = self.match_tokens(token_list![Caret]) {
            let lhs = expr;
            let rhs = self.unary();
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if let Some(op) = self.match_tokens(token_list![Minus]) {
            let rhs = self.unary();
            Expr::make_unary(op.into(), rhs)
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        let token = match self.tokens.next() {
            Some(token) => token,
            _ => panic!("expected primary"),
        };

        match token.kind {
            TokenKind::Number => Expr::make_literal(token.literal.clone().unwrap()),
            TokenKind::LeftParen => {
                let expr = self.expression();

                if self.match_tokens(token_list![RightParen]).is_none() {
                    panic!("expected ')' after expression.");
                }

                Expr::make_grouping(expr)
            }
            _ => panic!("unexpected token: {}", token.lexeme),
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct ParserError {
    pub message: String,
}
