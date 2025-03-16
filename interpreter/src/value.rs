use std::cell::RefCell;
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
    Function(Function),
    List(Rc<RefCell<Vec<Value>>>),
    Range(f64, f64),
    Nil,
}

impl Value {
    pub fn kind(&self) -> ValueType {
        use Value::*;

        match self {
            Number(_) => ValueType::Number,
            Boolean(_) => ValueType::Boolean,
            String(_) => ValueType::String,
            Function(_) => ValueType::Function,
            List(_) => ValueType::List,
            Range(_, _) => ValueType::Range,
            Nil => ValueType::Nil,
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Value::Number(value) => *value != 0.0,
            Value::Boolean(value) => *value,
            Value::String(_) => true,
            Value::Function(_) => true,
            Value::List(_) => true,
            Value::Range(_, _) => true,
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
                List(value) => format!(
                    "[{}]",
                    value
                        .borrow()
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Range(start, end) => format!("{} até {}", start, end),
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
    List,
    Range,
    Nil,
}

impl From<Value> for ValueType {
    fn from(value: Value) -> Self {
        value.kind()
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
            List => "lista".to_string(),
            Range => "intervalo".to_string(),
            Nil => "Nada".to_string(),
        };

        write!(f, "{}", str)
    }
}
