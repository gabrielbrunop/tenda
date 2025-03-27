use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use scanner::token::Literal;

use crate::associative_array::{AssociativeArray, AssociativeArrayKey};
use crate::date::Date;
use crate::function::Function;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(Function),
    List(Rc<RefCell<Vec<Value>>>),
    Range(usize, usize),
    AssociativeArray(Rc<RefCell<AssociativeArray>>),
    Date(Date),
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
            AssociativeArray(_) => ValueType::AssociativeArray,
            Date(_) => ValueType::Date,
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
            Value::AssociativeArray(_) => true,
            Value::Date(_) => true,
        }
    }

    pub fn is_iterable(&self) -> bool {
        matches!(self, Value::List(_) | Value::Range(_, _))
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
                    "<função {}({})>",
                    value.id,
                    value
                        .get_params()
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                List(value) => format!(
                    "[{}]",
                    value
                        .borrow()
                        .iter()
                        .map(|v| match v {
                            Value::String(s) => format!("\"{}\"", escape_special_chars(s)),
                            _ => v.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Range(start, end) => format!("{} até {}", start, end),
                Nil => Literal::NIL_LITERAL.to_string(),
                AssociativeArray(value) => format!(
                    "{{ {} }}",
                    value
                        .borrow()
                        .iter()
                        .map(|(k, v)| match v {
                            Value::String(s) => (k, format!("\"{}\"", escape_special_chars(s))),
                            _ => (k, v.to_string()),
                        })
                        .map(|(k, v)| match k {
                            AssociativeArrayKey::String(key) => format!("\"{}\": {}", key, v),
                            AssociativeArrayKey::Number(key) => format!("{}: {}", key, v),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Date(value) => value.to_iso_string(),
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

impl IntoIterator for Value {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        if !self.is_iterable() {
            panic!("Value is not iterable");
        }

        match self {
            Value::List(list) => list.borrow_mut().clone().into_iter(),
            Value::Range(start, end) => (start..=end)
                .map(|i| Value::Number(i as f64))
                .collect::<Vec<_>>()
                .into_iter(),
            _ => unreachable!(),
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
    AssociativeArray,
    Date,
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
            AssociativeArray => "dicionário".to_string(),
            Date => "data".to_string(),
            Nil => "Nada".to_string(),
        };

        write!(f, "{}", str)
    }
}

fn escape_special_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if let Some(&'\n') = chars.peek() {
                    result.push_str("\\r\\n");
                    chars.next();
                } else {
                    result.push_str("\\r");
                }
            }
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }

    result
}
