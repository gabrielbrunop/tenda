use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;
use tenda_scanner::Literal;

use crate::associative_array::{AssociativeArray, AssociativeArrayKey};
use crate::date::Date;
use crate::function::Function;
use crate::Environment;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Function(Function),
    Range(usize, usize),
    List(Rc<DynamicValue<Vec<Value>>>),
    AssociativeArray(Rc<DynamicValue<AssociativeArray>>),
    Date(Date),
    Module(Environment),
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
            Module(_) => ValueType::Module,
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
            Value::Module(_) => true,
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
                Number(value) => match value {
                    v if v.is_infinite() => {
                        if v.is_sign_positive() {
                            Literal::POSITIVE_INFINITY_LITERAL.to_string()
                        } else {
                            Literal::NEGATIVE_INFINITY_LITERAL.to_string()
                        }
                    }
                    v if v.is_nan() => Literal::NAN_LITERAL.to_string(),
                    _ => value.to_string(),
                },
                Boolean(value) => match *value {
                    true => Literal::TRUE_LITERAL.to_string(),
                    false => Literal::FALSE_LITERAL.to_string(),
                },
                String(value) => format!("\"{}\"", value),
                Function(value) => format!("<função {:#x}>", value.id),
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
                Module(_) => "<módulo>".to_string(),
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
            panic!("value is not iterable");
        }

        match self {
            Value::List(list) => {
                let inner = list.borrow().clone();
                inner.into_iter()
            }
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
    Module,
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
            Module => "módulo".to_string(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug)]
pub struct DynamicValue<T> {
    value: RefCell<T>,
    frozen: Cell<bool>,
}

impl<T: Clone> Clone for DynamicValue<T> {
    fn clone(&self) -> Self {
        let cloned_inner = { (*self.value.borrow()).clone() };

        Self {
            value: RefCell::new(cloned_inner),
            frozen: Cell::new(self.frozen.get()),
        }
    }
}

impl<T: PartialEq> PartialEq for DynamicValue<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.value.borrow() == *other.value.borrow()
    }
}

impl<T> DynamicValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            frozen: Cell::new(false),
        }
    }

    pub fn new_frozen(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            frozen: Cell::new(true),
        }
    }

    pub fn freeze(&self) {
        self.frozen.set(true);
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen.get()
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.value.borrow()
    }

    pub fn borrow_mut_checked(&self) -> Result<RefMut<'_, T>, DynamicValueError> {
        if self.is_frozen() {
            Err(DynamicValueError::Frozen)
        } else {
            Ok(self.value.borrow_mut())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DynamicValueError {
    #[error("cannot modify a frozen value")]
    Frozen,
}

pub fn escape_special_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '\0' => result.push_str("\\0"),
            '\x07' => result.push_str("\\a"),
            '\x08' => result.push_str("\\b"),
            '\x0C' => result.push_str("\\f"),
            '\x0B' => result.push_str("\\v"),
            '\x1B' => result.push_str("\\e"),
            '\r' => result.push_str("\\r"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\'' => result.push_str("\\\'"),

            c if c.is_control() => {
                let byte = c as u32;
                result.push_str(&format!("\\x{byte:02X}"));
            }

            _ => result.push(c),
        }
    }

    result
}

pub fn escape_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", escape_special_chars(s)),
        _ => value.to_string(),
    }
}
