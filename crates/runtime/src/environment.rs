use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::value::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    state: HashMap<String, ValueCell>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            state: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&ValueCell> {
        self.state.get(name)
    }

    pub fn has(&self, name: &String) -> bool {
        self.state.contains_key(name)
    }

    pub fn set(&mut self, name: String, value: ValueCell) {
        match self.state.get_mut(&name) {
            Some(val) => match val {
                ValueCell::Shared(val) => {
                    *val.borrow_mut() = value.extract();
                }
                ValueCell::Owned(_) => {
                    self.state.insert(name, value);
                }
            },
            None => {
                self.state.insert(name, value);
            }
        }
    }
}

impl<'a> IntoIterator for &'a Environment {
    type Item = (&'a String, &'a ValueCell);
    type IntoIter = std::collections::hash_map::Iter<'a, String, ValueCell>;

    fn into_iter(self) -> Self::IntoIter {
        self.state.iter()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum ValueCell {
    Owned(Value),
    Shared(Rc<RefCell<Value>>),
}

impl ValueCell {
    pub fn new(value: Value) -> Self {
        ValueCell::Owned(value)
    }

    pub fn new_shared(value: Value) -> Self {
        ValueCell::Shared(Rc::new(RefCell::new(value)))
    }

    pub fn extract(&self) -> Value {
        match self {
            ValueCell::Owned(val) => val.clone(),
            ValueCell::Shared(val) => val.borrow().clone(),
        }
    }
}
