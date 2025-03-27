use std::rc::Rc;

use scanner::token::{Token, TokenKind};

#[derive(Debug, PartialEq, Clone)]
pub struct AstSpan {
    pub start: usize,
    pub end: usize,
    pub source: Rc<NamedSource>,
}

impl AstSpan {
    pub fn new(start: usize, end: usize, source: Rc<NamedSource>) -> Self {
        AstSpan { start, end, source }
    }

    pub fn from_token(token: &Token, source: Rc<NamedSource>) -> Self {
        AstSpan {
            start: token.span.start,
            end: token.span.end,
            source,
        }
    }
}

impl From<&AstSpan> for miette::SourceSpan {
    fn from(val: &AstSpan) -> Self {
        let length = val.end - val.start;
        miette::SourceSpan::new(val.start.into(), length)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NamedSource {
    pub name: String,
    pub source: String,
}

impl NamedSource {
    pub fn new(name: String, source: String) -> Self {
        NamedSource { name, source }
    }
}

impl From<NamedSource> for miette::NamedSource<String> {
    fn from(val: NamedSource) -> Self {
        miette::NamedSource::new(val.name, val.source)
    }
}

pub trait RcNamedSourceExt {
    fn as_optional(&self) -> Option<NamedSource>;
}

impl RcNamedSourceExt for Rc<NamedSource> {
    fn as_optional(&self) -> Option<NamedSource> {
        Some(self.as_ref().clone())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ast {
    pub inner: Vec<Stmt>,
    pub span: AstSpan,
}

impl Ast {
    pub fn new(span: AstSpan) -> Self {
        Ast {
            inner: vec![],
            span,
        }
    }
}

impl IntoIterator for Ast {
    type Item = Stmt;
    type IntoIter = std::vec::IntoIter<Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl From<Vec<Stmt>> for Ast {
    fn from(value: Vec<Stmt>) -> Self {
        let span = if value.is_empty() {
            AstSpan {
                start: 0,
                end: 0,
                source: Rc::new(NamedSource {
                    name: "".to_string(),
                    source: "".to_string(),
                }),
            }
        } else {
            AstSpan {
                start: value[0].get_span().start,
                end: value[value.len() - 1].get_span().end,
                source: value[0].get_span().source.clone(),
            }
        };

        Ast { inner: value, span }
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
    pub fn get_span(&self) -> &AstSpan {
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

pub trait StmtVisitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_decl(&mut self, decl: &Decl) -> T;
    fn visit_cond(&mut self, cond: &Cond) -> T;
    fn visit_block(&mut self, block: &Block) -> T;
    fn visit_return(&mut self, return_stmt: &Return) -> T;
    fn visit_while(&mut self, while_stmt: &While) -> T;
    fn visit_for_each(&mut self, for_each: &ForEach) -> T;
    fn visit_break(&mut self, break_stmt: &Break) -> T;
    fn visit_continue(&mut self, continue_stmt: &Continue) -> T;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub value: Option<Expr>,
    pub span: AstSpan,
}

impl Return {
    pub fn new(value: Option<Expr>, span: AstSpan) -> Self {
        Return { value, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Break {
    pub span: AstSpan,
}

impl Break {
    pub fn new(span: AstSpan) -> Self {
        Break { span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Continue {
    pub span: AstSpan,
}

impl Continue {
    pub fn new(span: AstSpan) -> Self {
        Continue { span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub inner: Ast,
    pub span: AstSpan,
}

impl Block {
    pub fn new(inner: Ast, span: AstSpan) -> Self {
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

pub trait DeclVisitor<T> {
    fn visit_local(&mut self, local: &LocalDecl) -> T;
    fn visit_function(&mut self, function: &FunctionDecl) -> T;
}

#[derive(Debug, PartialEq, Clone)]
pub struct LocalDecl {
    pub name: String,
    pub value: Expr,
    pub captured: bool,
    pub uid: usize,
    pub span: AstSpan,
}

impl LocalDecl {
    pub fn new(name: String, value: Expr, uid: usize, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl FunctionParam {
    pub fn new(name: String, uid: usize, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl FunctionDecl {
    pub fn new(
        name: String,
        params: Vec<FunctionParam>,
        body: Stmt,
        uid: usize,
        span: AstSpan,
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
    pub span: AstSpan,
}

impl Cond {
    pub fn new(cond: Expr, then: Stmt, or_else: Option<Stmt>, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl While {
    pub fn new(cond: Expr, body: Stmt, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl ForEachItem {
    pub fn new(name: String, uid: usize, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl ForEach {
    pub fn new(item: ForEachItem, iterable: Expr, body: Stmt, span: AstSpan) -> Self {
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
    pub fn get_span(&self) -> &AstSpan {
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

pub trait ExprVisitor<T> {
    fn visit_binary(&mut self, binary: &BinaryOp) -> T;
    fn visit_unary(&mut self, unary: &UnaryOp) -> T;
    fn visit_call(&mut self, call: &Call) -> T;
    fn visit_access(&mut self, index: &Access) -> T;
    fn visit_assign(&mut self, assign: &Assign) -> T;
    fn visit_list(&mut self, list: &List) -> T;
    fn visit_grouping(&mut self, grouping: &Grouping) -> T;
    fn visit_literal(&mut self, literal: &Literal) -> T;
    fn visit_variable(&mut self, variable: &Variable) -> T;
    fn visit_associative_array(&mut self, associative_array: &AssociativeArray) -> T;
    fn visit_anonymous_function(&mut self, anonymous_function: &AnonymousFunction) -> T;
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOp {
    pub lhs: Box<Expr>,
    pub op: BinaryOperator,
    pub rhs: Box<Expr>,
    pub span: AstSpan,
}

impl BinaryOp {
    pub fn new(lhs: Expr, op: BinaryOperator, rhs: Expr, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl UnaryOp {
    pub fn new(op: UnaryOperator, rhs: Expr, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl Call {
    pub fn new(callee: Expr, args: Vec<Expr>, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl Assign {
    pub fn new(name: Expr, value: Expr, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl Access {
    pub fn new(subscripted: Expr, index: Expr, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl List {
    pub fn new(elements: Vec<Expr>, span: AstSpan) -> Self {
        List { elements, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AssociativeArray {
    pub elements: Vec<(Literal, Expr)>,
    pub span: AstSpan,
}

impl AssociativeArray {
    pub fn new(elements: Vec<(Literal, Expr)>, span: AstSpan) -> Self {
        AssociativeArray { elements, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Grouping {
    pub expr: Box<Expr>,
    pub span: AstSpan,
}

impl Grouping {
    pub fn new(expr: Expr, span: AstSpan) -> Self {
        Grouping {
            expr: Box::new(expr),
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub value: scanner::token::Literal,
    pub span: AstSpan,
}

impl Literal {
    pub fn new(value: scanner::token::Literal, span: AstSpan) -> Self {
        Literal { value, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub name: String,
    pub uid: usize,
    pub captured: bool,
    pub span: AstSpan,
}

impl Variable {
    pub fn new(name: String, id: usize, span: AstSpan) -> Self {
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
    pub span: AstSpan,
}

impl AnonymousFunction {
    pub fn new(params: Vec<FunctionParam>, body: Stmt, uid: usize, span: AstSpan) -> Self {
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
