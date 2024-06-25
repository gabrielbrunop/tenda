pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {}
    }

    pub fn interpret(&mut self, string: String) -> String {
        string
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
