use crate::stmt::{BinaryOp, Cond, Decl, Expr, Stmt};
use crate::token::{Token, TokenIterator, TokenKind};
use crate::value::Value;
use crate::{token_iter, token_vec, with_ignoring_newline};
use std::fmt;

macro_rules! parser_error {
    ($kind:expr, $line:expr) => {{
        use ParserErrorKind::*;
        ParserError {
            kind: $kind,
            line: $line,
        }
    }};
}

macro_rules! unexpected_token {
    ($token:expr, $line:expr) => {{
        let token = $token;
        parser_error!(UnexpectedToken($token), token.line)
    }};
}

macro_rules! unexpected_eoi {
    ($self:ident) => {{
        vec![parser_error!(UnexpectedEoi, $self.tokens.get_last_line())]
    }};
}

pub struct Parser<'a> {
    tokens: TokenIterator<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [Token]) -> Parser<'a> {
        Parser {
            tokens: source.into(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParserError>> {
        let program = self.program()?;

        match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Eof => Ok(program),
            Some(token) => Err(vec![unexpected_token!((*token).clone(), self.line)]),
            None => Err(unexpected_eoi!(self)),
        }
    }

    fn program(&mut self) -> Result<Vec<Stmt>, Vec<ParserError>> {
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
            Ok(stmt_list)
        }
    }

    fn statement(&mut self) -> Result<Stmt, Vec<ParserError>> {
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
            _ => self.expression().map_err(|err| vec![err]).map(Stmt::Expr),
        }?;

        self.consume_newline().map_err(|err| vec![err])?;

        Ok(result)
    }

    fn block(
        &mut self,
        end_token_types: Vec<TokenKind>,
    ) -> Result<(Stmt, TokenKind), Vec<ParserError>> {
        self.tokens.next();

        let mut stmt_list: Vec<Stmt> = vec![];

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
                Ok(stmt) => stmt_list.push(stmt),
                Err(e) => return Err(e),
            };
        };

        if block_end_delimiter == TokenKind::Eof {
            return Err(unexpected_eoi!(self));
        }

        self.consume_newline().ok();
        self.skip_token(block_end_delimiter).map_err(|e| vec![e])?;

        Ok((Stmt::Block(stmt_list), block_end_delimiter))
    }

    fn if_statement(&mut self) -> Result<Stmt, Vec<ParserError>> {
        self.tokens.next();

        let condition = self.expression().map_err(|err| vec![err])?;

        let (then_branch, block_end_delimiter) = match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Then => {
                self.block(token_vec![BlockEnd, Else])?
            }
            Some(token) => return Err(vec![unexpected_token!(token.clone(), token.line)]),
            None => return Err(unexpected_eoi!(self)),
        };

        let stmt = match block_end_delimiter {
            TokenKind::Else => {
                let (else_branch, _) = self.block(token_vec![BlockEnd])?;
                Cond::make_if_statement(condition, then_branch, Some(else_branch))
            }
            TokenKind::BlockEnd => Cond::make_if_statement(condition, then_branch, None),
            _ => unreachable!(),
        };

        Ok(Stmt::Cond(stmt))
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        self.tokens.next();

        let name = self.consume_identifier()?;

        self.skip_token(TokenKind::EqualSign)?;

        let value = Decl::make_local_declaration(name.to_string(), self.expression()?);

        Ok(Stmt::Decl(value))
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.logical()?;

        if let Some(equal_sign) = self.tokens.match_tokens(token_iter![EqualSign]) {
            let value = self.assignment()?;

            return match expr {
                Expr::Variable { name } => {
                    let name: Expr = Expr::make_literal(Value::String(name));
                    Ok(Expr::make_binary(name, BinaryOp::Assignment, value))
                }
                _ => Err(parser_error!(InvalidAssignmentTarget, equal_sign.line)),
            };
        }

        Ok(expr)
    }

    fn logical(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Or, And]) {
            let lhs = expr;
            let rhs = self.equality()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        loop {
            let op: Option<BinaryOp> = {
                if let Some(token) = self.tokens.match_tokens(token_iter![Equals]) {
                    Some(token.into())
                } else if self.tokens.matches_sequence(token_iter![Not, Equals]) {
                    Some(BinaryOp::Inequality)
                } else if let Some(token) = self.tokens.match_tokens(token_iter![Not]) {
                    return Err(unexpected_token!(token.clone(), token.line));
                } else {
                    None
                }
            };

            if let Some(op) = op {
                let lhs = expr;
                let rhs = self.comparison()?;
                expr = Expr::make_binary(lhs, op, rhs);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;

        while let Some(op) =
            self.tokens
                .match_tokens(token_iter![Greater, GreaterOrEqual, Less, LessOrEqual])
        {
            let lhs = expr;
            let rhs = self.term()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Plus, Minus]) {
            let lhs = expr;
            let rhs = self.factor()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.exponent()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Star, Slash, Percent]) {
            let lhs = expr;
            let rhs = self.exponent()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn exponent(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while let Some(op) = self.tokens.match_tokens(token_iter![Caret]) {
            let lhs = expr;
            let rhs = self.unary()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if let Some(op) = self.tokens.match_tokens(token_iter![Minus, Not]) {
            let rhs = self.unary()?;
            let expr = Expr::make_unary(op.into(), rhs);

            Ok(expr)
        } else {
            self.function_call()
        }
    }

    fn function_call(&mut self) -> Result<Expr, ParserError> {
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
                return Err(parser_error!(
                    MissingFunctionParentheses,
                    self.tokens.next().unwrap().line
                ));
            }
        }

        Ok(Expr::make_call(name, arguments))
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        use TokenKind::*;

        let token = match self.tokens.next() {
            Some(token) => token,
            _ => unreachable!(),
        };

        let line = token.line;

        match token.kind {
            Number | True | False | String | Nil => {
                Ok(Expr::make_literal(token.literal.clone().unwrap()))
            }
            LeftParen => with_ignoring_newline!(self.tokens, {
                let expr = self.expression()?;

                if self.tokens.match_tokens(token_iter![RightParen]).is_none() {
                    return Err(parser_error!(MissingParentheses, line));
                }

                Ok(Expr::make_grouping(expr))
            }),
            Identifier => {
                let name = match token.literal.as_ref().unwrap() {
                    Value::String(string) => string,
                    _ => unreachable!(),
                };

                Ok(Expr::make_variable(name.clone()))
            }
            Eof => Err(parser_error!(UnexpectedEoi, line)),
            _ => Err(unexpected_token!(token.clone(), line)),
        }
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
            Some(token) => Err(unexpected_token!((*token).clone(), token.line)),
            None => Err(parser_error!(UnexpectedEoi, self.tokens.get_last_line())),
        }
    }

    fn consume_identifier(&mut self) -> Result<String, ParserError> {
        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Identifier => {
                match token.literal.as_ref().unwrap() {
                    Value::String(string) => Ok(string.to_string()),
                    _ => unreachable!(),
                }
            }
            Some(token) => Err(unexpected_token!(token.clone(), token.line)),
            None => Err(parser_error!(UnexpectedEoi, self.tokens.get_last_line())),
        }
    }

    fn skip_token(&mut self, token_kind: TokenKind) -> Result<(), ParserError> {
        match self.tokens.next() {
            Some(token) if token.kind == token_kind => Ok(()),
            Some(token) if token.kind == TokenKind::Eof => {
                Err(parser_error!(UnexpectedEoi, token.line))
            }
            Some(token) => Err(unexpected_token!(token.clone(), token.line)),
            None => Err(parser_error!(UnexpectedEoi, self.tokens.get_last_line())),
        }
    }
}

#[derive(Debug)]
pub struct ParserError {
    line: usize,
    kind: ParserErrorKind,
}

impl ParserError {
    pub fn message(&self) -> String {
        use ParserErrorKind::*;

        match &self.kind {
            UnexpectedEoi => "fim inesperado de input".to_string(),
            MissingParentheses => "esperado ')' após a expressão".to_string(),
            MissingFunctionParentheses => "esperado ')' ao fim da chamada de função".to_string(),
            UnexpectedToken(token) => format!("token inesperado: {}", token.lexeme),
            InvalidAssignmentTarget => {
                "o valor à direita do '=' não é um valor válido para receber atribuições"
                    .to_string()
            }
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (na linha {})", self.message(), self.line)
    }
}

#[derive(Debug)]
pub enum ParserErrorKind {
    UnexpectedEoi,
    UnexpectedToken(Token),
    MissingParentheses,
    MissingFunctionParentheses,
    InvalidAssignmentTarget,
}

#[cfg(test)]
mod tests;
