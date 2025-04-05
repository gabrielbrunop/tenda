use thiserror::Error;

use crate::{environment::ValueCell, frame::Frame};

type Result<T> = std::result::Result<T, StackError>;

#[derive(Debug)]
pub struct Stack {
    global: Frame,
    frame: Vec<Frame>,
    has_break: bool,
    has_continue: bool,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            global: Frame::new(),
            frame: vec![],
            has_break: false,
            has_continue: false,
        }
    }

    pub fn is_name_in_local_scope(&self, name: &String) -> bool {
        self.get_innermost_frame().get_env().has(name)
    }

    pub fn define(&mut self, name: String, value: ValueCell) -> Result<()> {
        let scope = self.get_innermost_scope_mut();

        if scope.get_env().has(&name) {
            return Err(StackError::AlreadyDeclared);
        }

        scope.get_env_mut().set(name, value);

        Ok(())
    }

    pub fn assign(&mut self, name: String, value: ValueCell) -> Result<()> {
        let frame = self
            .frame
            .iter_mut()
            .rev()
            .find(|frame| frame.get_env().has(&name))
            .unwrap_or(&mut self.global);

        if frame.get_env().has(&name) {
            frame.get_env_mut().set(name, value);
            Ok(())
        } else {
            Err(StackError::AssignToUndefined(name))
        }
    }

    pub fn lookup(&mut self, name: &String) -> Option<&ValueCell> {
        for frame in self.frame.iter().rev() {
            if let Some(var) = frame.get_env().get(name) {
                return Some(var);
            }
        }

        self.global.get_env().get(name)
    }

    pub fn push(&mut self, frame: Frame) {
        self.frame.push(frame);
    }

    pub fn pop(&mut self) {
        if self.get_innermost_frame().get_return_value().is_some() {
            self.shift_return_to_upper_frame();
        }

        self.frame.pop();
    }

    pub fn set_return_value(&mut self, value: ValueCell) {
        self.get_innermost_scope_mut().set_return_value(value);
    }

    pub fn has_return_value(&self) -> bool {
        self.get_innermost_frame().get_return_value().is_some()
    }

    pub fn consume_return_value(&mut self) -> Option<ValueCell> {
        let value = self.get_innermost_frame().get_return_value().cloned();

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

    pub fn global(&self) -> &Frame {
        &self.global
    }

    pub fn global_mut(&mut self) -> &mut Frame {
        &mut self.global
    }
}

impl Stack {
    fn get_innermost_frame(&self) -> &Frame {
        self.frame.last().unwrap_or(&self.global)
    }

    fn get_innermost_scope_mut(&mut self) -> &mut Frame {
        self.frame.last_mut().unwrap_or(&mut self.global)
    }

    fn shift_return_to_upper_frame(&mut self) {
        let len = self.frame.len();

        let return_value = match self.get_innermost_frame().get_return_value().cloned() {
            Some(value) => value,
            None => return,
        };

        let last_index = len - 1;
        let decremented_index = last_index - 1;

        let scope_above = match self.frame.get_mut(decremented_index) {
            Some(scope) => scope,
            None => return,
        };

        scope_above.set_return_value(return_value.clone());
    }
}

impl<'a> IntoIterator for &'a Stack {
    type Item = &'a Frame;
    type IntoIter = std::vec::IntoIter<&'a Frame>;

    fn into_iter(self) -> Self::IntoIter {
        let mut frames: Vec<&Frame> = Vec::with_capacity(self.frame.len() + 1);
        frames.push(&self.global);
        frames.extend(self.frame.iter());
        frames.into_iter()
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
