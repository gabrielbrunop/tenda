#[derive(Debug)]
pub struct Platform;

impl interpreter::platform::Platform for Platform {
    fn println(&self, message: &str) {
        println!("{}", message);
    }

    fn write(&self, message: &str) {
        print!("{}", message);
    }

    fn rand(&self) -> f64 {
        rand::random()
    }
}
