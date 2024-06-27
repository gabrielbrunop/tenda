use std::fmt;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<Value> for f64 {
    fn from(val: Value) -> Self {
        match val {
            Value::Number(n) => n,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Value::Number(n) => n.to_string(),
            }
        )
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs + rhs),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs - rhs),
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs * rhs),
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs / rhs),
        }
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self {
        match self {
            Value::Number(n) => Value::Number(-n),
        }
    }
}

impl Rem for Value {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs % rhs),
        }
    }
}

impl Value {
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
        }
    }
}
