use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::value::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    state: HashMap<String, StoredValue>,
    return_value: Option<StoredValue>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            state: HashMap::new(),
            return_value: None,
        }
    }

    pub fn get(&self, name: &String) -> Option<&StoredValue> {
        self.state.get(name)
    }

    pub fn has(&self, name: &String) -> bool {
        self.state.contains_key(name)
    }

    pub fn set(&mut self, name: String, value: StoredValue) {
        match self.state.get_mut(&name) {
            Some(val) => match val {
                StoredValue::Shared(val) => {
                    *val.borrow_mut() = value.clone_value();
                }
                StoredValue::Unique(_) => {
                    self.state.insert(name, value);
                }
            },
            None => {
                self.state.insert(name, value);
            }
        }
    }

    pub fn set_return(&mut self, value: StoredValue) {
        self.return_value = Some(value);
    }

    pub fn get_return(&self) -> Option<&StoredValue> {
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

#[derive(Debug, Clone)]
pub enum StoredValue {
    Unique(Value),
    Shared(Rc<RefCell<Value>>),
}

impl StoredValue {
    pub fn new_unique(value: Value) -> Self {
        StoredValue::Unique(value)
    }

    pub fn new_shared(value: Value) -> Self {
        StoredValue::Shared(Rc::new(RefCell::new(value)))
    }

    pub fn clone_value(&self) -> Value {
        match self {
            StoredValue::Unique(val) => val.clone(),
            StoredValue::Shared(val) => val.borrow().clone(),
        }
    }
}
