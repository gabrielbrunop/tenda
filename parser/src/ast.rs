use scanner::token::{Literal, Token, TokenKind};

#[derive(Debug, PartialEq, Clone)]
pub struct Ast(pub Vec<Stmt>);

impl Ast {
    pub fn new() -> Self {
        Ast(vec![])
    }

    pub fn push(&mut self, statement: Stmt) {
        self.get_statements_mut().push(statement);
    }

    fn get_statements_mut(&mut self) -> &mut Vec<Stmt> {
        &mut self.0
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Ast {
    type Item = Stmt;
    type IntoIter = std::vec::IntoIter<Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<Stmt>> for Ast {
    fn from(value: Vec<Stmt>) -> Self {
        Ast(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr),
    Decl(Decl),
    Cond(Cond),
    Block(Block),
    Return(Return),
}

pub type Return = Option<Expr>;

pub type Block = Ast;

#[derive(Debug, PartialEq, Clone)]
pub enum Decl {
    Local {
        name: String,
        value: Box<Expr>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
    },
}

impl Decl {
    pub fn make_local_declaration(name: String, value: Expr) -> Self {
        Decl::Local {
            name,
            value: Box::new(value),
        }
    }

    pub fn make_function_declaration(name: String, params: Vec<String>, body: Stmt) -> Self {
        Decl::Function {
            name,
            params,
            body: Box::new(body),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Cond {
    pub cond: Box<Expr>,
    pub then: Box<Stmt>,
    pub or_else: Option<Box<Stmt>>,
}

impl Cond {
    pub fn make_if_statement(cond: Expr, then: Stmt, or_else: Option<Stmt>) -> Self {
        Cond {
            cond: Box::new(cond),
            then: Box::new(then),
            or_else: or_else.map(Box::new),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Literal,
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

    pub fn make_literal(value: Literal) -> Self {
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

    pub fn make_call(callee: Expr, args: Vec<Expr>) -> Self {
        Expr::Call {
            callee: Box::new(callee),
            args,
        }
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
    LogicalAnd,
    LogicalOr,
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
            TokenKind::Or => LogicalOr,
            TokenKind::And => LogicalAnd,
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
