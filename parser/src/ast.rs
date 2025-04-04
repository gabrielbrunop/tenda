use common::span::SourceSpan;
use scanner::{Token, TokenKind};

#[derive(Debug, PartialEq, Clone)]
pub struct Ast {
    pub inner: Vec<Stmt>,
    pub span: SourceSpan,
}

impl Ast {
    pub fn new(span: SourceSpan) -> Self {
        Ast {
            inner: vec![],
            span,
        }
    }

    pub fn from(inner: Vec<Stmt>, span: SourceSpan) -> Self {
        Ast { inner, span }
    }
}

impl IntoIterator for Ast {
    type Item = Stmt;
    type IntoIter = std::vec::IntoIter<Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr),
    Decl(Decl),
    Cond(Cond),
    While(While),
    ForEach(ForEach),
    Block(Block),
    Return(Return),
    Break(Break),
    Continue(Continue),
}

impl Stmt {
    pub fn get_span(&self) -> &SourceSpan {
        match self {
            Stmt::Expr(expr) => expr.get_span(),
            Stmt::Decl(decl) => match decl {
                Decl::Local(local) => &local.span,
                Decl::Function(function) => &function.span,
            },
            Stmt::Cond(cond) => &cond.span,
            Stmt::While(while_stmt) => &while_stmt.span,
            Stmt::ForEach(for_each) => &for_each.span,
            Stmt::Block(block) => &block.span,
            Stmt::Return(return_stmt) => &return_stmt.span,
            Stmt::Break(break_stmt) => &break_stmt.span,
            Stmt::Continue(continue_stmt) => &continue_stmt.span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub value: Option<Expr>,
    pub span: SourceSpan,
}

impl Return {
    pub fn new(value: Option<Expr>, span: SourceSpan) -> Self {
        Return { value, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Break {
    pub span: SourceSpan,
}

impl Break {
    pub fn new(span: SourceSpan) -> Self {
        Break { span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Continue {
    pub span: SourceSpan,
}

impl Continue {
    pub fn new(span: SourceSpan) -> Self {
        Continue { span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub inner: Ast,
    pub span: SourceSpan,
}

impl Block {
    pub fn new(inner: Ast, span: SourceSpan) -> Self {
        Block { inner, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Decl {
    Local(LocalDecl),
    Function(FunctionDecl),
}

impl Decl {
    pub fn get_name(&self) -> &str {
        match self {
            Decl::Local(local) => &local.name,
            Decl::Function(function) => &function.name,
        }
    }

    pub fn get_uid(&self) -> usize {
        match self {
            Decl::Local(local) => local.uid,
            Decl::Function(function) => function.uid,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LocalDecl {
    pub name: String,
    pub value: Expr,
    pub captured: bool,
    pub uid: usize,
    pub span: SourceSpan,
}

impl LocalDecl {
    pub fn new(name: String, value: Expr, uid: usize, span: SourceSpan) -> Self {
        LocalDecl {
            name,
            value,
            captured: false,
            uid,
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub uid: usize,
    pub captured: bool,
    pub span: SourceSpan,
}

impl FunctionParam {
    pub fn new(name: String, uid: usize, span: SourceSpan) -> Self {
        FunctionParam {
            name,
            uid,
            captured: false,
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub body: Box<Stmt>,
    pub free_vars: Vec<String>,
    pub captured: bool,
    pub uid: usize,
    pub span: SourceSpan,
}

impl FunctionDecl {
    pub fn new(
        name: String,
        params: Vec<FunctionParam>,
        body: Stmt,
        uid: usize,
        span: SourceSpan,
    ) -> Self {
        FunctionDecl {
            name,
            params,
            body: Box::new(body),
            free_vars: vec![],
            captured: false,
            uid,
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Cond {
    pub cond: Expr,
    pub then: Box<Stmt>,
    pub or_else: Option<Box<Stmt>>,
    pub span: SourceSpan,
}

impl Cond {
    pub fn new(cond: Expr, then: Stmt, or_else: Option<Stmt>, span: SourceSpan) -> Self {
        Cond {
            cond,
            then: Box::new(then),
            or_else: or_else.map(Box::new),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct While {
    pub cond: Expr,
    pub body: Box<Stmt>,
    pub span: SourceSpan,
}

impl While {
    pub fn new(cond: Expr, body: Stmt, span: SourceSpan) -> Self {
        While {
            cond,
            body: Box::new(body),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForEachItem {
    pub name: String,
    pub uid: usize,
    pub captured: bool,
    pub span: SourceSpan,
}

impl ForEachItem {
    pub fn new(name: String, uid: usize, span: SourceSpan) -> Self {
        ForEachItem {
            name,
            uid,
            captured: false,
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForEach {
    pub item: ForEachItem,
    pub iterable: Expr,
    pub body: Box<Stmt>,
    pub span: SourceSpan,
}

impl ForEach {
    pub fn new(item: ForEachItem, iterable: Expr, body: Stmt, span: SourceSpan) -> Self {
        ForEach {
            item,
            iterable,
            body: Box::new(body),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(BinaryOp),
    Unary(UnaryOp),
    Call(Call),
    Assign(Assign),
    Access(Access),
    List(List),
    Grouping(Grouping),
    Literal(Literal),
    Variable(Variable),
    AssociativeArray(AssociativeArray),
    AnonymousFunction(AnonymousFunction),
}

impl Expr {
    pub fn get_span(&self) -> &SourceSpan {
        match self {
            Expr::Binary(binary_op) => &binary_op.span,
            Expr::Unary(unary_op) => &unary_op.span,
            Expr::Call(call) => &call.span,
            Expr::Assign(assign) => &assign.span,
            Expr::Access(access) => &access.span,
            Expr::List(list) => &list.span,
            Expr::Grouping(grouping) => &grouping.span,
            Expr::Literal(literal) => &literal.span,
            Expr::Variable(variable) => &variable.span,
            Expr::AssociativeArray(associative_array) => &associative_array.span,
            Expr::AnonymousFunction(anonymous_function) => &anonymous_function.span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOp {
    pub lhs: Box<Expr>,
    pub op: BinaryOperator,
    pub rhs: Box<Expr>,
    pub span: SourceSpan,
}

impl BinaryOp {
    pub fn new(lhs: Expr, op: BinaryOperator, rhs: Expr, span: SourceSpan) -> Self {
        BinaryOp {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryOp {
    pub op: UnaryOperator,
    pub rhs: Box<Expr>,
    pub span: SourceSpan,
}

impl UnaryOp {
    pub fn new(op: UnaryOperator, rhs: Expr, span: SourceSpan) -> Self {
        UnaryOp {
            op,
            rhs: Box::new(rhs),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: SourceSpan,
}

impl Call {
    pub fn new(callee: Expr, args: Vec<Expr>, span: SourceSpan) -> Self {
        Call {
            callee: Box::new(callee),
            args,
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub name: Box<Expr>,
    pub value: Box<Expr>,
    pub span: SourceSpan,
}

impl Assign {
    pub fn new(name: Expr, value: Expr, span: SourceSpan) -> Self {
        Assign {
            name: Box::new(name),
            value: Box::new(value),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Access {
    pub subscripted: Box<Expr>,
    pub index: Box<Expr>,
    pub span: SourceSpan,
}

impl Access {
    pub fn new(subscripted: Expr, index: Expr, span: SourceSpan) -> Self {
        Access {
            subscripted: Box::new(subscripted),
            index: Box::new(index),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct List {
    pub elements: Vec<Expr>,
    pub span: SourceSpan,
}

impl List {
    pub fn new(elements: Vec<Expr>, span: SourceSpan) -> Self {
        List { elements, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AssociativeArray {
    pub elements: Vec<(Literal, Expr)>,
    pub span: SourceSpan,
}

impl AssociativeArray {
    pub fn new(elements: Vec<(Literal, Expr)>, span: SourceSpan) -> Self {
        AssociativeArray { elements, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Grouping {
    pub expr: Box<Expr>,
    pub span: SourceSpan,
}

impl Grouping {
    pub fn new(expr: Expr, span: SourceSpan) -> Self {
        Grouping {
            expr: Box::new(expr),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub value: scanner::Literal,
    pub span: SourceSpan,
}

impl Literal {
    pub fn new(value: scanner::Literal, span: SourceSpan) -> Self {
        Literal { value, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub name: String,
    pub uid: usize,
    pub captured: bool,
    pub span: SourceSpan,
}

impl Variable {
    pub fn new(name: String, id: usize, span: SourceSpan) -> Self {
        Variable {
            name,
            uid: id,
            captured: false,
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AnonymousFunction {
    pub params: Vec<FunctionParam>,
    pub body: Box<Stmt>,
    pub uid: usize,
    pub free_vars: Vec<String>,
    pub span: SourceSpan,
}

impl AnonymousFunction {
    pub fn new(params: Vec<FunctionParam>, body: Stmt, uid: usize, span: SourceSpan) -> Self {
        AnonymousFunction {
            params,
            body: Box::new(body),
            uid,
            free_vars: vec![],
            span,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinaryOperator {
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
    LogicalAnd,
    LogicalOr,
    Range,
    Has,
    Lacks,
}

impl From<Token> for BinaryOperator {
    fn from(value: Token) -> Self {
        use BinaryOperator::*;

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
            TokenKind::Or => LogicalOr,
            TokenKind::And => LogicalAnd,
            TokenKind::Until => Range,
            _ => panic!("invalid token for binary operation"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UnaryOperator {
    Negative,
    LogicalNot,
}

impl From<Token> for UnaryOperator {
    fn from(value: Token) -> Self {
        use UnaryOperator::*;

        match value.kind {
            TokenKind::Minus => Negative,
            TokenKind::Not => LogicalNot,
            _ => panic!("invalid token for unary operation"),
        }
    }
}
