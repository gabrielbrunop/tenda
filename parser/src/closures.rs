use crate::ast::{self, Ast};

type ClosureVarRef = (usize, usize);

#[derive(Debug)]
pub struct Closure {
    pub func_decl: usize,
    pub reference: usize,
    pub var_decl: usize,
    pub var_name: String,
}

impl Closure {
    pub fn new(func_decl: usize, reference: usize, var_decl: usize, var_name: String) -> Self {
        Closure {
            func_decl,
            reference,
            var_decl,
            var_name,
        }
    }
}

#[derive(Debug)]
struct ClosureList(Vec<Closure>);

impl ClosureList {
    pub fn new(closures: Vec<Closure>) -> Self {
        ClosureList(closures)
    }

    pub fn is_captured_var_decl(&self, var_decl: usize) -> bool {
        self.0.iter().any(|closure| closure.var_decl == var_decl)
    }

    pub fn is_captured_var_ref(&self, var_ref: usize) -> bool {
        self.0.iter().any(|closure| closure.reference == var_ref)
    }

    pub fn get_closures(&self, func_decl: usize) -> Vec<&Closure> {
        self.0
            .iter()
            .filter(|closure| closure.func_decl == func_decl)
            .collect()
    }

    pub fn get_closures_names(&self, func_decl: usize) -> Vec<String> {
        self.get_closures(func_decl)
            .iter()
            .map(|closure| closure.var_name.clone())
            .collect()
    }
}

pub fn apply_closures_in_ast(ast: &mut Ast) {
    use ast::*;

    let closure_list = get_closures_from_ast(ast);
    let closure_list = ClosureList::new(closure_list);

    let Ast(ast) = ast;

    for stmt in ast.iter_mut() {
        apply_closures_in_stmt(stmt, &closure_list);
    }
}

fn apply_closures_in_stmt(stmt: &mut ast::Stmt, closure_list: &ClosureList) {
    use ast::*;

    match stmt {
        Stmt::Expr(expr) => apply_closures_in_expr(expr, closure_list),
        Stmt::Decl(Decl::Local(LocalDecl {
            uid,
            is_captured_var,
            ..
        })) => {
            if closure_list.is_captured_var_decl(*uid) {
                *is_captured_var = true;
            }
        }
        Stmt::Decl(ast::Decl::Function(ast::FunctionDecl {
            body,
            is_captured_var,
            captured_vars,
            uid,
            ..
        })) => {
            if closure_list.is_captured_var_decl(*uid) {
                *is_captured_var = true;
            }

            *captured_vars = closure_list.get_closures_names(*uid);

            apply_closures_in_stmt(body, closure_list);
        }
        Stmt::Cond(ast::Cond { then, or_else, .. }) => {
            apply_closures_in_stmt(then, closure_list);

            if let Some(or_else) = or_else {
                apply_closures_in_stmt(or_else, closure_list);
            }
        }
        Stmt::While(ast::While { cond, body }) => {
            apply_closures_in_expr(cond, closure_list);
            apply_closures_in_stmt(body, closure_list);
        }
        Stmt::Block(Block(Ast(block))) => block
            .iter_mut()
            .for_each(|stmt| apply_closures_in_stmt(stmt, closure_list)),
        Stmt::Return(Return(Some(expr))) => apply_closures_in_expr(expr, closure_list),
        Stmt::Return(Return(None)) => {}
        Stmt::Continue(_) => {}
        Stmt::Break(_) => {}
    }
}

fn apply_closures_in_expr(stmt: &mut ast::Expr, closure_list: &ClosureList) {
    use ast::*;

    match stmt {
        Expr::Variable(Variable {
            uid,
            is_captured_var,
            ..
        }) => {
            if closure_list.is_captured_var_ref(*uid) {
                *is_captured_var = true;
            }
        }
        Expr::Call(Call { args, callee }) => {
            args.iter_mut()
                .for_each(|arg| apply_closures_in_expr(arg, closure_list));
            apply_closures_in_expr(callee, closure_list);
        }
        Expr::Access(Access { index, subscripted }) => {
            apply_closures_in_expr(index, closure_list);
            apply_closures_in_expr(subscripted, closure_list);
        }
        Expr::Assign(Assign { name, value }) => {
            apply_closures_in_expr(name, closure_list);
            apply_closures_in_expr(value, closure_list);
        }
        Expr::List(List { elements }) => {
            elements
                .iter_mut()
                .for_each(|e| apply_closures_in_expr(e, closure_list));
        }
        Expr::Binary(BinaryOp { lhs, rhs, .. }) => {
            apply_closures_in_expr(lhs, closure_list);
            apply_closures_in_expr(rhs, closure_list);
        }
        Expr::Unary(UnaryOp { rhs, .. }) => apply_closures_in_expr(rhs, closure_list),
        Expr::Grouping(Grouping { expr }) => apply_closures_in_expr(expr, closure_list),
        Expr::Literal(_) => {}
    }
}

fn get_closures_from_ast(ast: &Ast) -> Vec<Closure> {
    use ast::*;

    let Ast(ast) = ast;

    let iter = ast.iter().enumerate();
    let iter = iter.flat_map(|(i, stmt)| match stmt {
        Stmt::Decl(decl) => {
            let name = decl.get_name();

            let closures_from_fn_body = match decl {
                Decl::Function(FunctionDecl { body, .. }) => match body.as_ref() {
                    Stmt::Block(Block(block)) => get_closures_from_ast(block),
                    _ => unreachable!(),
                },
                _ => vec![],
            };

            let closures_from_fn_args = match decl {
                Decl::Function(FunctionDecl { body, params, .. }) => params
                    .iter()
                    .map(|param| (param, get_closures_from_stmt(body, param)))
                    .flat_map(|(param, closures)| {
                        closures.into_iter().map(|(var_ref, fn_decl)| {
                            Closure::new(fn_decl, var_ref, decl.get_uid(), param.to_string())
                        })
                    })
                    .collect(),
                _ => vec![],
            };

            ast[i + 1..]
                .iter()
                .flat_map(|sibling| get_closures_from_stmt(sibling, name))
                .map(|(var_ref, fn_decl)| {
                    Closure::new(fn_decl, var_ref, decl.get_uid(), name.to_string())
                })
                .chain(closures_from_fn_body)
                .chain(closures_from_fn_args)
                .collect()
        }
        Stmt::Cond(Cond { then, or_else, .. }) => {
            let mut closure_list = vec![];

            if let Stmt::Block(Block(then)) = then.as_ref() {
                closure_list.extend(get_closures_from_ast(then));
            }

            if let Some(Stmt::Block(Block(or_else))) = or_else.as_deref() {
                closure_list.extend(get_closures_from_ast(or_else));
            }

            closure_list
        }
        Stmt::Block(Block(block)) => get_closures_from_ast(block),
        _ => vec![],
    });

    iter.collect()
}

fn get_closures_from_stmt(stmt: &ast::Stmt, name: &str) -> Vec<ClosureVarRef> {
    use ast::*;

    match stmt {
        Stmt::Decl(decl) if decl.get_name() == name => vec![],
        Stmt::Decl(ast::Decl::Function(ast::FunctionDecl { body, uid, .. })) => {
            get_stmt_var_refs(body, name, *uid)
        }
        Stmt::Cond(ast::Cond { then, or_else, .. }) => {
            let mut references = get_closures_from_stmt(then, name);

            if let Some(or_else) = or_else {
                references.extend(get_closures_from_stmt(or_else, name));
            }

            references
        }
        Stmt::Block(ast::Block(ast::Ast(block))) => block
            .iter()
            .flat_map(|stmt| get_closures_from_stmt(stmt, name))
            .collect::<Vec<_>>(),
        _ => vec![],
    }
}

fn get_stmt_var_refs(stmt: &ast::Stmt, name: &str, closure_fn: usize) -> Vec<ClosureVarRef> {
    use ast::*;

    match stmt {
        Stmt::Expr(expr) => get_expr_var_refs(expr, name)
            .iter()
            .map(|expr| (*expr, closure_fn))
            .collect::<Vec<_>>(),
        Stmt::Decl(decl) if decl.get_name() == name => vec![],
        Stmt::Decl(Decl::Function(ast::FunctionDecl { body, uid, .. })) => {
            get_stmt_var_refs(body, name, *uid)
        }
        Stmt::Decl(Decl::Local(ast::LocalDecl { value, .. })) => get_expr_var_refs(value, name)
            .into_iter()
            .map(|expr| (expr, closure_fn))
            .collect::<Vec<_>>(),
        Stmt::Cond(ast::Cond {
            cond,
            then,
            or_else,
        }) => {
            let cond_references = get_expr_var_refs(cond, name)
                .into_iter()
                .map(|expr| (expr, closure_fn));

            let then_references = get_stmt_var_refs(then, name, closure_fn);

            let or_else_references = match or_else {
                Some(or_else) => get_stmt_var_refs(or_else, name, closure_fn),
                None => vec![],
            };

            cond_references
                .chain(then_references)
                .chain(or_else_references)
                .collect::<Vec<_>>()
        }
        Stmt::While(ast::While { cond, body }) => {
            let cond_references = get_expr_var_refs(cond, name)
                .into_iter()
                .map(|expr| (expr, closure_fn));

            let body_references = get_stmt_var_refs(body, name, closure_fn);

            cond_references.chain(body_references).collect::<Vec<_>>()
        }
        Stmt::Block(ast::Block(ast::Ast(block))) => block
            .iter()
            .take_while(|stmt| !matches!(stmt, Stmt::Decl(decl) if decl.get_name() == name))
            .flat_map(|stmt| get_stmt_var_refs(stmt, name, closure_fn))
            .collect::<Vec<_>>(),
        Stmt::Return(ast::Return(Some(expr))) => get_expr_var_refs(expr, name)
            .iter()
            .map(|expr| (*expr, closure_fn))
            .collect::<Vec<_>>(),
        Stmt::Return(ast::Return(None)) => vec![],
        Stmt::Break(_) => vec![],
        Stmt::Continue(_) => vec![],
    }
}

fn get_expr_var_refs(expr: &ast::Expr, name: &str) -> Vec<usize> {
    use ast::Expr::*;

    match expr {
        Binary(ast::BinaryOp { lhs, rhs, .. }) => get_expr_var_refs(lhs, name)
            .into_iter()
            .chain(get_expr_var_refs(rhs, name))
            .collect(),
        Unary(ast::UnaryOp { rhs, .. }) => get_expr_var_refs(rhs, name),
        Call(ast::Call { args, callee }) => args
            .iter()
            .flat_map(|arg| get_expr_var_refs(arg, name))
            .chain(get_expr_var_refs(callee, name))
            .collect::<Vec<_>>(),
        Access(ast::Access { index, subscripted }) => get_expr_var_refs(index, name)
            .into_iter()
            .chain(get_expr_var_refs(subscripted, name))
            .collect(),
        Assign(ast::Assign {
            name: var_name,
            value,
        }) => get_expr_var_refs(var_name, name)
            .into_iter()
            .chain(get_expr_var_refs(value, name))
            .collect(),
        List(ast::List { elements }) => elements
            .iter()
            .flat_map(|e| get_expr_var_refs(e, name))
            .collect(),
        Grouping(ast::Grouping { expr }) => get_expr_var_refs(expr, name),
        Literal(_) => vec![],
        Variable(ast::Variable {
            name: var_name,
            uid,
            ..
        }) => {
            if var_name == name {
                vec![*uid]
            } else {
                vec![]
            }
        }
    }
}
