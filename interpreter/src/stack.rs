use thiserror::Error;

use crate::environment::{Environment, StoredValue};

type Result<T> = std::result::Result<T, StackError>;

#[derive(Debug)]
pub struct Stack {
    global: Environment,
    scopes: Vec<Environment>,
    has_break: bool,
    has_continue: bool,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            global: Environment::new(),
            scopes: vec![],
            has_break: false,
            has_continue: false,
        }
    }

    pub fn local_exists(&self, name: &String) -> bool {
        self.get_innermost().has(name)
    }

    pub fn define(&mut self, name: String, value: StoredValue) -> Result<()> {
        let scope = self.get_innermost_mut();

        if scope.has(&name) {
            return Err(StackError::AlreadyDeclared);
        }

        scope.set(name, value);

        Ok(())
    }

    pub fn set(&mut self, name: String, value: StoredValue) -> Result<()> {
        let scope = self
            .scopes
            .iter_mut()
            .rev()
            .find(|scope| scope.has(&name))
            .unwrap_or(&mut self.global);

        if scope.has(&name) {
            scope.set(name, value);
            Ok(())
        } else {
            Err(StackError::AssignToUndefined(name))
        }
    }

    pub fn find(&mut self, name: &String) -> Option<&StoredValue> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var);
            }
        }

        self.global.get(name)
    }

    pub fn push(&mut self, environment: Environment) {
        self.scopes.push(environment);
    }

    pub fn pop(&mut self) {
        if self.get_innermost().get_return().is_some() {
            self.move_return_up();
        }

        self.scopes.pop();
    }

    pub fn set_return(&mut self, value: StoredValue) {
        self.get_innermost_mut().set_return(value);
    }

    pub fn has_return(&self) -> bool {
        self.get_innermost().get_return().is_some()
    }

    pub fn consume_return(&mut self) -> Option<StoredValue> {
        let value = self.get_innermost().get_return().cloned();
        self.get_innermost_mut().clear_return();
        value
    }

    pub fn set_break(&mut self, value: bool) {
        self.has_break = value;
    }

    pub fn has_break(&self) -> bool {
        self.has_break
    }

    pub fn set_continue(&mut self, value: bool) {
        self.has_continue = value;
    }

    pub fn has_continue(&self) -> bool {
        self.has_continue
    }
}

impl Stack {
    fn get_innermost(&self) -> &Environment {
        self.scopes.last().unwrap_or(&self.global)
    }

    fn get_innermost_mut(&mut self) -> &mut Environment {
        self.scopes.last_mut().unwrap_or(&mut self.global)
    }

    fn move_return_up(&mut self) {
        let len = self.scopes.len();

        let return_value = match self.get_innermost().get_return().cloned() {
            Some(value) => value,
            None => return,
        };

        let last_index = len - 1;
        let decremented_index = last_index - 1;

        let scope_above = match self.scopes.get_mut(decremented_index) {
            Some(scope) => scope,
            None => return,
        };

        scope_above.set_return(return_value.clone());
    }
}

impl<'a> IntoIterator for &'a Stack {
    type Item = &'a Environment;
    type IntoIter = std::vec::IntoIter<&'a Environment>;

    fn into_iter(self) -> Self::IntoIter {
        let mut scopes: Vec<&Environment> = Vec::with_capacity(self.scopes.len() + 1);
        scopes.push(&self.global);
        scopes.extend(self.scopes.iter());
        scopes.into_iter()
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum StackError {
    #[error("variable already declared")]
    AlreadyDeclared,

    #[error("assignment to undefined variable")]
    AssignToUndefined(String),
}
