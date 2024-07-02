use crate::token::{Token, TokenKind};
use crate::value::Value;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Decl(Decl),
    Cond(Cond),
    Block(Block),
}

pub type Block = Vec<Stmt>;

#[derive(Debug, PartialEq)]
pub enum Decl {
    Local { name: String, value: Box<Expr> },
}

impl Decl {
    pub fn make_local_declaration(name: String, value: Expr) -> Self {
        Decl::Local {
            name,
            value: Box::new(value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Cond {
    If { cond: Box<Expr>, then: Box<Stmt> },
}

impl Cond {
    pub fn make_if_statement(cond: Expr, then: Stmt) -> Self {
        Cond::If {
            cond: Box::new(cond),
            then: Box::new(then),
        }
    }
}

#[derive(Debug, PartialEq)]
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
    Variable {
        name: String,
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

    pub fn make_variable(value: String) -> Self {
        Expr::Variable { name: value }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiation,
    Modulo,
    Equality,
    Inequality,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Assignment,
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
            TokenKind::Greater => Greater,
            TokenKind::GreaterOrEqual => GreaterOrEqual,
            TokenKind::Less => Less,
            TokenKind::LessOrEqual => LessOrEqual,
            TokenKind::EqualSign => Assignment,
            _ => panic!("invalid token for binary operation"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnaryOp {
    Negative,
    LogicalNot,
}

impl From<Token> for UnaryOp {
    fn from(value: Token) -> Self {
        use UnaryOp::*;

        match value.kind {
            TokenKind::Minus => Negative,
            TokenKind::Not => LogicalNot,
            _ => panic!("invalid token for unary operation"),
        }
    }
}
