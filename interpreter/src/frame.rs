use crate::environment::{Environment, StoredValue};

#[derive(Debug, Clone)]
pub struct Frame {
    pub env: Environment,
    pub return_value: Option<StoredValue>,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            env: Environment::new(),
            return_value: None,
        }
    }

    pub fn set_return_value(&mut self, value: StoredValue) {
        self.return_value = Some(value);
    }

    pub fn get_return_value(&self) -> Option<&StoredValue> {
        self.return_value.as_ref()
    }

    pub fn clear_return_value(&mut self) {
        self.return_value = None;
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}
