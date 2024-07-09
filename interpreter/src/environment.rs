use std::collections::{hash_map::Entry, HashMap};

use thiserror::Error;

use super::value::Value;

type Result<T> = std::result::Result<T, EnvironmentError>;

pub struct Environment {
    state: HashMap<String, Value>,
    return_value: Option<Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            state: HashMap::new(),
            return_value: None,
        }
    }

    pub fn get(&self, name: &String) -> Option<&Value> {
        self.state.get(name)
    }

    pub fn exists(&self, name: &String) -> bool {
        self.state.contains_key(name)
    }

    pub fn define(&mut self, name: String, value: Value) -> Result<&Value> {
        if let Entry::Vacant(e) = self.state.entry(name) {
            Ok(e.insert(value))
        } else {
            Err(EnvironmentError::AlreadyDeclared)
        }
    }

    pub fn set(&mut self, name: String, value: Value) -> Result<()> {
        if let Entry::Occupied(mut e) = self.state.entry(name) {
            e.insert(value);
            Ok(())
        } else {
            Err(EnvironmentError::NotFound)
        }
    }

    pub fn set_return(&mut self, value: Value) {
        self.return_value = Some(value);
    }

    pub fn get_return(&self) -> Option<&Value> {
        self.return_value.as_ref()
    }

    pub fn clear_return(&mut self) {
        self.return_value = None;
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum EnvironmentError {
    #[error("variável já declarada")]
    AlreadyDeclared,
    #[error("variável não encontrada")]
    NotFound,
}
