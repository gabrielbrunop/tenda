use std::fmt::Debug;

pub trait Platform: Debug {
    fn println(&self, message: &str);
    fn write(&self, message: &str);
    fn rand(&self) -> f64;
}
