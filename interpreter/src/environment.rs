use std::collections::HashMap;

use super::value::Value;

#[derive(Debug)]
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

    pub fn has(&self, name: &String) -> bool {
        self.state.contains_key(name)
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.state.insert(name, value);
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
