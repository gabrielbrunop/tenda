use std::collections::{hash_map::Entry, HashMap};

use crate::value::Value;

pub struct Stack {
    global: Environment,
    scopes: Vec<Environment>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            global: Environment::new(),
            scopes: vec![],
        }
    }

    pub fn local_exists(&self, name: &String) -> bool {
        self.get_innermost().exists(name)
    }

    pub fn define(&mut self, name: String, value: Value) -> Result<&Value, ()> {
        self.get_innermost_mut().define(name, value)
    }

    pub fn set(&mut self, name: String, value: Value) -> Result<(), ()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.exists(&name) {
                let _ = scope.set(name, value);
                return Ok(());
            }
        }

        self.global.set(name, value)
    }

    pub fn find(&mut self, name: &String) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var);
            }
        }

        self.global.get(name)
    }

    pub fn allocate(&mut self) {
        self.scopes.push(Environment::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }
}

impl Stack {
    fn get_innermost(&self) -> &Environment {
        self.scopes.last().unwrap_or(&self.global)
    }

    fn get_innermost_mut(&mut self) -> &mut Environment {
        self.scopes.last_mut().unwrap_or(&mut self.global)
    }
}

struct Environment {
    state: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            state: HashMap::new(),
        }
    }

    pub fn get(&self, name: &String) -> Option<&Value> {
        self.state.get(name)
    }

    pub fn exists(&self, name: &String) -> bool {
        self.state.contains_key(name)
    }

    pub fn define(&mut self, name: String, value: Value) -> Result<&Value, ()> {
        if let Entry::Vacant(e) = self.state.entry(name) {
            Ok(e.insert(value))
        } else {
            Err(())
        }
    }

    pub fn set(&mut self, name: String, value: Value) -> Result<(), ()> {
        if let Entry::Occupied(mut e) = self.state.entry(name) {
            e.insert(value);
            Ok(())
        } else {
            Err(())
        }
    }
}
