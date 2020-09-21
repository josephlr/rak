//! Test the public interface to the library
use std::{io::{Stdout, stdout, Write}};
use uk::Plat;

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
    plat.print_random_val(&[42]);
}