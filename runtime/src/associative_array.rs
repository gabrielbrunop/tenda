use std::fmt;
use std::fmt::Display;

use crate::value::Value;

pub type AssociativeArray = indexmap::IndexMap<AssociativeArrayKey, Value>;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AssociativeArrayKey {
    String(String),
    Number(i64),
}

impl Display for AssociativeArrayKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssociativeArrayKey::String(key) => write!(f, "{}", key),
            AssociativeArrayKey::Number(key) => write!(f, "{}", key),
        }
    }
}
