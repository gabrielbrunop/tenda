use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use scanner::token::Literal;

use crate::function::Function;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(Rc<Function>),
    Nil,
}

impl Value {
    pub fn kind(&self) -> ValueType {
        use Value::*;

        match self {
            Number(_) => ValueType::Number,
            Boolean(_) => ValueType::Boolean,
            String(_) => ValueType::String,
            Function(..) => ValueType::Function,
            Nil => ValueType::Nil,
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Value::Number(value) => *value != 0.0,
            Value::Boolean(value) => *value,
            Value::String(_) => true,
            Value::Function(..) => true,
            Value::Nil => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Value::*;

        write!(
            f,
            "{}",
            match self {
                Number(value) => value.to_string(),
                Boolean(value) => match *value {
                    true => Literal::TRUE_LITERAL.to_string(),
                    false => Literal::FALSE_LITERAL.to_string(),
                },
                String(value) => format!("\"{}\"", value),
                Function(value) => format!(
                    "{}({})",
                    value.context.name,
                    value.context.params.join(", ")
                ),
                Nil => Literal::NIL_LITERAL.to_string(),
            }
        )
    }
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        use Literal::*;

        match literal {
            Number(value) => Value::Number(value),
            String(value) => Value::String(value),
            Boolean(value) => Value::Boolean(value),
            Nil => Value::Nil,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    Number,
    Boolean,
    String,
    Function,
    Nil,
}

impl From<Value> for ValueType {
    fn from(value: Value) -> Self {
        use Value::*;
        match value {
            Number(_) => ValueType::Number,
            Boolean(_) => ValueType::Boolean,
            String(_) => ValueType::String,
            Function(_) => ValueType::Function,
            Nil => ValueType::Nil,
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValueType::*;

        let str = match self {
            Number => "número".to_string(),
            Boolean => "lógico".to_string(),
            String => "texto".to_string(),
            Function => "função".to_string(),
            Nil => "Nada".to_string(),
        };

        write!(f, "{}", str)
    }
}
