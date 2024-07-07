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
        if self.get_innermost().get_return().is_some() {
            self.move_return_up().ok();
        }

        self.scopes.pop();
    }

    pub fn set_return(&mut self, value: Value) {
        self.get_innermost_mut().set_return(value);
    }

    pub fn consume_return(&mut self) -> Option<Value> {
        let value = self.get_innermost().get_return().cloned();
        self.get_innermost_mut().return_value = None;
        value
    }
}

impl Stack {
    fn get_innermost(&self) -> &Environment {
        self.scopes.last().unwrap_or(&self.global)
    }

    fn get_innermost_mut(&mut self) -> &mut Environment {
        self.scopes.last_mut().unwrap_or(&mut self.global)
    }

    fn move_return_up(&mut self) -> Result<(), ()> {
        let len = self.scopes.len();
        let return_value = self.get_innermost().get_return().ok_or(()).cloned()?;

        self.scopes
            .get_mut(len - 2)
            .ok_or(())?
            .set_return(return_value.clone());

        Ok(())
    }
}

struct Environment {
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

    pub fn set_return(&mut self, value: Value) {
        self.return_value = Some(value);
    }

    pub fn get_return(&self) -> Option<&Value> {
        self.return_value.as_ref()
    }
}
