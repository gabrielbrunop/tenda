use scanner::token::{Token, TokenKind};

#[derive(Debug, PartialEq, Clone)]
pub struct Ast(pub Vec<Stmt>);

impl Ast {
    pub fn new() -> Self {
        Ast(vec![])
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
    While(While),
    Block(Block),
    Return(Return),
    Break(Break),
    Continue(Continue),
}

pub trait StmtVisitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_decl(&mut self, decl: &Decl) -> T;
    fn visit_cond(&mut self, cond: &Cond) -> T;
    fn visit_block(&mut self, block: &Block) -> T;
    fn visit_return(&mut self, return_stmt: &Return) -> T;
    fn visit_while(&mut self, while_stmt: &While) -> T;
    fn visit_break(&mut self, break_stmt: &Break) -> T;
    fn visit_continue(&mut self, continue_stmt: &Continue) -> T;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return(pub Option<Expr>);

#[derive(Debug, PartialEq, Clone)]
pub struct Break;

#[derive(Debug, PartialEq, Clone)]
pub struct Continue;

#[derive(Debug, PartialEq, Clone)]
pub struct Block(pub Ast);

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
    pub is_captured_var: bool,
    pub uid: usize,
}

impl LocalDecl {
    pub fn new(name: String, value: Expr, uid: usize) -> Self {
        LocalDecl {
            name,
            value,
            is_captured_var: false,
            uid,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<Stmt>,
    pub captured_vars: Vec<String>,
    pub is_captured_var: bool,
    pub uid: usize,
}

impl FunctionDecl {
    pub fn new(name: String, params: Vec<String>, body: Stmt, uid: usize) -> Self {
        FunctionDecl {
            name,
            params,
            body: Box::new(body),
            captured_vars: vec![],
            is_captured_var: false,
            uid,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Cond {
    pub cond: Expr,
    pub then: Box<Stmt>,
    pub or_else: Option<Box<Stmt>>,
}

impl Cond {
    pub fn new(cond: Expr, then: Stmt, or_else: Option<Stmt>) -> Self {
        Cond {
            cond,
            then: Box::new(then),
            or_else: or_else.map(Box::new),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct While {
    pub cond: Expr,
    pub body: Box<Stmt>,
}

impl While {
    pub fn new(cond: Expr, body: Stmt) -> Self {
        While {
            cond,
            body: Box::new(body),
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOp {
    pub lhs: Box<Expr>,
    pub op: BinaryOperator,
    pub rhs: Box<Expr>,
}

impl BinaryOp {
    pub fn new(lhs: Expr, op: BinaryOperator, rhs: Expr) -> Self {
        BinaryOp {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryOp {
    pub op: UnaryOperator,
    pub rhs: Box<Expr>,
}

impl UnaryOp {
    pub fn new(op: UnaryOperator, rhs: Expr) -> Self {
        UnaryOp {
            op,
            rhs: Box::new(rhs),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

impl Call {
    pub fn new(callee: Expr, args: Vec<Expr>) -> Self {
        Call {
            callee: Box::new(callee),
            args,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assign {
    pub name: Box<Expr>,
    pub value: Box<Expr>,
}

impl Assign {
    pub fn new(name: Expr, value: Expr) -> Self {
        Assign {
            name: Box::new(name),
            value: Box::new(value),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Access {
    pub subscripted: Box<Expr>,
    pub index: Box<Expr>,
}

impl Access {
    pub fn new(subscripted: Expr, index: Expr) -> Self {
        Access {
            subscripted: Box::new(subscripted),
            index: Box::new(index),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct List {
    pub elements: Vec<Expr>,
}

impl List {
    pub fn new(elements: Vec<Expr>) -> Self {
        List { elements }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Grouping {
    pub expr: Box<Expr>,
}

impl Grouping {
    pub fn new(expr: Expr) -> Self {
        Grouping {
            expr: Box::new(expr),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub value: scanner::token::Literal,
}

impl Literal {
    pub fn new(value: scanner::token::Literal) -> Self {
        Literal { value }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub name: String,
    pub uid: usize,
    pub is_captured_var: bool,
}

impl Variable {
    pub fn new(name: String, id: usize) -> Self {
        Variable {
            name,
            uid: id,
            is_captured_var: false,
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

macro_rules! make_function_decl {
    ($name:expr, $parameters:expr, $body:expr, $uid:expr) => {{
        use $crate::ast::{Decl, FunctionDecl, Stmt};
        Stmt::Decl(Decl::Function(FunctionDecl::new(
            $name.to_string(),
            $parameters,
            $body,
            $uid,
        )))
    }};
}

macro_rules! make_local_decl {
    ($name:expr, $value:expr, $uid:expr) => {{
        use $crate::ast::{Decl, LocalDecl, Stmt};
        Stmt::Decl(Decl::Local(LocalDecl::new($name.to_string(), $value, $uid)))
    }};
}

macro_rules! make_literal_expr {
    ($value:expr) => {{
        #[allow(unused_imports)]
        use scanner::token::Literal::*;
        $crate::ast::Expr::Literal($crate::ast::Literal::new($value))
    }};
}

macro_rules! make_return_stmt {
    ($value:expr) => {{
        use $crate::ast::Stmt;
        Stmt::Return($crate::ast::Return($value))
    }};
}

macro_rules! make_break_stmt {
    () => {{
        use $crate::ast::Stmt;
        Stmt::Break($crate::ast::Break)
    }};
}

macro_rules! make_continue_stmt {
    () => {{
        use $crate::ast::Stmt;
        Stmt::Continue($crate::ast::Continue)
    }};
}

macro_rules! make_binary_expr {
    ($lhs:expr, $op:expr, $rhs:expr) => {{
        use $crate::ast::Expr;
        Expr::Binary($crate::ast::BinaryOp::new($lhs, $op, $rhs))
    }};
}

macro_rules! make_unary_expr {
    ($op:expr, $rhs:expr) => {{
        use $crate::ast::Expr;
        Expr::Unary($crate::ast::UnaryOp::new($op, $rhs))
    }};
}

macro_rules! make_call_expr {
    ($callee:expr, $args:expr) => {{
        use $crate::ast::Expr;
        Expr::Call($crate::ast::Call::new($callee, $args))
    }};
}

macro_rules! make_assign_expr {
    ($name:expr, $value:expr) => {{
        use $crate::ast::Expr;
        Expr::Assign($crate::ast::Assign::new($name, $value))
    }};
}

macro_rules! make_access_expr {
    ($subscripted:expr, $index:expr) => {{
        use $crate::ast::Expr;
        Expr::Access($crate::ast::Access::new($subscripted, $index))
    }};
}

macro_rules! make_grouping_expr {
    ($expr:expr) => {{
        use $crate::ast::Expr;
        Expr::Grouping($crate::ast::Grouping::new($expr))
    }};
}

macro_rules! make_variable_expr {
    ($name:expr, $uid:expr) => {{
        use $crate::ast::Expr;
        Expr::Variable($crate::ast::Variable::new($name.to_string(), $uid))
    }};
}

macro_rules! make_list_expr {
    ($elements:expr) => {{
        use $crate::ast::Expr;
        Expr::List($crate::ast::List::new($elements))
    }};
}

macro_rules! make_cond_stmt {
    ($cond:expr, $then:expr, $or_else:expr) => {{
        use $crate::ast::{Cond, Stmt};
        Stmt::Cond(Cond::new($cond, $then, $or_else))
    }};
}

macro_rules! make_while_stmt {
    ($cond:expr, $body:expr) => {{
        use $crate::ast::{Stmt, While};
        Stmt::While(While::new($cond, $body))
    }};
}

macro_rules! make_block_stmt {
    ($statements:expr) => {{
        use $crate::ast::{Block, Stmt};
        Stmt::Block(Block($statements))
    }};
}

pub(crate) use make_access_expr;
pub(crate) use make_assign_expr;
pub(crate) use make_binary_expr;
pub(crate) use make_block_stmt;
pub(crate) use make_break_stmt;
pub(crate) use make_call_expr;
pub(crate) use make_cond_stmt;
pub(crate) use make_continue_stmt;
pub(crate) use make_function_decl;
pub(crate) use make_grouping_expr;
pub(crate) use make_list_expr;
pub(crate) use make_literal_expr;
pub(crate) use make_local_decl;
pub(crate) use make_return_stmt;
pub(crate) use make_unary_expr;
pub(crate) use make_variable_expr;
pub(crate) use make_while_stmt;
