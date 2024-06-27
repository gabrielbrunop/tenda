use crate::{
    ast::{BinaryOp, Expr, UnaryOp},
    value::Value,
};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn interpret_expr(&self, expr: Expr) -> Value {
        use Expr::*;

        match expr {
            Binary { lhs, op, rhs } => self.interpret_binary_op(*lhs, op, *rhs),
            Unary { op, rhs } => self.interpret_unary_op(op, *rhs),
            Grouping { expr } => self.interpret_expr(*expr),
            Literal { value } => value,
        }
    }

    fn interpret_binary_op(&self, lhs: Expr, op: BinaryOp, rhs: Expr) -> Value {
        use crate::ast::BinaryOp::*;

        match op {
            Add => self.interpret_expr(lhs) + self.interpret_expr(rhs),
            Subtract => self.interpret_expr(lhs) - self.interpret_expr(rhs),
            Multiply => self.interpret_expr(lhs) * self.interpret_expr(rhs),
            Divide => self.interpret_expr(lhs) / self.interpret_expr(rhs),
            Exponentiation => self
                .interpret_expr(lhs)
                .to_number()
                .powf(self.interpret_expr(rhs).into())
                .into(),
            Modulo => self.interpret_expr(lhs) % self.interpret_expr(rhs),
        }
    }

    fn interpret_unary_op(&self, op: UnaryOp, rhs: Expr) -> Value {
        use crate::ast::UnaryOp::*;

        match op {
            Negative => -self.interpret_expr(rhs),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
