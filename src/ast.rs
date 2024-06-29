use crate::ast::UnaryOp::Negative;
use crate::token::{Token, TokenKind};
use crate::value::Value;

#[derive(Debug)]
pub enum Expr {
    Binary {
        lhs: Box<Expr>,
        op: BinaryOp,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        rhs: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Value,
    },
}

impl Expr {
    pub fn make_binary(lhs: Expr, op: BinaryOp, rhs: Expr) -> Self {
        Expr::Binary {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }

    pub fn make_unary(op: UnaryOp, rhs: Expr) -> Self {
        Expr::Unary {
            op,
            rhs: Box::new(rhs),
        }
    }

    pub fn make_literal(value: Value) -> Self {
        Expr::Literal { value }
    }

    pub fn make_grouping(expr: Expr) -> Self {
        Expr::Grouping {
            expr: Box::new(expr),
        }
    }
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiation,
    Modulo,
    Equality,
    Inequality,
}

impl From<Token> for BinaryOp {
    fn from(value: Token) -> Self {
        use BinaryOp::*;

        match value.kind {
            TokenKind::Plus => Add,
            TokenKind::Minus => Subtract,
            TokenKind::Star => Multiply,
            TokenKind::Slash => Divide,
            TokenKind::Percent => Modulo,
            TokenKind::Caret => Exponentiation,
            TokenKind::Equals => Equality,
            _ => panic!("invalid token for binary operation"),
        }
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    Negative,
}

impl From<Token> for UnaryOp {
    fn from(value: Token) -> Self {
        match value.kind {
            TokenKind::Minus => Negative,
            _ => panic!("invalid token for unary operation"),
        }
    }
}
