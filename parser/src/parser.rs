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
        let program = self.parse_program()?;

        let mut ast = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Eof => program,
            Some(token) => Err(vec![unexpected_token!(token)])?,
            None => Err(vec![unexpected_eoi!(self)])?,
        };

        closures::apply_closures_in_ast(&mut ast);

        Ok(ast)
    }

    fn parse_program(&mut self) -> Result<ast::Ast, Vec<ParserError>> {
        let mut stmt_list = vec![];
        let mut errors: Vec<ParserError> = vec![];

        while let Some(token) = self.tokens.peek() {
            if token.kind == TokenKind::Eof {
                break;
            }

            match self.parse_statement() {
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

    fn parse_statement(&mut self) -> Result<ast::Stmt, Vec<ParserError>> {
        let token = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Newline => {
                self.tokens.next().unwrap();
                return self.parse_statement();
            }
            Some(token) => token,
            _ => unreachable!(),
        };

        let result = match token.kind {
            TokenKind::Let => self.parse_declaration().map_err(|err| vec![err]),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Function => self.parse_function_declaration(),
            TokenKind::Return => self.parse_return_statement().map_err(|err| vec![err]),
            TokenKind::Continue => self.parse_continue_statement().map_err(|err| vec![err]),
            TokenKind::Break => self.parse_break_statement().map_err(|err| vec![err]),
            _ => self
                .parse_expression()
                .map_err(|err| vec![err])
                .map(ast::Stmt::Expr),
        }?;

        self.consume_newline().map_err(|err| vec![err])?;

        Ok(result)
    }

    fn parse_block(
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

            match self.parse_statement() {
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

    fn parse_if_statement(&mut self) -> Result<ast::Stmt, Vec<ParserError>> {
        self.tokens.next();

        let condition = self.parse_expression().map_err(|err| vec![err])?;

        let (then_branch, block_end_delimiter) = match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Then => {
                self.tokens.next();
                self.parse_block(token_vec![BlockEnd, Else], BlockScope::If)?
            }
            Some(token) => return Err(vec![unexpected_token!(token)]),
            None => return Err(vec![unexpected_eoi!(self)]),
        };

        let stmt = match block_end_delimiter {
            TokenKind::Else => {
                self.tokens.next();
                let (else_branch, _) = self.parse_block(token_vec![BlockEnd], BlockScope::Else)?;
                ast::make_cond_stmt!(condition, then_branch, Some(else_branch))
            }
            TokenKind::BlockEnd => ast::make_cond_stmt!(condition, then_branch, None),
            _ => unreachable!(),
        };

        Ok(stmt)
    }

    fn parse_while_statement(&mut self) -> Result<ast::Stmt, Vec<ParserError>> {
        self.tokens.next();

        let condition = self.parse_expression().map_err(|err| vec![err])?;

        self.skip_token(TokenKind::Do).map_err(|e| vec![e])?;

        let (body, _) = self.parse_block(token_vec![BlockEnd], BlockScope::Loop)?;

        Ok(ast::make_while_stmt!(condition, body))
    }

    fn parse_function_declaration(&mut self) -> Result<ast::Stmt, Vec<ParserError>> {
        self.tokens.next();

        let name = self.consume_identifier().map_err(|e| vec![e])?;

        self.skip_token(TokenKind::LeftParen).map_err(|e| vec![e])?;

        let guard = self.tokens.set_ignoring_newline();

        let parameters = match self.tokens.match_tokens(token_iter![RightParen]) {
            Some(_) => vec![],
            None => {
                let parameters = self.parse_function_parameters(&name).map_err(|e| vec![e])?;

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

        drop(guard);

        let (body, _) = self.parse_block(token_vec![BlockEnd], BlockScope::Function)?;

        Ok(ast::make_function_decl!(
            name.to_string(),
            parameters,
            body,
            self.gen_uid()
        ))
    }

    fn parse_function_parameters(
        &mut self,
        function_name: &str,
    ) -> Result<Vec<String>, ParserError> {
        let mut parameters: Vec<String> = vec![];

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

            parameters.push(parameter);

            if self.tokens.match_tokens(token_iter![Comma]).is_none() {
                break;
            }
        }

        Ok(parameters.into_iter().collect())
    }

    fn parse_declaration(&mut self) -> Result<ast::Stmt, ParserError> {
        self.tokens.next();

        let name = self.consume_identifier()?;

        self.skip_token(TokenKind::EqualSign)?;

        Ok(ast::make_local_decl!(
            name.to_string(),
            self.parse_expression()?,
            self.gen_uid()
        ))
    }

    fn parse_return_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        let return_token = self.tokens.next().unwrap();

        if !self.scope.has_scope(BlockScope::Function) {
            return Err(parser_err!(IllegalReturn, return_token.line));
        }

        let expr = match self.tokens.peek() {
            Some(token) if token.kind != TokenKind::Newline => self.parse_expression(),
            _ => Ok(ast::make_literal_expr!(Nil)),
        }?;

        Ok(ast::make_return_stmt!(Some(expr)))
    }

    fn parse_break_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        let break_token = self.tokens.next().unwrap();

        if !self.scope.has_scope(BlockScope::Loop) {
            return Err(parser_err!(IllegalBreak, break_token.line));
        }

        Ok(ast::make_break_stmt!())
    }

    fn parse_continue_statement(&mut self) -> Result<ast::Stmt, ParserError> {
        let continue_token = self.tokens.next().unwrap();

        if !self.scope.has_scope(BlockScope::Loop) {
            return Err(parser_err!(IllegalContinue, continue_token.line));
        }

        Ok(ast::make_continue_stmt!())
    }

    fn parse_expression(&mut self) -> Result<ast::Expr, ParserError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<ast::Expr, ParserError> {
        let expr = self.parse_logical()?;

        if let Some(equal_sign) = self.tokens.match_tokens(token_iter![EqualSign]) {
            let value = self.parse_assignment()?;

            return match expr {
                ast::Expr::Variable(_) | ast::Expr::Access(_) => {
                    Ok(ast::make_assign_expr!(expr, value))
                }
                _ => Err(parser_err!(
                    InvalidAssignmentTarget(equal_sign.clone_ref()),
                    equal_sign.line
                )),
            };
        }

        Ok(expr)
    }

    fn parse_logical(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.parse_equality()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Or, And]) {
            let lhs = expr;
            let rhs = self.parse_equality()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.parse_comparison()?;

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
                let rhs = self.parse_comparison()?;
                expr = ast::make_binary_expr!(lhs, op, rhs);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.parse_term()?;

        while let Some(op) =
            self.tokens
                .match_tokens(token_iter![Greater, GreaterOrEqual, Less, LessOrEqual])
        {
            let lhs = expr;
            let rhs = self.parse_term()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.parse_factor()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Plus, Minus]) {
            let lhs = expr;
            let rhs = self.parse_factor()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.parse_exponent()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Star, Slash, Percent]) {
            let lhs = expr;
            let rhs = self.parse_exponent()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn parse_exponent(&mut self) -> Result<ast::Expr, ParserError> {
        let mut expr = self.parse_unary()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Caret]) {
            let lhs = expr;
            let rhs = self.parse_unary()?;
            expr = ast::make_binary_expr!(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<ast::Expr, ParserError> {
        if let Some(op) = self.tokens.match_tokens(token_iter![Minus, Not]) {
            let rhs = self.parse_unary()?;
            let expr = ast::make_unary_expr!(op.into(), rhs);

            Ok(expr)
        } else {
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> Result<ast::Expr, ParserError> {
        let mut lhs = self.parse_primary()?;

        while let Some(token) = self
            .tokens
            .match_tokens(token_iter![LeftParen, LeftBracket])
        {
            match token.kind {
                TokenKind::LeftParen => lhs = self.parse_function_call(lhs)?,
                TokenKind::LeftBracket => lhs = self.parse_access(lhs)?,
                _ => unreachable!(),
            }
        }

        Ok(lhs)
    }

    fn parse_function_call(&mut self, name: ast::Expr) -> Result<ast::Expr, ParserError> {
        let mut arguments = vec![];
        let _guard = self.tokens.set_ignoring_newline();

        if self.tokens.match_tokens(token_iter![RightParen]).is_none() {
            loop {
                arguments.push(self.parse_expression()?);

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

    fn parse_access(&mut self, name: ast::Expr) -> Result<ast::Expr, ParserError> {
        let _guard = self.tokens.set_ignoring_newline();
        let index = self.parse_expression()?;

        let next_token_is_bracket = self
            .tokens
            .match_tokens(token_iter![RightBracket])
            .is_some();

        if !next_token_is_bracket {
            Err(parser_err!(
                MissingBrackets,
                self.tokens.next().unwrap().line,
                "esperado ']' em indexação".to_string()
            ))?;
        }

        Ok(ast::make_access_expr!(name, index))
    }

    fn parse_primary(&mut self) -> Result<ast::Expr, ParserError> {
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
                self.parse_list()
            }
            LeftParen => {
                let _guard = self.tokens.set_ignoring_newline();
                let expr = self.parse_expression()?;

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

    fn parse_list(&mut self) -> Result<ast::Expr, ParserError> {
        if self
            .tokens
            .match_tokens(token_iter![RightBracket])
            .is_some()
        {
            return Ok(ast::make_list_expr!(vec![]));
        }

        let mut elements = vec![];

        loop {
            elements.push(self.parse_expression()?);

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

impl Parser<'_> {
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
