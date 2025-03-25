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

    pub fn is_name_in_local_scope(&self, name: &String) -> bool {
        self.get_innermost_scope().has(name)
    }

    pub fn define(&mut self, name: String, value: StoredValue) -> Result<()> {
        let scope = self.get_innermost_scope_mut();

        if scope.has(&name) {
            return Err(StackError::AlreadyDeclared);
        }

        scope.set(name, value);

        Ok(())
    }

    pub fn assign(&mut self, name: String, value: StoredValue) -> Result<()> {
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

    pub fn lookup(&mut self, name: &String) -> Option<&StoredValue> {
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
        if self.get_innermost_scope().get_return_value().is_some() {
            self.shift_return_to_upper_scope();
        }

        self.scopes.pop();
    }

    pub fn set_return_value(&mut self, value: StoredValue) {
        self.get_innermost_scope_mut().set_return_value(value);
    }

    pub fn has_return_value(&self) -> bool {
        self.get_innermost_scope().get_return_value().is_some()
    }

    pub fn consume_return_value(&mut self) -> Option<StoredValue> {
        let value = self.get_innermost_scope().get_return_value().cloned();

        self.get_innermost_scope_mut().clear_return_value();

        value
    }

    pub fn set_loop_break_flag(&mut self, value: bool) {
        self.has_break = value;
    }

    pub fn has_loop_break_flag(&self) -> bool {
        self.has_break
    }

    pub fn set_loop_continue_flag(&mut self, value: bool) {
        self.has_continue = value;
    }

    pub fn has_loop_continue_flag(&self) -> bool {
        self.has_continue
    }

    pub fn global(&self) -> &Environment {
        &self.global
    }
}

impl Stack {
    fn get_innermost_scope(&self) -> &Environment {
        self.scopes.last().unwrap_or(&self.global)
    }

    fn get_innermost_scope_mut(&mut self) -> &mut Environment {
        self.scopes.last_mut().unwrap_or(&mut self.global)
    }

    fn shift_return_to_upper_scope(&mut self) {
        let len = self.scopes.len();

        let return_value = match self.get_innermost_scope().get_return_value().cloned() {
            Some(value) => value,
            None => return,
        };

        let last_index = len - 1;
        let decremented_index = last_index - 1;

        let scope_above = match self.scopes.get_mut(decremented_index) {
            Some(scope) => scope,
            None => return,
        };

        scope_above.set_return_value(return_value.clone());
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
