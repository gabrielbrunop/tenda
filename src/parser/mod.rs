use crate::ast::{BinaryOp, Decl, Expr, Stmt};
use crate::token::{Token, TokenKind};
use crate::token_list;
use crate::value::Value;
use peekmore::{PeekMore, PeekMoreIterator};
use std::fmt;
use std::slice::Iter;

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
        if token.kind == TokenKind::Newline {
            parser_error!(UnexpectedEoi, token.line)
        } else {
            parser_error!(UnexpectedToken($token), token.line)
        }
    }};
}

macro_rules! with_ignoring_newline {
    ($self:ident, $block:block) => {{
        $self.ignoring_newline = true;
        let result = $block;
        $self.ignoring_newline = false;
        result
    }};
}

pub struct Parser<'a> {
    tokens: PeekMoreIterator<Iter<'a, Token>>,
    ignoring_newline: bool,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a [Token]) -> Parser<'a> {
        Parser {
            tokens: source.iter().peekmore(),
            ignoring_newline: false,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParserError>> {
        let program = self.program()?;

        match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Eof => Ok(program),
            Some(token) => Err(vec![unexpected_token!((*token).clone(), self.line)]),
            None => Err(vec![parser_error!(
                UnexpectedEoi,
                self.tokens
                    .peek_backward_or_first(0)
                    .map(|t| t.line)
                    .unwrap_or(0)
            )]),
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
                Err(err) => errors.push(err),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(stmt_list)
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        let token = match self.tokens.peek() {
            Some(token) => token,
            _ => unreachable!(),
        };

        let result = match token.kind {
            TokenKind::Let => self.declaration()?,
            _ => Stmt::Expr(self.expression()?),
        };

        self.consume_newline()?;

        Ok(result)
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        self.tokens.next();

        let name = self.consume_identifier()?;

        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::EqualSign => (),
            Some(token) => return unexpected_token!(token.clone(), token.line).into(),
            None => return parser_error!(UnexpectedEoi, self.get_last_line()).into(),
        };

        let value = Decl::make_local_declaration(name.to_string(), self.expression()?);

        Ok(Stmt::Decl(value))
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.equality()?;

        if let Some(equal_sign) = self.match_tokens(token_list![EqualSign]) {
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

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        loop {
            let op: Option<BinaryOp> = {
                if let Some(token) = self.match_tokens(token_list![Equals]) {
                    Some(token.into())
                } else if self.matches_sequence(token_list![Not, Equals]) {
                    Some(BinaryOp::Inequality)
                } else if let Some(token) = self.match_tokens(token_list![Not]) {
                    return unexpected_token!(token.clone(), token.line).into();
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
            self.match_tokens(token_list![Greater, GreaterOrEqual, Less, LessOrEqual])
        {
            let lhs = expr;
            let rhs = self.term()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while let Some(op) = self.match_tokens(token_list![Plus, Minus]) {
            let lhs = expr;
            let rhs = self.factor()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.exponent()?;

        while let Some(op) = self.match_tokens(token_list![Star, Slash, Percent]) {
            let lhs = expr;
            let rhs = self.exponent()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn exponent(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while let Some(op) = self.match_tokens(token_list![Caret]) {
            let lhs = expr;
            let rhs = self.unary()?;
            expr = Expr::make_binary(lhs, op.into(), rhs);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if let Some(op) = self.match_tokens(token_list![Minus, Not]) {
            let rhs = self.unary()?;
            let expr = Expr::make_unary(op.into(), rhs);

            Ok(expr)
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        use TokenKind::*;

        let token = match self.tokens.next() {
            Some(token) => token,
            _ => unreachable!(),
        };

        match token.kind {
            Number | True | False | String | Nil => {
                Ok(Expr::make_literal(token.literal.clone().unwrap()))
            }
            LeftParen => with_ignoring_newline!(self, {
                let expr = self.expression()?;

                if self.match_tokens(token_list![RightParen]).is_none() {
                    return parser_error!(MissingParentheses, token.line).into();
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
            Eof => parser_error!(UnexpectedEoi, token.line).into(),
            _ => unexpected_token!(token.clone(), token.line).into(),
        }
    }
}

impl<'a> Parser<'a> {
    fn consume_newline(&mut self) -> Result<(), ParserError> {
        use TokenKind::*;

        if self.match_tokens(token_list![Newline]).is_some() {
            return Ok(());
        }

        match self.tokens.peek() {
            Some(token) if matches!(token.kind, Eof) => Ok(()),
            Some(token) => Err(unexpected_token!((*token).clone(), token.line)),
            None => Err(parser_error!(UnexpectedEoi, self.get_last_line())),
        }
    }

    fn ignore_newline(&mut self) -> Option<&Token> {
        if !self.ignoring_newline {
            None
        } else if matches!(self.tokens.peek(), Some(token) if token.kind == TokenKind::Newline) {
            self.tokens.next()
        } else {
            None
        }
    }

    fn match_tokens(&mut self, token_types: Iter<TokenKind>) -> Option<Token> {
        self.ignore_newline();

        let next = self.tokens.peek();

        for t in token_types {
            match next {
                Some(token) if token.kind == *t => {
                    return Some(self.tokens.next().unwrap().clone())
                }
                _ => (),
            }
        }

        None
    }

    fn matches_sequence(&mut self, token_types: Iter<TokenKind>) -> bool {
        assert_eq!(self.tokens.cursor(), 0, "cursor is already in use");

        for token_type in token_types {
            if self.ignore_newline().is_some() {
                continue;
            }

            let matched_sequence =
                matches!(self.tokens.peek(), Some(token) if *token_type == token.kind);

            if !matched_sequence {
                self.tokens.reset_cursor();
                return false;
            }

            self.tokens.advance_cursor();
        }

        for _ in 0..self.tokens.cursor() {
            self.tokens.next();
        }

        true
    }

    fn get_last_line(&mut self) -> usize {
        self.tokens
            .peek_backward_or_first(0)
            .map(|t| t.line)
            .unwrap_or(0)
    }

    fn consume_identifier(&mut self) -> Result<String, ParserError> {
        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Identifier => {
                match token.literal.as_ref().unwrap() {
                    Value::String(string) => Ok(string.to_string()),
                    _ => unreachable!(),
                }
            }
            Some(token) => unexpected_token!(token.clone(), token.line).into(),
            None => parser_error!(UnexpectedEoi, self.get_last_line()).into(),
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
            UnexpectedToken(token) => format!("token inesperado: {}", token.lexeme),
            InvalidAssignmentTarget => {
                "o valor à direita do '=' não é um valor válido para receber atribuições"
                    .to_string()
            }
        }
    }
}

impl<T> From<ParserError> for Result<T, ParserError> {
    fn from(val: ParserError) -> Self {
        Err(val)
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
    InvalidAssignmentTarget,
}

#[cfg(test)]
mod tests;
