use tenda_common::{
    source::IdentifiedSource,
    span::{SourceSpan, Span},
};
use tenda_scanner::{self, Token, TokenKind};

use crate::{
    ast::{self, FunctionParam},
    closures,
    parser_error::{unexpected_token, ParserError, Result},
    scope_tracker::{BlockScope, ScopeTracker},
    token_iter::TokenIterator,
    token_stream, token_vec,
};

pub struct Parser<'a> {
    tokens: TokenIterator<'a>,
    scope: ScopeTracker,
    uid_counter: usize,
    source_id: IdentifiedSource,
}

impl<'a> Parser<'a> {
    pub fn new(stream: &'a [Token], source_id: IdentifiedSource) -> Parser<'a> {
        Parser {
            tokens: stream.into(),
            scope: ScopeTracker::new(),
            uid_counter: 0,
            source_id,
        }
    }

    pub fn parse(&mut self) -> Result<ast::Ast> {
        let program = self.parse_program()?;

        let mut ast = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Eof => program,
            Some(token) => return Err(vec![unexpected_token!(token)]),
            None => unreachable!(),
        };

        closures::annotate_ast_with_var_captures(&mut ast);

        Ok(ast)
    }

    fn parse_program(&mut self) -> Result<ast::Ast> {
        let mut stmt_list = vec![];
        let mut errors: Vec<ParserError> = vec![];

        let span_start = match self.tokens.peek() {
            Some(token) => token.span.start(),
            None => self.tokens.last_token().span.end(),
        };

        while self.tokens.is_next_valid() {
            match self.parse_statement() {
                Ok(stmt) => stmt_list.push(stmt),
                Err(e) => {
                    errors.extend(e);

                    break;
                }
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            let span_end = stmt_list
                .last()
                .map_or(span_start, |stmt| stmt.get_span().end());

            let span = SourceSpan::new(span_start, span_end, self.source_id);
            let ast = ast::Ast::from(stmt_list, span);

            Ok(ast)
        }
    }

    fn parse_statement(&mut self) -> Result<ast::Stmt> {
        let token = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Newline => {
                self.tokens.advance_while(token_stream![Newline]);

                return self.parse_statement();
            }
            Some(token) => token,
            _ => unreachable!(),
        };

        let result = match token.kind {
            TokenKind::Let => self.parse_declaration(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::ForOrBreak => {
                if self.tokens.check_sequence(token_stream![ForOrBreak, Each]) {
                    self.parse_for_each_statement()
                } else {
                    self.parse_break_statement()
                }
            }
            TokenKind::Function => self.parse_function_declaration(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            _ => self.parse_expression().map(ast::Stmt::Expr),
        }?;

        match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Newline => {
                self.tokens.advance_while(token_stream![Newline]);
            }
            Some(token)
                if matches!(
                    token.kind,
                    TokenKind::Eof | TokenKind::BlockEnd | TokenKind::Else
                ) => {}
            Some(token) => {
                return Err(vec![unexpected_token!(token)]);
            }
            None => unreachable!(),
        }

        Ok(result)
    }

    fn parse_block(
        &mut self,
        end_token_types: Vec<TokenKind>,
        scope: BlockScope,
    ) -> Result<(ast::Stmt, TokenKind)> {
        let span_start = self.tokens.next().unwrap().span.start();

        self.parse_block_contents(end_token_types, scope, Some(span_start))
    }

    fn parse_block_contents(
        &mut self,
        end_token_types: Vec<TokenKind>,
        scope: BlockScope,
        span_start: Option<usize>,
    ) -> Result<(ast::Stmt, TokenKind)> {
        let _guard = self.scope.guard(scope);
        let _newline_guard = self.tokens.halt_ignoring_newline();

        self.tokens.advance_while(token_stream![Newline]);

        let block_first_token_span = match self.tokens.peek() {
            Some(token) => &token.span,
            None => {
                return Err(vec![ParserError::UnexpectedEoi {
                    span: self.tokens.last_token().span.clone(),
                }])
            }
        };

        let span_start = span_start.unwrap_or_else(|| block_first_token_span.start());
        let inner_span_start = block_first_token_span.start();
        let mut current_inner_span_end = block_first_token_span.end();

        let mut stmt_list = vec![];

        let end_token = loop {
            let token = match self.tokens.peek() {
                Some(token) => token,
                None => unreachable!(),
            };

            if end_token_types.contains(&token.kind) {
                break Ok(self.tokens.next().unwrap());
            }

            current_inner_span_end = token.span.end();

            match self.parse_statement() {
                Ok(stmt) => stmt_list.push(stmt),
                Err(e) => break Err(e),
            };
        }?;

        let span_end = end_token.span.end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);
        let inner_span = SourceSpan::new(inner_span_start, current_inner_span_end, self.source_id);

        let ast = ast::Ast::from(stmt_list, inner_span);
        let block_stmt = ast::Block::new(ast, span);
        let block_stmt = ast::Stmt::Block(block_stmt);

        Ok((block_stmt, end_token.kind))
    }

    fn parse_if_statement(&mut self) -> Result<ast::Stmt> {
        let span_start = self.tokens.next().unwrap().span.start();
        let condition = self.parse_expression()?;

        let (then_branch, block_end_delimiter) = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Then => {
                self.parse_block(token_vec![BlockEnd, Else], BlockScope::If)?
            }
            Some(_) => return Err(vec![unexpected_token!(self.tokens.next().unwrap())]),
            None => unreachable!(),
        };

        let else_branch = match block_end_delimiter {
            TokenKind::Else => {
                let (else_branch, _) = self.parse_block(token_vec![BlockEnd], BlockScope::Else)?;
                Some(else_branch)
            }
            TokenKind::BlockEnd => None,
            _ => unreachable!(),
        };

        let span_end = else_branch
            .as_ref()
            .map_or(then_branch.get_span().end(), |else_branch| {
                else_branch.get_span().end()
            });

        let span = SourceSpan::new(span_start, span_end, self.source_id);
        let stmt = ast::Cond::new(condition, then_branch, else_branch, span);

        Ok(ast::Stmt::Cond(stmt))
    }

    fn parse_while_statement(&mut self) -> Result<ast::Stmt> {
        let span_start = self.tokens.next().unwrap().span.start();
        let condition = self.parse_expression()?;

        if !self.tokens.is_next_token(TokenKind::Do) {
            let token = self.tokens.next().unwrap();
            return Err(vec![unexpected_token!(token)]);
        }

        let (body, _) = self.parse_block(token_vec![BlockEnd], BlockScope::Loop)?;

        let span_end = body.get_span().end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let while_stmt = ast::While::new(condition, body, span);
        let while_stmt = ast::Stmt::While(while_stmt);

        Ok(while_stmt)
    }

    fn parse_for_each_statement(&mut self) -> Result<ast::Stmt> {
        let span_start = self.tokens.next().unwrap().span.start();

        self.skip_token(TokenKind::Each)?;

        let (name, name_span) = self.consume_identifier()?;

        self.skip_token(TokenKind::In)?;

        let iterable = self.parse_expression()?;

        if !self.tokens.is_next_token(TokenKind::Do) {
            let token = self.tokens.next().unwrap();

            return Err(vec![unexpected_token!(token)]);
        }

        let (body, _) = self.parse_block(token_vec![BlockEnd], BlockScope::Loop)?;

        let span_end = body.get_span().end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let for_each_item = ast::ForEachItem::new(name, self.gen_uid(), name_span);
        let for_each_stmt = ast::ForEach::new(for_each_item, iterable, body, span);
        let for_each_stmt = ast::Stmt::ForEach(for_each_stmt);

        Ok(for_each_stmt)
    }

    fn parse_function_declaration(&mut self) -> Result<ast::Stmt> {
        let span_start = self.tokens.next().unwrap().span.start();

        let (name, _) = self.consume_identifier()?;
        let parameters = self.parse_function_parameters_signature()?;

        let (body, _) =
            self.parse_block_contents(token_vec![BlockEnd], BlockScope::Function, None)?;

        let span_end = body.get_span().end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let function_decl = ast::FunctionDecl::new(name, parameters, body, self.gen_uid(), span);
        let function_decl = ast::Decl::Function(function_decl);
        let function_decl = ast::Stmt::Decl(function_decl);

        Ok(function_decl)
    }

    fn parse_declaration(&mut self) -> Result<ast::Stmt> {
        let span_start = self.tokens.next().unwrap().span.start();
        let (name, _) = self.consume_identifier()?;

        self.skip_token(TokenKind::EqualSign)?;

        let expr = self.parse_expression()?;

        let span_end = expr.get_span().end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let local_decl = ast::LocalDecl::new(name, expr, self.gen_uid(), span);
        let local_decl = ast::Decl::Local(local_decl);

        Ok(ast::Stmt::Decl(local_decl))
    }

    fn parse_return_statement(&mut self) -> Result<ast::Stmt> {
        let return_token = self.tokens.next().unwrap();

        if !self.scope.has_scope(BlockScope::Function) {
            return Err(vec![ParserError::IllegalReturn {
                span: return_token.span.clone(),
            }]);
        }

        let expr = match self.tokens.peek() {
            Some(token) if token.kind != TokenKind::Newline => Some(self.parse_expression()?),
            _ => None,
        };

        let span_start = return_token.span.start();
        let span_end = expr
            .as_ref()
            .map_or(return_token.span.end(), |expr| expr.get_span().end());
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let return_stmt = ast::Return::new(expr, span);

        Ok(ast::Stmt::Return(return_stmt))
    }

    fn parse_break_statement(&mut self) -> Result<ast::Stmt> {
        let break_token = self.tokens.next().unwrap();

        if !self.scope.has_scope(BlockScope::Loop) {
            return Err(vec![ParserError::IllegalBreak {
                span: break_token.span.clone(),
            }]);
        }

        let break_stmt = ast::Break::new(break_token.span.clone());
        let break_stmt = ast::Stmt::Break(break_stmt);

        Ok(break_stmt)
    }

    fn parse_continue_statement(&mut self) -> Result<ast::Stmt> {
        let continue_token = self.tokens.next().unwrap();

        if !self.scope.has_scope(BlockScope::Loop) {
            return Err(vec![ParserError::IllegalContinue {
                span: continue_token.span.clone(),
            }]);
        }

        let continue_stmt = ast::Continue::new(continue_token.span.clone());
        let continue_stmt = ast::Stmt::Continue(continue_stmt);

        Ok(continue_stmt)
    }

    fn parse_expression(&mut self) -> Result<ast::Expr> {
        let _guard = self.tokens.set_ignoring_newline();

        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<ast::Expr> {
        let expr = self.parse_logical_or()?;

        if let Some(equal_sign) = self.tokens.consume_one_of(token_stream![EqualSign]) {
            let value = self.parse_assignment()?;

            return match expr {
                ast::Expr::Variable(_) | ast::Expr::Access(_) => {
                    let span_start = expr.get_span().start();
                    let span_end = value.get_span().end();
                    let span = SourceSpan::new(span_start, span_end, self.source_id);

                    let assign_expr = ast::Assign::new(expr, value, span);
                    let assign_expr = ast::Expr::Assign(assign_expr);

                    Ok(assign_expr)
                }
                _ => Err(vec![ParserError::InvalidAssignmentTarget {
                    span: equal_sign.span.clone(),
                    token: equal_sign.clone_ref(),
                }]),
            };
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<ast::Expr> {
        let mut expr = self.parse_logical_and()?;

        while let Some(op) = self.tokens.consume_one_of(token_stream![Or]) {
            let lhs = expr;
            let rhs = self.parse_logical_and()?;

            let span_start = lhs.get_span().start();
            let span_end = rhs.get_span().end();
            let span = SourceSpan::new(span_start, span_end, self.source_id);

            let binary_op = ast::BinaryOp::new(lhs, op.into(), rhs, span);

            expr = ast::Expr::Binary(binary_op);
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<ast::Expr> {
        let mut expr = self.parse_equality()?;

        while let Some(op) = self.tokens.consume_one_of(token_stream![And]) {
            let lhs = expr;
            let rhs = self.parse_equality()?;

            let span_start = lhs.get_span().start();
            let span_end = rhs.get_span().end();
            let span = SourceSpan::new(span_start, span_end, self.source_id);

            let binary_op = ast::BinaryOp::new(lhs, op.into(), rhs, span);

            expr = ast::Expr::Binary(binary_op);
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<ast::Expr> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op: Option<ast::BinaryOperator> = {
                if self.tokens.consume_one_of(token_stream![Equals]).is_some() {
                    Some(ast::BinaryOperator::Equality)
                } else if self.tokens.consume_one_of(token_stream![Has]).is_some() {
                    Some(ast::BinaryOperator::Has)
                } else if self
                    .tokens
                    .consume_sequence(token_stream![Not, Has])
                    .is_some()
                {
                    Some(ast::BinaryOperator::Lacks)
                } else if self
                    .tokens
                    .consume_sequence(token_stream![Not, Equals])
                    .is_some()
                {
                    Some(ast::BinaryOperator::Inequality)
                } else {
                    None
                }
            };

            if let Some(op) = op {
                let lhs = expr;
                let rhs = self.parse_comparison()?;

                let span_start = lhs.get_span().start();
                let span_end = rhs.get_span().end();
                let span = SourceSpan::new(span_start, span_end, self.source_id);

                let binary_op = ast::BinaryOp::new(lhs, op, rhs, span);

                expr = ast::Expr::Binary(binary_op);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<ast::Expr> {
        let mut expr = self.parse_range()?;

        while let Some(op) =
            self.tokens
                .consume_one_of(token_stream![Greater, GreaterOrEqual, Less, LessOrEqual])
        {
            let lhs = expr;
            let rhs = self.parse_range()?;

            let span_start = lhs.get_span().start();
            let span_end = rhs.get_span().end();
            let span = SourceSpan::new(span_start, span_end, self.source_id);

            let binary_op = ast::BinaryOp::new(lhs, op.into(), rhs, span);

            expr = ast::Expr::Binary(binary_op);
        }

        Ok(expr)
    }

    fn parse_range(&mut self) -> Result<ast::Expr> {
        let lhs = self.parse_term()?;

        if let Some(op) = self.tokens.consume_one_of(token_stream![Until]) {
            let rhs = self.parse_term()?;

            let span_start = lhs.get_span().start();
            let span_end = rhs.get_span().end();
            let span = SourceSpan::new(span_start, span_end, self.source_id);

            let binary_op = ast::BinaryOp::new(lhs, op.into(), rhs, span);

            return Ok(ast::Expr::Binary(binary_op));
        }

        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<ast::Expr> {
        let mut expr = self.parse_factor()?;

        while let Some(op) = self.tokens.consume_one_of(token_stream![Plus, Minus]) {
            let lhs = expr;
            let rhs = self.parse_factor()?;

            let span_start = lhs.get_span().start();
            let span_end = rhs.get_span().end();
            let span = SourceSpan::new(span_start, span_end, self.source_id);

            let binary_op = ast::BinaryOp::new(lhs, op.into(), rhs, span);

            expr = ast::Expr::Binary(binary_op);
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ast::Expr> {
        let mut expr = self.parse_exponent()?;

        while let Some(op) = self
            .tokens
            .consume_one_of(token_stream![Star, Slash, Percent])
        {
            let lhs = expr;
            let rhs = self.parse_exponent()?;

            let span_start = lhs.get_span().start();
            let span_end = rhs.get_span().end();
            let span = SourceSpan::new(span_start, span_end, self.source_id);

            let binary_op = ast::BinaryOp::new(lhs, op.into(), rhs, span);

            expr = ast::Expr::Binary(binary_op);
        }

        Ok(expr)
    }

    fn parse_exponent(&mut self) -> Result<ast::Expr> {
        let mut expr = self.parse_unary()?;

        while let Some(op) = self.tokens.consume_one_of(token_stream![Caret]) {
            let lhs = expr;
            let rhs = self.parse_unary()?;

            let span_start = lhs.get_span().start();
            let span_end = rhs.get_span().end();
            let span = SourceSpan::new(span_start, span_end, self.source_id);

            let binary_op = ast::BinaryOp::new(lhs, op.into(), rhs, span);

            expr = ast::Expr::Binary(binary_op);
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<ast::Expr> {
        if let Some(op) = self.tokens.consume_one_of(token_stream![Minus, Not]) {
            let rhs = self.parse_unary()?;

            let span_start = op.span.start();
            let span_end = rhs.get_span().end();
            let span = SourceSpan::new(span_start, span_end, self.source_id);

            let unary_op = ast::UnaryOp::new(op.into(), rhs, span);

            Ok(ast::Expr::Unary(unary_op))
        } else {
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> Result<ast::Expr> {
        let mut lhs = self.parse_primary()?;

        while let Some(token) = self
            .tokens
            .consume_one_of(token_stream![LeftParen, LeftBracket])
        {
            match token.kind {
                TokenKind::LeftParen => lhs = self.parse_function_call(lhs)?,
                TokenKind::LeftBracket => lhs = self.parse_access(lhs)?,
                _ => unreachable!(),
            }
        }

        Ok(lhs)
    }

    fn parse_function_call(&mut self, name: ast::Expr) -> Result<ast::Expr> {
        let mut arguments = vec![];

        if !self.tokens.is_next_token(TokenKind::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);

                if self.tokens.consume_one_of(token_stream![Comma]).is_none() {
                    break;
                }
            }
        }

        if self.tokens.is_next_eof() {
            return Err(vec![ParserError::UnexpectedEoi {
                span: self.tokens.last_token().span.clone(),
            }]);
        }

        let right_paran = match self.tokens.consume_one_of(token_stream![RightParen]) {
            Some(token) => token,
            _ => {
                return Err(vec![ParserError::MissingParentheses {
                    span: self.tokens.next().unwrap().span.clone(),
                }]);
            }
        };

        let span_start = name.get_span().start();
        let span_end = right_paran.span.end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let call_expr = ast::Call::new(name, arguments, span);
        let call_expr = ast::Expr::Call(call_expr);

        Ok(call_expr)
    }

    fn parse_access(&mut self, name: ast::Expr) -> Result<ast::Expr> {
        let index = self.parse_expression()?;

        if self.tokens.is_next_eof() {
            return Err(vec![ParserError::UnexpectedEoi {
                span: self.tokens.last_token().span.clone(),
            }]);
        }

        let closing_bracket = match self.tokens.consume_one_of(token_stream![RightBracket]) {
            Some(token) => token,
            _ => {
                return Err(vec![ParserError::MissingBrackets {
                    span: self.tokens.next().unwrap().span.clone(),
                }]);
            }
        };

        let span_start = name.get_span().start();
        let span_end = closing_bracket.span.end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let access_expr = ast::Access::new(name, index, span);
        let access_expr = ast::Expr::Access(access_expr);

        Ok(access_expr)
    }

    fn parse_primary(&mut self) -> Result<ast::Expr> {
        use TokenKind::*;

        let token = match self.tokens.peek() {
            Some(token) => token,
            _ => unreachable!(),
        };

        let span = token.span.clone();

        match token.kind {
            Number | True | False | String | Nil => self.parse_literal(),
            Identifier => self.parse_variable(),
            LeftParen => self.parse_grouping(),
            LeftBracket => self.parse_list(),
            LeftBrace => self.parse_associative_array(),
            Function => self.parse_anonymous_function(),
            Eof => {
                self.tokens.next().unwrap();
                Err(vec![ParserError::UnexpectedEoi { span }])
            }
            _ => {
                let token = self.tokens.next().unwrap();
                Err(vec![unexpected_token!(token)])
            }
        }
    }

    fn parse_literal(&mut self) -> Result<ast::Expr> {
        let token = self.tokens.next().unwrap();

        let literal_expr = ast::Literal::new(
            token.literal.as_ref().unwrap().clone(),
            token.clone().span.clone(),
        );

        let literal_expr = ast::Expr::Literal(literal_expr);

        Ok(literal_expr)
    }

    fn parse_variable(&mut self) -> Result<ast::Expr> {
        let token = self.tokens.next().unwrap();
        let span = token.span.clone();

        let name = match token.literal.as_ref().unwrap() {
            tenda_scanner::Literal::String(string) => string,
            _ => unreachable!(),
        };

        let id = self.gen_uid();

        if self.tokens.consume_one_of(token_stream![Dot]).is_some() {
            let variable_expr = ast::Variable::new(name.clone(), id, span);
            let variable_expr = ast::Expr::Variable(variable_expr);

            self.parse_dot_access(variable_expr)
        } else {
            let variable_expr = ast::Variable::new(name.clone(), id, span);

            Ok(ast::Expr::Variable(variable_expr))
        }
    }

    fn parse_grouping(&mut self) -> Result<ast::Expr> {
        let token = self.tokens.next().unwrap();
        let expr = self.parse_expression()?;

        let closing_paren = match self.tokens.consume_one_of(token_stream![RightParen]) {
            Some(token) => token,
            _ => {
                return Err(vec![ParserError::MissingParentheses {
                    span: self.tokens.next().unwrap().span.clone(),
                }])
            }
        };

        let span_start = token.span.start();
        let span_end = closing_paren.span.end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let grouping_expr = ast::Grouping::new(expr, span);
        let grouping_expr = ast::Expr::Grouping(grouping_expr);

        Ok(grouping_expr)
    }

    fn parse_anonymous_function(&mut self) -> Result<ast::Expr> {
        let function_token = self.tokens.next().unwrap();

        let parameters = self.parse_function_parameters_signature()?;
        let (body, _) =
            self.parse_block_contents(token_vec![BlockEnd], BlockScope::Function, None)?;

        let span_start = function_token.span.start();
        let span_end = body.get_span().end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let function_expr = ast::AnonymousFunction::new(parameters, body, self.gen_uid(), span);
        let function_expr = ast::Expr::AnonymousFunction(function_expr);

        Ok(function_expr)
    }

    fn parse_function_parameters_signature(&mut self) -> Result<Vec<ast::FunctionParam>> {
        self.skip_token(TokenKind::LeftParen)?;

        let _guard = self.tokens.set_ignoring_newline();

        if self.tokens.is_next_eof() {
            return Err(vec![ParserError::UnexpectedEoi {
                span: self.tokens.last_token().span.clone(),
            }]);
        }

        let parameters = match self.tokens.consume_one_of(token_stream![RightParen]) {
            Some(_) => vec![],
            None => {
                let parameters = self.parse_function_parameters()?;

                if self
                    .tokens
                    .consume_one_of(token_stream![RightParen])
                    .is_none()
                {
                    return Err(vec![ParserError::MissingParentheses {
                        span: self.tokens.next().unwrap().span.clone(),
                    }]);
                }

                parameters
            }
        };

        Ok(parameters)
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<FunctionParam>> {
        let mut parameters: Vec<FunctionParam> = vec![];

        loop {
            let (param_name, param_span) = self.consume_identifier()?;

            if parameters.iter().any(|p| p.name == param_name) {
                return Err(vec![ParserError::DuplicateParameter {
                    name: param_name.clone(),
                    span: param_span,
                }]);
            }

            parameters.push(FunctionParam::new(param_name, self.gen_uid(), param_span));

            if self.tokens.consume_one_of(token_stream![Comma]).is_none() {
                break;
            }
        }

        Ok(parameters.into_iter().collect())
    }

    fn parse_list(&mut self) -> Result<ast::Expr> {
        let span_start = self.tokens.next().unwrap().span.start();
        let mut elements = vec![];

        if !self.tokens.is_next_token(TokenKind::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);

                if self.tokens.consume_one_of(token_stream![Comma]).is_none() {
                    break;
                }
            }
        }

        if self.tokens.is_next_eof() {
            return Err(vec![ParserError::UnexpectedEoi {
                span: self.tokens.last_token().span.clone(),
            }]);
        }

        let closing_bracket = match self.tokens.consume_one_of(token_stream![RightBracket]) {
            Some(token) => token,
            _ => {
                return Err(vec![ParserError::MissingBrackets {
                    span: self.tokens.next().unwrap().span.clone(),
                }]);
            }
        };

        let span_end = closing_bracket.span.end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let elements = elements.into_iter().collect();
        let list_expr = ast::List::new(elements, span);
        let list_expr = ast::Expr::List(list_expr);

        Ok(list_expr)
    }

    fn parse_associative_array(&mut self) -> Result<ast::Expr> {
        let span_start = self.tokens.next().unwrap().span.start();
        let mut elements = vec![];

        if !self.tokens.is_next_token(TokenKind::RightBrace) {
            loop {
                let key = match self.tokens.consume_one_of(token_stream![Number, String]) {
                    Some(token) => {
                        ast::Literal::new(token.literal.clone().unwrap(), token.span.clone())
                    }
                    None => return Err(vec![unexpected_token!(self.tokens.next().unwrap())]),
                };

                if self.tokens.consume_one_of(token_stream![Colon]).is_none() {
                    return Err(vec![ParserError::MissingColon {
                        span: self.tokens.next().unwrap().span.clone(),
                    }]);
                }

                let value = self.parse_expression()?;

                elements.push((key, value));

                if self.tokens.consume_one_of(token_stream![Comma]).is_none() {
                    break;
                }
            }
        }

        if self.tokens.is_next_eof() {
            return Err(vec![ParserError::UnexpectedEoi {
                span: self.tokens.last_token().span.clone(),
            }]);
        }

        let closing_brace = match self.tokens.consume_one_of(token_stream![RightBrace]) {
            Some(token) => token,
            _ => {
                return Err(vec![ParserError::MissingBraces {
                    span: self.tokens.next().unwrap().span.clone(),
                }]);
            }
        };

        let span_end = closing_brace.span.end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let elements = elements.into_iter().collect();
        let associative_array_expr = ast::AssociativeArray::new(elements, span);
        let associative_array_expr = ast::Expr::AssociativeArray(associative_array_expr);

        Ok(associative_array_expr)
    }

    fn parse_dot_access(&mut self, name: ast::Expr) -> Result<ast::Expr> {
        let (field, field_span) = self.consume_identifier()?;
        let field = tenda_scanner::Literal::String(field);

        let index_expr = ast::Literal::new(field, field_span);
        let index_expr = ast::Expr::Literal(index_expr);

        let span_start = name.get_span().start();
        let span_end = index_expr.get_span().end();
        let span = SourceSpan::new(span_start, span_end, self.source_id);

        let access_expr = ast::Access::new(name, index_expr, span);
        let mut access_expr = ast::Expr::Access(access_expr);

        if self.tokens.consume_one_of(token_stream![Dot]).is_some() {
            access_expr = self.parse_dot_access(access_expr)?;
        }

        Ok(access_expr)
    }
}

impl Parser<'_> {
    fn consume_identifier(&mut self) -> Result<(String, SourceSpan)> {
        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Identifier => {
                match token.literal.as_ref().unwrap() {
                    tenda_scanner::Literal::String(string) => {
                        Ok((string.to_string(), token.span.clone()))
                    }
                    _ => unreachable!(),
                }
            }
            Some(token) => Err(vec![unexpected_token!(token)]),
            None => Err(vec![ParserError::UnexpectedEoi {
                span: self.tokens.last_token().span.clone(),
            }]),
        }
    }

    fn skip_token(&mut self, token_kind: TokenKind) -> Result<&Token> {
        match self.tokens.next() {
            Some(token) if token.kind == token_kind => Ok(token),
            Some(token) if token.kind == TokenKind::Eof => Err(vec![ParserError::UnexpectedEoi {
                span: token.span.clone(),
            }]),
            Some(token) => Err(vec![unexpected_token!(token)]),
            None => Err(vec![ParserError::UnexpectedEoi {
                span: self.tokens.last_token().span.clone(),
            }]),
        }
    }

    fn gen_uid(&mut self) -> usize {
        self.uid_counter += 1;
        self.uid_counter
    }
}
