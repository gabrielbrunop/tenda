use crate::environment::{Environment, ValueCell};

#[derive(Debug, Clone)]
pub struct Frame {
    env: Environment,
    state: FrameState,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            env: Environment::new(),
            state: FrameState::default(),
        }
    }

    pub fn from_env(env: Environment) -> Self {
        Frame {
            env,
            state: FrameState::default(),
        }
    }

    pub fn get_env(&self) -> &Environment {
        &self.env
    }

    pub fn get_env_mut(&mut self) -> &mut Environment {
        &mut self.env
    }

    pub fn get_return_value(&self) -> Option<&ValueCell> {
        self.state.return_value.as_ref()
    }

    pub fn set_return_value(&mut self, value: ValueCell) {
        self.state.return_value = Some(value);
    }

    pub fn clear_return_value(&mut self) {
        self.state.return_value = None;
    }

    pub fn has_loop_break_flag(&self) -> bool {
        self.state.loop_break_flag
    }

    pub fn set_loop_break_flag(&mut self, value: bool) {
        self.state.loop_break_flag = value;
    }

    pub fn has_loop_continue_flag(&self) -> bool {
        self.state.loop_continue_flag
    }

    pub fn set_loop_continue_flag(&mut self, value: bool) {
        self.state.loop_continue_flag = value;
    }

    pub fn get_state(&self) -> &FrameState {
        &self.state
    }

    pub fn set_state(&mut self, state: FrameState) {
        self.state = state;
    }

    pub fn is_state_dirty(&self) -> bool {
        self.state.return_value.is_some()
            || self.state.loop_break_flag
            || self.state.loop_continue_flag
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FrameState {
    return_value: Option<ValueCell>,
    loop_break_flag: bool,
    loop_continue_flag: bool,
}
