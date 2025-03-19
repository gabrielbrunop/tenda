use crate::ast::{self, Ast};

type VarRefId = usize;
type InnerFnDeclId = usize;
type FreeVarRef = (VarRefId, InnerFnDeclId);

#[derive(Debug)]
pub struct VarCapture {
    pub inner_fn_id: usize,
    pub free_variable_id: usize,
    pub enclosed_var_id: usize,
    pub var_name: String,
}

impl VarCapture {
    pub fn new(
        inner_fn_id: usize,
        free_variable_id: usize,
        enclosed_var_id: usize,
        var_name: String,
    ) -> Self {
        VarCapture {
            inner_fn_id,
            free_variable_id,
            enclosed_var_id,
            var_name,
        }
    }
}

#[derive(Debug)]
struct VarCaptureList(Vec<VarCapture>);

impl VarCaptureList {
    pub fn new(closures: Vec<VarCapture>) -> Self {
        VarCaptureList(closures)
    }

    pub fn is_enclosed_var_decl(&self, var_decl: usize) -> bool {
        self.0
            .iter()
            .any(|closure| closure.enclosed_var_id == var_decl)
    }

    pub fn is_free_variable_ref(&self, var_ref: usize) -> bool {
        self.0
            .iter()
            .any(|closure| closure.free_variable_id == var_ref)
    }

    pub fn get_free_vars_in_fn(&self, func_decl: usize) -> Vec<String> {
        self.0
            .iter()
            .filter(|var_capture| var_capture.inner_fn_id == func_decl)
            .map(|var_capture| var_capture.var_name.clone())
            .collect()
    }
}

pub fn annotate_ast_with_var_captures(ast: &mut Ast) {
    use ast::*;

    let closure_list = get_var_captures_from_ast(ast);
    let closure_list = VarCaptureList::new(closure_list);

    let Ast(ast) = ast;

    for stmt in ast.iter_mut() {
        annotate_stmt_with_var_captures(stmt, &closure_list);
    }
}

fn annotate_stmt_with_var_captures(stmt: &mut ast::Stmt, closure_list: &VarCaptureList) {
    use ast::*;

    match stmt {
        Stmt::Expr(expr) => annotate_expr_with_var_captures(expr, closure_list),
        Stmt::Decl(Decl::Local(LocalDecl {
            uid,
            captured: is_captured_var,
            ..
        })) => {
            if closure_list.is_enclosed_var_decl(*uid) {
                *is_captured_var = true;
            }
        }
        Stmt::Decl(ast::Decl::Function(ast::FunctionDecl {
            body,
            captured: is_captured_var,
            free_vars,
            uid,
            params,
            ..
        })) => {
            if closure_list.is_enclosed_var_decl(*uid) {
                *is_captured_var = true;
            }

            let mut free_vars_in_fn = closure_list.get_free_vars_in_fn(*uid);
            free_vars_in_fn.dedup();

            *free_vars = free_vars_in_fn;

            annotate_stmt_with_var_captures(body, closure_list);

            for param in params {
                if closure_list.is_enclosed_var_decl(param.uid) {
                    param.captured = true;
                }
            }
        }
        Stmt::Cond(ast::Cond { then, or_else, .. }) => {
            annotate_stmt_with_var_captures(then, closure_list);

            if let Some(or_else) = or_else {
                annotate_stmt_with_var_captures(or_else, closure_list);
            }
        }
        Stmt::While(ast::While { cond, body }) => {
            annotate_expr_with_var_captures(cond, closure_list);
            annotate_stmt_with_var_captures(body, closure_list);
        }
        Stmt::ForEach(ast::ForEach {
            iterable,
            body,
            item,
            ..
        }) => {
            annotate_expr_with_var_captures(iterable, closure_list);
            annotate_stmt_with_var_captures(body, closure_list);

            if closure_list.is_enclosed_var_decl(item.uid) {
                item.captured = true;
            }
        }
        Stmt::Block(Block(Ast(block))) => block
            .iter_mut()
            .for_each(|stmt| annotate_stmt_with_var_captures(stmt, closure_list)),
        Stmt::Return(Return(Some(expr))) => annotate_expr_with_var_captures(expr, closure_list),
        Stmt::Return(Return(None)) => {}
        Stmt::Continue(_) => {}
        Stmt::Break(_) => {}
    }
}

fn annotate_expr_with_var_captures(expr: &mut ast::Expr, closure_list: &VarCaptureList) {
    use ast::*;

    match expr {
        Expr::Variable(Variable {
            uid,
            captured: is_captured_var,
            ..
        }) => {
            if closure_list.is_free_variable_ref(*uid) {
                *is_captured_var = true;
            }
        }
        Expr::Call(Call { args, callee }) => {
            args.iter_mut()
                .for_each(|arg| annotate_expr_with_var_captures(arg, closure_list));
            annotate_expr_with_var_captures(callee, closure_list);
        }
        Expr::Access(Access { index, subscripted }) => {
            annotate_expr_with_var_captures(index, closure_list);
            annotate_expr_with_var_captures(subscripted, closure_list);
        }
        Expr::Assign(Assign { name, value }) => {
            annotate_expr_with_var_captures(name, closure_list);
            annotate_expr_with_var_captures(value, closure_list);
        }
        Expr::List(List { elements }) => {
            elements
                .iter_mut()
                .for_each(|e| annotate_expr_with_var_captures(e, closure_list));
        }
        Expr::AssociativeArray(AssociativeArray { elements }) => {
            elements
                .iter_mut()
                .for_each(|(_, value)| annotate_expr_with_var_captures(value, closure_list));
        }
        Expr::Binary(BinaryOp { lhs, rhs, .. }) => {
            annotate_expr_with_var_captures(lhs, closure_list);
            annotate_expr_with_var_captures(rhs, closure_list);
        }
        Expr::Unary(UnaryOp { rhs, .. }) => annotate_expr_with_var_captures(rhs, closure_list),
        Expr::Grouping(Grouping { expr }) => annotate_expr_with_var_captures(expr, closure_list),
        Expr::Literal(_) => {}
    }
}

fn get_var_captures_from_ast(ast: &Ast) -> Vec<VarCapture> {
    use ast::*;

    let Ast(ast) = ast;

    let iter = ast.iter().enumerate();
    let iter = iter.flat_map(|(i, stmt)| match stmt {
        Stmt::Decl(decl) => {
            let name = decl.get_name();

            ast[i + 1..]
                .iter()
                .flat_map(|sibling| get_free_vars_in_statement(sibling, name))
                .map(|(var_ref_id, inner_fn_decl_id)| {
                    VarCapture::new(
                        inner_fn_decl_id,
                        var_ref_id,
                        decl.get_uid(),
                        name.to_string(),
                    )
                })
                .chain(get_var_captures_from_fn_body(decl))
                .chain(get_var_captures_from_fn_args(decl))
                .collect()
        }
        Stmt::Cond(Cond { then, or_else, .. }) => {
            let mut closure_list = vec![];

            if let Stmt::Block(Block(then)) = then.as_ref() {
                closure_list.extend(get_var_captures_from_ast(then));
            }

            if let Some(Stmt::Block(Block(or_else))) = or_else.as_deref() {
                closure_list.extend(get_var_captures_from_ast(or_else));
            }

            closure_list
        }
        Stmt::While(While { body, .. }) => match body.as_ref() {
            Stmt::Block(Block(block)) => get_var_captures_from_ast(block),
            _ => vec![],
        },
        Stmt::ForEach(ForEach { body, item, .. }) => {
            let var_captures_in_body = match body.as_ref() {
                Stmt::Block(Block(block)) => get_var_captures_from_ast(block),
                _ => vec![],
            };

            let body = match body.as_ref() {
                Stmt::Block(Block(Ast(block))) => block,
                _ => unreachable!(),
            };

            let var_captures_from_item = body
                .iter()
                .flat_map(|sibling| get_free_vars_in_statement(sibling, &item.name))
                .map(|(var_ref_id, inner_fn_decl_id)| {
                    VarCapture::new(
                        inner_fn_decl_id,
                        var_ref_id,
                        item.uid,
                        item.name.to_string(),
                    )
                });

            var_captures_in_body
                .into_iter()
                .chain(var_captures_from_item)
                .collect()
        }
        Stmt::Block(Block(block)) => get_var_captures_from_ast(block),
        _ => vec![],
    });

    iter.collect()
}

fn get_var_captures_from_fn_args(decl: &ast::Decl) -> Vec<VarCapture> {
    use ast::*;
    match decl {
        Decl::Function(FunctionDecl { body, params, .. }) => params
            .iter()
            .flat_map(|FunctionParam { name, uid, .. }| {
                get_free_vars_in_statement(body, name).into_iter().map(
                    |(var_ref_id, inner_fn_decl_id)| {
                        VarCapture::new(inner_fn_decl_id, var_ref_id, *uid, name.to_string())
                    },
                )
            })
            .collect(),
        _ => vec![],
    }
}

fn get_var_captures_from_fn_body(decl: &ast::Decl) -> Vec<VarCapture> {
    use ast::*;
    match decl {
        Decl::Function(FunctionDecl { body, .. }) => match body.as_ref() {
            Stmt::Block(Block(block)) => get_var_captures_from_ast(block),
            _ => unreachable!(),
        },
        _ => vec![],
    }
}

fn get_free_vars_in_statement(stmt: &ast::Stmt, name: &str) -> Vec<FreeVarRef> {
    use ast::*;

    match stmt {
        Stmt::Decl(decl) if decl.get_name() == name => vec![],
        Stmt::Decl(ast::Decl::Function(ast::FunctionDecl { body, uid, .. })) => {
            get_free_vars_in_fn_body(body, name, *uid)
        }
        Stmt::Cond(ast::Cond { then, or_else, .. }) => {
            let mut references = get_free_vars_in_statement(then, name);

            if let Some(or_else) = or_else {
                references.extend(get_free_vars_in_statement(or_else, name));
            }

            references
        }
        Stmt::While(ast::While { body, .. }) => get_free_vars_in_statement(body, name),
        Stmt::ForEach(ast::ForEach { body, .. }) => get_free_vars_in_statement(body, name),
        Stmt::Block(ast::Block(ast::Ast(block))) => block
            .iter()
            .flat_map(|stmt| get_free_vars_in_statement(stmt, name))
            .collect::<Vec<_>>(),
        _ => vec![],
    }
}

fn get_free_vars_in_fn_body(stmt: &ast::Stmt, name: &str, closure_fn: usize) -> Vec<FreeVarRef> {
    use ast::*;

    match stmt {
        Stmt::Expr(expr) => get_var_refs_in_expr(expr, name)
            .iter()
            .map(|expr| (*expr, closure_fn))
            .collect::<Vec<_>>(),
        Stmt::Decl(decl) if decl.get_name() == name => vec![],
        Stmt::Decl(Decl::Function(ast::FunctionDecl { body, uid, .. })) => {
            get_free_vars_in_fn_body(body, name, *uid)
        }
        Stmt::Decl(Decl::Local(ast::LocalDecl { value, .. })) => get_var_refs_in_expr(value, name)
            .into_iter()
            .map(|expr| (expr, closure_fn))
            .collect::<Vec<_>>(),
        Stmt::Cond(ast::Cond {
            cond,
            then,
            or_else,
        }) => {
            let cond_references = get_var_refs_in_expr(cond, name)
                .into_iter()
                .map(|expr| (expr, closure_fn));

            let then_references = get_free_vars_in_fn_body(then, name, closure_fn);

            let or_else_references = match or_else {
                Some(or_else) => get_free_vars_in_fn_body(or_else, name, closure_fn),
                None => vec![],
            };

            cond_references
                .chain(then_references)
                .chain(or_else_references)
                .collect::<Vec<_>>()
        }
        Stmt::While(ast::While { cond, body }) => {
            let cond_references = get_var_refs_in_expr(cond, name)
                .into_iter()
                .map(|expr| (expr, closure_fn));

            let body_references = get_free_vars_in_fn_body(body, name, closure_fn);

            cond_references.chain(body_references).collect::<Vec<_>>()
        }
        Stmt::ForEach(ast::ForEach { iterable, body, .. }) => {
            let iterable_references = get_var_refs_in_expr(iterable, name)
                .into_iter()
                .map(|expr| (expr, closure_fn));

            let body_references = get_free_vars_in_fn_body(body, name, closure_fn);

            iterable_references
                .chain(body_references)
                .collect::<Vec<_>>()
        }
        Stmt::Block(ast::Block(ast::Ast(block))) => block
            .iter()
            .take_while(|stmt| !matches!(stmt, Stmt::Decl(decl) if decl.get_name() == name))
            .flat_map(|stmt| get_free_vars_in_fn_body(stmt, name, closure_fn))
            .collect::<Vec<_>>(),
        Stmt::Return(ast::Return(Some(expr))) => get_var_refs_in_expr(expr, name)
            .iter()
            .map(|expr| (*expr, closure_fn))
            .collect::<Vec<_>>(),
        Stmt::Return(ast::Return(None)) => vec![],
        Stmt::Break(_) => vec![],
        Stmt::Continue(_) => vec![],
    }
}

fn get_var_refs_in_expr(expr: &ast::Expr, name: &str) -> Vec<usize> {
    use ast::Expr::*;

    match expr {
        Binary(ast::BinaryOp { lhs, rhs, .. }) => get_var_refs_in_expr(lhs, name)
            .into_iter()
            .chain(get_var_refs_in_expr(rhs, name))
            .collect(),
        Unary(ast::UnaryOp { rhs, .. }) => get_var_refs_in_expr(rhs, name),
        Call(ast::Call { args, callee }) => args
            .iter()
            .flat_map(|arg| get_var_refs_in_expr(arg, name))
            .chain(get_var_refs_in_expr(callee, name))
            .collect::<Vec<_>>(),
        Access(ast::Access { index, subscripted }) => get_var_refs_in_expr(index, name)
            .into_iter()
            .chain(get_var_refs_in_expr(subscripted, name))
            .collect(),
        Assign(ast::Assign {
            name: var_name,
            value,
        }) => get_var_refs_in_expr(var_name, name)
            .into_iter()
            .chain(get_var_refs_in_expr(value, name))
            .collect(),
        List(ast::List { elements }) => elements
            .iter()
            .flat_map(|e| get_var_refs_in_expr(e, name))
            .collect(),
        AssociativeArray(ast::AssociativeArray { elements }) => elements
            .iter()
            .flat_map(|(_, value)| get_var_refs_in_expr(value, name))
            .collect(),
        Grouping(ast::Grouping { expr }) => get_var_refs_in_expr(expr, name),
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
