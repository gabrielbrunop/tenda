use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
}

impl Value {
    pub const TRUE_LITERAL: &'static str = "verdadeiro";
    pub const FALSE_LITERAL: &'static str = "falso";

    pub fn get_type(&self) -> ValueType {
        use Value::*;

        match self {
            Number(_) => ValueType::Number,
            Boolean(_) => ValueType::Boolean,
        }
    }

    pub fn to_number(self) -> Result<f64, ValueError> {
        use Value::*;

        match self {
            Number(value) => Ok(value),
            Boolean(_) => Err(ValueError::UnsupportedTypeConversion(
                self.get_type(),
                ValueType::Boolean,
            )),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
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
            }
        )
    }
}

pub enum ValueType {
    Number,
    Boolean,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValueType::*;

        let str = match self {
            Number => "number".to_string(),
            Boolean => "boolean".to_string(),
        };

        write!(f, "{}", str)
    }
}

pub enum ValueError {
    UnsupportedTypeConversion(ValueType, ValueType),
}
