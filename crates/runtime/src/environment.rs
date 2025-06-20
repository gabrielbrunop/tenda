use std::{
    cell::RefCell,
    collections::{hash_map, HashMap, HashSet},
    rc::Rc,
};

use super::value::Value;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Environment {
    state: HashMap<String, ValueCell>,
    sealed_vars: HashSet<String>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, name: &str) -> Option<&ValueCell> {
        self.state.get(name)
    }

    pub fn has(&self, name: &str) -> bool {
        self.state.contains_key(name)
    }

    pub fn set(&mut self, name: String, value: ValueCell) -> Result<(), EnvironmentSetError> {
        if self.sealed_vars.contains(&name) {
            return Err(EnvironmentSetError::SealedVariable(name));
        }

        self.set_or_replace(name, value);

        Ok(())
    }

    pub fn set_or_replace(&mut self, name: String, value: ValueCell) {
        use ValueCell::*;

        if let Some(Shared(rc) | External(rc)) = self.state.get_mut(&name) {
            *rc.borrow_mut() = value.extract();
            return;
        }

        self.state.insert(name, value);
    }

    pub fn seal(&mut self, name: String) -> Result<(), EnvironmentSealError> {
        if !self.state.contains_key(&name) {
            return Err(EnvironmentSealError::VariableNotFound(name));
        }

        self.sealed_vars.insert(name);

        Ok(())
    }

    pub fn external(&self) -> Self {
        Environment {
            state: self
                .state
                .iter()
                .filter(|(_, v)| matches!(v, ValueCell::External(_)))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            sealed_vars: self.sealed_vars.clone(),
        }
    }
}

impl<'a> IntoIterator for &'a Environment {
    type Item = (&'a String, &'a ValueCell);
    type IntoIter = hash_map::Iter<'a, String, ValueCell>;

    fn into_iter(self) -> Self::IntoIter {
        self.state.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueCell {
    Owned(Value),
    Shared(Rc<RefCell<Value>>),
    External(Rc<RefCell<Value>>),
}

impl ValueCell {
    pub fn new(value: Value) -> Self {
        ValueCell::Owned(value)
    }

    pub fn new_shared(value: Value) -> Self {
        ValueCell::Shared(Rc::new(RefCell::new(value)))
    }

    pub fn new_external(value: Value) -> Self {
        ValueCell::External(Rc::new(RefCell::new(value)))
    }

    pub fn extract(&self) -> Value {
        use ValueCell::*;

        match self {
            Owned(val) => val.clone(),
            Shared(val) | External(val) => val.borrow().clone(),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentSetError {
    #[error("cannot modify a sealed variable: '{0}'")]
    SealedVariable(String),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentSealError {
    #[error("variable '{0}' does not exist in the environment")]
    VariableNotFound(String),
}
