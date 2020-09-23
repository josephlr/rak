//! Test the public interface to the library
use rak::{print_random_val, Plat};
use std::io::{stdout, Stdout, Write};

struct Hosted(Stdout);

impl Hosted {
    fn new() -> Self {
        Self(stdout())
    }
}

impl std::fmt::Write for Hosted {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self.0.write(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

impl Plat for Hosted {
    fn logger(&mut self) -> &mut dyn std::fmt::Write {
        self
    }
}

#[test]
fn hosted_integration_test() {
    let mut plat = Hosted::new();
    print_random_val(&mut plat, &[42]).unwrap();
}
