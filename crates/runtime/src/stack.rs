use std::rc::Rc;
use thiserror::Error;

use crate::{environment::ValueCell, frame::Frame, Environment, EnvironmentSetError};

#[derive(Debug, Default)]
pub struct Stack {
    global: Frame,
    frames: Vec<Frame>,
    base: Option<Rc<Environment>>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            global: Frame::new(),
            frames: vec![],
            base: None,
        }
    }

    pub fn with_base(base: Rc<Environment>) -> Self {
        Stack {
            global: Frame::new(),
            frames: vec![],
            base: Some(base),
        }
    }

    pub fn is_name_in_local_scope(&self, name: &str) -> bool {
        self.get_innermost_frame().get_env().has(name)
    }

    pub fn is_in_base_env(&self, name: &str) -> bool {
        self.base.as_ref().map(|env| env.has(name)).unwrap_or(false)
    }

    pub fn define(&mut self, name: String, value: ValueCell) -> Result<(), StackDefinitionError> {
        let scope = self.get_innermost_frame_mut().get_env_mut();

        if scope.has(&name) {
            return Err(StackDefinitionError::AlreadyDeclared);
        }

        scope.set_or_replace(name, value);

        Ok(())
    }

    pub fn assign(&mut self, name: String, value: ValueCell) -> Result<(), StackAssignmentError> {
        let env = self
            .frames
            .iter_mut()
            .rev()
            .find(|frame| frame.get_env().has(&name))
            .unwrap_or(&mut self.global)
            .get_env_mut();

        if let Some(base_env) = &self.base {
            if base_env.has(&name) {
                return Err(StackAssignmentError::AssignToBaseEnvironment);
            }
        }

        if env.has(&name) {
            env.set(name, value).map_err(StackAssignmentError::from)?;
            Ok(())
        } else {
            Err(StackAssignmentError::AssignToUndefined(name))
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&ValueCell> {
        for frame in self.frames.iter().rev() {
            if let Some(var) = frame.get_env().get(name) {
                return Some(var);
            }
        }

        if let Some(var) = self.global.get_env().get(name) {
            return Some(var);
        }

        if let Some(base_env) = &self.base {
            if let Some(var) = base_env.get(name) {
                return Some(var);
            }
        }

        None
    }

    pub fn push(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn pop(&mut self) {
        if self.get_innermost_frame().is_state_dirty() {
            self.shift_frame_state_to_upper_frame();
        }

        self.frames.pop();
    }

    pub fn set_return_value(&mut self, value: ValueCell) {
        self.get_innermost_frame_mut().set_return_value(value);
    }

    pub fn has_return_value(&self) -> bool {
        self.frames
            .last()
            .and_then(|frame| frame.get_return_value())
            .is_some()
    }

    pub fn consume_return_value(&mut self) -> Option<ValueCell> {
        let frame = self.get_innermost_frame_mut();
        let value = frame.get_return_value().cloned();

        frame.clear_return_value();

        value
    }

    pub fn set_loop_break_flag(&mut self, value: bool) {
        self.get_innermost_frame_mut().set_loop_break_flag(value);
    }

    pub fn has_loop_break_flag(&self) -> bool {
        self.get_innermost_frame().has_loop_break_flag()
    }

    pub fn set_loop_continue_flag(&mut self, value: bool) {
        self.get_innermost_frame_mut().set_loop_continue_flag(value)
    }

    pub fn has_loop_continue_flag(&self) -> bool {
        self.get_innermost_frame().has_loop_continue_flag()
    }

    pub fn global(&self) -> &Environment {
        self.global.get_env()
    }

    pub fn global_mut(&mut self) -> &mut Environment {
        self.global.get_env_mut()
    }
}

impl Stack {
    fn get_innermost_frame(&self) -> &Frame {
        self.frames.last().unwrap_or(&self.global)
    }

    fn get_innermost_frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap_or(&mut self.global)
    }

    fn shift_frame_state_to_upper_frame(&mut self) {
        let innermost_frame_state = self.get_innermost_frame().get_state().clone();

        let scope_above = self
            .frames
            .iter_mut()
            .nth_back(1)
            .unwrap_or(&mut self.global);

        scope_above.set_state(innermost_frame_state);
    }
}

impl<'a> IntoIterator for &'a Stack {
    type Item = &'a Environment;
    type IntoIter = std::vec::IntoIter<&'a Environment>;

    fn into_iter(self) -> Self::IntoIter {
        let mut frames: Vec<&Environment> = Vec::with_capacity(self.frames.len() + 1);
        frames.push(self.global.get_env());
        frames.extend(self.frames.iter().map(|frame| frame.get_env()));
        frames.into_iter()
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum StackDefinitionError {
    #[error("variable already declared")]
    AlreadyDeclared,
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum StackAssignmentError {
    #[error("assignment to base environment is not allowed")]
    AssignToBaseEnvironment,

    #[error("assignment to undefined variable: '{0}'")]
    AssignToUndefined(String),

    #[error("cannot assign to unreassignable variable: '{0}'")]
    Unreassignable(String),
}

impl From<EnvironmentSetError> for StackAssignmentError {
    fn from(err: EnvironmentSetError) -> Self {
        match err {
            EnvironmentSetError::SealedVariable(name) => StackAssignmentError::Unreassignable(name),
        }
    }
}
