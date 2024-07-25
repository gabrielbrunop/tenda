use std::collections::HashSet;

use scanner::{
    token::{self, Token, TokenIterator, TokenKind},
    token_iter, token_vec,
};

use crate::{
    ast, closures, parser_err,
    parser_error::{unexpected_eoi, unexpected_token, ParserError, ParserErrorKind},
    scope_tracker::{BlockScope, ScopeTracker},
};

pub struct Parser<'a> {
    tokens: TokenIterator<'a>,
    scope: ScopeTracker,
    uid_counter: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [Token]) -> Parser<'a> {
        Parser {
            tokens: source.into(),
            scope: ScopeTracker::new(),
            uid_counter: 0,
        }
    }

    pub fn parse(&mut self) -> Result<ast::Ast, Vec<ParserError>> {
        let program = self.program()?;

        let mut ast = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Eof => program,
            Some(token) => Err(vec![unexpected_token!(token)])?,
            None => Err(vec![unexpected_eoi!(self)])?,
        };

        closures::apply_closures_in_ast(&mut ast);

        Ok(ast)
    }

    fn program(&mut self) -> Result<ast::Ast, Vec<ParserError>> {
        let mut stmt_list = vec![];
        let mut errors: Vec<ParserError> = vec![];

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::Eof {
                break;
            }

            match self.statement() {
                Ok(stmt) => stmt_list.push(stmt),
                Err(e) => e.into_iter().for_each(|err| errors.push(err)),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(stmt_list.into())
        }
    }

    fn statement(&mut self) -> Result<ast::Stmt, Vec<ParserError>> {
        let token = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Newline => {
                self.tokens.next().unwrap();
                return self.statement();
            }
            Some(token) => token,
            _ => unreachable!(),
        };

        let result = match token.kind {
            TokenKind::Let => self.declaration().map_err(|err| vec![err]),
            TokenKind::If => self.if_statement(),
            TokenKind::Function => self.function_declaration(),
            TokenKind::Return => self.return_statement().map_err(|err| vec![err]),
            _ => self
                .expression()
                .map_err(|err| vec![err])
                .map(ast::Stmt::Expr),
        }?;

        self.consume_newline().map_err(|err| vec![err])?;

        Ok(result)
    }

    fn block(
        &mut self,
        end_token_types: Vec<TokenKind>,
        scope: BlockScope,
    ) -> Result<(ast::Stmt, TokenKind), Vec<ParserError>> {
        let _guard = self.scope.guard(scope);
        let mut ast = ast::Ast::new();

        self.consume_newline().ok();

        let block_end_delimiter = loop {
            let token = match self.tokens.peek() {
                Some(token) => token,
                None => break TokenKind::Eof,
            };

            if end_token_types.contains(&token.kind) {
                break token.kind;
            }

            match self.statement() {
                Ok(stmt) => {
                    let ast::Ast(ast) = &mut ast;
                    ast.push(stmt);
                }
                Err(e) => return Err(e),
            };
        };

        if block_end_delimiter == TokenKind::Eof {
            return Err(vec![unexpected_eoi!(self)]);
        }

        self.consume_newline().ok();
        self.skip_token(block_end_delimiter).map_err(|e| vec![e])?;

        Ok((ast::make_block_stmt!(ast), block_end_delimiter))
    }

    fn if_statement(&mut self) -> Result<ast::Stmt, Vec<ParserError>> {
        self.tokens.next();

        let condition = self.expression().map_err(|err| vec![err])?;

        let (then_branch, block_end_delimiter) = match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Then => {
                self.tokens.next();
                self.block(token_vec![BlockEnd, Else], BlockScope::If)?
            }
            Some(token) => return Err(vec![unexpected_token!(token)]),
            None => return Err(vec![unexpected_eoi!(self)]),
        };

        let stmt = match block_end_delimiter {
            TokenKind::Else => {
                self.tokens.next();
                let (else_branch, _) = self.block(token_vec![BlockEnd], BlockScope::Else)?;
                ast::make_cond_stmt!(condition, then_branch, Some(else_branch))
            }
            TokenKind::BlockEnd => ast::make_cond_stmt!(condition, then_branch, None),
            _ => unreachable!(),
        };

        Ok(stmt)
    }

    fn function_declaration(&mut self) -> Result<ast::Stmt, Vec<ParserError>> {
        self.tokens.next();

        let name = self.consume_identifier().map_err(|e| vec![e])?;

        self.skip_token(TokenKind::LeftParen).map_err(|e| vec![e])?;

        let parameters = match self.tokens.match_tokens(token_iter![RightParen]) {
            Some(_) => vec![],
            None => {
                let parameters = self.function_parameters(&name).map_err(|e| vec![e])?;

                if self.tokens.match_tokens(token_iter![RightParen]).is_none() {
                    Err(vec![parser_err!(
                        MissingParentheses,
                        self.tokens.next().unwrap().line,
                        "esperado ')' após declaração de função".to_string()
                    )])?;
                }

                parameters
            }
        };

        let (body, _) = self.block(token_vec![BlockEnd], BlockScope::Function)?;

        Ok(ast::make_function_decl!(
            name.to_string(),
            parameters,
            body,
            self.gen_uid()
        ))
    }

    fn function_parameters(&mut self, function_name: &str) -> Result<Vec<String>, ParserError> {
        let mut parameters = HashSet::new();

        loop {
            let parameter = self.consume_identifier()?;

            if parameters.contains(&parameter) {
                Err(parser_err!(
                    DuplicateParameter(parameter.clone()),
                    self.tokens.get_last_line(),
                    format!(
                        "parâmetro '{}' duplicado na função '{}'",
                        parameter, function_name
                    )
                ))?;
            }

            parameters.insert(parameter);

            if self.tokens.match_tokens(token_iter![Comma]).is_none() {
                break;
            }
        }

        Ok(parameters.into_iter().collect())
    }

    fn declaration(&mut self) -> Result<ast::Stmt, ParserError> {
        self.tokens.next();

        let name = self.consume_identifier()?;

        self.skip_token(TokenKind::EqualSign)?;

        Ok(ast::make_local_decl!(
            name.to_string(),
            self.expression()?,
            self.gen_uid()
        ))
    }

    fn return_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        let return_token = self.tokens.next().unwrap();

        if !self.scope.has_scope(BlockScope::Function) {
            return Err(parser_err!(IllegalReturn, return_token.line));
        }

        let expr = match self.tokens.peek() {
            Some(token) if token.kind != TokenKind::Newline => self.expression(),
            _ => Ok(ast::make_literal_expr!(Nil)),
        }?;

        Ok(ast::make_return_stmt!(Some(expr)))
    }

    fn expression(&mut self) -> Result<ast::Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<ast::Expr, ParserError> {
        let expr = self.logical()?;

        if let Some(equal_sign) = self.tokens.match_tokens(token_iter![EqualSign]) {
            let value = self.assignment()?;

            return match expr {
                ast::Expr::Variable(ast::Variable { name, .. }) => {
                    let name: ast::Expr = ast::make_literal_expr!(String(name));

                    Ok(ast::make_binary_expr!(
                        name,
                        ast::BinaryOperator::Assignment,
                        value
                    ))
                }
                _ => Err(parser_err!(
                    InvalidAssignmentTarget(equal_sign.clone_ref()),
                    equal_sign.line
                )),
            };
        }

        Ok(expr)
    }

    fn logical(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.equality()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Or, And]) {
            let lhs = expr;
            let rhs = self.equality()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.comparison()?;

        loop {
            let op: Option<ast::BinaryOperator> = {
                if let Some(token) = self.tokens.match_tokens(token_iter![Equals]) {
                    Some(token.into())
                } else if self.tokens.matches_sequence(token_iter![Not, Equals]) {
                    Some(ast::BinaryOperator::Inequality)
                } else if let Some(token) = self.tokens.match_tokens(token_iter![Not]) {
                    return Err(unexpected_token!(token));
                } else {
                    None
                }
            };

            if let Some(op) = op {
                let lhs = expr;
                let rhs = self.comparison()?;
                expr = ast::make_binary_expr!(lhs, op, rhs);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.term()?;

        while let Some(op) =
            self.tokens
                .match_tokens(token_iter![Greater, GreaterOrEqual, Less, LessOrEqual])
        {
            let lhs = expr;
            let rhs = self.term()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.factor()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Plus, Minus]) {
            let lhs = expr;
            let rhs = self.factor()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.exponent()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Star, Slash, Percent]) {
            let lhs = expr;
            let rhs = self.exponent()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn exponent(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.unary()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Caret]) {
            let lhs = expr;
            let rhs = self.unary()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<ast::Expr, ParserError> {
        if let Some(op) = self.tokens.match_tokens(token_iter![Minus, Not]) {
            let rhs = self.unary()?;
            let expr = ast::make_unary_expr!(op.into(), rhs);

            Ok(expr)
        } else {
            self.function_call()
        }
    }

    fn function_call(&mut self) -> Result<ast::Expr, ParserError> {
        let name = self.primary()?;

        if self.tokens.match_tokens(token_iter![LeftParen]).is_none() {
            return Ok(name);
        }

        let mut arguments = vec![];

        if self.tokens.match_tokens(token_iter![RightParen]).is_none() {
            loop {
                arguments.push(self.expression()?);

                if self.tokens.match_tokens(token_iter![Comma]).is_none() {
                    break;
                }
            }

            if self.tokens.match_tokens(token_iter![RightParen]).is_none() {
                Err(parser_err!(
                    MissingParentheses,
                    self.tokens.next().unwrap().line,
                    "esperado ')' após chamada de função".to_string()
                ))?;
            }
        }

        Ok(ast::make_call_expr!(name, arguments))
    }

    fn primary(&mut self) -> Result<ast::Expr, ParserError> {
        use TokenKind::*;

        let token = match self.tokens.next() {
            Some(token) => token,
            _ => unreachable!(),
        };

        let line = token.line;

        match token.kind {
            Number | True | False | String | Nil => {
                Ok(ast::make_literal_expr!(token.literal.clone().unwrap()))
            }
            LeftBracket => {
                let _guard = self.tokens.set_ignoring_newline();
                self.list()
            }
            LeftParen => {
                let _guard = self.tokens.set_ignoring_newline();
                let expr = self.expression()?;

                if self.tokens.match_tokens(token_iter![RightParen]).is_none() {
                    return Err(parser_err!(MissingParentheses, line));
                }

                Ok(ast::make_grouping_expr!(expr))
            }
            Identifier => {
                let name = match token.literal.as_ref().unwrap() {
                    token::Literal::String(string) => string,
                    _ => unreachable!(),
                };

                Ok(ast::make_variable_expr!(name.clone(), self.gen_uid()))
            }
            Eof => Err(parser_err!(UnexpectedEoi, line)),
            _ => Err(unexpected_token!(token)),
        }
    }

    fn list(&mut self) -> Result<ast::Expr, ParserError> {
        if let Some(_) = self.tokens.match_tokens(token_iter![RightBracket]) {
            return Ok(ast::make_list_expr!(vec![]));
        }

        let mut elements = vec![];

        loop {
            elements.push(self.expression()?);

            if self.tokens.match_tokens(token_iter![Comma]).is_none() {
                break;
            }
        }

        let next_token_is_bracket = self
            .tokens
            .match_tokens(token_iter![RightBracket])
            .is_some();

        if !next_token_is_bracket {
            Err(parser_err!(
                MissingBrackets,
                self.tokens.next().unwrap().line,
                "esperado ']' ao final de lista".to_string()
            ))?
        }

        Ok(ast::make_list_expr!(elements))
    }
}

impl<'a> Parser<'a> {
    fn consume_newline(&mut self) -> Result<(), ParserError> {
        use TokenKind::*;

        if self.tokens.match_tokens(token_iter![Newline]).is_some() {
            return Ok(());
        }

        match self.tokens.peek() {
            Some(token) if matches!(token.kind, Eof | BlockEnd) => Ok(()),
            Some(token) => Err(unexpected_token!(token)),
            None => Err(parser_err!(UnexpectedEoi, self.tokens.get_last_line())),
        }
    }

    fn consume_identifier(&mut self) -> Result<String, ParserError> {
        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Identifier => {
                match token.literal.as_ref().unwrap() {
                    token::Literal::String(string) => Ok(string.to_string()),
                    _ => unreachable!(),
                }
            }
            Some(token) => Err(unexpected_token!(token)),
            None => Err(parser_err!(UnexpectedEoi, self.tokens.get_last_line())),
        }
    }

    fn skip_token(&mut self, token_kind: TokenKind) -> Result<(), ParserError> {
        match self.tokens.next() {
            Some(token) if token.kind == token_kind => Ok(()),
            Some(token) if token.kind == TokenKind::Eof => {
                Err(parser_err!(UnexpectedEoi, token.line))
            }
            Some(token) => Err(unexpected_token!(token)),
            None => Err(parser_err!(UnexpectedEoi, self.tokens.get_last_line())),
        }
    }

    fn gen_uid(&mut self) -> usize {
        self.uid_counter += 1;
        self.uid_counter
    }
}
