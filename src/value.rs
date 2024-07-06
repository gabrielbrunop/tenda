use std::fmt::Display;
use std::{fmt, rc::Rc};

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
    pub const TRUE_LITERAL: &'static str = "verdadeiro";
    pub const FALSE_LITERAL: &'static str = "falso";
    pub const NIL_LITERAL: &'static str = "Nada";

    pub fn get_type(&self) -> ValueType {
        use Value::*;

        match self {
            Number(_) => ValueType::Number,
            Boolean(_) => ValueType::Boolean,
            String(_) => ValueType::String,
            Function(..) => ValueType::Function,
            Nil => ValueType::Nil,
        }
    }

    pub fn to_number(&self) -> Result<f64, ValueError> {
        use Value::*;

        match self {
            Number(value) => Ok(*value),
            _ => Err(ValueError::UnsupportedTypeConversion(
                self.get_type(),
                ValueType::Number,
            )),
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
                    true => Value::TRUE_LITERAL.to_string(),
                    false => Value::FALSE_LITERAL.to_string(),
                },
                String(value) => format!("\"{}\"", value),
                Function(..) => "Função".to_string(),
                Nil => Value::NIL_LITERAL.to_string(),
            }
        )
    }
}

pub enum ValueType {
    Number,
    Boolean,
    String,
    Function,
    Nil,
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

pub enum ValueError {
    UnsupportedTypeConversion(ValueType, ValueType),
}
